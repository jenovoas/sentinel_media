//! 🛠️ Sentinel Tools: AGENTIC TOOLBOX 🛠️
//! ---------------------------------------------------------------------------
//! Gestiona el registro y ejecución de herramientas en el Sandbox WASM.

use anyhow::Result;
use std::path::PathBuf;
use sentinel_sandbox::{SentinelSandbox, SandboxConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub wasm_path: PathBuf,
}

pub struct ToolManager {
    tools_dir: PathBuf,
}

impl ToolManager {
    pub fn new(tools_dir: PathBuf) -> Self {
        Self { tools_dir }
    }

    /// Ejecuta una herramienta por nombre con los argumentos proporcionados
    pub fn call_tool(&self, name: &str, args: Vec<String>) -> Result<String> {
        let wasm_file = self.tools_dir.join(format!("{}.wasm", name));
        if !wasm_file.exists() {
            anyhow::bail!("Herramienta no encontrada: {}.wasm en {:?}", name, self.tools_dir);
        }

        let mut sandbox = SentinelSandbox::new(&wasm_file)?;
        
        // Configuración por defecto para herramientas
        let config = SandboxConfig {
            allowed_paths: vec![self.tools_dir.clone()], // Mapear el directorio de herramientas
            env_vars: vec![("SENTINEL_TOOL".to_string(), name.to_string())],
        };

        let (stdout, stderr) = sandbox.run(&config, args)?;
        
        if !stderr.is_empty() {
            log::warn!("Advertencia en herramienta {}: {}", name, stderr);
        }

        Ok(stdout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_tool_manager_probe() {
        // Esta prueba requiere que sentinel_probe.wasm exista en core/tools
        let tools_dir = Path::new("/home/jnovoas/Desarrollo/sentinel_media/core/tools");
        let manager = ToolManager::new(tools_dir.to_path_buf());
        
        if tools_dir.join("sentinel_probe.wasm").exists() {
            let result = manager.call_tool("sentinel_probe", vec!["test".to_string(), "arg".to_string()]);
            assert!(result.is_ok(), "El tool call falló: {:?}", result.err());
            let out = result.unwrap();
            assert!(out.contains("Found 2 arguments: test, arg"));
        } else {
            println!("⚠️ Saltando test_tool_manager_probe: sentinel_probe.wasm no encontrado.");
        }
    }
}
