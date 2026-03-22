//! # 🛡️ BASE-60 FIXED-POINT: RUST CORE 🛡️
//!
//! Pure sexagesimal arithmetic for ME-60OS.
//! Implementation: Zero-allocation, fixed-point (60^4 scaling).
//! Compliant with AI Prime Directives: ZERO DECIMAL CONTAMINATION.

use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

use serde::{Deserialize, Serialize};

/// Sexagesimal (Base-60) Fixed-Point Number.
///
/// Internal representation: 5-component array [degrees, minutes, seconds, thirds, fourths].
/// Order: High significance to low significance.
/// Precision: 1/12,960,000 (Fourths).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct SPA {
    pub components: [i64; 5],
}

impl SPA {
    pub const SCALE_0: i64 = 12_960_000; // 60^4
    pub const SCALE_1: i64 = 216_000; // 60^3
    pub const SCALE_2: i64 = 3_600; // 60^2
    pub const SCALE_3: i64 = 60; // 60^1
    pub const SCALE_4: i64 = 1; // 60^0

    /// Creates a new SPA from sexagesimal components.
    /// Components: [degrees, minutes, seconds, thirds, fourths]
    pub const fn new(d: i64, m: i64, s: i64, t: i64, q: i64) -> Self {
        // We store directly, normalization happens on arithmetic operations usually,
        // but for 'new' we assume inputs might need normalization if they exceed 60.
        // However, 'const fn' normalization is hard without loops.
        // We will store as-is and rely on 'to_raw' or arithmetic to normalize.
        Self {
            components: [d, m, s, t, q],
        }
    }

    /// Internal: Creates SPA from raw unit value (scalar).
    pub fn from_raw(raw: i64) -> Self {
        let sign = if raw < 0 { -1 } else { 1 };
        let mut val = raw.abs();

        let q = val % 60;
        val /= 60;
        let t = val % 60;
        val /= 60;
        let s = val % 60;
        val /= 60;
        let m = val % 60;
        val /= 60;
        let d = val;

        Self {
            components: [d * sign, m * sign, s * sign, t * sign, q * sign],
        }
    }

    pub const fn zero() -> Self {
        Self { components: [0; 5] }
    }

    pub const fn one() -> Self {
        Self {
            components: [1, 0, 0, 0, 0],
        }
    }

    /// **LEGACY IMPORT ONLY**: Avoid in core logic.
    pub fn from_decimal_for_import_only(decimal: f64) -> Self {
        let raw = (decimal * Self::SCALE_0 as f64).round() as i64;
        Self::from_raw(raw)
    }

    pub fn to_raw(&self) -> i64 {
        let raw_128 = (self.components[0] as i128 * Self::SCALE_0 as i128)
            + (self.components[1] as i128 * Self::SCALE_1 as i128)
            + (self.components[2] as i128 * Self::SCALE_2 as i128)
            + (self.components[3] as i128 * Self::SCALE_3 as i128)
            + (self.components[4] as i128 * Self::SCALE_4 as i128);

        // Saturate to i64 to prevent panic if value is extreme
        raw_128.clamp(i64::MIN as i128, i64::MAX as i128) as i64
    }

    /// Absolute value.
    pub fn abs(&self) -> Self {
        Self::from_raw(self.to_raw().abs())
    }

    /// Convert to degrees (integer part).
    pub fn to_degrees(&self) -> i64 {
        self.components[0]
    }

    /// Extract sexagesimal components [d, m, s, t, q].
    pub fn to_components(&self) -> [i64; 5] {
        self.components
    }

    /// Creates a new SPA from an array of components.
    pub fn from_components(c: &[i64]) -> Self {
        let mut arr = [0; 5];
        for i in 0..c.len().min(5) {
            arr[i] = c[i];
        }
        Self::from_raw(Self::compute_raw(&arr))
    }

    fn compute_raw(c: &[i64; 5]) -> i64 {
        c[0] * Self::SCALE_0
            + c[1] * Self::SCALE_1
            + c[2] * Self::SCALE_2
            + c[3] * Self::SCALE_3
            + c[4] * Self::SCALE_4
    }
}

// --- ARITHMETIC ---

impl Add for SPA {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        let mut res = [0i64; 5];
        // Use wrapping add to allow carry propagation logic
        // q (4)
        let q_sum = self.components[4] + other.components[4];
        res[4] = q_sum.rem_euclid(Self::SCALE_3); // % 60
        let mut carry = q_sum.div_euclid(Self::SCALE_3);

        // t (3)
        let t_sum = self.components[3] + other.components[3] + carry;
        res[3] = t_sum.rem_euclid(Self::SCALE_3);
        carry = t_sum.div_euclid(Self::SCALE_3);

        // s (2)
        let s_sum = self.components[2] + other.components[2] + carry;
        res[2] = s_sum.rem_euclid(Self::SCALE_3);
        carry = s_sum.div_euclid(Self::SCALE_3);

        // m (1)
        let m_sum = self.components[1] + other.components[1] + carry;
        res[1] = m_sum.rem_euclid(Self::SCALE_3);
        carry = m_sum.div_euclid(Self::SCALE_3);

        // d (0) - No modulo
        res[0] = self.components[0] + other.components[0] + carry;

        Self { components: res }
    }
}

