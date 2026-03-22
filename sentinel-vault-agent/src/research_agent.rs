/*
// src/research_agent.rs
//! 🛡️ ME-60OS: NATIVE RESEARCH AGENT 🛡️
//! ---------------------------------------------------------------------------
//! Portabilidad nativa de la lógica oracular (Imagina + Intuición).

use ignore::WalkBuilder;
use me60os_core::agent_manager::AgentSPA;
use me60os_core::cortex::CortexEngine;
use me60os_core::spa::SPA;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct ResearchContext {
    pub internal_notes: Vec<String>,
    pub cortex_resonance: SPA,
}

pub struct ResearchAgent {
    pub name: String,
    pub vault_path: PathBuf,
    pub last_context: Option<ResearchContext>,
    pub imagina_mode: bool,
    pub intuicion_mode: bool,
}

impl ResearchAgent {
    pub fn new(name: &str, vault_path: &str) -> Self {
        Self {
            name: name.to_string(),
            vault_path: PathBuf::from(vault_path),
            last_context: None,
            imagina_mode: true,
            intuicion_mode: true,
        }
    }

    /// Implementación de INTUICIÓN: Búsqueda ultra-rápida en la bóveda
    fn search_vault(&self, terms: Vec<&str>) -> Vec<String> {
        let mut results = Vec::new();
        let walker = WalkBuilder::new(&self.vault_path)
            .hidden(false)
            .git_ignore(true)
            .build();

        for entry in walker.filter_map(Result::ok) {
            if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                if entry.path().extension().map(|e| e == "md").unwrap_or(false) {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        let content_lower = content.to_lowercase();
                        for term in &terms {
                            if content_lower.contains(&term.to_lowercase()) {
                                results.push(entry.path().display().to_string());
                                break;
                            }
                        }
                    }
                }
            }
            if results.len() > 10 {
                break;
            } // Límite de resonancia
        }
        results
    }
}

impl AgentSPA for ResearchAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn percibir(&mut self, cortex: &CortexEngine) -> bool {
        let resonance = cortex.total_energy;
        let internal_notes = self.search_vault(vec!["SPA", "Sentinel"]);

        self.last_context = Some(ResearchContext {
            internal_notes,
            cortex_resonance: resonance,
        });

        true
    }

    fn decidir(&mut self) -> String {
        if let Some(ctx) = &self.last_context {
            if ctx.internal_notes.is_empty() {
                "EXPAND_RESEARCH".to_string()
            } else {
                "S60_SYNTHESIS_REQUIRED".to_string()
            }
        } else {
            "NO_OP".to_string()
        }
    }

    fn actuar(&mut self, action: String) {
        println!(
            "🔮 [ORACLE:{}] Decision: {} | Resonance: {}",
            self.name,
            action,
            self.last_context
                .as_ref()
                .map(|c| c.cortex_resonance.to_string())
                .unwrap_or_default()
        );

        if self.imagina_mode {
            let res_raw = self
                .last_context
                .as_ref()
                .map(|c| c.cortex_resonance.to_raw())
                .unwrap_or(0);

            let analogy = if res_raw > SPA::SCALE_0 {
                "Súper-posición Cuántica: La idea colapsa en múltiples realidades simultáneas."
            } else if res_raw > (SPA::SCALE_0 / 2) {
                "Lattice Harmónica: Los datos vibran en una red perfecta de cristal."
            } else {
                "Sopa Primordial: Entropía creativa en busca de orden."
            };

            println!("   🎨 MODO IMAGINA: {}", analogy);
        }
    }
}
*/
