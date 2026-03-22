//! # 🧠 ME-60OS Neural Memory — Pipeline Completa
//!
//! Integra las tres capas del sistema de memoria:
//! ```text
//! Ring 0 (eBPF)
//!     └─► RawCortexEvent
//!              └─► NeuronLayer (LIF + Hebbian + Homeostasis)
//!                       └─► CrystalLattice (propagación resonante)
//!                                └─► CrystalStore (mmap → disco .crystal)
//! ```
//!
//! Todo en aritmética Base-60 pura (SPA). Sin floats.

use crate::crystal_store::CrystalStore;
use crate::ebpf_cortex_bridge::RawCortexEvent;
use crate::lattice::CrystalLattice;
use crate::lif_neuron::NeuronLayer;
use crate::pai60_lib::pai60_divide;
use crate::shm_bridge::SharedBuffer;
use crate::spa::SPA;

// Tamaño de la red: 12 nodos (resonancia zodiacal S60)
const LATTICE_SIZE: usize = 12;

// Persistir cada 60 eventos (un ciclo YHWH)
const PERSIST_EVERY: usize = 60;

pub struct NeuralMemory {
    pub processed: usize,
    pub lattice: CrystalLattice,
    pub neurons: NeuronLayer,
    store: Option<CrystalStore>,
    hot: Option<SharedBuffer>,
}

impl NeuralMemory {
    /// Memoria volátil (sin persistencia a disco).
    pub fn new() -> Self {
        Self {
            processed: 0,
            lattice: CrystalLattice::new(LATTICE_SIZE),
            neurons: NeuronLayer::new(LATTICE_SIZE),
            store: None,
            hot: None,
        }
    }

    /// Memoria con Hot Layer L1 — escribe estado del lattice en /dev/shm tras cada evento.
    /// Permite lectura ultrarrápida desde otros procesos sin tocar disco.
    pub fn with_hot_memory() -> Self {
        // Layout: 4 bytes magic + 4 bytes node_count + 12 × 12 bytes (i64 amp + u32 pressure)
        let shm_size = 8 + LATTICE_SIZE * 12;
        let hot = SharedBuffer::new("/pai60_lattice".to_string(), shm_size, true).ok();
        if hot.is_some() {
            eprintln!("✅ Hot Memory L1 activa: /dev/shm/pai60_lattice ({} bytes)", shm_size);
        } else {
            eprintln!("⚠️ Hot Memory L1 no disponible, continuando sin ella");
        }
        Self {
            processed: 0,
            lattice: CrystalLattice::new(LATTICE_SIZE),
            neurons: NeuronLayer::new(LATTICE_SIZE),
            store: None,
            hot,
        }
    }

    /// Memoria con Liquid Persistence — restaura el estado del archivo `.crystal`.
    pub fn with_persistence(path: &std::path::Path) -> std::io::Result<Self> {
        let store = CrystalStore::open(path, LATTICE_SIZE)?;
        let lattice = store.load();
        Ok(Self {
            processed: 0,
            lattice,
            neurons: NeuronLayer::new(LATTICE_SIZE),
            store: Some(store),
            hot: None,
        })
    }

    /// Ingiere un evento del Ring 0 y lo procesa a través de la pipeline completa.
    pub fn ingest_event(&mut self, ev: &RawCortexEvent) {
        self.processed += 1;

        // 1. Calcular resonancia desde frecuencia S60 del evento
        //    60 / freq → si freq es número regular, resonancia alta
        let base_60 = SPA::new(60, 0, 0, 0, 0);
        let freq = ev.frequency.max(1); // evitar división por cero
        let resonance = pai60_divide(base_60, freq).unwrap_or(SPA::zero());

        // 2. Señal de amplitud del cristal (escalada a SPA)
        let amplitude_spa = SPA::from_raw(ev.amplitude_raw as i64);

        // 3. Construir vector de entrada para la capa neuronal
        //    El nodo objetivo se determina por frequency % LATTICE_SIZE (distribución anular)
        let target_node = (ev.frequency as usize) % LATTICE_SIZE;
        let mut inputs = vec![SPA::zero(); LATTICE_SIZE];
        // Combinar resonancia y amplitud en la señal de entrada
        inputs[target_node] = resonance + amplitude_spa * SPA::new(0, 1, 0, 0, 0);

        // 4. Procesar a través de la capa LIF (disparo + Hebbian + Homeostasis)
        let spikes = self.neurons.step(&inputs);

        // 5. Inyectar presión en el lattice solo en los nodos que dispararon
        let pressure = resonance.to_raw() / SPA::SCALE_0;
        for (i, &fired) in spikes.iter().enumerate() {
            if fired {
                self.lattice.inject(i, pressure);
            }
        }

        // 6. Evolucionar la red resonante (propagación entre cristales)
        self.lattice.step();

        // 7. Escribir snapshot en Hot Memory L1 (/dev/shm) si está disponible
        self.write_hot_snapshot();

        // 8. Persistencia cada PERSIST_EVERY eventos
        if self.processed % PERSIST_EVERY == 0 {
            if let Some(ref mut store) = self.store {
                store.save(&self.lattice);
                eprintln!(
                    "💾 Crystal guardado: {} eventos procesados | Nodos activos: {}",
                    self.processed,
                    self.neurons.active_count()
                );
            }
        }

        eprintln!(
            "🧠 Mem[{}]: freq={} Hz → nodo={} | disparos={}/{} | reg={}",
            self.processed,
            ev.frequency,
            target_node,
            spikes.iter().filter(|&&s| s).count(),
            LATTICE_SIZE,
            ev.is_regular != 0,
        );
    }

