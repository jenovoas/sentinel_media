//! # 📅 QUANTUM SCHEDULER: RUST CORE 📅
//!
//! Orchestrates system pulse, batch sizing, and bio-resonance alignment.
//! Implements Axiom V: "Bio-Centrism" & "The 17-Second Discovery".
//!
//! Protocols:
//! - P-Controller (Latency -> Batch Size)
//! - Dead Man's Switch (SoulVerifier)
//! - Venus Drift Correction (Phase Reset at T=68s)

use crate::bio::SoulVerifier;
use crate::spa::SPA;
use crate::spa_math::SPAMath;
use std::cmp;
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub enum SchedulerAction {
    Continue { batch_size: usize },
    Halt { reason: String },
    Emergency { phase: SPA },
}

pub struct QuantumScheduler {
    pub current_batch_size: usize,
    pub min_batch: usize,
    pub max_batch: usize,
    pub baseline_ms: SPA,

    // Bio-Resonance State
    pub ticks_since_reset: u64,
    pub venus_phase_error: SPA,
}

impl QuantumScheduler {
    pub fn new() -> Self {
        Self {
            current_batch_size: 1000,
            min_batch: 100,
            max_batch: 65536,
            baseline_ms: SPA::new(20, 0, 0, 0, 0), // 20ms

            ticks_since_reset: 0,
            venus_phase_error: SPA::zero(),
        }
    }

    /// Primary Tick Logic (41Hz)
    pub fn tick(&mut self, latency_ms: SPA, bio_signal: &[SPA]) -> SchedulerAction {
        // 1. Bio-Resonance Check (Dead Man's Switch)
        // Only run check if we enough signal data
        if bio_signal.len() >= 3 {
            let metrics = SoulVerifier::analyze(bio_signal);
            if !metrics.is_alive {
                return SchedulerAction::Halt {
                    reason: "PILOT_LOST: Bio-Resonance coherence failed".to_string(),
                };
            }
        }

        // 2. Venus Drift Correction (Axiom V)
        // Reset phase every 68s (approx 2788 ticks at 41Hz) -> logic says T=68s.
        // 41Hz * 68s = 2788 ticks.
        self.ticks_since_reset += 1;
        if self.ticks_since_reset >= 2788 {
            self.ticks_since_reset = 0;
            // Force phase reset
            return SchedulerAction::Emergency { phase: SPA::zero() };
        }

        // 3. Adaptive Batch Sizing (P-Controller)
        let epsilon = SPA::new(0, 6, 0, 0, 0); // 0.1ms
        let safe_latency = latency_ms + epsilon;

        // scale = baseline / latency
        let scale_factor = self.baseline_ms / safe_latency;

        // Clamp scale [0.5, 1.5]
        let lower = SPA::new(0, 30, 0, 0, 0); // 0.5
        let upper = SPA::new(1, 30, 0, 0, 0); // 1.5

        let clamped_scale = if scale_factor < lower {
            lower
        } else if scale_factor > upper {
            upper
        } else {
            scale_factor
        };

        // New batch
        // current * scale
        let current_spa = SPA::from_raw(self.current_batch_size as i64 * SPA::SCALE_0);
        let new_batch_spa = current_spa * clamped_scale;

        // Convert back to usize
        let new_batch = new_batch_spa.to_degrees() as usize;

        self.current_batch_size = cmp::max(self.min_batch, cmp::min(self.max_batch, new_batch));

        SchedulerAction::Continue {
            batch_size: self.current_batch_size,
        }
    }
}

// =============================================================================
// EXP-029 V2: PORTAL DETECTOR + ADIABTIC TASK QUEUE
// =============================================================================

/// Tipos de tarea del sistema (EXP-029 §4.1)
#[derive(Debug, Clone, PartialEq)]
pub enum TaskType {
    ZpeTune,    // Re-sintonización del reactor
    BciSync,    // Sincronización bio-máquina
    LatticeGc,  // Garbage collection de memoria líquida
    BackupS60,  // Snapshot de estado S60
    PhaseAlign, // Re-calibración de fase
}

