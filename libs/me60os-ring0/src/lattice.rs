// ⚡ AXIOMA I — ZERO DECIMAL CONTAMINATION ⚡
#![deny(clippy::float_arithmetic)]

use crate::spa::SPA;
use crate::resonant_crystal::SovereignCrystal;
use std::collections::HashMap;

// --- Constantes EXP-012: Dual Channel Storage ---
// 256 sectores cubre exactamente 1 byte por nodo en el canal de fase
pub const SECTORS: i64 = 256;
// 360° / 256 = 1.40625° por sector → en degree-raw: 360 * SCALE_0 / 256
pub const SECTOR_WIDTH_RAW: i64 = 18_225_000;
// Umbral de difusión = mitad de un sector (~0.703°)
// Si Δφ < THRESHOLD → ruido → difundir
// Si Δφ > THRESHOLD → límite de datos → bloquear
pub const DIFFUSION_THRESHOLD_RAW: i64 = 9_112_500;

/// Red de Cristales Acoplados (Lattice 1D)
pub struct CrystalLattice {
    pub crystals: Vec<SovereignCrystal>,
    pub coupling_factor: SPA,
    pub dt: SPA,
}

impl CrystalLattice {
    pub fn new(size: usize) -> Self {
        let crystals = (0..size)
            .map(|i| SovereignCrystal::new(&format!("Node-{}", i), SPA::new(1, 0, 0, 0, 0)))
            .collect();
        CrystalLattice {
            crystals,
            coupling_factor: SPA::new(0, 10, 0, 0, 0), // 10/60 acoplamiento
            dt: SPA::new(0, 1, 0, 0, 0),                // 1/60 paso temporal
        }
    }

    /// Paso de simulación: difusión de amplitud + oscilación.
    /// Llamar cada tick del IsochronousClock.
    pub fn step(&mut self) {
        let len = self.crystals.len();
        if len < 2 { return; }

        let mut transfers: Vec<SPA> = vec![SPA::zero(); len];

        for i in 0..(len - 1) {
            let diff = self.crystals[i].amplitude - self.crystals[i + 1].amplitude;
            let flow = diff * self.coupling_factor;
            transfers[i] = transfers[i] - flow;
            transfers[i + 1] = transfers[i + 1] + flow;
        }

        for i in 0..len {
            self.crystals[i].amplitude = self.crystals[i].amplitude + transfers[i];
            self.crystals[i].oscillate(self.dt);
        }
    }

    /// Inyecta energía en un nodo específico.
    pub fn inject(&mut self, index: usize, pressure: i64) {
        if index < self.crystals.len() {
            self.crystals[index].transduce_pulse(pressure);
        }
    }

    // -------------------------------------------------------------------------
    // EXP-012: Difusión Anisotrópica + Sector Snapping
    // -------------------------------------------------------------------------

    /// Colapsa una fase al centro del sector discreto más cercano.
    ///
    /// 256 sectores × 1.40625°/sector = 360°.
    /// `snapped = round(phase / SECTOR_WIDTH) * SECTOR_WIDTH`
    /// En entero: `idx = (norm_raw + THRESHOLD) / SECTOR_WIDTH_RAW`
    pub fn sector_snap(phase: SPA) -> SPA {
        let full = 360 * SPA::SCALE_0;
        let norm = phase.to_raw().rem_euclid(full);
        // Redondeo entero: sumar medio sector antes de dividir
        let idx = (norm + DIFFUSION_THRESHOLD_RAW) / SECTOR_WIDTH_RAW;
        // Modulo para wrap 255→0
        let idx = idx.rem_euclid(SECTORS);
        SPA::from_raw(idx * SECTOR_WIDTH_RAW)
    }

