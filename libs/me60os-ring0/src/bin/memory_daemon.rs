//! # 💎 PAI-60 Memory Daemon
//!
//! Pipeline completa: Ring 0 (eBPF) → NeuronLayer (LIF) → CrystalLattice → disco (.crystal)
//!
//! ## Uso
//! ```bash
//! sudo ./memory_daemon [ruta_crystal]
//! ```
//! Por defecto: `/var/lib/pai60/memory.crystal`
//!
//! ## Flujo
//! 1. Restaura estado del lattice desde archivo `.crystal` (si existe)
//! 2. Conecta al ring buffer del Ring 0 en `/sys/fs/bpf/sentinel/cortex_events`
//! 3. Por cada evento: LIF → disparo → lattice → persist cada 60 eventos
//! 4. Shutdown limpio con Ctrl+C (flush final a disco)

use me60os_core::ebpf_cortex_bridge::EbpfCortexBridge;
use me60os_core::neural_memory::NeuralMemory;
use me60os_core::buffer_system::ResonantBuffer;
use me60os_core::ebpf_cortex_bridge::RawCortexEvent;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

const RING_BUFFER_PATH: &str = "/sys/fs/bpf/sentinel/cortex_events";
const DEFAULT_CRYSTAL_PATH: &str = "/var/lib/pai60/memory.crystal";
// Tick YHWH: 23.9ms (base_interval del IsochronousClock)
const TICK_MS: u64 = 24;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════╗");
    println!("║   💎 PAI-60 Memory Daemon v1.0       ║");
    println!("║   Ring 0 → LIF → Crystal Store       ║");
    println!("╚══════════════════════════════════════╝");

    // Ruta del crystal file (argumento opcional)
    let crystal_path_str = std::env::args().nth(1)
        .unwrap_or_else(|| DEFAULT_CRYSTAL_PATH.to_string());
    let crystal_path = Path::new(&crystal_path_str);

    // Crear directorio si no existe
    if let Some(parent) = crystal_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Inicializar NeuralMemory con persistencia
    let mut memory = match NeuralMemory::with_persistence(crystal_path) {
        Ok(m) => {
            println!("✅ Estado restaurado desde: {}", crystal_path_str);
            m
        }
        Err(e) => {
            eprintln!("⚠️ No se pudo abrir crystal store ({}). Modo volátil.", e);
            NeuralMemory::new()
        }
    };

    println!("🧠 Lattice: {} nodos | Neuronas: activas", 12);

    // Señal de shutdown limpio (Ctrl+C)
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        eprintln!("\n⚡ Shutdown solicitado...");
        r.store(false, Ordering::SeqCst);
    }).unwrap_or_else(|_| eprintln!("⚠️ No se pudo registrar Ctrl+C handler"));

    // Inicializar puente eBPF
    let buffer = Arc::new(ResonantBuffer::new());
    let bridge = EbpfCortexBridge::new(buffer.clone());

    println!("🛰️ Conectando al Ring 0: {}", RING_BUFFER_PATH);
    println!("   Tick: {}ms (YHWH base interval)", TICK_MS);
    println!("   Crystal: {}", crystal_path_str);
    println!("──────────────────────────────────────");

    // Loop principal
    while running.load(Ordering::SeqCst) {
        match bridge.consume_pulse(RING_BUFFER_PATH, TICK_MS) {
            Ok(count) if count > 0 => {
                // Leer eventos del buffer intermediario y procesarlos
                // El bridge ya depositó las amplitudes en ResonantBuffer
                // Reconstruimos RawCortexEvent mínimo desde la amplitud SPA
                while let Some(amplitude_spa) = buffer.pop() {
                    let raw = amplitude_spa.to_raw();
                    // Recuperar frecuencia original desde los componentes SPA
                    let freq = (raw.unsigned_abs() % 100) as u32;
                    let ev = RawCortexEvent {
                        timestamp_ns: current_ns(),
                        frequency: freq.max(1),
                        _gap: 0,
                        amplitude_raw: raw.unsigned_abs().min(u64::MAX),
                        is_regular: is_s60_regular(freq) as u8,
                        hex_q: 0,
                        hex_r: 0,
                        _padding: [0; 5],
                    };
                    memory.ingest_event(&ev);
                }
            }
            Ok(_) => {
                // Sin eventos: dormir un tick YHWH
                std::thread::sleep(Duration::from_millis(TICK_MS));
            }
            Err(e) => {
                eprintln!("⚠️ Ring buffer error: {}. Reintentando en 100ms...", e);
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }

    // Shutdown limpio: flush final
    println!("💾 Guardando estado final...");
    memory.flush();
    println!("✅ Memory Daemon detenido correctamente.");

    Ok(())
}

/// Nanosegundos monotónicos del sistema
fn current_ns() -> u64 {
    let mut ts = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts); }
    (ts.tv_sec as u64) * 1_000_000_000 + (ts.tv_nsec as u64)
}

/// Verifica si un número es S60-regular (5-smooth: solo factores 2, 3, 5)
fn is_s60_regular(mut n: u32) -> bool {
    if n == 0 { return false; }
    for p in [2u32, 3, 5] {
        while n % p == 0 { n /= p; }
    }
    n == 1
}
