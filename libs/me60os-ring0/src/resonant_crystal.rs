// ⚡ AXIOMA I — ZERO DECIMAL CONTAMINATION ⚡
// Este módulo es física pura S60. f64/f32 están PROHIBIDOS.
// El compilador rechazará cualquier aritmética flotante.
#![deny(clippy::float_arithmetic)]

use crate::spa::SPA;
use crate::spa_math::SPAMath;


/// Cristal Soberano: Celda de Memoria Resonante
pub struct SovereignCrystal {
    pub name: String,
    pub natural_frequency: SPA, // Ratio de resonancia
    pub amplitude: SPA,         // Energía almacenada
    pub phase: SPA,             // Fase actual
    pub damping_factor: SPA,    // Q-Factor (pérdida por ciclo)
}

impl SovereignCrystal {
    pub fn new(name: &str, resonance_ratio: SPA) -> Self {
        // Default damping: S60(0, 0, 30) -> 0.5 sec decay approx
        let damping = SPA::new(0, 0, 30, 0, 0); 
        
        SovereignCrystal {
            name: name.to_string(),
            natural_frequency: resonance_ratio,
            amplitude: SPA::zero(),
            phase: SPA::zero(),
            damping_factor: damping,
        }
    }

    /// Inyecta energía (presión de datos)
    pub fn transduce_pulse(&mut self, pressure: i64) {
        let input_force = SPA::from_raw(pressure * SPA::SCALE_0);
        self.amplitude = self.amplitude + input_force;
    }

    /// Aplica entropía termodinámica
    fn apply_entropy(&mut self, dt: SPA) -> SPA {
        // Decay = A * lambda * dt
        // Multiplicación S60*S60 requiere cuidado de escala. 
        // Asumimos que SPA::mul maneja la escala (a*b)/SCALE.
        
        let decay = (self.amplitude * self.damping_factor) * dt;
        self.amplitude = self.amplitude - decay;

        // Ground state check
        if self.amplitude.to_raw() < SPA::new(0, 0, 1, 0, 0).to_raw() {
            self.amplitude = SPA::zero();
        }
        
        decay
    }

    /// Avanza un paso de simulación
    pub fn oscillate(&mut self, dt: SPA) -> SPA {
        // 1. Avanzar Fase
        let delta_phase = self.natural_frequency * dt;
        self.phase = self.phase + delta_phase;

        // 2. Calcular Señal (Sinusoidal) — Quantum Hack #2: Taylor Series Integer-Only
        let signal_wave = SPAMath::sin(self.phase);

        let output = self.amplitude * signal_wave;

        // 3. Entropía
        self.apply_entropy(dt);

        output
    }
}
