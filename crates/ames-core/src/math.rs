//! Just enough linear algebra for a spinning window.

use crate::Angle;
use std::ops::{Add, Mul, Sub};

/// A point or direction in 3‑D space.
///
/// Axes follow the usual right‑handed convention for a picture:
/// `x` points right, `y` points up, and `z` points *out of the screen*
/// toward the viewer.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    /// Rightward component.
    pub x: f32,
    /// Upward component.
    pub y: f32,
    /// Component toward the viewer.
    pub z: f32,
}

impl Vec3 {
    /// The origin.
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    /// Builds a vector from its components.
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Rotates the vector about the vertical `y` axis.
    ///
    /// A positive angle turns the `+x` axis toward the viewer (`+z`), which
    /// is counter‑clockwise when seen from above.
    #[must_use]
    pub fn rotate_y(self, angle: Angle) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self {
            x: self.x * cos - self.z * sin,
            y: self.y,
            z: self.x * sin + self.z * cos,
        }
    }

    /// Linear interpolation: `t = 0` gives `self`, `t = 1` gives `other`.
    #[must_use]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t
    }

    /// Euclidean length.
    #[must_use]
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// The vector scaled to unit length. The zero vector stays zero.
    #[must_use]
    pub fn normalized(self) -> Self {
        let len = self.length();
        if len == 0.0 { self } else { self * (1.0 / len) }
    }
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

/// A point on the picture plane.
///
/// Units are *view units*: the camera's focal length is chosen so that a
/// range of roughly `-1..=1` fills the shorter side of the picture. Flipping
/// `y` and scaling to pixels is left to whoever draws.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    /// Rightward component.
    pub x: f32,
    /// Upward component.
    pub y: f32,
}

impl Vec2 {
    /// Builds a point from its components.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Vec3, b: Vec3) -> bool {
        (a - b).length() < 1e-5
    }

    #[test]
    fn quarter_turn_brings_x_toward_viewer() {
        let turned = Vec3::new(1.0, 0.0, 0.0).rotate_y(Angle::QUARTER);
        assert!(close(turned, Vec3::new(0.0, 0.0, 1.0)));
    }

    #[test]
    fn rotation_preserves_height_and_length() {
        let v = Vec3::new(3.0, 2.0, -1.0);
        let turned = v.rotate_y(Angle::degrees(123.0));
        assert!((turned.y - v.y).abs() < 1e-6);
        assert!((turned.length() - v.length()).abs() < 1e-5);
    }

    #[test]
    fn lerp_hits_both_ends() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(2.0, 4.0, 6.0);
        assert!(close(a.lerp(b, 0.0), a));
        assert!(close(a.lerp(b, 1.0), b));
        assert!(close(a.lerp(b, 0.5), Vec3::new(1.0, 2.0, 3.0)));
    }
}