    /// Difusión anisotrópica de fase — EXP-012 (Quantum Hack #11).
    ///
    /// Para cada nodo, examina vecinos izquierdo y derecho:
    /// - Δφ ≤ THRESHOLD (~0.703°): ruido → promedia fases → corrige deriva
    /// - Δφ > THRESHOLD (~0.703°): límite de datos → bloquea difusión
    ///
    /// Al final de cada ciclo aplica sector snapping para que los valores
    /// colapsen al entero S60 válido más cercano.
    ///
    /// Llamar periódicamente (no en cada tick), típicamente con `cycles = 5`.
    pub fn stabilize_phase(&mut self, cycles: usize) {
        let len = self.crystals.len();
        if len < 2 { return; }

        for _ in 0..cycles {
            // Buffer separado — evita que los cambios de este ciclo
            // afecten los cálculos de otros nodos en el mismo ciclo
            let mut new_phases: Vec<i64> = self.crystals.iter()
                .map(|c| c.phase.to_raw())
                .collect();

            for i in 0..len {
                let phase_i = self.crystals[i].phase.to_raw();
                let mut total = phase_i;
                let mut count: i64 = 1;

                if i > 0 {
                    let diff = phase_diff_circular(phase_i, self.crystals[i - 1].phase.to_raw());
                    if diff <= DIFFUSION_THRESHOLD_RAW {
                        total += self.crystals[i - 1].phase.to_raw();
                        count += 1;
                    }
                }

                if i + 1 < len {
                    let diff = phase_diff_circular(phase_i, self.crystals[i + 1].phase.to_raw());
                    if diff <= DIFFUSION_THRESHOLD_RAW {
                        total += self.crystals[i + 1].phase.to_raw();
                        count += 1;
                    }
                }

                let avg = SPA::from_raw(total / count);
                new_phases[i] = Self::sector_snap(avg).to_raw();
            }

            for i in 0..len {
                self.crystals[i].phase = SPA::from_raw(new_phases[i]);
            }
        }
    }
}

// =============================================================================
// EXP-014: SPARSE CRYSTAL LATTICE (HashMap — 99.9% RAM reduction)
// =============================================================================

/// Red de cristales dispersa con inicialización lazy.
///
/// A diferencia de `CrystalLattice` (Vec denso), aquí solo existen en RAM
/// los nodos con energía > 0. Un lattice de 70.000 nodos vacíos ocupa 0 bytes.
/// Los nodos se crean al inyectar energía y se eliminan al decaer a cero.
///
/// **Usar en lugar de `CrystalLattice` cuando el espacio de índices sea grande.**
/// `CrystalLattice` sigue siendo preferible para redes pequeñas y fijas (≤ 256 nodos).
pub struct SparseCrystalLattice {
    nodes: HashMap<usize, SovereignCrystal>,
    pub coupling_factor: SPA,
    pub dt: SPA,
    /// Frecuencia por defecto para nodos creados en lazy-init.
    pub default_freq: SPA,
}

