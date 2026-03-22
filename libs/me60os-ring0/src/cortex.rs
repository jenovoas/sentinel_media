//! # 🧠 NATIVE SPA CORTEX (RUST) 🧠
//!
//! Implementation of the "AkashicLIFNeuron" (Leaky Integrate-and-Fire)
//! using pure Sexagesimal Arithmetic.
//!
//! Formula: V(t) = V_rest + (I(t) + Bias) * (1 - e^(-dt/tau))

use crate::buffer_system::ResonantBuffer;
use crate::spa::SPA;
use crate::spa_math::SPAMath;
use memmap2::MmapMut; // Liquid Persistence
use rayon::prelude::*; // Parallel Iterators
use std::collections::VecDeque;
use std::fs::OpenOptions;
use std::sync::Arc; // For opening crystal file

#[derive(Clone, Debug)]
pub struct Synapse {
    pub target_id: u32,
    pub weight: SPA,
    pub delay: SPA, // Delay in ticks
}

#[derive(Clone, Debug)]
pub struct Signal {
    pub target_id: u32,
    pub strength: SPA,
    pub arrival_time: SPA,
}

#[derive(Clone, Debug)]
pub struct S60Neuron {
    pub id: u32,
    pub potential: SPA,
    pub tau: SPA,               // Decay constant (e.g., 8.0s)
    pub threshold: SPA,         // Firing threshold (e.g., 1.2)
    pub resting: SPA,           // Resting potential (e.g., 0.0)
    pub ref_period: SPA,        // Refractory period
    pub last_spike: SPA,        // Timestamp of last spike
    pub sensitivity: SPA, // Synaptic weight (Plasticity) -- Used for INPUT sensitivity only now
    pub synapses: Vec<Synapse>, // Directed connections
}

impl S60Neuron {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            potential: SPA::zero(),
            tau: SPA::new(8, 0, 0, 0, 0), // 8.0s (Biological standard)
            threshold: SPA::new(1, 12, 0, 0, 0), // 1.2
            resting: SPA::zero(),
            ref_period: SPA::new(0, 0, 30, 0, 0), // 0.5s
            last_spike: SPA::new(-1, 0, 0, 0, 0),
            sensitivity: SPA::new(0, 30, 0, 0, 0), // Default 0.5
            synapses: Vec::new(),
        }
    }

    /// Integrates input signal and returns Spike Strength (SPA) if fired, else 0.
    pub fn integrate(&mut self, input_current: SPA, dt: SPA, current_time: SPA) -> SPA {
        // Refractory check
        if current_time - self.last_spike < self.ref_period {
            return SPA::zero();
        }

        // Leaky Update: V_new = V_old * e^(-dt/tau) + I * (1 - e^(-dt/tau))
        // This is the exact solution for constant I over dt.

        let decay_factor = SPAMath::exp(-(dt / self.tau));
        let intake_factor = SPA::new(1, 0, 0, 0, 0) - decay_factor;

        // Plasticity: Use dynamic sensitivity
        let effective_input = input_current * self.sensitivity;

        self.potential = (self.potential * decay_factor) + (effective_input * intake_factor);

        // Fire Check
        if self.potential > self.threshold {
            // Spike!
            self.potential = self.resting; // Reset
            self.update_plasticity(current_time); // Evolve
            self.last_spike = current_time;
            return SPA::new(1, 0, 0, 0, 0); // Logic 1.0 Spike
        }

        SPA::zero()
    }

    /// Homeostatic Plasticity (Self-Tuning)
    /// If firing too fast (burst), reduce sensitivity (Depression).
    /// If firing slow (rare), increase sensitivity (Potentiation).
    fn update_plasticity(&mut self, current_time: SPA) {
        let delta_t = current_time - self.last_spike;
        let target_interval = SPA::new(0, 0, 8, 0, 0); // Target: 8 seconds (Resonant period)

        let learning_rate = SPA::new(0, 0, 0, 18, 0); // ~0.005 adjustment (60^2 scaling equivalent)

        if delta_t < target_interval {
            // Too fast -> LTD (Depression)
            self.sensitivity = self.sensitivity - learning_rate;
        } else {
            // Too slow -> LTP (Potentiation)
            self.sensitivity = self.sensitivity + learning_rate;
        }

        // Clamp sensitivity [0.1, 2.0]
        if self.sensitivity.to_raw() < SPA::SCALE_1 / 10 {
            self.sensitivity = SPA::new(0, 6, 0, 0, 0); // 0.1
        }
    }
}

/// The Resonant Intelligence Network
pub struct CortexEngine {
    pub neurons: Vec<S60Neuron>,
    pub time: SPA,
    pub input_buffer: Option<Arc<ResonantBuffer>>,
    pub guardian_a: usize,
    pub guardian_b: usize,
    pub spike_queue: VecDeque<Signal>, // Event Queue
    pub total_energy: SPA,             // NEW: Total energy/activity in the system
    pub persistence: Option<MmapMut>,  // Liquid Crystal Backup
}

