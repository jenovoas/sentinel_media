use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

// Import run functions and arg structs from the library crates
use sentinel_scanner::{run as scan, Args as ScanArgs, ScanOutput};
use sentinel_research::{run as research, Args as ResearchArgs};
use sentinel_media::{run as produce, Args as MediaArgs};
use sentinel_publisher::{run as publish, Args as PublishArgs};

#[derive(Parser, Debug)]
#[command(name = "sentinel", author = "Sentinel Swarm", version = "0.9.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Args, Debug)]
pub struct FactoryArgs {
    #[arg(long, default_value = ".")]
    pub vault: String,
    #[arg(long, default_value_t = 0.95)]
    pub min_score: f64,
    #[arg(long, default_value = "gemini")]
    pub provider: String,
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long)]
    pub verbose: bool,
    #[arg(long)]
    pub publish: bool,
    #[arg(long)]
    pub shorts: bool,
    #[arg(long)]
    pub longform: bool,
    #[arg(long)]
    pub stitch: bool,
    #[arg(long)]
    pub local: bool,
    #[arg(long)]
    pub remotion_render: bool,
    #[arg(long)]
    pub file: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Full YouTube Factory Orchestrator (Scan -> Research -> Media -> Publish)
    Factory(FactoryArgs),
    /// Scan the vault for video candidates
    Scan(ScanArgs),
}

async fn factory_pipeline(args: &FactoryArgs) -> Result<()> {
    println!("{}", "🏭 [FACTORY] Iniciando Pipeline End-to-End...".cyan().bold());

    // 1. SCAN
    println!("{}", "🔍 [1/4] Escaneando bóveda en busca de candidatos...".cyan());
    let scan_args = ScanArgs {
        vault: args.vault.clone(),
        min_score: args.min_score,
        verbose: args.verbose,
    };
    let scan_output = scan(scan_args)?;
    println!("   => Se encontraron {} candidatos listos.", scan_output.ready.len());

    if scan_output.ready.is_empty() {
        println!("{}", "✅ No hay candidatos para procesar. Ciclo de Factory completado.".green());
        return Ok(());
    }

    // 2. RESEARCH
    println!("{}", "📝 [2/4] Generando guiones para los candidatos...".magenta());
    for candidate in &scan_output.ready {
        println!("   -> Investigando: {}", candidate.rel_path.bright_blue());
        
        // Determinar canal basado en la ruta (heurística: nombre de carpeta raíz del canal)
        let channel = if candidate.rel_path.contains("SecurePenguin") {
            "secure_penguin"
        } else if candidate.rel_path.contains("CodePenguin") {
            "code_penguin"
        } else if candidate.rel_path.contains("QuantumPenguin") {
            "quantum_penguin"
        } else if candidate.rel_path.contains("SumerPenguin") {
            "sumer_penguin"
        } else if candidate.rel_path.contains("KernelPenguin") {
            "kernel_penguin"
        } else if candidate.rel_path.contains("AutoPenguin") {
            "auto_penguin"
        } else {
            "secure_penguin" // Default
        };

        let research_args = ResearchArgs {
            file: Some(candidate.file.clone()),
            prompt: Some("Genera un guion para un video de YouTube basado en este contenido. Incluye título, descripción y etiquetas.".to_string()),
            deep: true,
            imagina: false,
            intuicion: false,
            refactor: false,
            translate: false,
            target_lang: "es".to_string(),
            interactive: false,
            system: false,
            memory_tier: "warm".to_string(),
            telos_context: false,
            hook: vec![],
            groq: args.provider == "groq",
            openai: args.provider == "openai",
            perplexity: args.provider == "perplexity",
            antigravity: false,
            target: "youtube".to_string(),
        };
        
        match research(research_args).await {
            Ok(script_content) => {
                let script_path = format!("{}.md", candidate.file);
                tokio::fs::write(&script_path, &script_content).await?;
                println!("      ✅ Guion guardado en: {}", script_path.green());

                if args.dry_run {
                    println!("      ⚠️ MODO DRY RUN: Saltando generación de medios y publicación.");
                    continue;
                }

                // 3. MEDIA
                println!("   -> 🎬 Produciendo video para: {}", candidate.rel_path.bright_blue());
                let video_output_name = format!("{}_gen.mp4", PathBuf::from(&candidate.file).file_stem().unwrap_or_default().to_string_lossy());
                let media_args = MediaArgs {
                    file: Some(script_path.clone()),
                    video: true,
                    image: false,
                    pdf: false,
                    duration: 60,
                    aspect_ratio: "16:9".to_string(),
                    resolution: "1080p".to_string(),
                    image_aspect: "16:9".to_string(),
                    local: false,
                    concat: false,
                    inputs: vec![],
                    output: Some(video_output_name.clone()),
                    remotion_render: false,
                    gpu: false,
                };
                
                if let Err(e) = produce(media_args).await {
                    eprintln!("      ❌ Error en generación de medios: {}", e);
                    continue; 
                }
                
                // 4. PUBLISH
                if args.publish {
                    println!("   -> 🚀 Publicando video para el canal: {}", channel.yellow());
                    let publish_args = PublishArgs {
                        file: video_output_name,
                        channel: channel.to_string(),
                        title: Some(candidate.rel_path.clone()),
                        description: Some(script_content),
                        privacy: "private".to_string(),
                        dry_run: args.dry_run,
                    };

                    if let Err(e) = publish(publish_args).await {
                        eprintln!("      ❌ Publicación fallida: {}", e);
                    }
                }

            },
            Err(e) => {
                eprintln!("      ❌ Error en investigación: {}", e);
            }
        }
    }

    println!("{}", "✅ Ciclo de Factory completado exitosamente.".green().bold());
    Ok(())
}


#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Factory(args) => {
            if let Err(e) = factory_pipeline(&args).await {
                eprintln!("{} {}", "ERROR:".red().bold(), e);
                std::process::exit(1);
            }
        }
        Commands::Scan(args) => {
            let output = scan(args)?;
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
    }
    Ok(())
}