impl Sub for SPA {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        self.add(-other)
    }
}

impl Mul<i64> for SPA {
    type Output = Self;
    fn mul(self, scalar: i64) -> Self {
        Self::from_raw(self.to_raw() * scalar)
    }
}

impl Mul for SPA {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        // Scalar fallback: (val1 * val2) / SCALE_0
        let val1 = self.to_raw() as i128; // Use i128 to prevent overflow
        let val2 = other.to_raw() as i128;
        let res = (val1 * val2) / Self::SCALE_0 as i128;
        Self::from_raw(res as i64)
    }
}

impl Div<i64> for SPA {
    type Output = Self;
    fn div(self, divisor: i64) -> Self {
        Self::from_raw(self.to_raw() / divisor)
    }
}

impl Div for SPA {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        if other.to_raw() == 0 {
            panic!("SPA Division by zero");
        }
        let val1 = self.to_raw() as i128;
        let val2 = other.to_raw() as i128;
        let res = (val1 * Self::SCALE_0 as i128) / val2;
        Self::from_raw(res as i64)
    }
}

impl Rem<i64> for SPA {
    type Output = Self;
    fn rem(self, rhs: i64) -> Self::Output {
        let mod_val = rhs * Self::SCALE_0;
        Self::from_raw(self.to_raw() % mod_val)
    }
}

impl Rem for SPA {
    type Output = Self;
    fn rem(self, rhs: Self) -> Self::Output {
        Self::from_raw(self.to_raw() % rhs.to_raw())
    }
}

impl Neg for SPA {
    type Output = Self;
    fn neg(self) -> Self {
        let mut c = self.components;
        for i in 0..5 {
            c[i] = -c[i];
        }
        // Normalization is messy when negating components individually if they are mixed
        // But from_raw handles signs correctly.
        // Actually, strictly: - (1;30) should be -1; -30.
        // from_raw logic: -1.5 deg -> -1 deg, -30 min. Components: [-1, -30...]
        // So simple negation works.
        // But wait: -1, -30 in Add logic:
        // -30 (mod 60) -> 30, carry -1.
        // -1 + (-1) = -2.
        // Result: -2, 30.
        // -2 + 30/60 = -1.5. Correct.
        Self { components: c }
    }
}

// --- FORMATTING ---

impl fmt::Display for SPA {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // For display logic, we might want "canonical" positive minutes?
        // SPA::new(2, 15) printed "SPA[02; 15...]"
        // SPA::new(-2, -30) -> components [-2, 30].
        // Wait, from_raw(-1.5) -> components [-1, -30].
        // Display logic in from_value usually separates sign.
        // Let's rely on to_components of the abs value for formatting?
        // No, we have the components directly now.
        // If we trust the components are consistent (all same sign or normalized), we can just print.
        // But Add logic can produce mixed signs (e.g. -2, 30).
        // Let's normalize for display by converting to raw and back?
        // That guarantees canonical form: Sign + [d, m, s, t, q] all positive.

        let raw = self.to_raw();
        let sign = if raw < 0 { "-" } else { "" };
        let abs_spa = self.abs();
        let c = abs_spa.components; // Should be all positive

        write!(
            f,
            "SPA[{}{:03}; {:02}, {:02}, {:02}, {:02}]",
            sign, c[0], c[1], c[2], c[3], c[4]
        )
    }
}

impl fmt::Debug for SPA {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let val = SPA::new(1, 30, 0, 0, 0);
        assert_eq!(val.to_raw(), SPA::SCALE_0 + 30 * SPA::SCALE_1);
        assert_eq!(val.components, [1, 30, 0, 0, 0]);
    }

    #[test]
    fn test_add() {
        let a = SPA::new(1, 30, 0, 0, 0);
        let b = SPA::new(0, 45, 0, 0, 0);
        let sum = a + b; // 1:30 + 0:45 = 2:15
        assert_eq!(sum.to_degrees(), 2);
        // We need to verify components specifically
        // 30+45 = 75 -> 15, carry 1.
        // 1+0+1 = 2.
        assert_eq!(sum.components, [2, 15, 0, 0, 0]);
    }

    #[test]
    fn test_sub_mixed() {
        // 2:00 - 0:30 = 1:30
        let a = SPA::new(2, 0, 0, 0, 0);
        let b = SPA::new(0, 30, 0, 0, 0);
        let res = a - b;
        // Logic:
        // -b components: [0, -30]
        // 0 + (-30) = -30. mod 60 -> 30, carry -1.
        // 2 + 0 + (-1) = 1.
        // res: [1, 30]
        assert_eq!(res.components, [1, 30, 0, 0, 0]);
    }

    #[test]
    fn test_mul_scaling() {
        let a = SPA::new(1, 0, 0, 0, 0);
        let b = SPA::new(2, 0, 0, 0, 0);
        assert_eq!((a * b).components, [2, 0, 0, 0, 0]);

        let half = SPA::new(0, 30, 0, 0, 0);
        let res = a * half; // 1 * 0.5 = 0.5
        assert_eq!(res.components, [0, 30, 0, 0, 0]);
    }

    #[test]
    fn test_display() {
        let val = SPA::new(15, 5, 24, 0, 0);
        assert_eq!(format!("{}", val), "SPA[015; 05, 24, 00, 00]");
    }
}
