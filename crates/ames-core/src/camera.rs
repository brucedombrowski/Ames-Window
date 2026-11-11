//! A pinhole camera on the `z` axis, looking at the origin.

use crate::{Vec2, Vec3};

/// A perspective camera.
///
/// The eye sits at `(0, 0, distance)` and looks down the negative `z` axis.
/// Points nearer the eye project larger; that foreshortening is the whole
/// trick of the Ames window, so the camera is deliberately simple and
/// explicit rather than hidden behind a matrix.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    /// How far the eye is from the window's pivot, in world units.
    pub distance: f32,
    /// Scale from world units at the pivot to view units on the picture plane.
    pub focal: f32,
}

impl Default for Camera {
    /// A viewer standing well back from a window about one unit tall.
    ///
    /// Distance matters: the illusion relies on perspective being too weak
    /// to betray which edge is really nearer.
    fn default() -> Self {
        Self {
            distance: 12.0,
            focal: 0.72,
        }
    }
}

impl Camera {
    /// Projects a world point onto the picture plane.
    ///
    /// Returns the 2‑D position and the point's distance from the eye, which
    /// is useful for painting far things first.
    #[must_use]
    pub fn project(&self, point: Vec3) -> (Vec2, f32) {
        let depth = self.distance - point.z;
        // The pivot sits at `distance`, so this keeps world scale there.
        let scale = self.focal * self.distance / depth;
        (Vec2::new(point.x * scale, point.y * scale), depth)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pivot_projects_to_centre_at_unit_scale() {
        let camera = Camera::default();
        let (p, depth) = camera.project(Vec3::new(0.5, 0.25, 0.0));
        assert!((p.x - 0.5 * camera.focal).abs() < 1e-6);
        assert!((p.y - 0.25 * camera.focal).abs() < 1e-6);
        assert!((depth - camera.distance).abs() < 1e-6);
    }

    #[test]
    fn nearer_points_project_larger() {
        let camera = Camera::default();
        let (near, _) = camera.project(Vec3::new(1.0, 0.0, 1.0));
        let (far, _) = camera.project(Vec3::new(1.0, 0.0, -1.0));
        assert!(near.x > far.x);
    }
}
