// ⚡ AXIOMA I — ZERO DECIMAL CONTAMINATION ⚡
#![deny(clippy::float_arithmetic)]

use crate::spa::SPA;
use crate::resonant_crystal::SovereignCrystal;
use crate::lattice::{SECTOR_WIDTH_RAW, DIFFUSION_THRESHOLD_RAW, CrystalLattice};
use std::collections::HashMap;

// =============================================================================
// EXP-009: HexLattice — Red 2D Hexagonal Von Neumann (72% retención)
// =============================================================================
//
// Coordenadas axiales (q, r). Vecinos de (q,r):
//   (q+1,r), (q-1,r), (q,r+1), (q,r-1), (q+1,r-1), (q-1,r+1)
//
// Para evitar doble contabilización en la difusión solo se procesan
// las 3 direcciones "positivas" HEX_DIRS_HALF al calcular flujos.
//
// Encoding byte: 256 sectores = 1 byte por nodo en canal de fase.
//   write_byte(coord, val) → phase = val * SECTOR_WIDTH_RAW
//   read_byte(coord)       → val = phase_raw / SECTOR_WIDTH_RAW
//
// Topología rings=5: 1 + 3×5×6 = 91 nodos, ~91 bytes almacenables.
// =============================================================================

/// Coordenada hexagonal axial.
pub type HexCoord = (i32, i32);

/// 6 direcciones de vecinos en rejilla hexagonal (axial q,r).
pub const HEX_DIRS: [HexCoord; 6] = [
    (1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1),
];

/// Mitad "positiva" — para procesar cada arista exactamente una vez en step().
const HEX_DIRS_HALF: [HexCoord; 3] = [(1, 0), (1, -1), (0, -1)];

/// Red hexagonal 2D de cristales resonantes — EXP-009 Von Neumann.
///
/// Pre-aloca `1 + 3×rings×(rings+1)` nodos en coordenadas axiales.
/// Difusión en 6 direcciones → 72% retención vs 44% en topología lineal.
pub struct HexLattice {
    nodes: HashMap<HexCoord, SovereignCrystal>,
    pub coupling_factor: SPA,
    pub dt: SPA,
    pub rings: usize,
}

impl HexLattice {
    /// Crea la red hexagonal con `rings` anillos concéntricos.
    /// rings=0 → 1 nodo (solo centro). rings=5 → 91 nodos.
    pub fn new(rings: usize) -> Self {
        let mut nodes = HashMap::new();
        let rings_i = rings as i32;

        for q in -rings_i..=rings_i {
            let r_min = (-rings_i).max(-q - rings_i);
            let r_max = rings_i.min(-q + rings_i);
            for r in r_min..=r_max {
                let name = format!("H{},{}", q, r);
                nodes.insert(
                    (q, r),
                    SovereignCrystal::new(&name, SPA::new(1, 0, 0, 0, 0)),
                );
            }
        }

        HexLattice {
            nodes,
            coupling_factor: SPA::new(0, 10, 0, 0, 0),
            dt: SPA::new(0, 1, 0, 0, 0),
            rings,
        }
    }

    /// Número total de nodos para `rings` anillos: `1 + 3×rings×(rings+1)`.
    pub fn node_count_for_rings(rings: usize) -> usize {
        1 + 3 * rings * (rings + 1)
    }