/// Tarea con costo energético intrínseco E₀.
#[derive(Debug, Clone)]
pub struct QuantumTask {
    pub task_type: TaskType,
    /// Energía intrínseca E₀ (en SPA). Costo en portal = E₀; fuera = 3×E₀.
    pub cost: SPA,
}

impl QuantumTask {
    pub fn new(task_type: TaskType, cost_raw: i64) -> Self {
        Self { task_type, cost: SPA::from_raw(cost_raw * SPA::SCALE_0) }
    }
}

/// Detecta portales de convergencia armónica (EXP-029 §2.2).
///
/// φ(t) = ⅓ × [sin(2π×t/T_Bio) + sin(2π×t/T_Crys) + sin(2π×t/T_Venus)]
///
/// Donde:
/// - T_Bio   = 17.0s  (pulso humano)
/// - T_Crys  = 4.25s  (YHWH cycle = 17/4)
/// - T_Venus = 16.18s (ratio Phi 13:8)
pub struct PortalDetector;

impl PortalDetector {
    const T_BIO_NS: i64   = 17_000_000_000;  // 17.0s
    const T_CRYS_NS: i64  =  4_250_000_000;  // 4.25s
    const T_VENUS_NS: i64 = 16_180_000_000;  // 16.18s

    // Umbrales en SPA raw  (SCALE_0 = 12_960_000 = 1.0)
    const PORTAL_OPEN: i64   =  9_720_000;  // 0.75 — portal abierto (V1 ajustado a margen)
    const PORTAL_MEDIUM: i64 = 11_016_000;  // 0.85 — portal medio  (V2 adaptive batch)
    const PORTAL_STRONG: i64 = 11_664_000;  // 0.90 — portal fuerte (V2 adaptive batch)

    /// Calcula resonancia φ(t) dado tiempo en nanosegundos.
    /// Retorna SPA en [-1.0, +1.0].
    pub fn resonance(t_ns: i64) -> SPA {
        let sin_bio   = SPAMath::sin(Self::angle_deg(t_ns, Self::T_BIO_NS));
        let sin_crys  = SPAMath::sin(Self::angle_deg(t_ns, Self::T_CRYS_NS));
        let sin_venus = SPAMath::sin(Self::angle_deg(t_ns, Self::T_VENUS_NS));
        (sin_bio + sin_crys + sin_venus) / 3i64
    }

    /// Portal abierto si φ > 0.75.
    pub fn is_open(resonance: SPA) -> bool {
        resonance.to_raw() > Self::PORTAL_OPEN
    }

    /// Batch size adaptativo según intensidad del portal (V2).
    ///
    /// | φ        | batch |
    /// |----------|-------|
    /// | > 0.90   |   5   |
    /// | > 0.85   |   4   |
    /// | > 0.75   |   2   |
    pub fn batch_size(resonance: SPA) -> usize {
        let r = resonance.to_raw();
        if r > Self::PORTAL_STRONG      { 5 }
        else if r > Self::PORTAL_MEDIUM { 4 }
        else                            { 2 }
    }

    /// Ángulo en SPA grados para un período dado.
    /// angle = 360° × (t mod T) / T
    /// Usa i128 para evitar overflow: 360 × SCALE_0 × T_max ≈ 7.5×10^22 > i64::MAX.
    fn angle_deg(t_ns: i64, period_ns: i64) -> SPA {
        let t_mod = t_ns.rem_euclid(period_ns) as i128;
        let raw = (360i128 * SPA::SCALE_0 as i128 * t_mod) / period_ns as i128;
        SPA::from_raw(raw as i64)
    }
}

/// Cola de tareas adiabática — EXP-029 V2.
///
/// Las tareas se ejecutan SOLO en portales (φ > 0.75).
/// Fuera de portal se acumulan. Si la cola desborda el límite, se fuerza
/// ejecución con penalización energética.
///
/// Optimizaciones V2:
/// - `overflow_limit = 20` (V1 era 10)
/// - Batch size adaptativo según intensidad del portal
/// - Pre-flush a T=60s si cola > 12 (antes del valle T=60-68s)
pub struct AdiabticTaskQueue {
    queue: VecDeque<QuantumTask>,
    /// Límite antes de forzar ejecución (V2: 20).
    pub overflow_limit: usize,
    /// Umbral para pre-flush antes del valle T=60-68s (V2: 12).
    pub pre_flush_threshold: usize,
    /// Duración del ciclo Quantum Leap en ns (68s).
    pub cycle_ns: i64,

