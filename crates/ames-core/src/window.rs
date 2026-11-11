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

impl Face {
    /// A quadrilateral from four corners.
    fn quad(a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> Self {
        Self {
            points: vec![a, b, c, d],
        }
    }
}

/// Which side of the window faces the viewer.
///
/// The physical model is painted on both sides so that its shadows always
/// fall the same way on screen. A flat model has to mirror them instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// The side seen while the tall edge is on the left.
    Front,
    /// The side seen once the window has turned past edge‑on.
    Back,
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
    /// Width of the painted shadow beside each bar, as a fraction of `tall`.
    pub shadow: f32,
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
            shadow: 0.035,
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
        let half = |t: f32| self.mullion * self.shape.tall * self.scale(t) / 2.0;
        self.vertical_bars(|x, half_width| (x - half_width, x + half_width), half)
            .chain(self.horizontal_bars(|y, half_height| (y - half_height, y + half_height), half))
            .collect()
    }

    /// The painted shadows: dark strips beside each upright and below each
    /// horizontal, as if lit from the upper left of the screen.
    ///
    /// Seen from the `Front`, the left frame bar and the right side of each
    /// mullion cast into the panes. Seen from the `Back` the picture is
    /// mirrored, so it is the right frame bar and the left side of each
    /// mullion. The top bar and the undersides stay where they are.
    #[must_use]
    pub fn shadow_faces(&self, side: Side) -> Vec<Face> {
        let shape = &self.shape;
        let inset = |t: f32| self.frame * shape.height_at(t);
        let bar = |t: f32| self.mullion * shape.tall * self.scale(t) / 2.0;
        let strip = |t: f32| self.shadow * shape.tall * self.scale(t);
        let (frame_t, toward) = match side {
            Side::Front => (0.0, 1.0),
            Side::Back => (1.0, -1.0),
        };

        // Uprights, as (column fraction, x of the edge the shadow starts at).
        let frame_edge = (frame_t, shape.x_at(frame_t) + toward * inset(frame_t));
        let mullion_edges = (1..self.columns).map(|c| {
            let t = f32::from(c) / f32::from(self.columns);
            (t, shape.x_at(t) + toward * bar(t))
        });
        let uprights = std::iter::once(frame_edge)
            .chain(mullion_edges)
            .map(|(t, edge)| {
                let far = edge + toward * strip(t);
                let h = shape.height_at(t) / 2.0;
                Face::quad(
                    Vec3::new(edge.min(far), -h, 0.0),
                    Vec3::new(edge.max(far), -h, 0.0),
                    Vec3::new(edge.max(far), h, 0.0),
                    Vec3::new(edge.min(far), h, 0.0),
                )
            });

        // Rails, as functions giving the underside's y at column fraction t.
        let top_bar = move |t: f32| shape.point(t, 1.0).y - inset(t);
        let rails: Vec<Box<dyn Fn(f32) -> f32>> =
            std::iter::once(Box::new(top_bar) as Box<dyn Fn(f32) -> f32>)
                .chain((1..self.rows).map(|r| {
                    let s = f32::from(r) / f32::from(self.rows);
                    Box::new(move |t: f32| shape.point(t, s).y - bar(t)) as Box<dyn Fn(f32) -> f32>
                }))
                .collect();
        let rails = rails.into_iter().map(|underside| {
            let (l, r) = (underside(0.0), underside(1.0));
            Face::quad(
                Vec3::new(shape.x_at(0.0), l - strip(0.0), 0.0),
                Vec3::new(shape.x_at(1.0), r - strip(1.0), 0.0),
                Vec3::new(shape.x_at(1.0), r, 0.0),
                Vec3::new(shape.x_at(0.0), l, 0.0),
            )
        });

        uprights.chain(rails).collect()
    }

    /// How much everything shrinks at horizontal fraction `t`.
    fn scale(&self, t: f32) -> f32 {
        self.shape.height_at(t) / self.shape.tall
    }

    /// Full‑height quadrilaterals at each interior column boundary.
    fn vertical_bars<'a>(
        &'a self,
        span: impl Fn(f32, f32) -> (f32, f32) + 'a,
        half: impl Fn(f32) -> f32 + 'a,
    ) -> impl Iterator<Item = Face> + 'a {
        (1..self.columns).map(move |c| {
            let t = f32::from(c) / f32::from(self.columns);
            let (lo, hi) = span(self.shape.x_at(t), half(t));
            let h = self.shape.height_at(t) / 2.0;
            Face::quad(
                Vec3::new(lo, -h, 0.0),
                Vec3::new(hi, -h, 0.0),
                Vec3::new(hi, h, 0.0),
                Vec3::new(lo, h, 0.0),
            )
        })
    }

    /// Edge‑to‑edge quadrilaterals at each interior row boundary, whose
    /// thickness follows the local height so they converge like the frame.
    fn horizontal_bars<'a>(
        &'a self,
        span: impl Fn(f32, f32) -> (f32, f32) + 'a,
        half: impl Fn(f32) -> f32 + 'a,
    ) -> impl Iterator<Item = Face> + 'a {
        (1..self.rows).map(move |r| {
            let s = f32::from(r) / f32::from(self.rows);
            let (l, rgt) = (self.shape.point(0.0, s), self.shape.point(1.0, s));
            let (l_lo, l_hi) = span(l.y, half(0.0));
            let (r_lo, r_hi) = span(rgt.y, half(1.0));
            Face::quad(
                Vec3::new(l.x, l_lo, 0.0),
                Vec3::new(rgt.x, r_lo, 0.0),
                Vec3::new(rgt.x, r_hi, 0.0),
                Vec3::new(l.x, l_hi, 0.0),
            )
        })
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
    fn shadows_hug_their_bars() {
        let window = Window::classic();
        let faces = window.shadow_faces(Side::Front);
        let inner_left = window.shape.x_at(0.0) + window.frame * window.shape.tall;
        let min_x = faces[0].points.iter().map(|p| p.x).fold(f32::MAX, f32::min);
        assert!((min_x - inner_left).abs() < 1e-6, "{min_x} vs {inner_left}");
    }

    #[test]
    fn shadows_mirror_between_sides() {
        let window = Window::classic();
        let front = window.shadow_faces(Side::Front);
        let back = window.shadow_faces(Side::Back);
        assert_eq!(front.len(), back.len());
        // The strip beside the middle upright moves from its right to its left.
        let upright_x = window.shape.x_at(0.5);
        let side_of = |faces: &[Face]| faces[1].points.iter().map(|p| p.x - upright_x).sum::<f32>();
        assert!(side_of(&front) > 0.0);
        assert!(side_of(&back) < 0.0);
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
