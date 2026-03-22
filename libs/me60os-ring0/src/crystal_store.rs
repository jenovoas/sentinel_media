//! # 💾 Liquid Persistence — Crystal Store
//!
//! Persistencia del estado del lattice mediante `mmap` a archivos `.crystal`.
//!
//! ## Layout del archivo (binario, little-endian)
//! ```text
//! Offset  Size  Campo
//! ------  ----  -----
//! 0       8     Magic: b"CRYSTAL1"
//! 8       4     node_count: u32
//! 12      4     padding: u32 (reservado, cero)
//! 16      N*16  Nodos: [amplitude: i64, phase: i64] por nodo
//! ```
//!
//! ## Garantías
//! - Todo el estado es SPA raw (i64 LE) → sin floats, sin decimales
//! - Escritura atómica por flush de página de kernel
//! - Restauración instantánea al arrancar: `load()` → `CrystalLattice`

use crate::lattice::CrystalLattice;
use crate::spa::SPA;
use memmap2::MmapMut;
use std::fs::OpenOptions;
use std::path::Path;

const MAGIC: &[u8; 8] = b"CRYSTAL1";
const HEADER_SIZE: usize = 16; // magic(8) + count(4) + pad(4)
const NODE_SIZE: usize = 16;   // amplitude(8) + phase(8)

/// Store de cristales con persistencia mmap.
pub struct CrystalStore {
    mmap: MmapMut,
    pub node_count: usize,
}

impl CrystalStore {
    /// Abre o crea un archivo `.crystal` con mmap.
    /// Si el archivo es nuevo, escribe la cabecera y lo inicializa a cero.
    pub fn open(path: &Path, node_count: usize) -> std::io::Result<Self> {
        let file_size = HEADER_SIZE + node_count * NODE_SIZE;

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        file.set_len(file_size as u64)?;

        let mut mmap = unsafe { MmapMut::map_mut(&file)? };

        if &mmap[0..8] != MAGIC {
            // Archivo nuevo: escribir cabecera
            mmap[0..8].copy_from_slice(MAGIC);
            mmap[8..12].copy_from_slice(&(node_count as u32).to_le_bytes());
            mmap[12..16].copy_from_slice(&[0u8; 4]);
            mmap.flush()?;
            eprintln!("💾 Crystal Store: archivo nuevo creado en {:?}", path);
        } else {
            eprintln!("💾 Crystal Store: estado existente cargado desde {:?}", path);
        }

        Ok(Self { mmap, node_count })
    }

    /// Escribe el estado actual del lattice en el mmap (flush a disco).
    pub fn save(&mut self, lattice: &CrystalLattice) {
        let nodes = lattice.crystals.len().min(self.node_count);
        for i in 0..nodes {
            let offset = HEADER_SIZE + i * NODE_SIZE;
            let amp = lattice.crystals[i].amplitude.to_raw();
            let phase = lattice.crystals[i].phase.to_raw();
            self.mmap[offset..offset + 8].copy_from_slice(&amp.to_le_bytes());
            self.mmap[offset + 8..offset + 16].copy_from_slice(&phase.to_le_bytes());
        }
        if let Err(e) = self.mmap.flush() {
            eprintln!("⚠️ Crystal Store: error en flush: {}", e);
        }
    }

    /// Restaura un `CrystalLattice` desde el mmap.
    pub fn load(&self) -> CrystalLattice {
        let mut lattice = CrystalLattice::new(self.node_count);
        for i in 0..self.node_count {
            let offset = HEADER_SIZE + i * NODE_SIZE;
            let amp = self.read_i64(offset);
            let phase = self.read_i64(offset + 8);
            lattice.crystals[i].amplitude = SPA::from_raw(amp);
            lattice.crystals[i].phase = SPA::from_raw(phase);
        }
        lattice
    }

    /// Devuelve la amplitud de un nodo individual (lectura directa del mmap).
    pub fn read_amplitude(&self, node: usize) -> SPA {
        if node >= self.node_count {
            return SPA::zero();
        }
        SPA::from_raw(self.read_i64(HEADER_SIZE + node * NODE_SIZE))
    }

    fn read_i64(&self, offset: usize) -> i64 {
        let bytes: [u8; 8] = self.mmap[offset..offset + 8].try_into().unwrap_or([0; 8]);
        i64::from_le_bytes(bytes)
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resonant_crystal::SovereignCrystal;

    #[test]
    fn test_crystal_store_roundtrip() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_memory.crystal");

        // Crear lattice con datos
        let mut lattice = CrystalLattice::new(4);
        lattice.crystals[0].amplitude = SPA::new(1, 30, 0, 0, 0);
        lattice.crystals[1].amplitude = SPA::new(0, 45, 0, 0, 0);
        lattice.crystals[2].phase = SPA::new(0, 15, 0, 0, 0);

        // Guardar
        {
            let mut store = CrystalStore::open(&path, 4).unwrap();
            store.save(&lattice);
        }

        // Restaurar en nuevo store
        let store2 = CrystalStore::open(&path, 4).unwrap();
        let restored = store2.load();

        assert_eq!(
            restored.crystals[0].amplitude.to_raw(),
            lattice.crystals[0].amplitude.to_raw(),
            "amplitud nodo 0 debe persistir"
        );
        assert_eq!(
            restored.crystals[1].amplitude.to_raw(),
            lattice.crystals[1].amplitude.to_raw(),
            "amplitud nodo 1 debe persistir"
        );
        assert_eq!(
            restored.crystals[2].phase.to_raw(),
            lattice.crystals[2].phase.to_raw(),
            "fase nodo 2 debe persistir"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_magic_detecta_archivo_existente() {
        let dir = std::env::temp_dir();
        let path = dir.join("test_magic.crystal");

        let lattice = CrystalLattice::new(2);
        {
            let mut s = CrystalStore::open(&path, 2).unwrap();
            s.save(&lattice);
        }
        // Segunda apertura no debe sobrescribir magic
        let s2 = CrystalStore::open(&path, 2).unwrap();
        assert_eq!(&s2.mmap[0..8], MAGIC);

        let _ = std::fs::remove_file(&path);
    }
}
