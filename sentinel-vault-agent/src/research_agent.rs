// src/research_agent.rs
//! 🛡️ Sentinel Vault: NATIVE RESEARCH AGENT 🛡️
//! ---------------------------------------------------------------------------
//! Maneja el ciclo de investigación profunda (RAG) y síntesis multicanal.

use anyhow::Result;
use colored::Colorize;
use sentinel_research::Args as ResearchArgs;
use sentinel_core::FactoryConfig;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResearchTask {
    pub topic: String,
    pub file_path: Option<PathBuf>,
    pub target: String, // "youtube", "x-thread", "linkedin", "marketing-pack"
}

pub struct ResearchAgent {
    pub name: String,
    pub vault_path: PathBuf,
    pub config: FactoryConfig,
}

impl ResearchAgent {
    pub fn new(name: &str, vault_path: &str) -> Self {
        let config = FactoryConfig::load().unwrap_or_default();
        Self {
            name: name.to_string(),
            vault_path: PathBuf::from(vault_path),
            config,
        }
    }

    /// Ejecuta una investigación profunda para un objetivo específico
    pub async fn solve_task(&self, task: &ResearchTask) -> Result<String> {
        println!("🔮 [{}] Investigando tópico: '{}' para el target: {}", 
            self.name.cyan().bold(), task.topic.yellow(), task.target.magenta());

        
        // Preparar argumentos para el motor de investigación
        let args = ResearchArgs {
            file: task.file_path.as_ref().map(|p| p.to_string_lossy().to_string()),
            prompt: Some(task.topic.clone()),
            imagina: true,
            intuicion: true,
            deep: true,
            refactor: false,
            translate: false,
            target_lang: "es".to_string(),
            interactive: false,
            system: true,
            memory_tier: "warm".to_string(),
            telos_context: true,
            groq: false,
            openai: false,
            antigravity: true,
            perplexity: false,
            target: task.target.clone(),
            hook: vec![],
        };

        // En la arquitectura Fenix v2, delegamos la síntesis al crate research
        let synthesis = sentinel_research::run(args).await?;
        
        Ok(synthesis)
    }
}
