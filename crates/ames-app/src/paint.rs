//! Rasterises a [`Scene`] into a pixmap.
//!
//! This module knows about pixels and colours but nothing about windows or
//! input, which keeps it easy to test: give it a pixmap, get back a picture.

use ames_core::{Angle, Rod, Scene, Shape, Style, Vec2, Vec3, perceived_angle};
use tiny_skia::{
    FillRule, GradientStop, LineCap, LinearGradient, Paint, Path, PathBuilder, Pixmap, Point,
    SpreadMode, Stroke, Transform,
};

use crate::font;
use crate::palette::{self, Swatch};

/// Maps view units onto a region of the pixmap.
#[derive(Clone, Copy, Debug)]
pub struct Viewport {
    /// Pixel position of the scene's origin.
    centre: Point,
    /// Pixels per view unit.
    scale: f32,
}

impl Viewport {
    /// Fits a square view of `-1..=1` into the picture, a little above centre
    /// to leave room for the dial underneath.
    #[must_use]
    pub fn fit(width: u32, height: u32) -> Self {
        #[allow(clippy::cast_precision_loss)]
        let (w, h) = (width as f32, height as f32);
        Self {
            centre: Point::from_xy(w / 2.0, h * 0.42),
            scale: w.min(h) / 2.0,
        }
    }

    /// Pixels per view unit, for turning drag distances into angles.
    #[must_use]
    pub fn scale(&self) -> f32 {
        self.scale
    }

    fn to_pixels(self, p: Vec2) -> Point {
        // View `y` points up; pixel `y` points down.
        Point::from_xy(
            self.centre.x + p.x * self.scale,
            self.centre.y - p.y * self.scale,
        )
    }
}

/// Draws the whole frame: backdrop, window and, when `reveal` is set, the
/// little dial that shows what is really happening.
///
/// The application does the two halves separately, because the backdrop never
/// changes and is worth keeping; see `Canvas` in `app.rs`.
// The tests and the snapshot example draw whole frames; the app does not.
#[cfg_attr(not(test), allow(dead_code))]
pub fn paint(pixmap: &mut Pixmap, scene: &Scene, reveal: bool) {
    backdrop(pixmap);
    paint_scene(pixmap, scene, reveal);
}

/// Draws everything that moves, over a backdrop that is already there.
pub fn paint_scene(pixmap: &mut Pixmap, scene: &Scene, reveal: bool) {
    let viewport = Viewport::fit(pixmap.width(), pixmap.height());
    for shape in scene.render() {
        draw_shape(pixmap, &viewport, &shape);
    }
    if reveal {
        dial(pixmap, &viewport, scene);
    }
}

/// What Rust says first.
pub const GREETING: &str = "Hello, world!";

/// Lays the splash screen over whatever is already drawn.
///
/// `opacity` runs from `1.0` (only the splash) to `0.0` (nothing), so the
/// greeting can dissolve into the window behind it. The cursor is drawn when
/// `cursor_on`, which the caller toggles to make it blink.
pub fn splash(pixmap: &mut Pixmap, opacity: f32, cursor_on: bool) {
    if opacity <= 0.0 {
        return;
    }
    #[allow(clippy::cast_precision_loss)]
    let (w, h) = (pixmap.width() as f32, pixmap.height() as f32);
    let alpha = opacity.clamp(0.0, 1.0);

    let mut backdrop = Paint::default();
    backdrop.set_color(palette::SPLASH.with_opacity(alpha).into());
    if let Some(rect) = tiny_skia::Rect::from_xywh(0.0, 0.0, w, h) {
        pixmap.fill_rect(rect, &backdrop, Transform::identity(), None);
    }

    // Size the lettering to the width, leaving room for the cursor cell.
    #[allow(clippy::cast_precision_loss)]
    let cells = (font::width_in_cells(GREETING) + font::COLUMNS + 2) as f32;
    let cell = (w * 0.86 / cells).min(h / 40.0);
    let text_width = cells * cell;
    let x = (w - text_width) / 2.0;
    #[allow(clippy::cast_precision_loss)]
    let y = (h - font::ROWS as f32 * cell) / 2.0;

    let ink = palette::SPLASH_TEXT.with_opacity(alpha);
    font::draw(pixmap, GREETING, x, y, cell, ink);
    if cursor_on {
        #[allow(clippy::cast_precision_loss)]
        let cursor_x = x + (font::width_in_cells(GREETING) + 2) as f32 * cell;
        let mut paint = Paint::default();
        paint.set_color(ink.into());
        #[allow(clippy::cast_precision_loss)]
        let size = (font::COLUMNS as f32 * cell, font::ROWS as f32 * cell);
        if let Some(rect) = tiny_skia::Rect::from_xywh(cursor_x, y, size.0, size.1) {
            pixmap.fill_rect(rect, &paint, Transform::identity(), None);
        }
    }
}