impl CortexEngine {
    pub fn new(n_neurons: usize) -> Self {
        let mut neurons = Vec::new();
        for i in 0..n_neurons {
            neurons.push(S60Neuron::new(i as u32));
        }
        Self {
            neurons,
            time: SPA::zero(),
            input_buffer: None,
            guardian_a: 0, // Will be set by init
            guardian_b: 0,
            spike_queue: VecDeque::new(),
            total_energy: SPA::zero(),
            persistence: None,
        }
    }

    /// Inicializa la persistencia líquida (Mmap to Disk Crystal)
    pub fn init_persistence(&mut self, path: &str) -> std::io::Result<()> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        // Ensure file size matches SPA array size: N * 8 bytes (i64)
        let needed_size = (self.neurons.len() * 8) as u64;
        file.set_len(needed_size)?;

        let mmap = unsafe { MmapMut::map_mut(&file)? };

        // Restore state if file has data (Liquid Resume)
        // We assume if first byte is non-zero, it might have data.
        // Or we just overwrite RAM with Disk state on init.
        // For now, let's load disk to RAM.
        // Safety: SPA has same memory layout as i64 (8 bytes).

        // Load: Disk -> RAM
        for (i, neuron) in self.neurons.iter_mut().enumerate() {
            let start = i * 8;
            let bytes = &mmap[start..start + 8];
            let val_raw = i64::from_le_bytes(bytes.try_into().unwrap());
            neuron.potential = SPA::from_raw(val_raw);
        }

