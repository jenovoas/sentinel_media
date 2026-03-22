//! # TruthSync — Guardián de Coherencia del Agente
//!
//! Expone los tipos que espeja el mapa BPF `truth_sync_map` de guardian_alpha.
//! El estado real vive en Ring 0; este módulo es la interfaz userspace.

use crate::spa::SPA;

/// Ratio de Plimpton 322 escalado en campo SPA.
/// Representa la razón matemática sexagesimal usada para detectar alucinaciones.
#[derive(Debug, Clone, Copy)]
pub struct PlimptonRatio(pub SPA);

impl PlimptonRatio {
    pub fn new(raw: i64) -> Self {
        Self(SPA::from_raw(raw))
    }
}

/// Guarda el estado TruthSync de un agente activo.
/// `status == true` → UNISON (coherente). `false` → DISSONANCE (alucinando).
#[derive(Debug, Clone)]
pub struct TruthSyncGuard {
    pub agent_id: u32,
    pub last_ratio: PlimptonRatio,
    pub status: bool,
}

impl TruthSyncGuard {
    pub fn new(agent_id: u32) -> Self {
        Self {
            agent_id,
            last_ratio: PlimptonRatio::new(0),
            status: false,
        }
    }

    pub fn is_unison(&self) -> bool {
        self.status
    }
}
