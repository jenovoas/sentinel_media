//! # 🛰️ eBPF → Cortex Bridge 🛰️
//!
//! Polls BPF ring buffer for kernel events and feeds them directly
//! to the Cortex neural network.
//!
//! Architecture:
//! ```text
//! Kernel eBPF -> Ring Buffer -> This Bridge -> ResonantBuffer -> Cortex
//! ```
//!
//! Performance:
//! - Zero-copy from kernel to userspace
//! - Lock-free ring buffer
//! - Sub-millisecond latency

use crate::buffer_system::ResonantBuffer;
use crate::spa::SPA;
use crate::maat_regulator::{MaatStabilizer, MaatStatus};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// Event type constants (must match cortex_events.h)
#[allow(dead_code)]
const EVENT_FILE_BLOCKED: u32 = 1;
#[allow(dead_code)]
const EVENT_EXEC_BLOCKED: u32 = 2;
#[allow(dead_code)]
const EVENT_FILE_ALLOWED: u32 = 3;
#[allow(dead_code)]
const EVENT_EXEC_ALLOWED: u32 = 4;
#[allow(dead_code)]
const EVENT_NETWORK_BURST: u32 = 5;
#[allow(dead_code)]
const EVENT_NETWORK_NORMAL: u32 = 6;

// Severity levels
#[allow(dead_code)]
const SEVERITY_LOW: u8 = 0;
#[allow(dead_code)]
const SEVERITY_MEDIUM: u8 = 1;
#[allow(dead_code)]
const SEVERITY_HIGH: u8 = 2;
#[allow(dead_code)]
const SEVERITY_CRITICAL: u8 = 3;


/// Evento crudo del Ring 0 — layout idéntico a `cortex_event` en guardian_alpha.bpf.c.
/// El compilador C inserta 4 bytes de padding entre `frequency` (u32) y `amplitude_raw` (u64)
/// por alineación natural. `#[repr(C)]` replica ese comportamiento → 32 bytes exactos.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct RawCortexEvent {
    pub timestamp_ns: u64,
    pub frequency: u32,
    pub _gap: u32,          // padding implícito del compilador C (alineación de u64)
    pub amplitude_raw: u64,
    pub is_regular: u8,
    pub hex_q: u8,          // Coordenada Axial Q
    pub hex_r: u8,          // Coordenada Axial R
    pub _padding: [u8; 5],
}

/// Cortex Event (Python-friendly, unpacked)
#[derive(Debug, Clone)]
pub struct CortexEvent {
    pub timestamp_ns: u64,
    pub frequency: u32,
    pub amplitude_raw: u64,
    pub is_regular: bool,
    pub hex_q: u8,
    pub hex_r: u8,
}

impl CortexEvent {
    pub fn new(
        timestamp_ns: u64,
        frequency: u32,
        amplitude_raw: u64,
        is_regular: bool,
        hex_q: u8,
        hex_r: u8,
    ) -> Self {
        Self {
            timestamp_ns,
            frequency,
            amplitude_raw,
            is_regular,
            hex_q,
            hex_r,
        }
    }
}

/// eBPF to Cortex Bridge
pub struct EbpfCortexBridge {
    buffer: Arc<ResonantBuffer>,
    #[allow(dead_code)]
    neuron_map: NeuronMapping,
    pub maat: MaatStabilizer,
}

/// Maps event types to specific neurons
#[allow(dead_code)]
struct NeuronMapping {
    file_blocked: usize,
    exec_blocked: usize,
    file_allowed: usize,
    exec_allowed: usize,
    network_burst: usize,
    network_normal: usize,
}

impl Default for NeuronMapping {
    fn default() -> Self {
        Self {
            file_blocked: 0,     // Neuron 0: File access threats
            exec_blocked: 64,    // Neuron 64: Execution threats
            file_allowed: 128,   // Neuron 128: Normal file ops
            exec_allowed: 192,   // Neuron 192: Normal exec ops
            network_burst: 256,  // Neuron 256: Network anomalies
            network_normal: 320, // Neuron 320: Normal network
        }
    }
}

impl EbpfCortexBridge {
    pub fn new(buffer: Arc<ResonantBuffer>) -> Self {
        Self {
            buffer,
            neuron_map: NeuronMapping::default(),
            maat: MaatStabilizer::new(),
        }
    }

