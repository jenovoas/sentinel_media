//! # 🛡️ BASE-60 COMPLEX ARITHMETIC - RUST CORE 🛡️
//!
//! Complex numbers using SPA fixed-point arithmetic.
//! Compliant with AI Prime Directives.

use crate::spa::SPA;
use crate::spa_math::SPAMath;
use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ComplexSPA {
    pub real: SPA,
    pub imag: SPA,
}

impl ComplexSPA {
    pub const fn new(real: SPA, imag: SPA) -> Self {
        Self { real, imag }
    }

    pub fn magnitude(&self) -> SPA {
        SPAMath::sqrt(self.real * self.real + self.imag * self.imag)
    }

    pub fn conjugate(&self) -> Self {
        Self::new(self.real, -self.imag)
    }

    pub fn exp_i_theta(phi: SPA) -> Self {
        Self::new(SPAMath::cos(phi), SPAMath::sin(phi))
    }
}

// --- ARITHMETIC ---

impl Add for ComplexSPA {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.real + other.real, self.imag + other.imag)
    }
}

impl Sub for ComplexSPA {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.real - other.real, self.imag - other.imag)
    }
}

impl Mul for ComplexSPA {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        let r = (self.real * other.real) - (self.imag * other.imag);
        let i = (self.real * other.imag) + (self.imag * other.real);
        Self::new(r, i)
    }
}

impl Mul<SPA> for ComplexSPA {
    type Output = Self;
    fn mul(self, scalar: SPA) -> Self {
        Self::new(self.real * scalar, self.imag * scalar)
    }
}

impl Div<SPA> for ComplexSPA {
    type Output = Self;
    fn div(self, scalar: SPA) -> Self {
        Self::new(self.real / scalar, self.imag / scalar)
    }
}

impl Neg for ComplexSPA {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.real, -self.imag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complex_add() {
        let a = ComplexSPA::new(SPA::new(1, 0, 0, 0, 0), SPA::new(0, 30, 0, 0, 0));
        let b = ComplexSPA::new(SPA::new(0, 45, 0, 0, 0), SPA::new(1, 15, 0, 0, 0));
        let sum = a + b;
        assert_eq!(sum.real, SPA::new(1, 45, 0, 0, 0));
        assert_eq!(sum.imag, SPA::new(1, 45, 0, 0, 0));
    }
}

// Constantes
pub const I: ComplexSPA = ComplexSPA::new(SPA::new(0, 0, 0, 0, 0), SPA::new(1, 0, 0, 0, 0));
pub const ONE: ComplexSPA = ComplexSPA::new(SPA::new(1, 0, 0, 0, 0), SPA::new(0, 0, 0, 0, 0));
