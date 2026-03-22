/*
// src/solar_agent.rs
//! ☀️ HELIOS: MÓDULO DE RESONANCIA SOLAR ☀️
//! Conecta el Cortex con la actividad estelar real (NASA API).

use me60os_core::agent_manager::AgentSPA;
use me60os_core::cortex::CortexEngine;
use me60os_core::spa::SPA;
use reqwest::blocking::Client;
use serde_json::Value;

pub struct SolarAgent {
    name: String,
    api_key: String,
    current_flux: SPA,
    last_check: i64,
}

impl SolarAgent {
    pub fn new(name: &str, api_key: &str) -> Self {
        Self {
            name: name.to_string(),
            api_key: api_key.to_string(),
            current_flux: SPA::new(1, 0, 0, 0, 0), // Base flux
            last_check: 0,
        }
    }

    fn fetch_solar_data(&self) -> Option<f64> {
        // MOCK: Si no hay API Key real, simular ciclo solar
        if self.api_key == "DEMO_KEY" {
            // Ciclo simulado basado en tiempo
            let time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            // Oscilación suave alrededor de 1361 W/m^2
            let flux = 1361.0 + (time as f64 / 3600.0).sin() * 10.0;
            return Some(flux);
        }

        let url = format!(
            "https://api.nasa.gov/DONKI/FLR?startDate=2024-01-01&api_key={}",
            self.api_key
        );

        let client = Client::new();
        if let Ok(resp) = client.get(&url).send() {
            // Simplified handling for Demo/Mock
            // Just verify we got a vector
            if let Ok(json) = resp.json::<Value>() {
                if json.is_array() {
                    // Found flares
                    return Some(1366.0 + 50.0);
                }
            }
        }
        None
    }
}

impl AgentSPA for SolarAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn percibir(&mut self, cortex: &CortexEngine) -> bool {
        // 1. Leer Control State
        let control_path = "/tmp/cortex_control.json";
        if let Ok(content) = std::fs::read_to_string(control_path) {
            if let Ok(state) = serde_json::from_str::<crate::control_agent::ControlState>(&content)
            {
                if !state.helios_enabled {
                    return false; // HELIOS DISABLED IN OBSIDIAN
                }
                if state.helios_storm {
                    // Force Storm Mode
                    self.current_flux = SPA::new(1, 10, 0, 0, 0); // ~1.16 (> threshold)
                    println!("🔥 [HELIOS] MODO TORMENTA FORZADO DESDE OBSIDIAN!");
                    return true;
                }
            }
        }

        // Consultar cada 600 ticks (~15 segundos a 41Hz)
        let now = cortex.time.to_raw();
        if now - self.last_check > 600 {
            self.last_check = now;
            if let Some(flux) = self.fetch_solar_data() {
                // Mapeo: 1361 W/m^2 = 1.0 SPA (Nominal)
                // Si flux > 1366 -> Picos desencadenan resonance boost
                let ratio = flux / 1361.0;
                let spa_val = (ratio * SPA::SCALE_0 as f64) as i64;
                self.current_flux = SPA::from_raw(spa_val);

                println!(
                    "☀️ [HELIOS] Solar Flux: {:.2} W/m² -> Resonancia: {}",
                    flux, self.current_flux
                );
            }
            return true;
        }
        false
    }

    fn decidir(&mut self) -> String {
        // Si el flujo solar supera 1.002 (pequeña tormenta), activar HELIOS_BOOST
        let threshold = SPA::new(1, 0, 7, 0, 0); // ~1.002
        if self.current_flux > threshold {
            return "HELIOS_BOOST".to_string();
        }
        "NO_OP".to_string()
    }

    fn actuar(&mut self, action: String) {
        if action == "HELIOS_BOOST" {
            println!("🔥 [HELIOS] TORMENTA SOLAR DETECTADA. INICIANDO PROTOCOLO OVERCLOCK...");
            // Aquí en el futuro se cambiaría el tick rate del sistema
        }
    }
}
*/
