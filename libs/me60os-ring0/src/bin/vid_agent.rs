//! ❄️ VID AGENT (Variable Inertia Dynamics)
//! ========================================
//! Reemplazo en Rust de `quantum_cooling_v3.py`.
//! Gestiona la "temperatura" (latencia) del sistema mediante
//! ajustes dinámicos de buffer (Masa Efectiva).

use me60os_core::spa::SPA;
use std::collections::VecDeque;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
struct BufferState {
    size: usize,
    utilization: SPA,
    _timestamp: f64, // Seconds
}

struct VidAgent {
    history: VecDeque<BufferState>,
    velocity_history: VecDeque<SPA>,
    acceleration_history: VecDeque<SPA>,
    _last_action_time: Instant,
    start_time: Instant,

    // Config
    _base_damping: SPA,
    _current_damping: SPA,
}

impl VidAgent {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(60),
            velocity_history: VecDeque::with_capacity(20),
            acceleration_history: VecDeque::with_capacity(10),
            _last_action_time: Instant::now(),
            start_time: Instant::now(),
            _base_damping: SPA::new(0, 48, 0, 0, 0), // 0.8
            _current_damping: SPA::new(0, 48, 0, 0, 0),
        }
    }

    pub fn run_loop(&mut self) {
        println!("❄️  VID AGENT ONLINE (SPA Quantum Cooling)");

        // Mock loop
        let mut current_size = 1000;
        let mut sim_util = SPA::new(0, 30, 0, 0, 0); // 0.5
        let mut direction = 1;

        loop {
            // 1. Measure (Improved Simulation)
            // Oscilar util entre 0.1 y 0.9 para probar todas las acciones
            if direction == 1 {
                sim_util = sim_util + SPA::new(0, 2, 0, 0, 0); // +0.033
                if sim_util > SPA::new(0, 54, 0, 0, 0) {
                    // 0.9
                    direction = -1;
                }
            } else {
                sim_util = sim_util - SPA::new(0, 2, 0, 0, 0);
                if sim_util < SPA::new(0, 6, 0, 0, 0) {
                    // 0.1
                    direction = 1;
                }
            }

            let state = BufferState {
                size: current_size,
                utilization: sim_util,
                _timestamp: self.start_time.elapsed().as_secs_f64(),
            };

            // 2. Predict & Act
            let (new_size, action) = self.predict(state);

            if new_size != current_size {
                // Apply cap to prevent overflow in logs and logic
                let capped_size = if new_size > 1_000_000 {
                    1_000_000
                } else {
                    new_size
                };

                if capped_size != current_size {
                    println!(
                        "⚡ ACTION: {} | Resize {} -> {}",
                        action, current_size, capped_size
                    );
                    current_size = capped_size;
                }
            }

            thread::sleep(Duration::from_millis(100));
        }
    }

    fn predict(&mut self, state: BufferState) -> (usize, String) {
        self.history.push_back(state);
        if self.history.len() > 60 {
            self.history.pop_front();
        }

        // Calc Velocity
        let velocity = self.measure_velocity();
        self.velocity_history.push_back(velocity);
        if self.velocity_history.len() > 20 {
            self.velocity_history.pop_front();
        }

        // Calc Acceleration
        let acceleration = self.measure_acceleration();
        self.acceleration_history.push_back(acceleration);
        if self.acceleration_history.len() > 10 {
            self.acceleration_history.pop_front();
        }

        // 1. Runaway Detection
        if self.detect_runaway() {
            return (state.size * 5, "EMERGENCY: RUNAWAY DETECTED".to_string());
        }

        // 2. Standard Logic
        let _ground_state = SPA::new(0, 30, 0, 0, 0); // 0.5

        if state.utilization > SPA::new(0, 48, 0, 0, 0) {
            // > 0.8
            // Cool Down (Expand Buffer)
            let new_size = (state.size as f64 * 1.1) as usize;
            return (new_size, format!("COOLING (v={})", velocity));
        } else if state.utilization < SPA::new(0, 12, 0, 0, 0) {
            // < 0.2
            // Contract
            let new_size = (state.size as f64 * 0.9) as usize;
            return (new_size, "CONTRACTING".to_string());
        }

        (state.size, "STABLE".to_string())
    }

    fn measure_velocity(&self) -> SPA {
        if self.history.len() < 2 {
            return SPA::zero();
        }
        let curr = self.history.back().unwrap();
        let prev = self.history.get(self.history.len() - 2).unwrap();

        curr.utilization - prev.utilization
    }

    fn measure_acceleration(&self) -> SPA {
        if self.velocity_history.len() < 2 {
            return SPA::zero();
        }
        let curr = self.velocity_history.back().unwrap();
        let _prev = self
            .velocity_history
            .get(self.velocity_history.len() - 2)
            .unwrap();
        *curr
    }

    fn detect_runaway(&self) -> bool {
        false
    }
}

fn main() {
    let mut agent = VidAgent::new();
    agent.run_loop();
}
