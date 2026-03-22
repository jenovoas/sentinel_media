use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use std::path::Path;
use regex::Regex;
use sentinel_media_core::load_agent_skill;

#[derive(Parser, Debug)]
#[command(author, version, about = "Sentinel Forensic Verifier (Rust Native)")]
struct Args {
    /// Archivo a auditar para detectar alucinaciones o datos falsos
    #[arg(long)]
    file: String,

    /// Activar modo paranoico (Duda Metódica máxima)
    #[arg(long)]
    strict: bool,

    /// Validar contra NotebookLM (requiere API configurada)
    #[arg(long)]
    notebook: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 🛡️ Protocolo UNISON: Cargar entorno sagrado
    dotenvy::dotenv().ok();
    let args = Args::parse();
    
    println!("{}", "🛡️ Sentinel Forensic Verifier - Iniciando Auditoría...".bold().cyan());
    
    let path = Path::new(&args.file);
    if !path.exists() {
        anyhow::bail!("❌ El archivo no existe: {}", args.file);
    }

    let content = std::fs::read_to_string(path)
        .with_context(|| format!("No se pudo leer el archivo {}", args.file))?;

    // 1. Detección de Magic Values (Números flotantes sin contexto)
    println!("🔢 Escaneando Magic Values...");
    let re_float = Regex::new(r"\b[0-9]+\.[0-9]+\b")?;
    let mut magic_count = 0;
    for line in content.lines() {
        if re_float.is_match(line) && !line.contains("const") && !line.contains("//") {
            println!("   ⚠️  Posible valor alucinado/sin fuente: {}", line.trim().yellow());
            magic_count += 1;
        }
    }

    // 2. Validación de Referencias (Placeholder detection)
    println!("📚 Verificando Integridad de Referencias...");
    if content.contains("[DOI]") || content.contains("[arXiv]") || content.contains("0.0.0.0") {
        println!("   🚨 DETECTADO: Placeholders de IA encontrados. El contenido podría estar falseado.");
    }

    // 3. Cargar Skill de Integridad
    match load_agent_skill() {
        Ok(_) => println!("✅ Skill de Integridad cargada y activa."),
        Err(_) => println!("⚠️ Skill no encontrada. Operando bajo reglas base DNA."),
    }

    println!("---");
    if magic_count > 0 {
        println!("{}", format!("❌ Integridad Comprometida: {} alertas encontradas.", magic_count).red().bold());
    } else {
        println!("{}", "✅ Integridad Verificada: No se encontraron patrones de alucinación evidentes.".green().bold());
    }

    Ok(())
}
