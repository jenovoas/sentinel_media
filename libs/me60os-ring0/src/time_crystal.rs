//! # 💎 CRISTAL DE TIEMPO - Motor de Sincronización Temporal 💎
//!
//! Implementación del reloj maestro y control de inyección de energía.
//! Garantiza que el Ring 0 y el Ring 3 latan al mismo ritmo.
// ⚡ AXIOMA I — ZERO DECIMAL CONTAMINATION ⚡
#![deny(clippy::float_arithmetic)]

use crate::spa::SPA;
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;
use serde::{Serialize, Deserialize};

// =============================================================================
// ISOCHRONOUS CLOCK (YHWH MODULATED)
// =============================================================================

/// Clock de sincronización maestro con Modulación de Fase YHWH (10-5-6-5).
/// Implementa el sistema de "respiración" para evitar saturación de CPU y deriva térmica.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IsochronousClock {
    pub base_interval_ns: u64,
    pub phase_intervals: [u64; 4],
    pub start_time_ns: u64,
    pub platonic_target_ns: u64,
    pub ticks: u64,
    pub drift_history: VecDeque<i64>,
    pub last_leap_ns: u64,
}

impl IsochronousClock {
    /// Obtiene nanosegundos monotónicos reales del sistema (Ring 0 sync).
    fn get_system_ns() -> u64 {
        let mut ts = libc::timespec { tv_sec: 0, tv_nsec: 0 };
        unsafe {
            libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts);
        }
        (ts.tv_sec as u64 * 1_000_000_000) + (ts.tv_nsec as u64)
    }

    pub fn new() -> Self {
        let base_interval_ns = 23_939_835;
        let i_yod = 22_977_941;
        let i_he1 = 24_378_352;
        let i_vav = 24_084_778;
        let i_he2 = 24_378_352;

        let now_ns = Self::get_system_ns();

        Self {
            base_interval_ns,
            phase_intervals: [i_yod, i_he1, i_vav, i_he2],
            start_time_ns: now_ns,
            platonic_target_ns: now_ns,
            ticks: 0,
            drift_history: VecDeque::with_capacity(64),
            last_leap_ns: now_ns,
        }
    }

    pub fn tick(&mut self) {
        let phase_idx = (self.ticks % 4) as usize;
        let current_interval = self.phase_intervals[phase_idx];
        
        let hiccup_correction = if self.ticks % 17 == 0 && self.ticks > 0 {
            700_000 
        } else {
            0
        };

        self.ticks += 1;
        self.platonic_target_ns += current_interval + hiccup_correction;

        let current_now = Self::get_system_ns();

        // 2. Quantum Leap (cada 68s)
        if current_now > self.last_leap_ns && current_now - self.last_leap_ns > 68_000_000_000 {
            self.start_time_ns = current_now;
            self.platonic_target_ns = current_now;
            self.ticks = 0;
            self.last_leap_ns = current_now;
            self.drift_history.clear();
            println!("🌀 QUANTUM LEAP: Fase del sistema reseteada.");
            return;
        }

        // 3. Respiración
        if self.platonic_target_ns > current_now {
            let sleep_ns = self.platonic_target_ns - current_now;
            if sleep_ns > 100_000 {
                thread::sleep(Duration::from_nanos(sleep_ns));
            }
        } else {
            let drift = (current_now - self.platonic_target_ns) as i64;
            self.drift_history.push_back(drift);
            if self.drift_history.len() > 64 {
                self.drift_history.pop_front();
            }
        }
    }

    pub fn get_drift_avg_ns(&self) -> i64 {
        if self.drift_history.is_empty() { return 0; }
        self.drift_history.iter().sum::<i64>() / self.drift_history.len() as i64
    }

    pub fn get_nanos(&self) -> u64 {
        Self::get_system_ns()
    }

    pub fn get_ticks(&self) -> u64 { self.ticks }
}

impl Default for IsochronousClock {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// S60 PID CONTROLLER
// =============================================================================

/// Controlador PID discreto en campo SPA.
/// u(t) = Kp*e(t) + Ki*∫e(t) + Kd*de(t)/dt
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct S60PID {
    pub kp: SPA,
    pub ki: SPA,
    pub kd: SPA,
    pub setpoint: SPA,
    pub integral: SPA,
    pub prev_error: SPA,
}

impl S60PID {
    pub fn new(kp_raw: i64, ki_raw: i64, kd_raw: i64, setpoint_raw: i64) -> Self {
        Self {
            kp: SPA::from_raw(kp_raw),
            ki: SPA::from_raw(ki_raw),
            kd: SPA::from_raw(kd_raw),
            setpoint: SPA::from_raw(setpoint_raw),
            integral: SPA::zero(),
            prev_error: SPA::zero(),
        }
    }

    /// Calcula salida del PID dado un valor medido y delta de tiempo.
    pub fn update(&mut self, measured_raw: i64, dt_raw: i64) -> i64 {
        let measured = SPA::from_raw(measured_raw);
        let dt = SPA::from_raw(dt_raw);
        let error = self.setpoint - measured;

        // Término Proporcional
        let p_term = (self.kp * error) / SPA::new(1, 0, 0, 0, 0);

        // Término Integral
        self.integral = self.integral + (error * dt) / SPA::new(1, 0, 0, 0, 0);
        let i_term = (self.ki * self.integral) / SPA::new(1, 0, 0, 0, 0);

        // Término Derivativo
        let d_term = if dt.to_raw() > 0 {
            let d_error = error - self.prev_error;
            let derivative = (d_error * SPA::new(1, 0, 0, 0, 0)) / dt;
            (self.kd * derivative) / SPA::new(1, 0, 0, 0, 0)
        } else {
            SPA::zero()
        };

        self.prev_error = error;

        (p_term + i_term + d_term).to_raw()
    }

    pub fn reset(&mut self) {
        self.integral = SPA::zero();
        self.prev_error = SPA::zero();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_clock_breathing() {
        let mut clock = IsochronousClock::new();
        let start = Instant::now();
        // 4 ticks (one full YHWH cycle)
        for _ in 0..4 {
            clock.tick();
        }
        let elapsed = start.elapsed();
        // 4 ticks ≈ 4 * 23.9ms ≈ 95.6ms
        println!("Elapsed for 4 ticks (YHWH cycle): {:?}", elapsed);
        assert!(elapsed.as_millis() >= 80);
        assert!(elapsed.as_millis() <= 120);
    }

    #[test]
    fn test_pid_logic() {
        let mut pid = S60PID::new(
            SPA::new(0, 30, 0, 0, 0).to_raw(), // 0.5
            SPA::new(0, 10, 0, 0, 0).to_raw(), // 0.16
            0,
            SPA::new(1, 0, 0, 0, 0).to_raw(),  // Setpoint 1.0
        );

        let dt = SPA::new(0, 1, 0, 0, 0).to_raw();
        let output = pid.update(SPA::new(0, 30, 0, 0, 0).to_raw(), dt); // Measure 0.5
        assert!(output > 0); // Should try to increase
    }
}
