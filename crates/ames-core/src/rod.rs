//! The rod that seems to slice through the frame.

use crate::{Angle, Vec3};

/// A straight rod, a pen in the classic demonstration, fixed through the
/// window's centre.
///
/// Because the rod turns with the window it should, and does, stay rigid.
/// But the eye sees the window swinging the other way, and to reconcile the
/// two it lets the rod appear to bend and pass through solid wood.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rod {
    /// Total length, in world units.
    pub length: f32,
    /// How far the rod is turned out of the window plane.
    pub tilt: Angle,
    /// How steeply the rod climbs from one end to the other.
    pub rise: Angle,
    /// Where the rod pierces the window plane, in window coordinates.
    pub anchor: Vec3,
}

impl Default for Rod {
    fn default() -> Self {
        Self {
            length: 2.0,
            tilt: Angle::degrees(55.0),
            rise: Angle::ZERO,
            anchor: Vec3::ZERO,
        }
    }
}

impl Rod {
    /// The two halves of the rod, split where it pierces the window plane.
    ///
    /// Each half is a pair of endpoints in the window's own coordinates.
    /// Splitting at the plane lets the scene draw one half behind the frame
    /// and the other in front, whichever way the window is turned.
    #[must_use]
    pub fn halves(&self) -> [[Vec3; 2]; 2] {
        let (sin, cos) = self.rise.sin_cos();
        let reach = Vec3::new(cos, sin, 0.0).rotate_y(self.tilt) * (self.length / 2.0);
        [
            [self.anchor, self.anchor + reach],
            [self.anchor, self.anchor - reach],
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn halves_meet_at_the_anchor_and_span_the_length() {
        let rod = Rod::default();
        let [front, back] = rod.halves();
        assert_eq!(front[0], rod.anchor);
        assert_eq!(back[0], rod.anchor);
        let span = (front[1] - back[1]).length();
        assert!((span - rod.length).abs() < 1e-5);
    }

    #[test]
    fn halves_lie_on_opposite_sides_of_the_plane() {
        let [front, back] = Rod::default().halves();
        assert!(front[1].z > 0.0);
        assert!(back[1].z < 0.0);
    }
}
