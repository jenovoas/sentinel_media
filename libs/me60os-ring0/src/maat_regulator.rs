//! # ⚖️ MAAT STABILIZER (ATLANTEAN REGULATOR)
//! 
//! Maintains the balance between Acceleration (Velocity) and Truth (Accuracy).
//! Ported from Atlantean Python logic to Pure S60 Rust.
//! 
//! AXIOM COMPLIANCE:
//! - ZERO DECIMAL CONTAMINATION.
//! - ALL arithmetic in S60 (SPA).

use crate::spa::SPA;

/// Maat Status for telemetry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaatStatus {
    VelocitySacrifice,
    MaxResonance,
    CrystalPureAccel,
    MaatHarmonic,
}

pub struct MaatStabilizer {
    /// 95% Truth target
    pub target_truth: SPA,
    /// 99% Pure threshold
    pub pure_threshold: SPA,
    /// Absolute max speed limit
    pub max_speed: SPA,
    /// Absolute min speed
    pub min_speed: SPA,
}

impl MaatStabilizer {
    pub fn new() -> Self {
        Self {
            // 95% = 57/60 arcminutes
            target_truth: SPA::new(0, 57, 0, 0, 0),
            // 99% = 59;24 (59.4/60)
            pure_threshold: SPA::new(0, 59, 24, 0, 0),
            // Max speed limit: 31.0
            max_speed: SPA::new(31, 0, 0, 0, 0),
            // Min speed limit: 1.0
            min_speed: SPA::new(1, 0, 0, 0, 0),
        }
    }

    /// Regulates the system speed based on the current Truth Score.
    /// Returns: (new_speed: SPA, status: MaatStatus)
    pub fn regulate(&self, current_truth: SPA, current_speed: SPA) -> (SPA, MaatStatus) {
        if current_truth < self.target_truth {
            // ⚠️ SACRIFICIO ARMÓNICO
            // New Speed = (Current Speed * Current Truth) / Target Truth
            // Throttles aggressively when accuracy drops below 95%.
            let mut new_speed = (current_speed * current_truth) / self.target_truth;
            
            if new_speed < self.min_speed {
                new_speed = self.min_speed;
            }
            (new_speed, MaatStatus::VelocitySacrifice)

        } else if current_truth > self.pure_threshold {
            // 💎 RESONANCIA PURA (> 99%)
            // Safe to accelerate towards max potential.
            if current_speed < self.max_speed {
                // Factor 1.1 = current + (current / 10)
                let mut new_speed = current_speed + (current_speed / 10);
                if new_speed > self.max_speed {
                    new_speed = self.max_speed;
                }
                (new_speed, MaatStatus::CrystalPureAccel)
            } else {
                (self.max_speed, MaatStatus::MaxResonance)
            }

        } else {
            // ✅ ESTABILIDAD (95-99%)
            (current_speed, MaatStatus::MaatHarmonic)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maat_throttle() {
        let maat = MaatStabilizer::new();
        let speed = SPA::new(30, 0, 0, 0, 0);
        // 80% Truth = 48/60 arcminutes
        let truth = SPA::new(0, 48, 0, 0, 0);
        
        let (new_speed, status) = maat.regulate(truth, speed);
        assert_eq!(status, MaatStatus::VelocitySacrifice);
        // (30 * 48) / 57 = 1440 / 57 ≈ 25.26
        assert!(new_speed < speed);
        assert!(new_speed >= maat.min_speed);
    }

    #[test]
    fn test_maat_accel() {
        let maat = MaatStabilizer::new();
        let speed = SPA::new(10, 0, 0, 0, 0);
        // 100% Truth = 1.0
        let truth = SPA::one();
        
        let (new_speed, status) = maat.regulate(truth, speed);
        assert_eq!(status, MaatStatus::CrystalPureAccel);
        // 10 + 1 = 11
        assert_eq!(new_speed, SPA::new(11, 0, 0, 0, 0));
    }
}
