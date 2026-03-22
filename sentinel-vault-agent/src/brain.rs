// src/brain.rs
//! 🧠 Sentinel Vault: AGENT BRAIN & ORCHESTRATOR 🧠
//! ---------------------------------------------------------------------------
//! Implementa el ciclo Plan-and-Solve inspirado en arquitecturas agénticas modernas.

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use sentinel_research::SentinelResearch;
use sentinel_core::FactoryConfig;
use colored::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentStep {
    pub id: u32,
    pub description: String,
    pub status: String, // "PENDING", "RUNNING", "COMPLETED", "FAILED"
    pub result: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentPlan {
    pub goal: String,
    pub steps: Vec<AgentStep>,
    pub final_summary: Option<String>,
}

pub struct SentinelBrain {
    research: SentinelResearch,
    config: FactoryConfig,
}

impl SentinelBrain {
    pub fn new() -> Result<Self> {
        let config = FactoryConfig::load()?;
        let research = SentinelResearch::new()?;
        Ok(Self { research, config })
    }

    /// Genera un plan estructurado basado en un objetivo
    pub async fn plan(&self, goal: &str) -> Result<AgentPlan> {
        println!("🧠 [{}] Diseñando plan para: {}", "BRAIN".blue().bold(), goal.yellow());
        
        let system_prompt = r#"Eres el Arquitecto de Sentinel Media. 
Tu tarea es descomponer un objetivo complejo en UN MÁXIMO de 5 pasos lógicos y secuenciales. 
Cada paso debe ser una acción concreta (ej: 'Analizar código', 'Sintetizar reporte', 'Generar hilo de X').
DEBES responder EXCLUSIVAMENTE en formato JSON válido con esta estructura:
{
  "goal": "objetivo",
  "steps": [
    { "id": 1, "description": "descripción del paso", "status": "PENDING" }
  ]
}"#;

        let response = self.research.synthesize_vertex(&self.config, system_prompt, goal).await?;
        self.parse_plan(response.as_str())
    }

    /// Limpia y parsea un plan desde una respuesta de IA
    fn parse_plan(&self, response: &str) -> Result<AgentPlan> {
        // Limpiar posible markdown del LLM
        let json_str = response.trim_start_matches("```json").trim_end_matches("```").trim();
        
        let plan: AgentPlan = serde_json::from_str(json_str)
            .context("Error al parsear el plan generado por la IA")?;
            
        Ok(plan)
    }

    /// Resuelve un paso individual del plan
    pub async fn solve_step(&self, step: &mut AgentStep, context: &str) -> Result<()> {
        step.status = "RUNNING".to_string();
        println!("⚙️ [{}] Ejecutando paso {}: {}", "CORE".magenta(), step.id, step.description.cyan());

        let system_prompt = "Eres un Agente Ejecutivo de Sentinel Media. Resuelve la tarea descrita basándote en el contexto proporcionado. Sé conciso y técnico.";
        let user_msg = format!("CONTEXTO PREVIO: {}\n\nTAREA ACTUAL: {}", context, step.description);

        let result = self.research.synthesize_vertex(&self.config, system_prompt, &user_msg).await?;
        
        step.result = Some(result);
        step.status = "COMPLETED".to_string();
        
        Ok(())
    }

    /// Ejecuta una inferencia directa sin modificar un paso (para síntesis final)
    pub async fn solve_step_direct(&self, system_prompt: &str, user_msg: &str) -> Result<String> {
        self.research.synthesize_vertex(&self.config, system_prompt, user_msg).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_plan_valid() {
        // En una prueba real necesitaríamos un SentinelBrain mockeado o parcial
        // Para probar la lógica de parse_plan, simularemos la limpieza de JSON
        let raw_json = r#"```json
        {
          "goal": "Test Goal",
          "steps": [
            { "id": 1, "description": "Step 1", "status": "PENDING" }
          ]
        }
        ```"#;
        
        // Limpieza manual similar a parse_plan para validación de estructura
        let json_str = raw_json.trim_start_matches("```json").trim_end_matches("```").trim();
        let plan: AgentPlan = serde_json::from_str(json_str).unwrap();
        
        assert_eq!(plan.goal, "Test Goal");
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].description, "Step 1");
    }

    #[test]
    fn test_agent_step_serialization() {
        let step = AgentStep {
            id: 1,
            description: "Test".to_string(),
            status: "PENDING".to_string(),
            result: None,
        };
        let json = serde_json::to_string(&step).unwrap();
        assert!(json.contains("\"status\":\"PENDING\""));
    }
}
