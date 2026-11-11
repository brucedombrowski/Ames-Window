//! What the viewer *thinks* they see.

use crate::Angle;

/// The orientation an observer perceives for a window actually turned by
/// `actual`.
///
/// A flat picture cannot say which of a window's two upright edges is the
/// nearer: turned by some angle, or by the same angle the other way, the
/// window covers the same width on the picture and only its depth differs.
/// The eye settles the question by size. It takes the tall edge to be the
/// near one, because that is how a real rectangle looks.
///
/// So while the tall edge really is nearer, perception matches reality. For
/// the other half of each turn the tall edge is behind, and the eye sees the
/// same picture with its depth reversed: the window turned by the opposite
/// angle, tall edge in front. The believed window therefore never completes
/// a turn. Its tall edge sweeps across in front of the pivot and back again,
/// through half a turn, while the real one goes all the way round.
///
/// The window's tall edge is the left one (`x` negative), and a positive
/// turn carries the right-hand side toward the viewer, so the tall edge is
/// nearer exactly when the angle is negative.
#[must_use]
pub fn perceived_angle(actual: Angle) -> Angle {
    let signed = actual.signed();
    if signed > Angle::ZERO {
        -signed
    } else {
        signed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Trapezoid, Vec3};

    fn deg(d: f32) -> Angle {
        Angle::degrees(d)
    }

    fn close(a: Angle, b: Angle) -> bool {
        (a.as_degrees() - b.as_degrees()).abs() < 1e-3
    }

    /// The tall edge of a window, as [`Trapezoid`] places it.
    fn tall_edge() -> Vec3 {
        let shape = Trapezoid {
            width: 2.0,
            tall: 1.0,
            short: 0.5,
        };
        Vec3::new(shape.x_at(0.0), 0.0, 0.0)
    }

    #[test]
    fn perception_is_honest_while_the_tall_edge_is_nearer() {
        assert!(close(perceived_angle(deg(0.0)), deg(0.0)));
        assert!(close(perceived_angle(deg(-60.0)), deg(-60.0)));
        assert!(close(perceived_angle(deg(-135.0)), deg(-135.0)));
        assert!(close(perceived_angle(deg(300.0)), deg(-60.0)));
    }

    #[test]
    fn perception_reverses_depth_while_it_is_farther() {
        assert!(close(perceived_angle(deg(45.0)), deg(-45.0)));
        assert!(close(perceived_angle(deg(135.0)), deg(-135.0)));
        assert!(close(perceived_angle(deg(170.0)), deg(-170.0)));
    }

    #[test]
    fn the_believed_window_swings_through_half_a_turn() {
        for step in 0..720 {
            #[allow(clippy::cast_precision_loss)]
            let seen = perceived_angle(deg(step as f32 * 0.5)).as_degrees();
            assert!((-180.001..=0.0).contains(&seen), "{seen}");
        }
    }

    /// The two properties that define the illusion, checked against the
    /// geometry itself rather than against angles worked out by hand.
    #[test]
    fn the_picture_is_unchanged_and_the_tall_edge_is_never_behind() {
        for step in 0..720 {
            #[allow(clippy::cast_precision_loss)]
            let actual = deg(step as f32 * 0.5);
            let real = tall_edge().rotate_y(actual);
            let believed = tall_edge().rotate_y(perceived_angle(actual));
            // Same place across the picture...
            assert!((real.x - believed.x).abs() < 1e-4, "{actual:?}");
            // ...at the same distance from the picture plane, but in front.
            assert!((real.z.abs() - believed.z).abs() < 1e-4, "{actual:?}");
            assert!(believed.z >= -1e-4, "{actual:?}");
        }
    }
}
