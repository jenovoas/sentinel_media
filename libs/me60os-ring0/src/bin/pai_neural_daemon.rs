// src/bin/pai_neural_daemon.rs
//! PAI‑60 Neural Daemon
//! Polls eBPF ring buffer, converts entropy to SPA, and updates neural memory.

use libbpf_rs::{MapHandle, RingBufferBuilder};
use me60os_core::ebpf_cortex_bridge::RawCortexEvent;
use me60os_core::neural_memory::NeuralMemory;
use std::path::Path;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🛡️ ME‑60OS: PAI‑60 Neural Daemon Starting...");

    // 1. Initialize Neural Memory with Hot Layer L1 (/dev/shm)
    let mut memory = NeuralMemory::with_hot_memory();

    // 2. Open eBPF Ring Buffer
    // Default path for pinned maps
    let ringbuf_path = "/sys/fs/bpf/sentinel/cortex_events";
    if !Path::new(ringbuf_path).exists() {
        eprintln!("❌ Error: Ring buffer map not found at {}", ringbuf_path);
        eprintln!(
            "   Please ensure eBPF program is loaded: sudo ./ebpf/scripts/load_ai_guardian.sh"
        );
        std::process::exit(1);
    }

    let map = MapHandle::from_pinned_path(ringbuf_path)
        .map_err(|e| format!("Failed to open pinned map at {}: {}", ringbuf_path, e))?;

    println!("✅ Ring buffer map opened: {}", ringbuf_path);

    // 3. Setup Ring Buffer Polling
    let mut builder = RingBufferBuilder::new();

    // We use a closure that captures the memory.
    // Note: NeuralMemory is not Thread-Safe by default, but RingBuffer callback
    // runs in the polling thread synchronously for each event.
    builder.add(&map, move |data: &[u8]| -> i32 {
        if data.len() < std::mem::size_of::<RawCortexEvent>() {
            return 0;
        }

        // SAFETY: Size is verified.
        let raw_ev: RawCortexEvent =
            unsafe { std::ptr::read_unaligned(data.as_ptr() as *const RawCortexEvent) };

        if raw_ev.amplitude_raw == 0 { return 0; } // Dissonance rejection

        // Pipeline directa: RawCortexEvent → NeuralMemory
        memory.ingest_event(&raw_ev);

        0 // Continue polling
    })?;

    let ringbuf = builder.build()?;

    println!("🚀 Daemon Active. Polling for events...");

    // 4. Main Loop
    loop {
        match ringbuf.poll(Duration::from_millis(100)) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("⚠️ Poll error: {}", e);
                std::thread::sleep(Duration::from_millis(500));
            }
        }
    }
}