    // Métricas acumuladas
    pub tasks_in_portal: u64,
    pub tasks_forced: u64,
    pub energy_saved: SPA,
}

impl AdiabticTaskQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            overflow_limit: 20,
            pre_flush_threshold: 12,
            cycle_ns: 68_000_000_000,
            tasks_in_portal: 0,
            tasks_forced: 0,
            energy_saved: SPA::zero(),
        }
    }

    pub fn push(&mut self, task: QuantumTask) {
        self.queue.push_back(task);
    }

    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    /// Procesa un tick. Retorna las tareas a ejecutar ahora (puede ser vacío).
    ///
    /// `t_ns` — tiempo absoluto en nanosegundos desde inicio del ciclo.
    pub fn tick(&mut self, t_ns: i64) -> Vec<QuantumTask> {
        if self.queue.is_empty() {
            return vec![];
        }

        let resonance = PortalDetector::resonance(t_ns);
        let t_in_cycle = t_ns.rem_euclid(self.cycle_ns);

        // V2: Pre-flush estratégico antes del valle T=60-68s
        // Si estamos cerca del Quantum Leap y la cola está cargada, ejecutar
        // un batch pequeño ahora aunque no sea portal completo.
        if t_in_cycle > 60_000_000_000 && self.queue.len() > self.pre_flush_threshold {
            let tasks = self.drain(2);
            self.tasks_forced += tasks.len() as u64;
            return tasks;
        }

        if PortalDetector::is_open(resonance) {
            // Portal abierto: ejecutar batch adaptativo
            let batch_size = PortalDetector::batch_size(resonance);
            let tasks = self.drain(batch_size);
            for task in &tasks {
                // Ahorro = 2×E₀ (evitamos el 3×E₀ del modo resistivo)
                self.energy_saved = self.energy_saved + task.cost * 2i64;
            }
            self.tasks_in_portal += tasks.len() as u64;
            tasks
        } else if self.queue.len() > self.overflow_limit {
            // Overflow: forzar ejecución con penalización (3×E₀ sin ahorro)
            let tasks = self.drain(1);
            self.tasks_forced += tasks.len() as u64;
            tasks
        } else {
            // Esperar el próximo portal
            vec![]
        }
    }

    /// Eficiencia portal-lock: N_portal / (N_portal + N_forced) × 100%.
    pub fn efficiency_pct(&self) -> u64 {
        let total = self.tasks_in_portal + self.tasks_forced;
        if total == 0 { return 0; }
        (self.tasks_in_portal * 100) / total
    }

    fn drain(&mut self, n: usize) -> Vec<QuantumTask> {
        let take = n.min(self.queue.len());
        (0..take).filter_map(|_| self.queue.pop_front()).collect()
    }
}

impl Default for AdiabticTaskQueue {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_init() {
        let sched = QuantumScheduler::new();
        assert_eq!(sched.current_batch_size, 1000);
    }

    #[test]
    fn test_p_controller_increase() {
        let mut sched = QuantumScheduler::new();
        // Latency 10ms (half of 20ms baseline) -> Scale 2.0 -> clamped to 1.5
        // New batch = 1000 * 1.5 = 1500
        let latency = SPA::new(10, 0, 0, 0, 0);
        let action = sched.tick(latency, &[]);

        if let SchedulerAction::Continue { batch_size } = action {
            assert_eq!(batch_size, 1500);
            assert_eq!(sched.current_batch_size, 1500);
        } else {
            panic!("Expected Continue action");
        }
    }

    #[test]
    fn test_p_controller_decrease() {
        let mut sched = QuantumScheduler::new();
        // Latency 40ms (double 20ms baseline) -> Scale 0.5 -> clamped to 0.5
        // New batch = 1000 * 0.5 = 500
        let latency = SPA::new(40, 0, 0, 0, 0);
        let action = sched.tick(latency, &[]);

        if let SchedulerAction::Continue { batch_size } = action {
            assert_eq!(batch_size, 500);
        } else {
            panic!("Expected Continue action");
        }
    }