    /// Synchronizes the Time Crystal clock and YHWH pattern to the kernel maps.
    pub fn sync_clock_to_kernel(
        &self,
        clock: &crate::time_crystal::IsochronousClock,
        maps_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use libbpf_rs::{MapHandle, MapCore};
        use std::path::Path;

        let path = Path::new(maps_path);
        let anchor_path = path.join("time_anchor_map");
        let tensor_path = path.join("yhwh_tensor_map");

        // 1. Sync Time Anchor
        let anchor_map = MapHandle::from_pinned_path(&anchor_path)?;
        let mut anchor_data = [0u8; 24]; // 3 * u64
        anchor_data[0..8].copy_from_slice(&(clock.start_time_ns.to_le_bytes()));
        anchor_data[8..16].copy_from_slice(&(clock.base_interval_ns.to_le_bytes()));
        anchor_data[16..24].copy_from_slice(&(clock.last_leap_ns.to_le_bytes()));

        let key_zero = 0u32.to_le_bytes();
        anchor_map.update(&key_zero, &anchor_data, libbpf_rs::MapFlags::ANY)?;

        // 2. Sync YHWH Tensor
        let tensor_map = MapHandle::from_pinned_path(&tensor_path)?;
        for i in 0..4 {
            let key = (i as u32).to_le_bytes();
            let val = clock.phase_intervals[i].to_le_bytes();
            tensor_map.update(&key, &val, libbpf_rs::MapFlags::ANY)?;
        }

        eprintln!("💎 Time Crystal Anchored to Kernel: start={}ns", clock.start_time_ns);
        Ok(())
    }

