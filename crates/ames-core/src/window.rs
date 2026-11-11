//! The trapezoid that pretends to be a rectangle.

use crate::Vec3;

/// The outline of the window: a trapezoid standing on its edge.
///
/// The left edge is `tall` and the right edge is `short`. Horizontal members
/// run straight between the two edges, so they converge toward the right,
/// exactly as a real rectangle's edges would if it receded into the distance.
/// That painted perspective is what the eye refuses to let go of.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Trapezoid {
    /// Distance between the two vertical edges.
    pub width: f32,
    /// Height of the left edge.
    pub tall: f32,
    /// Height of the right edge.
    pub short: f32,
}

impl Trapezoid {
    /// Height of the window at fraction `t` of the way from left to right.
    #[must_use]
    pub fn height_at(&self, t: f32) -> f32 {
        self.tall + (self.short - self.tall) * t
    }

    /// Horizontal position at fraction `t` of the way from left to right.
    #[must_use]
    pub fn x_at(&self, t: f32) -> f32 {
        -self.width / 2.0 + self.width * t
    }

    /// A point on the window plane, given a horizontal fraction `t` and a
    /// vertical fraction `s` (both `0..=1`, bottom‑left to top‑right).
    #[must_use]
    pub fn point(&self, t: f32, s: f32) -> Vec3 {
        let height = self.height_at(t);
        Vec3::new(self.x_at(t), (s - 0.5) * height, 0.0)
    }

    /// The four corners, counter‑clockwise from the bottom‑left.
    #[must_use]
    pub fn corners(&self) -> [Vec3; 4] {
        [
            self.point(0.0, 0.0),
            self.point(1.0, 0.0),
            self.point(1.0, 1.0),
            self.point(0.0, 1.0),
        ]
    }
}

/// One flat, filled piece of the window, in the window's own coordinates.
#[derive(Clone, Debug, PartialEq)]
pub struct Face {
    /// Corners in order around the face.
    pub points: Vec<Vec3>,
}

/// A complete Ames window: outline, frame and mullions.
#[derive(Clone, Debug, PartialEq)]
pub struct Window {
    /// The overall outline.
    pub shape: Trapezoid,
    /// Thickness of the frame bars at the tall edge, as a fraction of `tall`.
    pub frame: f32,
    /// Thickness of the mullions at the tall edge, as a fraction of `tall`.
    pub mullion: f32,
    /// Number of panes across.
    pub columns: u8,
    /// Number of panes down.
    pub rows: u8,
}

impl Window {
    /// The proportions Ames used: six panes, strongly foreshortened.
    #[must_use]
    pub fn classic() -> Self {
        Self {
            shape: Trapezoid {
                width: 1.6,
                tall: 1.2,
                short: 0.6,
            },
            frame: 0.06,
            mullion: 0.03,
            columns: 2,
            rows: 3,
        }
    }

    /// The frame bars, as four quadrilaterals.
    ///
    /// The ring is built from an outer and an inner trapezoid. The inner one
    /// is inset by a thickness that shrinks toward the short edge, so the
    /// frame too looks like it recedes.
    #[must_use]
    pub fn frame_faces(&self) -> Vec<Face> {
        let outer = self.shape.corners();
        let inner = self.inset_corners();
        (0..4)
            .map(|i| {
                let j = (i + 1) % 4;
                Face {
                    points: vec![outer[i], outer[j], inner[j], inner[i]],
                }
            })
            .collect()
    }

    /// The mullions dividing the panes, as thin quadrilaterals.
    #[must_use]
    pub fn mullion_faces(&self) -> Vec<Face> {
        let mut faces = Vec::new();
        let shape = &self.shape;
        let scale = |t: f32| shape.height_at(t) / shape.tall;

        // Vertical mullions: full height at their own `t`, width shrinking
        // with the local scale.
        for c in 1..self.columns {
            let t = f32::from(c) / f32::from(self.columns);
            let half = self.mullion * shape.tall * scale(t) / 2.0;
            let x = shape.x_at(t);
            let h = shape.height_at(t) / 2.0;
            faces.push(Face {
                points: vec![
                    Vec3::new(x - half, -h, 0.0),
                    Vec3::new(x + half, -h, 0.0),
                    Vec3::new(x + half, h, 0.0),
                    Vec3::new(x - half, h, 0.0),
                ],
            });
        }

        // Horizontal mullions: a fixed fraction of the local height on each
        // edge, so they converge like the frame does.
        for r in 1..self.rows {
            let s = f32::from(r) / f32::from(self.rows);
            let half_at = |t: f32| self.mullion * shape.tall * scale(t) / 2.0;
            let (l, r) = (shape.point(0.0, s), shape.point(1.0, s));
            faces.push(Face {
                points: vec![
                    Vec3::new(l.x, l.y - half_at(0.0), 0.0),
                    Vec3::new(r.x, r.y - half_at(1.0), 0.0),
                    Vec3::new(r.x, r.y + half_at(1.0), 0.0),
                    Vec3::new(l.x, l.y + half_at(0.0), 0.0),
                ],
            });
        }

        faces
    }

    /// Corners of the opening inside the frame.
    fn inset_corners(&self) -> [Vec3; 4] {
        let shape = &self.shape;
        let inset = |t: f32| self.frame * shape.height_at(t);
        let corner = |t: f32, s: f32| {
            let p = shape.point(t, s);
            let dx = if t < 0.5 { inset(t) } else { -inset(t) };
            let dy = if s < 0.5 { inset(t) } else { -inset(t) };
            Vec3::new(p.x + dx, p.y + dy, 0.0)
        };
        [
            corner(0.0, 0.0),
            corner(1.0, 0.0),
            corner(1.0, 1.0),
            corner(0.0, 1.0),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edges_have_their_own_heights() {
        let shape = Window::classic().shape;
        let [bl, br, tr, tl] = shape.corners();
        assert!((tl.y - bl.y - shape.tall).abs() < 1e-6);
        assert!((tr.y - br.y - shape.short).abs() < 1e-6);
        assert!(bl.x < br.x);
    }

    #[test]
    fn window_lies_flat_in_its_plane() {
        let window = Window::classic();
        let faces = window
            .frame_faces()
            .into_iter()
            .chain(window.mullion_faces());
        assert!(faces.flat_map(|f| f.points).all(|p| p.z == 0.0));
    }

    #[test]
    fn mullion_count_matches_panes() {
        let window = Window::classic();
        let expected = usize::from(window.columns - 1) + usize::from(window.rows - 1);
        assert_eq!(window.mullion_faces().len(), expected);
        assert_eq!(window.frame_faces().len(), 4);
    }

    #[test]
    fn inner_opening_is_inside_the_outline() {
        let window = Window::classic();
        let outer = window.shape.corners();
        let inner = window.inset_corners();
        for (o, i) in outer.iter().zip(&inner) {
            assert!(i.x.abs() < o.x.abs());
            assert!(i.y.abs() < o.y.abs());
        }
    }
}
