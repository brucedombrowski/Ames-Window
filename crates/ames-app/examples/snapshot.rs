//! Renders a few frames to PNG without opening a window.
//!
//! ```sh
//! cargo run -p ames-app --example snapshot -- out_dir
//! ```

#[path = "../src/font.rs"]
#[allow(dead_code)]
mod font;
#[path = "../src/paint.rs"]
#[allow(dead_code)]
mod paint;
#[path = "../src/palette.rs"]
mod palette;

use ames_core::{Angle, Camera, Rod, Scene, Window};
use std::path::PathBuf;
use tiny_skia::Pixmap;

fn main() -> std::io::Result<()> {
    let out = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&out)?;

    let mut scene = Scene::new(Window::classic(), Camera::default());
    scene.rod = Some(Rod::default());

    let mut pixmap = Pixmap::new(390, 844).expect("non-zero size");
    paint::paint(&mut pixmap, &scene);
    paint::splash(&mut pixmap, 1.0, true);
    let png = pixmap.encode_png().map_err(std::io::Error::other)?;
    std::fs::write(out.join("ames-splash.png"), png)?;

    for degrees in [0.0, 30.0, 60.0, 120.0, 150.0] {
        scene.set_angle(Angle::degrees(degrees));
        let mut pixmap = Pixmap::new(390, 844).expect("non-zero size");
        paint::paint(&mut pixmap, &scene);
        let png = pixmap.encode_png().map_err(std::io::Error::other)?;
        std::fs::write(out.join(format!("ames-{degrees:03}.png")), png)?;
    }
    Ok(())
}