    /// Start polling ring buffer (blocking, run in dedicated thread)
    ///
    /// Polls the eBPF ring buffer for cortex events and feeds them to the neural network.
    /// This function blocks and should be run in a dedicated thread.
    ///
    /// # Arguments
    /// * `ringbuf_path` - Path to the pinned ring buffer map (e.g., "/sys/fs/bpf/ai_guardian_maps/cortex_events")
    ///
    /// # Returns
    /// Returns an error if the ring buffer cannot be opened or polling fails
    pub fn start_polling(&self, ringbuf_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        use libbpf_rs::{MapHandle, RingBufferBuilder};
        use std::path::Path;

        eprintln!("🛰️ eBPF Cortex Bridge: Starting ring buffer polling...");
        eprintln!("   Ring buffer: {}", ringbuf_path);

        // Check if path is a directory (old API) or file (new API)
        let map_path = if Path::new(ringbuf_path).is_dir() {
            // Directory provided, append cortex_events
            format!("{}/cortex_events", ringbuf_path)
        } else {
            // Direct path to ring buffer
            ringbuf_path.to_string()
        };

        eprintln!("   Using map path: {}", map_path);

        // Open the pinned ring buffer map
        let map = MapHandle::from_pinned_path(&map_path)
            .map_err(|e| format!("Failed to open pinned map at {}: {}", map_path, e))?;

        eprintln!("✅ Ring buffer map opened successfully");

        // Create ring buffer with callback
        let mut builder = RingBufferBuilder::new();

        // Clone self for the closure
        let buffer = self.buffer.clone();

        builder.add(&map, move |data: &[u8]| -> i32 {
            // Handle event in the callback
            if data.len() < std::mem::size_of::<RawCortexEvent>() {
                eprintln!("⚠️ Invalid event size: {}", data.len());
                return 0;
            }

            // SAFETY: We've verified the size matches RawCortexEvent
            let event: RawCortexEvent =
                unsafe { std::ptr::read_unaligned(data.as_ptr() as *const RawCortexEvent) };

            // Copy fields to avoid unaligned references in packed struct
            let freq = event.frequency;
            let amp = event.amplitude_raw;
            let is_reg = event.is_regular;
            let q = event.hex_q;
            let r = event.hex_r;

            // Sanitization: If amplitude is 0, it was flagged as Dissonance in Ring 0
            if amp == 0 {
                eprintln!("🛑 DISSONANCE DETECTED: Hallucinated Event at Freq {}", freq);
                return 0;
            }

            // Convert amplitude signal to SPA
            let amplitude_spa = SPA::from_raw(amp as i64);

            // Push to resonant buffer
            buffer.push(amplitude_spa);

            // Log event with resonance status and hex coordinates
            eprintln!(
                "📡 eBPF Frequency: {} Hz | Amplitude: {} | Regular: {} | Hex: ({}, {})",
                freq, amplitude_spa, is_reg != 0, q, r
            );

            0 // Return 0 to continue polling
        })?;

        let ringbuf = builder.build()?;

        eprintln!("✅ Ring buffer polling active! Waiting for kernel events...");
        eprintln!("   Press Ctrl+C to stop");

        // Poll loop - blocks here
        loop {
            match ringbuf.poll(Duration::from_millis(100)) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("⚠️ Ring buffer poll error: {}", e);
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }

    /// Consumes events for a limited time (Energy Saving Mode)
    #[allow(dead_code)]
    pub fn consume_pulse(
        &self,
        ringbuf_path: &str,
        timeout_ms: u64,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        use libbpf_rs::{MapHandle, RingBufferBuilder};
        use std::path::Path;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        use std::time::Duration;

        let path = Path::new(ringbuf_path);

        let map_path = if path.display().to_string().ends_with("cortex_events") {
            path.to_path_buf()
        } else if path.is_dir() {
            path.join("cortex_events")
        } else {
            path.to_path_buf()
        };

        let map = MapHandle::from_pinned_path(&map_path)
            .map_err(|e| format!("Failed to open pinned map at {}: {}", map_path.display(), e))?;

        let mut builder = RingBufferBuilder::new();
        let buffer = self.buffer.clone();

        let event_count = Arc::new(AtomicUsize::new(0));
        let count_clone = event_count.clone();

        builder
            .add(&map, move |data: &[u8]| -> i32 {
                if data.len() < std::mem::size_of::<RawCortexEvent>() {
                    return 0;
                }
                let event: RawCortexEvent =
                    unsafe { std::ptr::read_unaligned(data.as_ptr() as *const RawCortexEvent) };
                
                if event.amplitude_raw > 0 {
                    let amplitude_spa = SPA::from_raw(event.amplitude_raw as i64);
                    buffer.push(amplitude_spa);
                    count_clone.fetch_add(1, Ordering::Relaxed);
                }
                0
            })
            .map_err(|e| format!("Failed to add map to ring buffer: {}", e))?;

        let ringbuf = builder
            .build()
            .map_err(|e| format!("Could not build ringbuf: {}", e))?;

        // Poll and ignore return value (since it might be unit)
        let _ = ringbuf.poll(Duration::from_millis(timeout_ms));

        Ok(event_count.load(Ordering::Relaxed))
    }

    /// Handle incoming event from ring buffer
    #[allow(dead_code)]
    fn handle_event(&self, data: &[u8]) {
        if data.len() < std::mem::size_of::<RawCortexEvent>() {
            eprintln!("⚠️ Invalid event size: {}", data.len());
            return;
        }

        // SAFETY: We've verified the size matches RawCortexEvent
        let event: RawCortexEvent =
            unsafe { std::ptr::read_unaligned(data.as_ptr() as *const RawCortexEvent) };

        // Copy fields
        let freq = event.frequency;
        let amp = event.amplitude_raw;
        let is_reg = event.is_regular;
        let q = event.hex_q;
        let r = event.hex_r;

        if amp == 0 { return; }

        let amp_spa = SPA::from_raw(amp as i64);
        self.buffer.push(amp_spa);

        eprintln!(
            "📡 eBPF Frequency: {} Hz | Amplitude: {} | Regular: {} | Hex: ({}, {})",
            freq, amp_spa, is_reg != 0, q, r
        );
    }

    /// Map frequency to neuron ID (Simplified for PAI-60)
    #[allow(dead_code)]
    fn map_event_to_neuron(&self, freq: u32) -> usize {
        (freq as usize) % 1024 
    }

    /// Reads the current state of the 19-node hexagonal lattice (rings=2) from the kernel.
    /// This is the "Fractal Snapshot" of the system resonance.
    pub fn get_resonance_lattice(&self, maps_path: &str) -> Result<Vec<(u32, u64, u64, u32)>, Box<dyn std::error::Error>> {
        use libbpf_rs::{MapHandle, MapCore};
        use std::path::Path;

        let lattice_path = Path::new(maps_path).join("hex_lattice_map");
        let map = MapHandle::from_pinned_path(&lattice_path)?;
        
        let mut results = Vec::with_capacity(19);
        
        for i in 0..19 {
            let key = (i as u32).to_le_bytes();
            if let Ok(Some(data)) = map.lookup(&key, libbpf_rs::MapFlags::ANY) {
                // struct hex_node: pressure (u64), phase (u64), last_fire (u64), hits (u32) = 28 bytes
                // aligned to 32 bytes by compiler.
                if data.len() >= 28 {
                    let pressure = u64::from_le_bytes(data[0..8].try_into().map_err(|e| format!("Byte conversion error: {}", e))?);
                    let phase = u64::from_le_bytes(data[8..16].try_into().map_err(|e| format!("Byte conversion error: {}", e))?);
                    let _last_fire = u64::from_le_bytes(data[16..24].try_into().map_err(|e| format!("Byte conversion error: {}", e))?);
                    let hits = u32::from_le_bytes(data[24..28].try_into().map_err(|e| format!("Byte conversion error: {}", e))?);
                    results.push((i, pressure, phase, hits));
                }
            }
        }
        
        Ok(results)
    }

    /// Calculates the 'Truth' score of the hexagonal lattice.
    /// Truth = Average (Pressure / Firing_Threshold) across active nodes.
    pub fn calculate_lattice_truth(&self, resonance_data: &[(u32, u64, u64, u32)]) -> SPA {
        if resonance_data.is_empty() {
            return SPA::one(); // Start pure
        }

        let mut total_truth = SPA::zero();
        let fire_threshold = SPA::new(1, 0, 0, 0, 0); // 1.0 unit

        for &(_idx, pressure, _phase, _hits) in resonance_data {
            let pressure_spa = SPA::from_raw(pressure as i64);
            // Normalized truth for this node: if pressure > threshold, it's 'overloaded'
            // We want truth to be 1.0 when perfectly resonant.
            // Simplified: Truth drops if pressure is low or too high?
            // User logic: "If Truth < 95% (Dissonance), SACRIFICE VELOCITY"
            // Let's assume Truth is a signal quality derived from lattice coherence.
            let node_truth = if pressure_spa > fire_threshold {
                fire_threshold / pressure_spa
            } else {
                pressure_spa / fire_threshold
            };
            
            total_truth = total_truth + node_truth;
        }

        total_truth / (resonance_data.len() as i64)
    }

    /// Regulates system speed based on kernel lattice state.
    pub fn regulate_system(&self, current_speed: SPA, maps_path: &str) -> (SPA, MaatStatus) {
        let lattice = self.get_resonance_lattice(maps_path).unwrap_or_default();
        let truth = self.calculate_lattice_truth(&lattice);
        
        self.maat.regulate(truth, current_speed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_size() {
        // Verify struct size matches C definition (32 bytes)
        assert_eq!(std::mem::size_of::<RawCortexEvent>(), 32);
    }

    #[test]
    fn test_neuron_mapping() {
        let bridge = EbpfCortexBridge::new(Arc::new(ResonantBuffer::new()));

        // map_event_to_neuron recibe frecuencia S60 y devuelve freq % 1024
        assert_eq!(bridge.map_event_to_neuron(60), 60);
        assert_eq!(bridge.map_event_to_neuron(0), 0);
        assert_eq!(bridge.map_event_to_neuron(1024), 0);
    }

    #[test]
    fn test_handle_event() {
        let bridge = EbpfCortexBridge::new(Arc::new(ResonantBuffer::new()));

        let event = RawCortexEvent {
            timestamp_ns: 1234567890,
            frequency: 60,           // número S60 (regular)
            _gap: 0,
            amplitude_raw: 39657600000,
            is_regular: 1,
            hex_q: 12,
            hex_r: 10,
            _padding: [0; 5],
        };

        let mut data = [0u8; 32];
        unsafe {
            std::ptr::write(data.as_mut_ptr() as *mut RawCortexEvent, event);
        }

        bridge.handle_event(&data);
        assert!(bridge.buffer.pop().is_some());
    }
}
