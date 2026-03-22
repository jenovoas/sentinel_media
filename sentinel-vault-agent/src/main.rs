// src/main.rs
use clap::{Parser, Subcommand};
use colored::*;
use sentinel_vault_agent::{FactoryAgent, CloudOpsAgent, SolarAgent};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Parser)]
#[command(name = "sentinel-vault-agent")]
#[command(about = "Orquestador del Enjambre de Sentinel Media", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 🔮 Inicia el ciclo del enjambre (Modo Demonio)
    Daemon {
        /// Ruta a la cola de tareas
        #[arg(short, long, default_value = "core/ready.json")]
        queue: String,
        /// Ruta a la bóveda
        #[arg(short, long, default_value = "vault")]
        vault: String,
        /// Ruta al almacén de operaciones en la nube
        #[arg(long, default_value = "core/operations.json")]
        ops_store: String,
    },
    /// 🛡️ Certificar nota (Pendiente de refactor)
    Certify {
        /// Archivo a certificar
        #[arg(short, long)]
        file: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Daemon { queue, vault, ops_store } => {
            println!("{}", "🔮 INICIANDO ENJAMBRE DE AGENTES (FENIX V2) 🔮".cyan().bold());
            
            let mut factory = FactoryAgent::new("Factory-Alpha", &queue, &vault);
            let mut cloud = CloudOpsAgent::new("Cloud-Watcher", &ops_store);
            let mut solar = SolarAgent::new("Helios-Scanner", "DEMO_KEY");
            
            println!("📡 Monitoreando cola: {}", queue.yellow());
            println!("📚 Bóveda activa en: {}", vault.yellow());
            println!("☁️ Ops Store: {}", ops_store.yellow());
            println!("🚀 Ciclo operativo del Enjambre iniciado...");

            loop {
                // Tictac de los agentes
                if let Err(e) = factory.tick().await {
                    eprintln!("⚠️ Error en FactoryAgent: {}", e);
                }
                
                if let Err(e) = cloud.tick().await {
                    eprintln!("⚠️ Error en CloudOpsAgent: {}", e);
                }
                
                if let Err(e) = solar.tick().await {
                    eprintln!("⚠️ Error en SolarAgent: {}", e);
                }

                sleep(Duration::from_secs(10)).await;
            }
        }
        Commands::Certify { file: _ } => {
            println!("Comando Certify no implementado en esta versión.");
        }
    }

    Ok(())
}