/// Fills the whole picture with the curtain behind the window.
pub fn backdrop(pixmap: &mut Pixmap) {
    #[allow(clippy::cast_precision_loss)]
    let (w, h) = (pixmap.width() as f32, pixmap.height() as f32);
    let Some(shader) = LinearGradient::new(
        Point::from_xy(0.0, 0.0),
        Point::from_xy(0.0, h),
        vec![
            GradientStop::new(0.0, palette::SKY_TOP.into()),
            GradientStop::new(1.0, palette::SKY_BOTTOM.into()),
        ],
        SpreadMode::Pad,
        Transform::identity(),
    ) else {
        return;
    };
    let paint = Paint {
        shader,
        ..Paint::default()
    };
    if let Some(rect) = tiny_skia::Rect::from_xywh(0.0, 0.0, w, h) {
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
    }
}

fn draw_shape(pixmap: &mut Pixmap, viewport: &Viewport, shape: &Shape) {
    let Some(path) = polyline(viewport, &shape.points, shape.style != Style::Rod) else {
        return;
    };
    match shape.style {
        Style::Frame => {
            fill(pixmap, &path, palette::FRAME);
            stroke(pixmap, &path, palette::FRAME_EDGE, viewport.scale * 0.004);
        }
        Style::Mullion => fill(pixmap, &path, palette::MULLION),
        Style::Reveal => fill(pixmap, &path, palette::REVEAL),
        Style::Rod => stroke(pixmap, &path, palette::ROD, viewport.scale * 0.055),
    }
}

/// A bird's‑eye view under the window, showing what is really happening.
///
/// Three things are drawn, all to one scale. The window, seen from above, is
/// a thin bar with a dot on its tall end. The pen is drawn through it at the
/// angle it is really fixed at, so the two can be seen turning together as
/// one rigid piece. Behind them, thick and in a colour of its own, is the
/// bar the viewer *believes* in: the window swinging back and forth. It is
/// the only thing here that is not real, and it is deliberately not blue, or
/// it reads as a pen that has come loose.
fn dial(pixmap: &mut Pixmap, viewport: &Viewport, scene: &Scene) {
    #[allow(clippy::cast_precision_loss)]
    let (w, h) = (pixmap.width() as f32, pixmap.height() as f32);
    let radius = viewport.scale * 0.16;
    let centre = Point::from_xy(w / 2.0, h - radius * 1.7);
    let actual = scene.angle();

    if let Some(ring) = PathBuilder::from_circle(centre.x, centre.y, radius) {
        stroke(pixmap, &ring, palette::DIAL, 2.0);
    }
    // A tick at the bottom marks where the viewer stands.
    if let Some(eye) = PathBuilder::from_circle(centre.x, centre.y + radius, 3.0) {
        fill(pixmap, &eye, palette::DIAL);
    }

    // Pixels per world unit: the window's half width reaches most of the way
    // to the ring.
    let unit = radius * 0.85 / (scene.window.shape.width / 2.0);
    // Plan view of a point given in the window's own coordinates and turned
    // by `angle`: picture `x` stays `x`, depth toward the viewer goes down.
    let plan = |local: Vec3, angle: Angle| {
        let turned = local.rotate_y(angle);
        Point::from_xy(centre.x + turned.x * unit, centre.y + turned.z * unit)
    };
    let segment = |from: Point, to: Point| {
        let mut pb = PathBuilder::new();
        pb.move_to(from.x, from.y);
        pb.line_to(to.x, to.y);
        pb.finish()
    };
    let [left, right, ..] = scene.window.shape.corners();
    let (tall_edge, short_edge) = (Vec3::new(left.x, 0.0, 0.0), Vec3::new(right.x, 0.0, 0.0));

    // What the eye believes, underneath everything real.
    let believed = perceived_angle(actual);
    if let Some(path) = segment(plan(tall_edge, believed), plan(short_edge, believed)) {
        stroke(pixmap, &path, palette::PERCEIVED, 7.0);
    }
    // Its tall end is marked too. This dot never leaves the viewer's side of
    // the dial, while the true one below goes all the way round.
    let believed_tall = plan(tall_edge, believed);
    if let Some(dot) = PathBuilder::from_circle(believed_tall.x, believed_tall.y, 6.5) {
        fill(pixmap, &dot, palette::PERCEIVED);
    }
    // The window as it truly is, with a dot on its tall end.
    if let Some(path) = segment(plan(tall_edge, actual), plan(short_edge, actual)) {
        stroke(pixmap, &path, palette::ACTUAL, 3.0);
    }
    let tall = plan(tall_edge, actual);
    if let Some(dot) = PathBuilder::from_circle(tall.x, tall.y, 4.5) {
        fill(pixmap, &dot, palette::ACTUAL);
    }
    // The pen, fixed through the window and turning with it.
    for [from, to] in scene.rod.iter().flat_map(Rod::halves) {
        if let Some(path) = segment(plan(from, actual), plan(to, actual)) {
            stroke(pixmap, &path, palette::ROD, 3.0);
        }
    }
}

