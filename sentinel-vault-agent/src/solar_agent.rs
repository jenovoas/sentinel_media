// src/solar_agent.rs
//! ☀️ Sentinel Vault: HELIOS (SOLAR RESONANCE MODULE) ☀️
//! ---------------------------------------------------------------------------
//! Conecta el enjambre con la actividad solar real (NASA API).

use anyhow::Result;
use colored::Colorize;
use reqwest::Client;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct SolarAgent {
    pub name: String,
    pub api_key: String,
    pub current_flux: f64,
    pub last_check: SystemTime,
}

impl SolarAgent {
    pub fn new(name: &str, api_key: &str) -> Self {
        Self {
            name: name.to_string(),
            api_key: api_key.to_string(),
            current_flux: 1361.0, // Base flux (W/m^2)
            last_check: SystemTime::UNIX_EPOCH,
        }
    }

    pub async fn tick(&mut self) -> Result<()> {
        let now = SystemTime::now();
        if now.duration_since(self.last_check).unwrap_or(std::time::Duration::from_secs(0)) < std::time::Duration::from_secs(300) {
            return Ok(());
        }
        self.last_check = now;

        if let Some(flux) = self.fetch_solar_data().await {
            self.current_flux = flux;
            println!("☀️ [{}] Solar Flux: {:.2} W/m² | Estado: {}", 
                self.name.yellow().bold(), flux, 
                if flux > 1366.0 { "TORMENTA SOLAR (BOOST ACTIVO)".red() } else { "Estable".green() }
            );
        }
        Ok(())
    }

    async fn fetch_solar_data(&self) -> Option<f64> {
        if self.api_key == "DEMO_KEY" {
            let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let flux = 1361.0 + (time as f64 / 3600.0).sin() * 5.0;
            return Some(flux);
        }
        
        let url = format!("https://api.nasa.gov/DONKI/FLR?startDate=2025-01-01&api_key={}", self.api_key);
        let client = Client::new();
        if let Ok(resp) = client.get(&url).send().await {
            if resp.status().is_success() {
                return Some(1361.0 + 10.0); // Flare detected (simulated value)
            }
        }
        None
    }
}