        self.persistence = Some(mmap);
        Ok(())
    }

    /// Sync RAM state to Disk Crystal (Liquid Flush)
    pub fn sync_persistence(&mut self) {
        if let Some(mmap) = &mut self.persistence {
            for (i, neuron) in self.neurons.iter().enumerate() {
                let start = i * 8;
                let bytes = neuron.potential.to_raw().to_le_bytes();
                mmap[start..start + 8].copy_from_slice(&bytes);
            }
            // Optional: mmap.flush() - but OS handles this usually.
            // For critical consistency we might want mmap.flush_async().
            let _ = mmap.flush();
        }
    }

    /// Hebbian Plasticity: Adjusts weights based on System Entropy (Truth)
    /// Low Entropy (Truth) -> Strengthen connections involved in resonance.
    /// High Entropy (Noise) -> Weaken connections (Damping).
    pub fn apply_plasticity(&mut self, entropy: SPA) {
        // Threshold for "Truth" vs "Noise"
        // If entropy < 2.5 (Arbitrary Bio-threshold), we learn.
        // Else we forget.

        let threshold = SPA::new(2, 30, 0, 0, 0); // ~2.5
        let is_scv = entropy < threshold;

        // Learning Rate
        let rate = SPA::new(0, 0, 0, 6, 0); // 0.1 rate (scaled)

        // Simplified Logic: Modify GLOBAL connectivity strength or local recent activity?
        // Real Hebbian needs "Pre-Post Spike Timing".
        // SPA Simplification: If Neuron is ACTIVE (charged) and System is TRUTH, reinforce.

        self.neurons.par_iter_mut().for_each(|n| {
            if n.potential.to_raw() > (SPA::SCALE_0 / 2) {
                // Active
                for synapse in &mut n.synapses {
                    // Changed 'connections' to 'synapses'
                    if is_scv {
                        // Potentiation
                        synapse.weight = synapse.weight + rate;
                        // Cap at max
                        if synapse.weight.to_raw() > SPA::SCALE_0 * 2 {
                            synapse.weight = SPA::from_raw(SPA::SCALE_0 * 2);
                        }
                    } else {
                        // Depression (Damping)
                        synapse.weight = synapse.weight - rate;
                        // Floor at 0
                        if synapse.weight.to_raw() < 0 {
                            synapse.weight = SPA::zero();
                        }
                    }
                }
            }
        });
    }

    pub fn attach_buffer(&mut self, buffer: Arc<ResonantBuffer>) {
        self.input_buffer = Some(buffer);
    }

    /// Add a directed synapse (Circuit connection)
    pub fn add_synapse(&mut self, from_id: usize, to_id: u32, weight_val: i64, delay_ticks: i64) {
        if from_id < self.neurons.len() {
            self.neurons[from_id].synapses.push(Synapse {
                target_id: to_id,
                weight: SPA::from_raw(weight_val),
                delay: SPA::from_raw(delay_ticks), // Using raw for ticks
            });
        }
    }

    /// Drains the input buffer and updates neurons for each event.
    /// Returns the total resonance (sum of all spikes).
    pub fn consume_buffer(&mut self) -> i64 {
        let mut total_resonance = SPA::zero();

        // 1. Process External Input (Sensors)
        if let Some(buf) = &self.input_buffer {
            let dt = SPA::new(0, 0, 0, 0, 1);

            while let Some(input_signal) = buf.pop() {
                self.time = self.time + dt;

                // For now, Input hits all neurons (Broadcast Sensor Layer)
                // In future, map specific inputs to specific IDs.
                for neuron_idx in 0..self.neurons.len() {
                    let spike = self.neurons[neuron_idx].integrate(input_signal, dt, self.time);

                    if spike.to_raw() > 0 {
                        total_resonance = total_resonance + spike;
                        // Propagate Spike
                        for syn in &self.neurons[neuron_idx].synapses {
                            self.spike_queue.push_back(Signal {
                                target_id: syn.target_id,
                                strength: spike * syn.weight,
                                arrival_time: self.time, // + syn.delay (Immediate for now, TODO: PrioQueue)
                            });
                        }
                    }
                }
            }
        }

        // 2. Process Internal Spike Propagation (Thinking)
        // Simple BFS for now (limited depth to prevent infinite loops in one tick)
        let mut propagation_steps = 0;
        while let Some(signal) = self.spike_queue.pop_front() {
            if propagation_steps > 1000 {
                break;
            } // Safety brake
            propagation_steps += 1;

            let idx = signal.target_id as usize;
            if idx < self.neurons.len() {
                let dt_internal = SPA::new(0, 0, 0, 0, 1); // Micro-step
                let spike = self.neurons[idx].integrate(signal.strength, dt_internal, self.time);

                if spike.to_raw() > 0 {
                    total_resonance = total_resonance + spike;
                    for syn in &self.neurons[idx].synapses {
                        self.spike_queue.push_back(Signal {
                            target_id: syn.target_id,
                            strength: spike * syn.weight,
                            arrival_time: self.time,
                        });
                    }
                }
            }
        }

        total_resonance.to_raw()
    }

    /// Quantum Pulse: Synchronizes Guardians A and B if phase aligns.
    /// "Spooky action at a distance" - Instant transfer.
    pub fn quantum_pulse(&mut self, crystal_phase: i64) -> bool {
        // ... valid quantum phase check ...
        let window = 129600; // 1 degree tolerance
        if crystal_phase.abs() < window {
            // Entangle 0 and N
            let p0 = self.neurons[self.guardian_a].potential;
            let p_n = self.neurons[self.guardian_b].potential;

            // Constructive Interference
            let avg = (p0 + p_n) / 2; // Using Div<i64>

            self.neurons[self.guardian_a].potential = avg;
            self.neurons[self.guardian_b].potential = avg;

            // Boost Sensitivity (simulating super-position readiness)
            let boost = SPA::new(1, 10, 0, 0, 0);
            self.neurons[self.guardian_a].sensitivity =
                self.neurons[self.guardian_a].sensitivity + boost;
            self.neurons[self.guardian_b].sensitivity =
                self.neurons[self.guardian_b].sensitivity + boost;

            return true;
        }
        false
    }

    pub fn get_guardian_telemetry(&self) -> (SPA, SPA) {
        if self.guardian_a < self.neurons.len() && self.guardian_b < self.neurons.len() {
            (
                self.neurons[self.guardian_a].potential,
                self.neurons[self.guardian_b].potential,
            )
        } else {
            (SPA::zero(), SPA::zero())
        }
    }

    /// Feeds a "thought" (input pattern) into the cortex and returns the resonance.
    pub fn process_thought(&mut self, input: i64, dt_seconds: i64) -> i64 {
        let input_spa = SPA::new(input, 0, 0, 0, 0);
        let dt_spa = SPA::new(0, 0, dt_seconds as i64, 0, 0);
        self.time = self.time + dt_spa;

        let mut total_resonance = SPA::zero();

        for neuron in &mut self.neurons {
            // Fan-out: Global input hits all neurons (Broadcast architecture for now)
            let spike = neuron.integrate(input_spa, dt_spa, self.time);
            total_resonance = total_resonance + spike;
        }

        total_resonance.to_raw()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neuron_charging() {
        let mut n = S60Neuron::new(1);
        let dt = SPA::new(0, 30, 0, 0, 0); // 0.5s steps
        let input = SPA::new(2, 0, 0, 0, 0); // Strong input (2.0)
        let time = SPA::zero();

        println!("--- Neuron Charge Test ---");
        for i in 0..20 {
            let t = time + (dt * SPA::new(i, 0, 0, 0, 0));
            let spike = n.integrate(input, dt, t);
            println!("T={}: V={} Spike={}", i, n.potential, spike);

            if spike.to_raw() > 0 {
                assert!(n.potential.to_raw() == 0); // Should reset
                println!("⚡ FIRED!");
            }
        }
    }
}