fn polyline(viewport: &Viewport, points: &[Vec2], closed: bool) -> Option<Path> {
    let (first, rest) = points.split_first()?;
    let mut pb = PathBuilder::new();
    let p = viewport.to_pixels(*first);
    pb.move_to(p.x, p.y);
    for &point in rest {
        let p = viewport.to_pixels(point);
        pb.line_to(p.x, p.y);
    }
    if closed {
        pb.close();
    }
    pb.finish()
}

fn fill(pixmap: &mut Pixmap, path: &Path, color: Swatch) {
    let mut paint = Paint::default();
    paint.set_color(color.into());
    pixmap.fill_path(path, &paint, FillRule::Winding, Transform::identity(), None);
}

fn stroke(pixmap: &mut Pixmap, path: &Path, color: Swatch, width: f32) {
    let mut paint = Paint::default();
    paint.set_color(color.into());
    let stroke = Stroke {
        width,
        line_cap: LineCap::Round,
        ..Stroke::default()
    };
    pixmap.stroke_path(path, &paint, &stroke, Transform::identity(), None);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ames_core::{Camera, Window};

    fn count(pixmap: &Pixmap, color: Swatch) -> usize {
        let target = tiny_skia::Color::from(color).premultiply().to_color_u8();
        pixmap.pixels().iter().filter(|&&p| p == target).count()
    }

    #[test]
    fn a_frame_paints_the_window_and_the_rod() {
        let mut scene = Scene::new(Window::classic(), Camera::default());
        scene.rod = Some(Rod::default());
        scene.set_angle(Angle::degrees(30.0));
        let mut pixmap = Pixmap::new(390, 844).unwrap();
        paint(&mut pixmap, &scene, false);
        assert!(count(&pixmap, palette::FRAME) > 1_000);
        assert!(count(&pixmap, palette::REVEAL) > 300);
        assert!(count(&pixmap, palette::ROD) > 100);
        assert_eq!(count(&pixmap, palette::PERCEIVED), 0);

        paint(&mut pixmap, &scene, true);
        assert!(count(&pixmap, palette::PERCEIVED) > 50);
    }

    /// Counts pixels of `color` in the band at the bottom where the dial is.
    fn count_in_dial(pixmap: &Pixmap, color: Swatch) -> usize {
        let target = tiny_skia::Color::from(color).premultiply().to_color_u8();
        let width = pixmap.width() as usize;
        let from = pixmap.height() as usize * 3 / 4 * width;
        pixmap.pixels()[from..]
            .iter()
            .filter(|&&p| p == target)
            .count()
    }

    #[test]
    fn the_dial_shows_the_pen_only_when_there_is_one() {
        let mut scene = Scene::new(Window::classic(), Camera::default());
        scene.set_angle(Angle::degrees(30.0));
        let mut pixmap = Pixmap::new(390, 844).unwrap();
        paint(&mut pixmap, &scene, true);
        assert_eq!(count_in_dial(&pixmap, palette::ROD), 0);

        scene.rod = Some(Rod::default());
        paint(&mut pixmap, &scene, true);
        assert!(count_in_dial(&pixmap, palette::ROD) > 20);
        // Hidden with the rest of the dial.
        paint(&mut pixmap, &scene, false);
        assert_eq!(count_in_dial(&pixmap, palette::ROD), 0);
    }

    #[test]
    fn the_splash_greets_in_green_and_fades_to_nothing() {
        let scene = Scene::new(Window::classic(), Camera::default());
        let mut pixmap = Pixmap::new(390, 844).unwrap();
        paint(&mut pixmap, &scene, false);
        splash(&mut pixmap, 1.0, true);
        assert!(count(&pixmap, palette::SPLASH_TEXT) > 500);
        assert_eq!(count(&pixmap, palette::FRAME), 0);

        let mut plain = Pixmap::new(390, 844).unwrap();
        paint(&mut plain, &scene, false);
        splash(&mut plain, 0.0, true);
        assert_eq!(count(&plain, palette::SPLASH_TEXT), 0);
    }

    #[test]
    fn viewport_flips_y_and_centres_x() {
        let v = Viewport::fit(200, 400);
        let origin = v.to_pixels(Vec2::new(0.0, 0.0));
        assert!((origin.x - 100.0).abs() < 1e-6);
        let up = v.to_pixels(Vec2::new(0.0, 1.0));
        assert!(up.y < origin.y);
    }
}