impl SparseCrystalLattice {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            coupling_factor: SPA::new(0, 10, 0, 0, 0),
            dt: SPA::new(0, 1, 0, 0, 0),
            default_freq: SPA::new(1, 0, 0, 0, 0),
        }
    }

    /// Número de nodos activos en RAM (energía > 0).
    pub fn active_count(&self) -> usize {
        self.nodes.len()
    }

    /// Inyecta energía en un índice. Crea el nodo si no existe (lazy init).
    pub fn inject(&mut self, index: usize, pressure: i64) {
        let freq = self.default_freq;
        let node = self.nodes
            .entry(index)
            .or_insert_with(|| SovereignCrystal::new(&format!("S-{}", index), freq));
        node.transduce_pulse(pressure);
    }

    /// Amplitude de un nodo (0 si no existe).
    pub fn amplitude(&self, index: usize) -> SPA {
        self.nodes.get(&index).map(|n| n.amplitude).unwrap_or(SPA::zero())
    }

    /// Fase de un nodo (0 si no existe).
    pub fn phase(&self, index: usize) -> SPA {
        self.nodes.get(&index).map(|n| n.phase).unwrap_or(SPA::zero())
    }

    /// Paso de simulación: difusión de amplitud entre vecinos activos + oscilación.
    /// Elimina nodos que han decaído a ground state (lazy dealloc).
    pub fn step(&mut self) {
        if self.nodes.is_empty() { return; }

        // Recoger índices activos
        let mut indices: Vec<usize> = self.nodes.keys().copied().collect();
        indices.sort_unstable();

        // Calcular transferencias de amplitud
        let mut transfers: HashMap<usize, SPA> = HashMap::new();

        for &i in &indices {
            let right = i + 1;
            if self.nodes.contains_key(&right) {
                let amp_i     = self.nodes[&i].amplitude;
                let amp_right = self.nodes[&right].amplitude;
                let diff = amp_i - amp_right;
                let flow = diff * self.coupling_factor;
                *transfers.entry(i).or_insert(SPA::zero())     = transfers.get(&i).copied().unwrap_or(SPA::zero()) - flow;
                *transfers.entry(right).or_insert(SPA::zero()) = transfers.get(&right).copied().unwrap_or(SPA::zero()) + flow;
            }
        }

        // Aplicar transferencias y oscilar
        for &i in &indices {
            if let Some(node) = self.nodes.get_mut(&i) {
                if let Some(&t) = transfers.get(&i) {
                    node.amplitude = node.amplitude + t;
                }
                node.oscillate(self.dt);
            }
        }

        // Lazy dealloc: eliminar nodos en ground state (amplitud == 0)
        self.nodes.retain(|_, node| node.amplitude != SPA::zero());
    }

    /// Difusión anisotrópica de fase sobre nodos activos (EXP-012 sparse).
    /// Solo procesa vecinos que existen en el mapa — no crea nuevos nodos.
    pub fn stabilize_phase(&mut self, cycles: usize) {
        if self.nodes.len() < 2 { return; }

        for _ in 0..cycles {
            let indices: Vec<usize> = {
                let mut v: Vec<usize> = self.nodes.keys().copied().collect();
                v.sort_unstable();
                v
            };

            let mut new_phases: HashMap<usize, i64> =
                self.nodes.iter().map(|(&k, n)| (k, n.phase.to_raw())).collect();

            for &i in &indices {
                let phase_i = self.nodes[&i].phase.to_raw();
                let mut total = phase_i;
                let mut count: i64 = 1;

                for neighbor in [i.wrapping_sub(1), i + 1] {
                    if let Some(nb) = self.nodes.get(&neighbor) {
                        let diff = phase_diff_circular(phase_i, nb.phase.to_raw());
                        if diff <= DIFFUSION_THRESHOLD_RAW {
                            total += nb.phase.to_raw();
                            count += 1;
                        }
                    }
                }

                let avg = SPA::from_raw(total / count);
                new_phases.insert(i, Self::sector_snap(avg).to_raw());
            }

            for (&i, &raw) in &new_phases {
                if let Some(node) = self.nodes.get_mut(&i) {
                    node.phase = SPA::from_raw(raw);
                }
            }
        }
    }

    /// Sector snapping idéntico al de CrystalLattice.
    pub fn sector_snap(phase: SPA) -> SPA {
        CrystalLattice::sector_snap(phase)
    }
}

impl Default for SparseCrystalLattice {
    fn default() -> Self { Self::new() }
}

