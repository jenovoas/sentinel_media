//! Núcleo Matemático S60 (Sexagesimal) - Piloto UNISON
//! Implementación de aritmética de punto fijo para alta precisión en ME-60OS.

use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub};

/// Factor de escala sexagesimal para precisión 60^4 (precisión tipo GPS/BCI)
const S60_SCALE: u128 = 60 * 60 * 60 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Sexagesimal(pub u128);

impl Sexagesimal {
    /// Crear desde un entero (escalado automáticamente)
    pub const fn from_int(val: u64) -> Self {
        Sexagesimal(val as u128 * S60_SCALE)
    }

    /// Obtener valor como float (Solo para telemetría visual, no para cálculos core)
    pub fn to_f64(&self) -> f64 {
        self.0 as f64 / S60_SCALE as f64
    }

    /// Suma segura con detección de overflow
    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Sexagesimal)
    }

    /// Multiplicación segura ajustando la escala sexagesimal
    pub fn checked_mul(self, other: Self) -> Option<Self> {
        // (a * scale) * (b * scale) / scale = (a * b) * scale
        let res = self.0.checked_mul(other.0)?;
        Some(Sexagesimal(res / S60_SCALE))
    }
}

// Implementación de Traits para ergonomía
impl Add for Sexagesimal {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Sexagesimal(self.0 + other.0)
    }
}

impl Sub for Sexagesimal {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Sexagesimal(self.0 - other.0)
    }
}

// --- LOGICA DE PRECISIÓN ---
// "Si no es exacto en Base-60, no es la verdad."

impl std::str::FromStr for Sexagesimal {
    type Err = String;

    /// Parsea formato Plimpton: "1; 59, 0, 15" -> Sexagesimal
    /// 1 * 60^0 + 59 * 60^-1 + 0 * 60^-2 + 15 * 60^-3 ... (ajustado a SCALE 60^4)
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(';').collect();
        if parts.is_empty() {
            return Err("Formato inválido".into());
        }

        let mut total: u128 = 0;

        // Parte entera
        let integer: u128 = parts[0].trim().parse().map_err(|_| "Entero inválido")?;
        total += integer * S60_SCALE;

        if parts.len() > 1 {
            let decimals: Vec<&str> = parts[1].split(',').collect();
            for (i, d) in decimals.iter().enumerate() {
                let val: u128 = d.trim().parse().map_err(|_| "Decimal inválido")?;
                let power = 3 - i as i32; // Ajuste para escala 60^4
                if power >= 0 {
                    total += val * 60u128.pow(power as u32);
                }
            }
        }

        Ok(Sexagesimal(total))
    }
}
