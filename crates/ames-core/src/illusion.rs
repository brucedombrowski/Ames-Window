//! What the viewer *thinks* they see.

use crate::Angle;

/// The orientation an observer perceives for a window actually turned by
/// `actual`.
///
/// The observer assumes the tall edge is the nearer one. While that is true
/// (the first and last quarter turns) perception matches reality. Once the
/// short edge swings nearer, the brain mirrors the motion instead of
/// accepting it, so the window appears to reverse and swing back. The result
/// oscillates within a quarter turn either side of face‑on.
#[must_use]
pub fn perceived_angle(actual: Angle) -> Angle {
    let signed = actual.signed();
    if signed > Angle::QUARTER {
        Angle::HALF - signed
    } else if signed < -Angle::QUARTER {
        -Angle::HALF - signed
    } else {
        signed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deg(d: f32) -> Angle {
        Angle::degrees(d)
    }

    fn close(a: Angle, b: Angle) -> bool {
        (a.as_degrees() - b.as_degrees()).abs() < 1e-4
    }

    #[test]
    fn perception_is_honest_near_face_on() {
        assert!(close(perceived_angle(deg(0.0)), deg(0.0)));
        assert!(close(perceived_angle(deg(45.0)), deg(45.0)));
        assert!(close(perceived_angle(deg(-60.0)), deg(-60.0)));
    }

    #[test]
    fn perception_reverses_past_edge_on() {
        assert!(close(perceived_angle(deg(135.0)), deg(45.0)));
        assert!(close(perceived_angle(deg(180.0)), deg(0.0)));
        assert!(close(perceived_angle(deg(225.0)), deg(-45.0)));
    }

    #[test]
    fn perception_never_leaves_a_half_turn() {
        for step in 0..720 {
            #[allow(clippy::cast_precision_loss)]
            let seen = perceived_angle(deg(step as f32 * 0.5)).as_degrees();
            assert!((-90.0..=90.0).contains(&seen), "{seen}");
        }
    }
}