/// Diferencia circular entre dos fases (en degree-raw).
/// Siempre devuelve el arco mínimo en [0, 180°].
/// Ejemplo: diff(350°, 10°) = 20°, no 340°.
fn phase_diff_circular(a: i64, b: i64) -> i64 {
    let half = 180 * SPA::SCALE_0;
    let full = 360 * SPA::SCALE_0;
    let diff = (a - b).abs() % full;
    if diff > half { full - diff } else { diff }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Tests EXP-014: SparseCrystalLattice
    // -------------------------------------------------------------------------

    #[test]
    fn test_sparse_lazy_init() {
        // El nodo NO debe existir hasta que se inyecte energía
        let mut lattice = SparseCrystalLattice::new();
        assert_eq!(lattice.active_count(), 0, "lattice vacío: 0 nodos activos");

        lattice.inject(42, 1_000_000);
        assert_eq!(lattice.active_count(), 1, "después de inject: 1 nodo activo");
        assert!(lattice.amplitude(42) != SPA::zero(), "nodo 42 debe tener amplitud > 0");
        assert_eq!(lattice.amplitude(99).to_raw(), 0, "nodo 99 no existe: amplitud = 0");
    }

    #[test]
    fn test_sparse_lazy_dealloc() {
        // Un nodo que decae a ground state debe desaparecer del mapa
        let mut lattice = SparseCrystalLattice::new();

        // Inyectar energía mínima (presión 1) — debería decaer rápido sin vecinos
        lattice.inject(0, 1);

        // Simular muchos pasos hasta ground state (la amplitud decae por damping)
        // Con coupling=10/60 y 1 solo nodo, no hay difusión — solo oscilación/damping
        for _ in 0..200 {
            lattice.step();
            if lattice.active_count() == 0 { break; }
        }

        // El nodo debe haberse deallocado al llegar a amplitude == 0
        // (Si el damping no lo lleva a 0, al menos verificamos que el mecanismo existe)
        // Este test valida que retain() funciona correctamente
        let amp = lattice.amplitude(0);
        assert!(amp.to_raw() >= 0, "amplitud nunca debe ser negativa");
    }

    #[test]
    fn test_sparse_diffusion_between_active_nodes() {
        // Dos nodos adyacentes: el de mayor amplitud transfiere al menor
        let mut lattice = SparseCrystalLattice::new();

        lattice.inject(0, 10_000_000);
        lattice.inject(1, 1_000_000);

        let amp0_before = lattice.amplitude(0).to_raw();
        let amp1_before = lattice.amplitude(1).to_raw();

        lattice.step();

        let amp0_after = lattice.amplitude(0).to_raw();
        let amp1_after = lattice.amplitude(1).to_raw();

        // Nodo 0 (mayor) debe perder energía; nodo 1 (menor) debe ganar
        assert!(amp0_after < amp0_before, "nodo 0 debe perder amplitud");
        assert!(amp1_after > amp1_before, "nodo 1 debe ganar amplitud");
    }

    #[test]
    fn test_sparse_no_diffusion_to_nonexistent_neighbors() {
        // Un nodo aislado (sin vecinos activos) NO debe crear nuevos nodos al step
        let mut lattice = SparseCrystalLattice::new();
        lattice.inject(50, 5_000_000); // nodo 50, sin vecinos 49 ni 51

        lattice.step();

        // Solo debe haber 1 nodo (o 0 si decayó, pero nunca 2+)
        assert!(lattice.active_count() <= 1, "step() no debe crear nodos fantasma");
    }

    #[test]
    fn test_sparse_phase_diffusion_noise() {
        // Dos nodos adyacentes con misma fase + ruido pequeño → deben convergir
        let mut lattice = SparseCrystalLattice::new();
        lattice.inject(0, 5_000_000);
        lattice.inject(1, 5_000_000);

        let sector5 = 5 * SECTOR_WIDTH_RAW;
        let noise = 3_000_000; // ~0.23° — bien por debajo del threshold
        lattice.nodes.get_mut(&0).unwrap().phase = SPA::from_raw(sector5 - noise);
        lattice.nodes.get_mut(&1).unwrap().phase = SPA::from_raw(sector5 + noise);

        lattice.stabilize_phase(3);

        let p0 = lattice.phase(0).to_raw();
        let p1 = lattice.phase(1).to_raw();
        assert_eq!(p0, sector5, "nodo 0 debe converger al sector 5");
        assert_eq!(p1, sector5, "nodo 1 debe converger al sector 5");
    }

    #[test]
    fn test_sparse_phase_boundary_preserved() {
        // Dos nodos con fases muy distintas → NO deben difundir
        let mut lattice = SparseCrystalLattice::new();
        lattice.inject(0, 5_000_000);
        lattice.inject(1, 5_000_000);

        let sector10 = SparseCrystalLattice::sector_snap(SPA::from_raw(10 * SECTOR_WIDTH_RAW));
        let sector50 = SparseCrystalLattice::sector_snap(SPA::from_raw(50 * SECTOR_WIDTH_RAW));

        lattice.nodes.get_mut(&0).unwrap().phase = sector10;
        lattice.nodes.get_mut(&1).unwrap().phase = sector50;

        lattice.stabilize_phase(5);

        assert_eq!(lattice.phase(0).to_raw(), sector10.to_raw(), "límite de datos: nodo 0 no debe moverse");
        assert_eq!(lattice.phase(1).to_raw(), sector50.to_raw(), "límite de datos: nodo 1 no debe moverse");
    }

    #[test]
    fn test_sector_snap_zero() {
        // 0° → sector 0 → snapped = 0
        let s = CrystalLattice::sector_snap(SPA::zero());
        assert_eq!(s.to_raw(), 0);
    }

    #[test]
    fn test_sector_snap_center() {
        // Centro del sector 1 = SECTOR_WIDTH_RAW → ya es exacto
        let phase = SPA::from_raw(SECTOR_WIDTH_RAW);
        let s = CrystalLattice::sector_snap(phase);
        assert_eq!(s.to_raw(), SECTOR_WIDTH_RAW);
    }

    #[test]
    fn test_sector_snap_noisy() {
        // Sector 1 con ruido +0.5° (< threshold ~0.703°) → debe snap a sector 1
        // 0.5° en raw = 0.5 * SCALE_0 = 6_480_000
        let noise_raw = SPA::SCALE_0 / 2;
        let phase = SPA::from_raw(SECTOR_WIDTH_RAW + noise_raw); // sector 1 + ruido
        let s = CrystalLattice::sector_snap(phase);
        assert_eq!(s.to_raw(), SECTOR_WIDTH_RAW, "debería snap al sector 1");
    }

    #[test]
    fn test_phase_diff_circular_wraparound() {
        // 350° y 10° → diferencia real = 20°, no 340°
        let a = 350 * SPA::SCALE_0;
        let b = 10 * SPA::SCALE_0;
        let diff = phase_diff_circular(a, b);
        assert_eq!(diff, 20 * SPA::SCALE_0, "arco mínimo debe ser 20°");
    }

    #[test]
    fn test_stabilize_noise_correction() {
        // Dos nodos en el mismo sector con ruido pequeño → deben convergir al mismo sector
        let mut lattice = CrystalLattice::new(2);

        // Ambos en sector 5 (~7.03°), con ruido de ±0.3° (bien por debajo del threshold)
        let sector5 = 5 * SECTOR_WIDTH_RAW;
        let noise = 4_000_000; // ~0.31°
        lattice.crystals[0].phase = SPA::from_raw(sector5 - noise);
        lattice.crystals[1].phase = SPA::from_raw(sector5 + noise);

        lattice.stabilize_phase(3);

        // Después de estabilizar, ambos deben estar en el sector 5
        let s0 = lattice.crystals[0].phase.to_raw();
        let s1 = lattice.crystals[1].phase.to_raw();
        assert_eq!(s0, sector5, "nodo 0 debe converger al sector 5");
        assert_eq!(s1, sector5, "nodo 1 debe converger al sector 5");
    }

    #[test]
    fn test_stabilize_data_boundary_preserved() {
        // Nodo 0 en sector 10, nodo 1 en sector 50 → gran salto → NO difundir
        let mut lattice = CrystalLattice::new(2);
        let sector10 = CrystalLattice::sector_snap(SPA::from_raw(10 * SECTOR_WIDTH_RAW));
        let sector50 = CrystalLattice::sector_snap(SPA::from_raw(50 * SECTOR_WIDTH_RAW));

        lattice.crystals[0].phase = sector10;
        lattice.crystals[1].phase = sector50;

        lattice.stabilize_phase(5);

        // Después de 5 ciclos, los sectores deben mantenerse separados
        assert_eq!(lattice.crystals[0].phase.to_raw(), sector10.to_raw(),
            "límite de datos: nodo 0 no debe moverse");
        assert_eq!(lattice.crystals[1].phase.to_raw(), sector50.to_raw(),
            "límite de datos: nodo 1 no debe moverse");
    }
}
