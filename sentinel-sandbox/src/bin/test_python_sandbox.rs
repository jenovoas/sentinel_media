use sentinel_sandbox::{SentinelSandbox, SandboxConfig};
use anyhow::Result;
use std::path::Path;

fn main() -> Result<()> {
    let wasm_path = Path::new("/home/jnovoas/Desarrollo/sentinel_media/core/tools/python.wasm");
    let script_dir = Path::new("/home/jnovoas/Desarrollo/sentinel_media/core/tools");
    
    if !wasm_path.exists() {
        anyhow::bail!("No se encontró python.wasm en core/tools/");
    }

    println!("🧪 [PYTHON SANDBOX TEST] Cargando motor de Python...");
    let mut sandbox = SentinelSandbox::new(wasm_path)?;

    // Configurar el mapeo de directorios
    // Mapeamos el directorio de scripts para que sea accesible desde el sandbox
    let config = SandboxConfig {
        allowed_paths: vec![script_dir.to_path_buf()],
        env_vars: vec![("PYTHONPATH".to_string(), ".".to_string())],
    };

    println!("🚀 [PYTHON SANDBOX TEST] Ejecutando: python.wasm test_tool.py greet Sentinel");
    
    // En el sandbox, el path será el path absoluto pero mapeado
    // El intérprete de python intentará abrir el archivo.
    let script_path = "/home/jnovoas/Desarrollo/sentinel_media/core/tools/test_tool.py";
    
    let (stdout, stderr) = sandbox.run(&config, vec![
        "python".to_string(), 
        script_path.to_string(), 
        "greet".to_string(), 
        "Sentinel-Media".to_string()
    ])?;

    println!("✅ [STDOUT]:\n{}", stdout);
    if !stderr.is_empty() {
        println!("⚠️ [STDERR]:\n{}", stderr);
    }

    Ok(())
}
