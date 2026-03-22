// src/research_agent.rs
//! 🛡️ Sentinel Vault: NATIVE RESEARCH AGENT 🛡️
//! ---------------------------------------------------------------------------
//! Maneja el ciclo de investigación profunda (RAG) mediante el bucle Plan-and-Solve.

use crate::brain::SentinelBrain;
use anyhow::Result;
use colored::Colorize;
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
    pub brain: SentinelBrain,
}

impl ResearchAgent {
    pub fn new(name: &str, vault_path: &str) -> Self {
        let brain = SentinelBrain::new().expect("No se pudo inicializar el cerebro del agente");
        Self {
            name: name.to_string(),
            vault_path: PathBuf::from(vault_path),
            brain,
        }
    }

    /// Ejecuta una investigación profunda usando el bucle Plan-and-Solve
    pub async fn solve_task(&self, task: &ResearchTask) -> Result<String> {
        println!("🔮 [{}] Iniciando Protocolo de Investigación: '{}'", 
            self.name.cyan().bold(), task.topic.yellow());

        // 1. Planificación
        let mut plan = self.brain.plan(&task.topic).await?;
        println!("📋 [{}] Plan generado con {} pasos lógicos.", self.name.cyan(), plan.steps.len());

        let mut cumulative_context = String::new();

        // 2. Ejecución Secuencial (Plan-and-Solve)
        for step in plan.steps.iter_mut() {
            self.brain.solve_step(step, &cumulative_context).await?;
            if let Some(res) = &step.result {
                cumulative_context.push_str(&format!("\n### Paso {}: {}\n{}\n", step.id, step.description, res));
            }
        }

        // 3. Síntesis Final
        println!("🧬 [{}] Sintetizando reporte final para el target: {}", self.name.cyan(), task.target.magenta());
        
        let synthesis_prompt = format!("Eres Sentinel Media - Assistant. Basándote en la siguiente investigación, genera un reporte final optimizado para: {}. El reporte debe ser técnico, profundo y profesional.", task.target);
        
        // Usamos la lógica de investigación para el formateo final
        let final_report = self.brain.solve_step_direct(&synthesis_prompt, &cumulative_context).await?;
        
        Ok(final_report)
    }
}
