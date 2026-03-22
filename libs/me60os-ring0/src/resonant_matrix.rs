//! # 🕸️ RESONANT LATTICE - RUST PURE 🕸️
use crate::isochronous_oscillator::IsochronousOscillator;
use crate::spa::SPA;
use crate::shm_bridge::SharedBuffer;
use std::io::{Error, ErrorKind, Result};

pub struct ResonantMatrix {
    pub crystals: Vec<IsochronousOscillator>,
    pub coupling_factor: SPA,
    pub dt: SPA,
}

impl ResonantMatrix {
    pub fn new(size: usize) -> Self {
        let crystals = (0..size)
            .map(|i| IsochronousOscillator::new(&format!("Node-{}", i)))
            .collect();

        Self {
            crystals,
            coupling_factor: SPA::new(0, 10, 0, 0, 0),
            dt: SPA::new(0, 0, 1, 0, 0),
        }
    }

    pub fn size(&self) -> usize {
        self.crystals.len()
    }

    pub fn step(&mut self) {
        let size = self.crystals.len();
        if size < 2 { return; }

        let mut transfers: Vec<SPA> = vec![SPA::zero(); size];

        for i in 0..(size - 1) {
            let amp_i = self.crystals[i].get_amplitude();
            let amp_next = self.crystals[i + 1].get_amplitude();
            let diff = amp_i - amp_next;
            let flow = (diff * self.coupling_factor) / SPA::new(1, 0, 0, 0, 0);

            transfers[i] = transfers[i] - flow;
            transfers[i + 1] = transfers[i + 1] + flow;
        }

        for i in 0..size {
            self.crystals[i].amplitude = self.crystals[i].amplitude + transfers[i];
            self.crystals[i].oscillate(self.dt);
        }
    }

    pub fn inject(&mut self, index: usize, pressure: i64) {
        if index < self.crystals.len() {
            self.crystals[index].transduce_pulse(pressure);
        }
    }

    pub fn get_amplitudes(&self) -> Vec<SPA> {
        self.crystals.iter().map(|c| c.get_amplitude()).collect()
    }

    pub fn sync_to_shm(&self, buffer: &mut SharedBuffer) -> Result<()> {
        let crystal_size = std::mem::size_of::<IsochronousOscillator>();
        let total_size = crystal_size * self.crystals.len();
        
        let shm_ptr = buffer.as_ptr();
        if shm_ptr.is_null() {
            return Err(Error::new(ErrorKind::Other, "SHM ptr null"));
        }

        if buffer.size < total_size {
            return Err(Error::new(ErrorKind::Other, "Buffer too small"));
        }

        unsafe {
            let src_ptr = self.crystals.as_ptr() as *const u8;
            std::ptr::copy_nonoverlapping(src_ptr, shm_ptr, total_size);
        }
        Ok(())
    }

    pub fn load_from_shm(&mut self, buffer: &SharedBuffer) -> Result<()> {
        let crystal_size = std::mem::size_of::<IsochronousOscillator>();
        let total_size = crystal_size * self.crystals.len();
        
        let shm_ptr = buffer.as_ptr();
        if shm_ptr.is_null() {
             return Err(Error::new(ErrorKind::Other, "SHM ptr null"));
        }
        
        if buffer.size < total_size {
             return Err(Error::new(ErrorKind::Other, "Buffer too small"));
        }

        unsafe {
            let dst_ptr = self.crystals.as_mut_ptr() as *mut u8;
            std::ptr::copy_nonoverlapping(shm_ptr, dst_ptr, total_size);
        }
        Ok(())
    }
}
