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

/// A complete Ames window: outline, frame and mullions.
#[derive(Clone, Debug, PartialEq)]
pub struct Window {
    /// The overall outline.
    pub shape: Trapezoid,
    /// Thickness of the frame bars at the tall edge, as a fraction of `tall`.
    pub frame: f32,
    /// Thickness of the mullions at the tall edge, as a fraction of `tall`.
    pub mullion: f32,
    /// Width of the painted reveal strips, as a fraction of `tall`.
    pub reveal: f32,
    /// Number of panes across.
    pub columns: u8,
    /// Number of panes down.
    pub rows: u8,
}

impl Window {
    /// The proportions of the classic demonstration: a wide six‑pane window,
    /// three across and two down, with bars as thick as the frame.
    #[must_use]
    pub fn classic() -> Self {
        Self {
            shape: Trapezoid {
                width: 2.4,
                tall: 1.2,
                short: 0.68,
            },
            frame: 0.075,
            mullion: 0.11,
            reveal: 0.03,
            columns: 3,
            rows: 2,
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

    /// The painted reveals: the coloured strips that stand in for the
    /// thickness of a real frame.
    ///
    /// They sit where a viewer level with, and a little to the right of, a
    /// receding rectangular window would see the inner faces of its
    /// openings: the left side of every pane, the top of the upper row, the
    /// bottom of the lower row, and the frame's outer left, top and bottom
    /// faces. The physical model carries the same paint on both sides of the
    /// board, so these are fixed in window coordinates and never mirrored.
    #[must_use]
    pub fn reveal_faces(&self) -> Vec<Face> {
        let shape = &self.shape;
        let t_of = |x: f32| (x - shape.x_at(0.0)) / shape.width;
        let inset = |t: f32| self.frame * shape.height_at(t);
        let bar = |t: f32| self.mullion * shape.tall * self.scale(t) / 2.0;
        let reveal = |t: f32| self.reveal * shape.tall * self.scale(t);
        let (cols, rows) = (f32::from(self.columns), f32::from(self.rows));

        let mut faces = Vec::new();

        // Outer faces of the frame.
        let (x0, x1) = (shape.x_at(0.0), shape.x_at(1.0));
        let h0 = shape.height_at(0.0) / 2.0;
        faces.push(Face::quad(
            Vec3::new(x0 - reveal(0.0), -h0, 0.0),
            Vec3::new(x0, -h0, 0.0),
            Vec3::new(x0, h0, 0.0),
            Vec3::new(x0 - reveal(0.0), h0, 0.0),
        ));
        for (s, out) in [(1.0, 1.0), (0.0, -1.0)] {
            let (l, r) = (shape.point(0.0, s), shape.point(1.0, s));
            faces.push(Face::quad(
                Vec3::new(x0, l.y, 0.0),
                Vec3::new(x1, r.y, 0.0),
                Vec3::new(x1, r.y + out * reveal(1.0), 0.0),
                Vec3::new(x0, l.y + out * reveal(0.0), 0.0),
            ));
        }

        // Inner faces of each pane.
        for c in 0..self.columns {
            let (t0, t1) = (f32::from(c) / cols, f32::from(c + 1) / cols);
            let left = shape.x_at(t0) + if c == 0 { inset(t0) } else { bar(t0) };
            let right = shape.x_at(t1)
                - if c + 1 == self.columns {
                    inset(t1)
                } else {
                    bar(t1)
                };
            let (tl, tr) = (t_of(left), t_of(right));
            let inner = left + reveal(tl);
            let ti = t_of(inner);

            for r in 0..self.rows {
                let (s0, s1) = (f32::from(r) / rows, f32::from(r + 1) / rows);
                let bottom = |t: f32| shape.point(t, s0).y + if r == 0 { inset(t) } else { bar(t) };
                let top = |t: f32| {
                    shape.point(t, s1).y - if r + 1 == self.rows { inset(t) } else { bar(t) }
                };

                faces.push(Face::quad(
                    Vec3::new(left, bottom(tl), 0.0),
                    Vec3::new(inner, bottom(ti), 0.0),
                    Vec3::new(inner, top(ti), 0.0),
                    Vec3::new(left, top(tl), 0.0),
                ));
                if r + 1 == self.rows {
                    faces.push(Face::quad(
                        Vec3::new(left, top(tl) - reveal(tl), 0.0),
                        Vec3::new(right, top(tr) - reveal(tr), 0.0),
                        Vec3::new(right, top(tr), 0.0),
                        Vec3::new(left, top(tl), 0.0),
                    ));
                }
                if r == 0 {
                    faces.push(Face::quad(
                        Vec3::new(left, bottom(tl), 0.0),
                        Vec3::new(right, bottom(tr), 0.0),
                        Vec3::new(right, bottom(tr) + reveal(tr), 0.0),
                        Vec3::new(left, bottom(tl) + reveal(tl), 0.0),
                    ));
                }
            }
        }

        faces
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
    fn reveals_cover_every_pane_and_the_outer_faces() {
        let window = Window::classic();
        let panes = usize::from(window.columns) * usize::from(window.rows);
        let lit_tops = usize::from(window.columns);
        let lit_bottoms = usize::from(window.columns);
        let outer = 3;
        assert_eq!(
            window.reveal_faces().len(),
            panes + lit_tops + lit_bottoms + outer
        );
    }

    #[test]
    fn pane_reveals_start_at_the_pane_edge() {
        let window = Window::classic();
        let faces = window.reveal_faces();
        let inner_left = window.shape.x_at(0.0) + window.frame * window.shape.tall;
        // Face 3 is the first pane's left reveal (after the three outer faces).
        let min_x = faces[3].points.iter().map(|p| p.x).fold(f32::MAX, f32::min);
        assert!((min_x - inner_left).abs() < 1e-6, "{min_x} vs {inner_left}");
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
