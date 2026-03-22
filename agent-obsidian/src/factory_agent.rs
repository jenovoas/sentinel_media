// src/factory_agent.rs
//! 🏭 ME-60OS: RESONANT FACTORY AGENT (RUST) 🏭
//! ---------------------------------------------------------------------------
//! Maneja el pipeline de producción de YouTube sincronizado a 41Hz.

use sentinel_memory::{CandleEmbedder, VectorStore};
// use me60os_core::agent_manager::AgentSPA;
// use me60os_core::cortex::CortexEngine;
// use me60os_core::spa::SPA;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

/*
pub struct FactoryAgent {
    name: String,
    queue_file: String,
    system_prompt_dir: String,
    system_prompt: String,   // Default/Cache
    current_persona: String, // Persona actual cargada
    current_load: SPA,
    is_busy: bool,
    // Neural Memory
    memory_store: Option<VectorStore>,
    embedder: Option<CandleEmbedder>,
}

impl FactoryAgent {
    pub fn new(name: &str, queue_file: &str) -> Self {
        println!("🧠 [Factory] Loading Neural Memory (Candle)...");
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let db_path = PathBuf::from(format!("{}/.sentinel_memory.json", home));

        let store = VectorStore::load(&db_path).ok();
        let embedder = CandleEmbedder::new().ok();

        if store.is_some() && embedder.is_some() {
            println!("✅ [Factory] Neural Memory Attached.");
        } else {
            println!("⚠️ [Factory] Memory load failed. Running in Lobotomy Mode.");
        }

        // Directorio de Prompts
        let prompt_dir = std::env::var("FACTORY_PROMPTS_PATH").unwrap_or_else(|_| "core/prompts".to_string());
        let default_prompt_path = format!("{}/youtube_architect.md", prompt_dir);
        let sys_prompt = fs::read_to_string(&default_prompt_path)
            .unwrap_or_else(|_| "You are a helpful AI.".to_string());

        Self {
            name: name.to_string(),
            queue_file: queue_file.to_string(),
            system_prompt_dir: prompt_dir.to_string(),
            system_prompt: sys_prompt,
            current_persona: "youtube_architect".to_string(),
            current_load: SPA::zero(),
            is_busy: false,
            memory_store: store,
            embedder,
        }
    }

    /// Lee la cola de tareas (ready.json)
    fn check_queue(&self) -> bool {
        if let Ok(content) = fs::read_to_string(&self.queue_file) {
            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                if let Some(tasks) = json.as_array() {
                    return !tasks.is_empty();
                }
            }
        }
        false
    }

    fn query_memory(&mut self, topic: &str) -> String {
        if let (Some(store), Some(embedder)) = (&self.memory_store, &mut self.embedder) {
            if let Ok(vec) = embedder.embed(topic) {
                // Buscamos los 5 fragmentos más relevantes para tener buen contexto
                let results = store.search(&vec, 5);
                let context: Vec<String> =
                    results.iter().map(|(doc, _)| doc.content.clone()).collect();
                return context.join("\n\n---\n\n");
            }
        }
        "NO_MEMORY_CONTEXT".to_string()
    }
}

impl AgentSPA for FactoryAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn percibir(&mut self, cortex: &CortexEngine) -> bool {
        // Recargar prompt actual si cambia (Hot Reloading)
        let tick_raw = cortex.time.to_raw();
        if (tick_raw % 120) == 0 {
            // Cada ~3 segs
            let current_path = format!("{}/{}.md", self.system_prompt_dir, self.current_persona);
            if let Ok(new_prompt) = fs::read_to_string(&current_path) {
                if new_prompt != self.system_prompt {
                    self.system_prompt = new_prompt;
                    println!("🎭 [Factory] Persona '{}' Updated!", self.current_persona);
                }
            }
        }

        self.current_load = cortex.total_energy;
        let has_tasks = self.check_queue();
        let high_load = SPA::new(2, 0, 0, 0, 0);

        if self.current_load < high_load {
            // Leer Control State para forzar/cancelar
            let control_path = std::env::var("CORTEX_CONTROL_PATH").unwrap_or_else(|_| "/tmp/cortex_control.json".to_string());
            if let Ok(content) = std::fs::read_to_string(control_path) {
                if let Ok(state) =
                    serde_json::from_str::<crate::control_agent::ControlState>(&content)
                {
                    if !state.factory_queue {
                        return false;
                    }
                }
            }
            self.is_busy = false;
            return has_tasks;
        }
        false
    }

    fn decidir(&mut self) -> String {
        let nominal = SPA::new(1, 0, 0, 0, 0);
        if self.current_load >= nominal {
            "TRIGGER_RESONANT_PRODUCTION".to_string()
        } else {
            "IDLE_WAITING_FOR_RESONANCE".to_string()
        }
    }

    fn actuar(&mut self, action: String) {
        if action == "TRIGGER_RESONANT_PRODUCTION" {
            println!(
                "🏭 [FACTORY:{}] Resonancia detectada. Iniciando producción...",
                self.name
            );

            if let Ok(content) = fs::read_to_string(&self.queue_file) {
                if let Ok(json) = serde_json::from_str::<Value>(&content) {
                    if let Some(tasks) = json.as_array() {
                        if let Some(first_task) = tasks.first() {
                            let topic = first_task["title"]
                                .as_str()
                                .or_else(|| first_task["rel_path"].as_str())
                                .unwrap_or("Untitled");

                            let channel = first_task["channel"]
                                .as_str()
                                .unwrap_or("youtube_architect");

                            // Si el canal cambió, cargamos la nueva personalidad
                            if channel != self.current_persona {
                                let new_path = format!("{}/{}.md", self.system_prompt_dir, channel);
                                if let Ok(new_prompt) = fs::read_to_string(&new_path) {
                                    self.system_prompt = new_prompt;
                                    self.current_persona = channel.to_string();
                                    println!("🎭 [Factory] Switched to Persona: '{}'", channel);
                                }
                            }

                            println!(
                                "🧠 [Factory] Consultando Oráculo ({}) sobre: '{}'",
                                self.current_persona, topic
                            );
                            let memory_context = self.query_memory(topic);

                            println!("📝 [Factory] Ensamblando Super-Prompt para {}...", channel);

                            let final_prompt = format!(
                                "SYSTEM (PERSONA: {}):\n{}\n\nCONTEXTO DE LA BÓVEDA (MEMORIA):\n{}\n\nTAREA ACTUAL:\nGenerar contenido para {} sobre: {}",
                                self.current_persona,
                                self.system_prompt,
                                memory_context,
                                channel,
                                topic
                            );

                            let _ = fs::write("/tmp/last_factory_prompt.txt", &final_prompt);
                            println!("✨ Prompt generado en /tmp/last_factory_prompt.txt. Listo para inferencia.");
                        }
                    }
                }
            }
            self.is_busy = true;
        }
    }
}
*/