    /// Número de nodos activos (pre-alocados o con energía).
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// `true` si la red no contiene nodos.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Vecinos válidos de `coord` (solo los que existen en la red).
    pub fn neighbors(&self, coord: HexCoord) -> impl Iterator<Item = HexCoord> + '_ {
        let (q, r) = coord;
        HEX_DIRS
            .iter()
            .map(move |&(dq, dr)| (q + dq, r + dr))
            .filter(|nb| self.nodes.contains_key(nb))
    }

    /// Distancia hexagonal desde el origen (en pasos de arista).
    pub fn hex_distance(coord: HexCoord) -> i32 {
        let (q, r) = coord;
        let s = -q - r;
        (q.abs().max(r.abs())).max(s.abs())
    }

    /// Coordenadas en orden espiral (anillo 0, luego 1, 2, …).
    /// Orden determinista para lectura/escritura secuencial de bytes.
    pub fn coords_spiral(&self) -> Vec<HexCoord> {
        let mut result = Vec::with_capacity(self.nodes.len());
        for ring in 0..=self.rings {
            if ring == 0 {
                if self.nodes.contains_key(&(0, 0)) {
                    result.push((0, 0));
                }
                continue;
            }
            // Empieza en la esquina (-r, r) y recorre el anillo en 6 segmentos
            // Esquina de inicio: hex_scale(HEX_DIRS[4], r) = (-r, r)
            let r = ring as i32;
            let mut q = -r;
            let mut row = r;
            for dir_idx in 0..6 {
                let (dq, dr) = HEX_DIRS[dir_idx];
                for _ in 0..r {
                    if self.nodes.contains_key(&(q, row)) {
                        result.push((q, row));
                    }
                    q += dq;
                    row += dr;
                }
            }
        }
        result
    }

    // -------------------------------------------------------------------------
    // Energía
    // -------------------------------------------------------------------------

    /// Inyecta energía en un nodo.
    pub fn inject(&mut self, coord: HexCoord, pressure: i64) {
        if let Some(node) = self.nodes.get_mut(&coord) {
            node.transduce_pulse(pressure);
        }
    }

    /// Amplitud de un nodo (0 si no existe).
    pub fn amplitude(&self, coord: HexCoord) -> SPA {
        self.nodes.get(&coord).map(|n| n.amplitude).unwrap_or(SPA::zero())
    }

    /// Fase de un nodo (0 si no existe).
    pub fn phase(&self, coord: HexCoord) -> SPA {
        self.nodes.get(&coord).map(|n| n.phase).unwrap_or(SPA::zero())
    }

    // -------------------------------------------------------------------------
    // Simulación
    // -------------------------------------------------------------------------

    /// Paso de simulación: difusión de amplitud en 6 vecinos + oscilación.
    ///
    /// Procesa cada arista exactamente una vez (HEX_DIRS_HALF).
    pub fn step(&mut self) {
        let coords: Vec<HexCoord> = self.nodes.keys().copied().collect();
        let mut transfers: HashMap<HexCoord, SPA> = HashMap::new();

        for &coord in &coords {
            for &(dq, dr) in &HEX_DIRS_HALF {
                let nb = (coord.0 + dq, coord.1 + dr);
                if self.nodes.contains_key(&nb) {
                    let amp_a = self.nodes[&coord].amplitude;
                    let amp_b = self.nodes[&nb].amplitude;
                    let diff = amp_a - amp_b;
                    let flow = diff * self.coupling_factor;

                    *transfers.entry(coord).or_insert(SPA::zero()) =
                        transfers.get(&coord).copied().unwrap_or(SPA::zero()) - flow;
                    *transfers.entry(nb).or_insert(SPA::zero()) =
                        transfers.get(&nb).copied().unwrap_or(SPA::zero()) + flow;
                }
            }
        }

        for &coord in &coords {
            if let Some(node) = self.nodes.get_mut(&coord) {
                if let Some(&t) = transfers.get(&coord) {
                    node.amplitude = node.amplitude + t;
                }
                node.oscillate(self.dt);
            }
        }
    }

    /// Difusión anisotrópica de fase en 6 vecinos — EXP-012 hex.
    ///
    /// - Δφ ≤ THRESHOLD: ruido → promedia → sector snap.
    /// - Δφ > THRESHOLD: límite de datos → bloquea difusión.
    pub fn stabilize_phase(&mut self, cycles: usize) {
        for _ in 0..cycles {
            let coords: Vec<HexCoord> = self.nodes.keys().copied().collect();
            let mut new_phases: HashMap<HexCoord, i64> =
                self.nodes.iter().map(|(&k, n)| (k, n.phase.to_raw())).collect();

            for &coord in &coords {
                let phase_i = self.nodes[&coord].phase.to_raw();
                let mut total = phase_i;
                let mut count: i64 = 1;

                for &(dq, dr) in &HEX_DIRS {
                    let nb = (coord.0 + dq, coord.1 + dr);
                    if let Some(nb_node) = self.nodes.get(&nb) {
                        let diff = phase_diff_circular(phase_i, nb_node.phase.to_raw());
                        if diff <= DIFFUSION_THRESHOLD_RAW {
                            total += nb_node.phase.to_raw();
                            count += 1;
                        }
                    }
                }

                let avg = SPA::from_raw(total / count);
                new_phases.insert(coord, CrystalLattice::sector_snap(avg).to_raw());
            }

            for (&coord, &raw) in &new_phases {
                if let Some(node) = self.nodes.get_mut(&coord) {
                    node.phase = SPA::from_raw(raw);
                }
            }
        }
    }

    // -------------------------------------------------------------------------
    // EXP-009: Byte Storage (1 byte/nodo vía canal de fase)
    // -------------------------------------------------------------------------
    //
    // 256 sectores × 1.40625°/sector = 360° → exactamente 1 byte por nodo.
    // El sector snapping (stabilize_phase) actúa como ECC natural.
    //
    // Encoding: sector_index = byte_value
    //   phase_raw = byte_value * SECTOR_WIDTH_RAW
    //   byte_value = phase_raw / SECTOR_WIDTH_RAW (con snapping ya aplicado)

    /// Escribe un byte en el canal de fase de un nodo.
    /// Activa el nodo si tenía amplitud cero.
    pub fn write_byte(&mut self, coord: HexCoord, val: u8) {
        if let Some(node) = self.nodes.get_mut(&coord) {
            node.phase = SPA::from_raw(val as i64 * SECTOR_WIDTH_RAW);
            // Mínima energía para que el nodo sea visible
            if node.amplitude == SPA::zero() {
                node.transduce_pulse(1_000_000);
            }
        }
    }

    /// Lee el byte almacenado en el canal de fase de un nodo.
    pub fn read_byte(&self, coord: HexCoord) -> u8 {
        let raw = self.phase(coord).to_raw();
        // Snap al sector más cercano antes de leer
        let snapped = CrystalLattice::sector_snap(SPA::from_raw(raw)).to_raw();
        (snapped / SECTOR_WIDTH_RAW).rem_euclid(256) as u8
    }

    /// Escribe un slice de bytes en orden espiral (centro → exterior).
    /// Escribe hasta `min(data.len(), nodos_disponibles)` bytes.
    pub fn write_bytes(&mut self, data: &[u8]) {
        let coords = self.coords_spiral();
        for (coord, &byte) in coords.iter().zip(data.iter()) {
            self.write_byte(*coord, byte);
        }
    }

    /// Lee `len` bytes en orden espiral.
    pub fn read_bytes(&self, len: usize) -> Vec<u8> {
        let coords = self.coords_spiral();
        coords.iter().take(len).map(|&c| self.read_byte(c)).collect()
    }
}

