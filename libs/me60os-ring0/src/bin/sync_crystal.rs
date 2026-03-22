use me60os_core::time_crystal::IsochronousClock;
use me60os_core::ebpf_cortex_bridge::EbpfCortexBridge;
use me60os_core::buffer_system::ResonantBuffer;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚛️  PAI-60 Time Crystal Synchronization...");
    
    // 1. Initialize Clock
    let clock = IsochronousClock::new();
    
    // 2. Initialize Bridge
    let bridge = EbpfCortexBridge::new(Arc::new(ResonantBuffer::new()));
    
    // 3. Sync to Kernel
    let maps_path = "/sys/fs/bpf/sentinel";
    bridge.sync_clock_to_kernel(&clock, maps_path)?;
    
    println!("✅ Synchronization Complete.");
    println!("   Start Time: {} ns", clock.start_time_ns);
    println!("   Tick Interval: {} ns", clock.base_interval_ns);
    
    Ok(())
}
