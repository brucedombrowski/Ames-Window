//! Rasterises a [`Scene`] into a pixmap.
//!
//! This module knows about pixels and colours but nothing about windows or
//! input, which keeps it easy to test: give it a pixmap, get back a picture.

use ames_core::{Angle, Scene, Shape, Style, Vec2, perceived_angle};
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

/// Draws the whole frame: backdrop, window and the little dial that shows
/// what is really happening.
pub fn paint(pixmap: &mut Pixmap, scene: &Scene) {
    let viewport = Viewport::fit(pixmap.width(), pixmap.height());
    backdrop(pixmap);
    for shape in scene.render() {
        draw_shape(pixmap, &viewport, &shape);
    }
    dial(pixmap, &viewport, scene.angle());
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

fn backdrop(pixmap: &mut Pixmap) {
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
        Style::Rod => stroke(pixmap, &path, palette::ROD, viewport.scale * 0.03),
    }
}

/// A bird's‑eye view in the corner: the window as a bar seen from above,
/// both as it truly turns and as the viewer believes it swings.
fn dial(pixmap: &mut Pixmap, viewport: &Viewport, actual: Angle) {
    #[allow(clippy::cast_precision_loss)]
    let (w, h) = (pixmap.width() as f32, pixmap.height() as f32);
    let radius = viewport.scale * 0.14;
    let centre = Point::from_xy(w / 2.0, h - radius * 1.8);

    if let Some(ring) = PathBuilder::from_circle(centre.x, centre.y, radius) {
        stroke(pixmap, &ring, palette::DIAL, 2.0);
    }
    // A tick at the bottom marks where the viewer stands.
    if let Some(eye) = PathBuilder::from_circle(centre.x, centre.y + radius, 3.0) {
        fill(pixmap, &eye, palette::DIAL);
    }

    let bar = |angle: Angle| {
        let (sin, cos) = angle.sin_cos();
        // Plan view: picture `x` stays `x`, depth toward the viewer goes down.
        let dir = Point::from_xy(cos * radius * 0.85, sin * radius * 0.85);
        let mut pb = PathBuilder::new();
        pb.move_to(centre.x - dir.x, centre.y - dir.y);
        pb.line_to(centre.x + dir.x, centre.y + dir.y);
        pb.finish()
    };
    if let Some(path) = bar(perceived_angle(actual)) {
        stroke(pixmap, &path, palette::PERCEIVED, 6.0);
    }
    if let Some(path) = bar(actual) {
        stroke(pixmap, &path, palette::ACTUAL, 2.0);
    }
    // A dot on the tall end so the true rotation is unmistakable.
    let (sin, cos) = actual.sin_cos();
    let tall_end = Point::from_xy(
        centre.x - cos * radius * 0.85,
        centre.y - sin * radius * 0.85,
    );
    if let Some(dot) = PathBuilder::from_circle(tall_end.x, tall_end.y, 4.0) {
        fill(pixmap, &dot, palette::ACTUAL);
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
    use ames_core::{Camera, Rod, Window};

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
        paint(&mut pixmap, &scene);
        assert!(count(&pixmap, palette::FRAME) > 1_000);
        assert!(count(&pixmap, palette::ROD) > 100);
    }

    #[test]
    fn the_splash_greets_in_green_and_fades_to_nothing() {
        let scene = Scene::new(Window::classic(), Camera::default());
        let mut pixmap = Pixmap::new(390, 844).unwrap();
        paint(&mut pixmap, &scene);
        splash(&mut pixmap, 1.0, true);
        assert!(count(&pixmap, palette::SPLASH_TEXT) > 500);
        assert_eq!(count(&pixmap, palette::FRAME), 0);

        let mut plain = Pixmap::new(390, 844).unwrap();
        paint(&mut plain, &scene);
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
