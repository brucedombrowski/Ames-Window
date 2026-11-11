//! Spin the window, project it, and sort what comes out.

use crate::{Angle, Camera, Rod, Vec2, Vec3, Window};

/// What a projected shape represents, so a renderer can pick colours.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    /// Part of the outer frame.
    Frame,
    /// One of the thin bars between panes.
    Mullion,
    /// The rod through the window.
    Rod,
}

/// A flat shape ready to draw.
///
/// `Frame` and `Mullion` shapes are closed polygons to fill. `Rod` shapes are
/// two‑point polylines to stroke. Shapes come out of [`Scene::render`] sorted
/// from farthest to nearest.
#[derive(Clone, Debug, PartialEq)]
pub struct Shape {
    /// What this shape is.
    pub style: Style,
    /// Picture‑plane points, in view units.
    pub points: Vec<Vec2>,
    /// Mean distance from the eye; larger is farther away.
    pub depth: f32,
}

/// The window, its optional rod, and the camera watching them.
#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    /// The spinning window.
    pub window: Window,
    /// A rod through the window, if one is present.
    pub rod: Option<Rod>,
    /// The viewer.
    pub camera: Camera,
    /// Angular velocity in radians per second. Negative spins the other way.
    pub spin: f32,
    angle: Angle,
}

impl Scene {
    /// A scene at rest, face‑on, turning at Ames's gentle pace.
    #[must_use]
    pub fn new(window: Window, camera: Camera) -> Self {
        Self {
            window,
            rod: None,
            camera,
            spin: 0.6,
            angle: Angle::ZERO,
        }
    }

    /// The window's true rotation about its vertical axis.
    #[must_use]
    pub fn angle(&self) -> Angle {
        self.angle
    }

    /// Sets the true rotation directly, for scrubbing.
    pub fn set_angle(&mut self, angle: Angle) {
        self.angle = angle.wrapped();
    }

    /// Advances time by `seconds`, turning the window at its current spin.
    pub fn advance(&mut self, seconds: f32) {
        self.set_angle(self.angle + Angle::radians(self.spin * seconds));
    }

    /// Projects everything onto the picture plane, back to front.
    ///
    /// The window is flat, so its own pieces never overlap in depth. Each rod
    /// half lies wholly on one side of that plane, so the only ordering that
    /// matters is: halves behind the glass, then the window, then halves in
    /// front of it.
    #[must_use]
    pub fn render(&self) -> Vec<Shape> {
        let mut behind = Vec::new();
        let mut in_front = Vec::new();
        for half in self.rod.iter().flat_map(Rod::halves) {
            let shape = self.shape(Style::Rod, &half);
            if self.is_in_front(half) {
                in_front.push(shape);
            } else {
                behind.push(shape);
            }
        }

        let frame = self
            .window
            .frame_faces()
            .into_iter()
            .map(|face| self.shape(Style::Frame, &face.points));
        let mullions = self
            .window
            .mullion_faces()
            .into_iter()
            .map(|face| self.shape(Style::Mullion, &face.points));

        behind
            .into_iter()
            .chain(frame)
            .chain(mullions)
            .chain(in_front)
            .collect()
    }

    /// Whether a rod half, running from the plane out to its tip, points
    /// toward the eye.
    fn is_in_front(&self, [anchor, tip]: [Vec3; 2]) -> bool {
        let depth = |p: Vec3| self.camera.project(p.rotate_y(self.angle)).1;
        depth(tip) < depth(anchor)
    }

    /// Turns window‑local points into one projected shape.
    fn shape(&self, style: Style, local: &[Vec3]) -> Shape {
        let mut depth = 0.0;
        let points = local
            .iter()
            .map(|&p| {
                let (projected, d) = self.camera.project(p.rotate_y(self.angle));
                depth += d;
                projected
            })
            .collect();
        #[allow(clippy::cast_precision_loss)]
        let depth = depth / local.len().max(1) as f32;
        Shape {
            style,
            points,
            depth,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scene() -> Scene {
        let mut scene = Scene::new(Window::classic(), Camera::default());
        scene.rod = Some(Rod::default());
        scene
    }

    #[test]
    fn exactly_one_rod_half_is_in_front_at_every_angle() {
        let mut scene = scene();
        // Sample between multiples of ten degrees so the one angle at which
        // the rod lies parallel to the picture plane (and has no front) is
        // skipped.
        for step in 0..36 {
            #[allow(clippy::cast_precision_loss)]
            scene.set_angle(Angle::degrees(step as f32 * 10.0 + 5.0));
            let shapes = scene.render();
            let rods = shapes.iter().filter(|s| s.style == Style::Rod).count();
            let first = shapes.iter().position(|s| s.style == Style::Rod).unwrap();
            let last = shapes.iter().rposition(|s| s.style == Style::Rod).unwrap();
            assert_eq!(rods, 2);
            assert_eq!(first, 0);
            assert_eq!(last, shapes.len() - 1);
        }
    }

    #[test]
    fn rod_halves_straddle_the_frame() {
        let mut scene = scene();
        scene.set_angle(Angle::degrees(20.0));
        let shapes = scene.render();
        let first = shapes.iter().position(|s| s.style == Style::Rod).unwrap();
        let last = shapes.iter().rposition(|s| s.style == Style::Rod).unwrap();
        let frame = shapes.iter().position(|s| s.style == Style::Frame).unwrap();
        assert!(first < frame && frame < last);
    }

    #[test]
    fn advancing_wraps_the_angle() {
        let mut scene = scene();
        scene.spin = Angle::FULL.as_radians();
        scene.advance(1.25);
        assert!((scene.angle().as_degrees() - 90.0).abs() < 1e-3);
    }

    #[test]
    fn face_on_window_is_wider_than_edge_on() {
        let mut scene = Scene::new(Window::classic(), Camera::default());
        let width = |s: &Scene| {
            s.render()
                .iter()
                .flat_map(|s| s.points.iter().map(|p| p.x))
                .fold((f32::MAX, f32::MIN), |(lo, hi), x| (lo.min(x), hi.max(x)))
        };
        scene.set_angle(Angle::ZERO);
        let (lo, hi) = width(&scene);
        scene.set_angle(Angle::QUARTER);
        let (lo2, hi2) = width(&scene);
        assert!(hi - lo > hi2 - lo2);
    }
}
