// src/factory_agent.rs
//! 🏭 Sentinel Vault: FACTORY AGENT (RUST) 🏭
//! ---------------------------------------------------------------------------
//! Orquestador del pipeline de producción y marketing.

use anyhow::Result;
use colored::Colorize;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use crate::research_agent::{ResearchAgent, ResearchTask};

pub struct FactoryAgent {
    pub name: String,
    pub queue_file: String,
    pub is_busy: bool,
    pub research_agent: ResearchAgent,
}

impl FactoryAgent {
    pub fn new(name: &str, queue_file: &str, vault_path: &str) -> Self {
        Self {
            name: name.to_string(),
            queue_file: queue_file.to_string(),
            is_busy: false,
            research_agent: ResearchAgent::new(&format!("{}-Research", name), vault_path),
        }
    }

    /// Revisa la cola de tareas (ready.json)
    pub async fn tick(&mut self) -> Result<()> {
        if self.is_busy {
            return Ok(());
        }

        if let Ok(content) = fs::read_to_string(&self.queue_file) {
            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                if let Some(tasks) = json.as_array() {
                    if !tasks.is_empty() {
                        println!("🏭 [{}] Tareas detectadas en la cola. Iniciando Enjambre...", self.name.green().bold());
                        self.process_next_task(tasks).await?;
                    }
                }
            }
        }
        Ok(())
    }

    async fn process_next_task(&mut self, tasks: &[Value]) -> Result<()> {
        if let Some(first_task) = tasks.first() {
            let title = first_task["title"].as_str().unwrap_or("Untitled");
            let file_path = first_task["rel_path"].as_str().map(PathBuf::from);

            println!("🚀 [Factory] Procesando: {}", title.yellow());

            // 1. Fase de Guion (YouTube)
            let yt_task = ResearchTask {
                topic: title.to_string(),
                file_path: file_path.clone(),
                target: "youtube".to_string(),
            };
            let script = self.research_agent.solve_task(&yt_task).await?;
            let _ = fs::write("/tmp/last_video_script.md", &script);
            println!("✅ [Factory] Guion de YouTube generado.");

            // 2. Fase de Marketing (Community Manager Poderes)
            println!("📣 [Factory] Iniciando Campaña de Marketing Digital...");
            
            // X-Thread
            let x_task = ResearchTask {
                topic: format!("Crea un hilo viral sobre: {}", title),
                file_path: file_path.clone(),
                target: "x-thread".to_string(),
            };
            let x_thread = self.research_agent.solve_task(&x_task).await?;
            let _ = fs::write("/tmp/last_x_thread.md", &x_thread);
            println!("🐦 [Factory] Hilo de X (Twitter) preparado.");

            // LinkedIn
            let li_task = ResearchTask {
                topic: format!("Escribe un post profesional para líderes técnicos sobre: {}", title),
                file_path: file_path.clone(),
                target: "linkedin".to_string(),
            };
            let li_post = self.research_agent.solve_task(&li_task).await?;
            let _ = fs::write("/tmp/last_linkedin_post.md", &li_post);
            println!("🏢 [Factory] Post de LinkedIn preparado.");

            // Facebook
            let fb_task = ResearchTask {
                topic: format!("Escribe un post inspirador para Facebook sobre: {}", title),
                file_path: file_path.clone(),
                target: "facebook_visionary".to_string(),
            };
            let fb_post = self.research_agent.solve_task(&fb_task).await?;
            let _ = fs::write("/tmp/last_facebook_post.md", &fb_post);
            println!("👥 [Factory] Post de Facebook preparado.");

            // Instagram
            let ig_task = ResearchTask {
                topic: format!("Crea una estructura de carrusel/reel para Instagram sobre: {}", title),
                file_path: file_path.clone(),
                target: "instagram_storyteller".to_string(),
            };
            let ig_post = self.research_agent.solve_task(&ig_task).await?;
            let _ = fs::write("/tmp/last_instagram_post.md", &ig_post);
            println!("📸 [Factory] Contenido de Instagram preparado.");

            // TikTok
            let tk_task = ResearchTask {
                topic: format!("Escribe un guion rápido para TikTok sobre: {}", title),
                file_path: file_path.clone(),
                target: "tiktok_trendsetter".to_string(),
            };
            let tk_script = self.research_agent.solve_task(&tk_task).await?;
            let _ = fs::write("/tmp/last_tiktok_script.md", &tk_script);
            println!("📱 [Factory] Guion de TikTok preparado.");

            println!("{}", "✨ ¡Campaña Multicanal Generada con Éxito! ✨".green().bold());
        }
        Ok(())
    }
}