impl Default for HexLattice {
    fn default() -> Self {
        Self::new(5) // 91 nodos por defecto
    }
}

/// Diferencia circular entre fases (en degree-raw). Arco mínimo en [0, 180°].
fn phase_diff_circular(a: i64, b: i64) -> i64 {
    let half = 180 * SPA::SCALE_0;
    let full = 360 * SPA::SCALE_0;
    let diff = (a - b).abs() % full;
    if diff > half { full - diff } else { diff }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_count_rings5() {
        // rings=5 → 1 + 3×5×6 = 91 nodos
        let lattice = HexLattice::new(5);
        assert_eq!(lattice.len(), 91, "rings=5 debe tener exactamente 91 nodos");
        assert_eq!(HexLattice::node_count_for_rings(5), 91);
    }

    #[test]
    fn test_node_count_formula() {
        // Verificar la fórmula para varios tamaños
        for rings in 0..=6 {
            let lattice = HexLattice::new(rings);
            let expected = HexLattice::node_count_for_rings(rings);
            assert_eq!(
                lattice.len(), expected,
                "rings={}: fórmula y count deben coincidir", rings
            );
        }
    }

    #[test]
    fn test_center_has_6_neighbors() {
        let lattice = HexLattice::new(2);
        let nb_count = lattice.neighbors((0, 0)).count();
        assert_eq!(nb_count, 6, "el centro tiene exactamente 6 vecinos");
    }

    #[test]
    fn test_edge_node_has_fewer_neighbors() {
        let lattice = HexLattice::new(2);
        // Nodo en el borde exterior del anillo 2 → menos de 6 vecinos
        let nb_count = lattice.neighbors((2, 0)).count();
        assert!(nb_count < 6, "nodo de borde exterior tiene < 6 vecinos (anillo exterior)");
    }

    #[test]
    fn test_diffusion_center_to_neighbors() {
        let mut lattice = HexLattice::new(2);
        // Inyectar solo en el centro
        lattice.inject((0, 0), 10_000_000);
        let amp_before = lattice.amplitude((0, 0)).to_raw();

        lattice.step();

        // El centro debe haber perdido energía hacia sus 6 vecinos
        let amp_after = lattice.amplitude((0, 0)).to_raw();
        assert!(amp_after < amp_before, "centro debe perder amplitud tras step()");

        // Al menos un vecino debe haber ganado energía
        let nb_gained = HEX_DIRS.iter().any(|&(dq, dr)| {
            lattice.amplitude((dq, dr)).to_raw() > 0
        });
        assert!(nb_gained, "al menos un vecino debe haber ganado energía");
    }

    #[test]
    fn test_write_read_byte() {
        let mut lattice = HexLattice::new(1);

        // Escribir y leer inmediatamente (sin difusión)
        lattice.write_byte((0, 0), 42u8);
        let val = lattice.read_byte((0, 0));
        assert_eq!(val, 42, "read_byte debe devolver el valor escrito");
    }

    #[test]
    fn test_write_read_all_bytes() {
        // 7 nodos en rings=1, escribir 7 bytes distintos
        let mut lattice = HexLattice::new(1);
        let data: Vec<u8> = (0u8..7).collect();

        lattice.write_bytes(&data);
        let recovered = lattice.read_bytes(7);

        assert_eq!(recovered, data, "todos los bytes deben recuperarse intactos");
    }

    #[test]
    fn test_write_read_with_stabilize() {
        // Escribir datos, añadir ruido, stabilize_phase debe corregir el ruido
        let mut lattice = HexLattice::new(1);
        let data: Vec<u8> = vec![10, 20, 30, 40, 50, 60, 70];

        lattice.write_bytes(&data);

        // Añadir ruido pequeño a la fase (< THRESHOLD)
        let noise = 3_000_000i64; // ~0.23° — por debajo del threshold
        for node in lattice.nodes.values_mut() {
            node.phase = SPA::from_raw(node.phase.to_raw() + noise);
        }

        // stabilize_phase debe recuperar los valores originales via sector snap
        lattice.stabilize_phase(3);

        let recovered = lattice.read_bytes(7);
        assert_eq!(recovered, data, "stabilize_phase debe corregir ruido sub-threshold");
    }

    #[test]
    fn test_coords_spiral_length() {
        let lattice = HexLattice::new(3);
        let spiral = lattice.coords_spiral();
        assert_eq!(spiral.len(), lattice.len(), "coords_spiral debe cubrir todos los nodos");
    }

    #[test]
    fn test_coords_spiral_starts_at_center() {
        let lattice = HexLattice::new(2);
        let spiral = lattice.coords_spiral();
        assert_eq!(spiral[0], (0, 0), "la espiral debe empezar en el centro");
    }

    #[test]
    fn test_hex_distance() {
        assert_eq!(HexLattice::hex_distance((0, 0)), 0);
        assert_eq!(HexLattice::hex_distance((1, 0)), 1);
        assert_eq!(HexLattice::hex_distance((2, -1)), 2);
        assert_eq!(HexLattice::hex_distance((-3, 1)), 3);
    }

    #[test]
    fn test_phase_boundary_preserved() {
        // Centro sector 10, vecino sector 50 → gran Δφ → no difundir
        let mut lattice = HexLattice::new(1);

        let sector10 = CrystalLattice::sector_snap(SPA::from_raw(10 * SECTOR_WIDTH_RAW));
        let sector50 = CrystalLattice::sector_snap(SPA::from_raw(50 * SECTOR_WIDTH_RAW));

        lattice.nodes.get_mut(&(0, 0)).unwrap().phase = sector10;
        for &(dq, dr) in &HEX_DIRS {
            if let Some(node) = lattice.nodes.get_mut(&(dq, dr)) {
                node.phase = sector50;
            }
        }

        lattice.stabilize_phase(5);

        assert_eq!(
            lattice.phase((0, 0)).to_raw(), sector10.to_raw(),
            "límite de datos: fase del centro no debe cambiar"
        );
    }
}
