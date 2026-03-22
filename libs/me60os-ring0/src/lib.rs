// 🛡️ ME-60OS CORE LIBRARY - RUST PURE 🛡️

pub mod agent_manager;
pub mod bci;
pub mod bio;
pub mod buffer_system;
pub mod cortex;
pub mod ebpf_cortex_bridge;
pub mod neural_memory;
pub mod pai60_lib;
pub mod adm;
pub mod isochronous_oscillator;
pub mod physics;
pub mod qhc;
pub mod resonant_matrix;
pub mod scheduler;
pub mod scv;
pub mod shm_bridge;
pub mod spa;
pub mod spa_complex;
pub mod spa_math;
pub mod time_crystal;
pub mod truth_sync;

// Sistema de Memoria (LIF + Hebbian + Liquid Persistence)
pub mod lif_neuron;
pub mod crystal_store;

// New Quantum Memory Modules (Redundancy check needed later)
pub mod resonant_crystal;
pub mod lattice;
pub mod hex_lattice;
pub mod maat_regulator;

// Re-exports pure Rust types
pub use agent_manager::{AgentManager, AgentSPA, EnergyMonitorAgent};
pub use bci::BCISystem;
pub use bio::SoulVerifier;
pub use buffer_system::ResonantBuffer;
pub use cortex::CortexEngine;
pub use isochronous_oscillator::IsochronousOscillator;
pub use physics::ResonantPhysics;
pub use qhc::QhcTensor;
pub use resonant_matrix::ResonantMatrix;
pub use scv::ScvEngine;
pub use spa::SPA;
pub use spa_complex::ComplexSPA;
pub use spa_math::SPAMath;
pub use resonant_crystal::SovereignCrystal;
pub use lattice::{CrystalLattice, SparseCrystalLattice};
pub use hex_lattice::{HexLattice, HexCoord};
pub use shm_bridge::SharedBuffer;
pub use time_crystal::{IsochronousClock, S60PID};
pub use scheduler::{PortalDetector, AdiabticTaskQueue, QuantumTask, TaskType};
pub use truth_sync::{TruthSyncGuard, PlimptonRatio};
pub use maat_regulator::{MaatStabilizer, MaatStatus};