    /// Serializa el estado actual del lattice en el SharedBuffer.
    /// Layout: [magic: 4B "PAI1"] [node_count: 4B i32] [12 × (amplitude: i64 + pressure: i32)]
    fn write_hot_snapshot(&self) {
        let hot = match &self.hot {
            Some(h) => h,
            None => return,
        };
        let mut buf = Vec::with_capacity(8 + LATTICE_SIZE * 12);
        // Magic
        buf.extend_from_slice(b"PAI1");
        // node_count
        buf.extend_from_slice(&(LATTICE_SIZE as i32).to_le_bytes());
        // Nodos del lattice
        for crystal in &self.lattice.crystals {
            let amp_raw = crystal.amplitude.to_raw();
            buf.extend_from_slice(&amp_raw.to_le_bytes());         // i64 (8 bytes)
            buf.extend_from_slice(&(amp_raw as i32).to_le_bytes()); // i32 saturado (4 bytes)
        }
        let _ = hot.write(0, &buf);
    }

    /// Estado actual del lattice como vector de strings (para diagnóstico).
    pub fn get_lattice_state(&self) -> Vec<String> {
        self.lattice
            .crystals
            .iter()
            .map(|c| format!("{}: amp={}", c.name, c.amplitude))
            .collect()
    }

    /// Fuerza un guardado inmediato (para shutdown limpio).
    pub fn flush(&mut self) {
        if let Some(ref mut store) = self.store {
            store.save(&self.lattice);
            eprintln!("💾 Crystal flushed: {} eventos totales", self.processed);
        }
    }
}

impl Default for NeuralMemory {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ebpf_cortex_bridge::RawCortexEvent;

    fn make_event(freq: u32, amplitude: u64, is_regular: u8) -> RawCortexEvent {
        RawCortexEvent {
            timestamp_ns: 1_000_000_000,
            frequency: freq,
            _gap: 0,
            amplitude_raw: amplitude,
            is_regular,
            hex_q: 0,
            hex_r: 0,
            _padding: [0; 5],
        }
    }

    #[test]
    fn test_ingest_evento_regular() {
        let mut mem = NeuralMemory::new();
        // freq=60 es S60 regular → resonancia = 60/60 = 1.0
        let ev = make_event(60, 1_000_000, 1);
        mem.ingest_event(&ev);
        assert_eq!(mem.processed, 1);
    }

    #[test]
    fn test_ingest_multiples_eventos() {
        let mut mem = NeuralMemory::new();
        for i in 1..=12u32 {
            mem.ingest_event(&make_event(i * 5, i as u64 * 100_000, 1));
        }
        assert_eq!(mem.processed, 12);
    }

    #[test]
    fn test_persistencia_roundtrip() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_neural_mem.crystal");

        // Procesar eventos y guardar
        {
            let mut mem = NeuralMemory::with_persistence(&path).unwrap();
            for i in 0..60u32 {
                mem.ingest_event(&make_event(60, i as u64 * 10_000, 1));
            }
            // Tras 60 eventos, debe haber hecho auto-save
        }

        // Restaurar y verificar que el lattice tiene estado
        let mem2 = NeuralMemory::with_persistence(&path).unwrap();
        let state = mem2.get_lattice_state();
        assert_eq!(state.len(), LATTICE_SIZE);

        let _ = std::fs::remove_file(&path);
    }
}