    // --- EXP-029: Portal Detector ---

    #[test]
    fn test_portal_resonance_range() {
        // φ(t) siempre en [-1.0, +1.0]
        for t_s in [0i64, 4, 8, 17, 34, 51, 68] {
            let r = PortalDetector::resonance(t_s * 1_000_000_000);
            assert!(r.to_raw() >= -SPA::SCALE_0, "φ debe ser >= -1.0 en t={}s", t_s);
            assert!(r.to_raw() <= SPA::SCALE_0,  "φ debe ser <= +1.0 en t={}s", t_s);
        }
    }

    #[test]
    fn test_portal_opens_somewhere_in_cycle() {
        // Debe existir al menos un portal en los 680 ticks de 68s (10Hz)
        let dt_ns = 100_000_000i64; // 0.1s
        let found = (0..680).any(|i| {
            let r = PortalDetector::resonance(i * dt_ns);
            PortalDetector::is_open(r)
        });
        assert!(found, "debe detectarse al menos un portal en 68s");
    }

    #[test]
    fn test_batch_size_adaptive() {
        // Portal fuerte (0.91) → batch 5
        let strong = SPA::from_raw(11_793_600); // 0.91
        assert_eq!(PortalDetector::batch_size(strong), 5);

        // Portal medio (0.87) → batch 4
        let medium = SPA::from_raw(11_275_200); // 0.87
        assert_eq!(PortalDetector::batch_size(medium), 4);

        // Portal mínimo (0.76) → batch 2
        let weak = SPA::from_raw(9_849_600); // 0.76
        assert_eq!(PortalDetector::batch_size(weak), 2);
    }

    #[test]
    fn test_adiabtic_queue_portal_execution() {
        let mut q = AdiabticTaskQueue::new();
        q.push(QuantumTask::new(TaskType::LatticeGc, 10));
        q.push(QuantumTask::new(TaskType::BackupS60, 15));

        // Buscar un t_ns donde el portal esté abierto
        let dt = 100_000_000i64;
        let mut portal_t = None;
        for i in 0..680 {
            let r = PortalDetector::resonance(i * dt);
            if PortalDetector::is_open(r) {
                portal_t = Some(i * dt);
                break;
            }
        }

        let t = portal_t.expect("debe existir un portal en 68s");
        let executed = q.tick(t);

        assert!(!executed.is_empty(), "en portal debe ejecutar tareas");
        assert!(q.tasks_in_portal > 0);
        assert_eq!(q.tasks_forced, 0);
    }

    #[test]
    fn test_adiabtic_queue_overflow() {
        let mut q = AdiabticTaskQueue::new();
        // Llenar más allá del límite (20)
        for i in 0..22 {
            q.push(QuantumTask::new(TaskType::ZpeTune, (5 + i % 15) as i64));
        }

        // Usar t=0 — casi seguro NO es portal (sin_bio=0, sin_crys=0, sin_venus=0 → φ=0)
        let executed = q.tick(0);
        assert!(!executed.is_empty(), "overflow debe forzar ejecución");
        assert!(q.tasks_forced > 0);
    }

    #[test]
    fn test_efficiency_formula() {
        let mut q = AdiabticTaskQueue::new();
        q.tasks_in_portal = 65;
        q.tasks_forced = 10;
        // efficiency = 65 / 75 = 86%
        assert_eq!(q.efficiency_pct(), 86);
    }

    #[test]
    fn test_venus_phase_reset() {
        let mut sched = QuantumScheduler::new();
        let latency = SPA::new(20, 0, 0, 0, 0);
        let signal = [];

        // Tick 2787 times
        for _ in 0..2787 {
            sched.tick(latency, &signal);
        }

        assert_eq!(sched.ticks_since_reset, 2787);

        // 2788th tick -> Should trigger reset
        let action = sched.tick(latency, &signal);
        match action {
            SchedulerAction::Emergency { phase } => {
                assert_eq!(phase, SPA::zero());
                assert_eq!(sched.ticks_since_reset, 0);
            }
            _ => panic!("Expected Emergency Phase Reset at tick 2788"),
        }
    }
}
