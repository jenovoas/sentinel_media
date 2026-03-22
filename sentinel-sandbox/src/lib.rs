//! 🛡️ Sentinel Sandbox: WASM SECURITY LAYER 🛡️
//! ---------------------------------------------------------------------------
//! Provee ejecución aislada para herramientas agénticas usando Wasmer y WASI.

use wasmer::{Instance, Module, Store, TypedFunction};
use wasmer_wasi::WasiState;
use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub allowed_paths: Vec<PathBuf>,
    pub env_vars: Vec<(String, String)>,
}

pub struct SentinelSandbox {
    store: Store,
    module: Module,
}

impl SentinelSandbox {
    /// Carga un módulo WASM desde un archivo
    pub fn new(wasm_path: &Path) -> Result<Self> {
        let store = Store::default();
        let wasm_bytes = std::fs::read(wasm_path)
            .with_context(|| format!("No se pudo leer el archivo WASM en {:?}", wasm_path))?;
        let module = Module::new(&store, wasm_bytes)?;
        
        Ok(Self { store, module })
    }

    /// Ejecuta el módulo en un entorno WASI controlado y captura la salida
    pub fn run(&mut self, config: &SandboxConfig, args: Vec<String>) -> Result<(String, String)> {
        use wasmer_wasi::Pipe;
        let stdout = Pipe::default();
        let stderr = Pipe::default();

        // 1. Configurar WASI
        let mut wasi_state_builder = WasiState::new("sentinel-tool");
        
        // Redirigir salidas
        wasi_state_builder
            .stdout(Box::new(stdout.clone()))
            .stderr(Box::new(stderr.clone()))
            .args(args);
        
        // Variables de Entorno
        for (k, v) in &config.env_vars {
            wasi_state_builder.env(k, v);
        }

        // Mapeo de Directorios (Aislamiento)
        for path in &config.allowed_paths {
            if path.exists() {
                wasi_state_builder.preopen_dir(path)?;
            }
        }

        let mut wasi_env = wasi_state_builder.finalize(&mut self.store)
            .map_err(|e| anyhow::anyhow!("Error en WasiState::finalize: {}", e))?;
        
        // 2. Importar Funciones WASI
        let import_object = wasi_env.import_object(&mut self.store, &self.module)
            .map_err(|e| anyhow::anyhow!("Error en WasiEnv::import_object: {}", e))?;
        
        // 3. Instanciar y Ejecutar
        let instance = Instance::new(&mut self.store, &self.module, &import_object)
            .map_err(|e| anyhow::anyhow!("Error en Instance::new: {}", e))?;
        
        // CRITICAL for Wasmer 3.x: Initialize WASI with the instance
        wasi_env.initialize(&mut self.store, &instance)
            .map_err(|e| anyhow::anyhow!("Error en WasiEnv::initialize: {}", e))?;
        
        // El punto de entrada estándar para WASI es _start
        let start: TypedFunction<(), ()> = instance.exports.get_typed_function(&self.store, "_start")?;
        
        match start.call(&mut self.store) {
            Ok(_) => {
                // Capturar la salida de los pipes
                use std::io::Read;
                let mut out_vec = Vec::new();
                let mut err_vec = Vec::new();
                let mut stdout_pipe = stdout;
                let mut stderr_pipe = stderr;
                
                stdout_pipe.read_to_end(&mut out_vec).ok();
                stderr_pipe.read_to_end(&mut err_vec).ok();

                let out_str = String::from_utf8_lossy(&out_vec).to_string();
                let err_str = String::from_utf8_lossy(&err_vec).to_string();
                Ok((out_str, err_str))
            },
            Err(e) => anyhow::bail!("Error durante la ejecución en el sandbox: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let config = SandboxConfig {
            allowed_paths: vec![PathBuf::from("/tmp")],
            env_vars: vec![("KEY".to_string(), "VALUE".to_string())],
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("allowed_paths"));
        assert!(json.contains("env_vars"));
    }

    #[test]
    fn test_sandbox_not_found() {
        let result = SentinelSandbox::new(Path::new("/non/existent/path.wasm"));
        assert!(result.is_err());
    }
}
