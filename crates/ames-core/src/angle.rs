//! A small newtype so radians and degrees never get mixed up.

use std::f32::consts::{PI, TAU};
use std::ops::{Add, AddAssign, Neg, Sub};

/// An angle, stored in radians.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Angle(f32);

impl Angle {
    /// No rotation at all.
    pub const ZERO: Self = Self(0.0);
    /// A quarter turn.
    pub const QUARTER: Self = Self(PI / 2.0);
    /// A half turn.
    pub const HALF: Self = Self(PI);
    /// A full turn.
    pub const FULL: Self = Self(TAU);

    /// Builds an angle from radians.
    #[must_use]
    pub const fn radians(radians: f32) -> Self {
        Self(radians)
    }

    /// Builds an angle from degrees.
    #[must_use]
    pub fn degrees(degrees: f32) -> Self {
        Self(degrees.to_radians())
    }

    /// The angle in radians.
    #[must_use]
    pub const fn as_radians(self) -> f32 {
        self.0
    }

    /// The angle in degrees.
    #[must_use]
    pub fn as_degrees(self) -> f32 {
        self.0.to_degrees()
    }

    /// The same direction, expressed in the half‑open range `[0, 2π)`.
    #[must_use]
    pub fn wrapped(self) -> Self {
        Self(self.0.rem_euclid(TAU))
    }

    /// The same direction, expressed in the range `(-π, π]`.
    #[must_use]
    pub fn signed(self) -> Self {
        let wrapped = self.wrapped().0;
        Self(if wrapped > PI { wrapped - TAU } else { wrapped })
    }

    /// Sine and cosine in one call, matching [`f32::sin_cos`].
    #[must_use]
    pub fn sin_cos(self) -> (f32, f32) {
        self.0.sin_cos()
    }
}

impl Add for Angle {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Angle {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for Angle {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl Neg for Angle {
    type Output = Self;
    fn neg(self) -> Self {
        Self(-self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn degrees_round_trip() {
        assert!(close(Angle::degrees(90.0).as_radians(), PI / 2.0));
        assert!(close(Angle::radians(PI).as_degrees(), 180.0));
    }

    #[test]
    fn wrapping_keeps_direction() {
        assert!(close(Angle::degrees(370.0).wrapped().as_degrees(), 10.0));
        assert!(close(Angle::degrees(-10.0).wrapped().as_degrees(), 350.0));
    }

    #[test]
    fn signed_centres_on_zero() {
        assert!(close(Angle::degrees(350.0).signed().as_degrees(), -10.0));
        assert!(close(Angle::degrees(180.0).signed().as_degrees(), 180.0));
        assert!(close(Angle::degrees(181.0).signed().as_degrees(), -179.0));
    }
}
