//! The Ames window, as an application.
//!
//! One binary runs everywhere `winit` does: iOS, macOS, Windows, Linux and
//! the browser. Frames are rasterised on the CPU with `tiny-skia` and
//! presented through `softbuffer`, so there is no GPU API and no shader.
//! The only platform‑specific lines are the handful below that hand the
//! event loop to the browser instead of blocking on it.

#![forbid(unsafe_code)]

mod app;
mod font;
mod gesture;
mod paint;
mod palette;

use winit::error::EventLoopError;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> Result<(), EventLoopError> {
    let event_loop = EventLoop::new()?;
    // Animate continuously; each frame asks for the next one.
    event_loop.set_control_flow(ControlFlow::Poll);
    run(event_loop)
}

#[cfg(not(target_arch = "wasm32"))]
fn run(event_loop: EventLoop<()>) -> Result<(), EventLoopError> {
    event_loop.run_app(&mut app::App::default())
}

/// In the browser the event loop cannot block, so it is handed over and
/// `main` returns at once.
#[cfg(target_arch = "wasm32")]
#[allow(clippy::unnecessary_wraps)]
fn run(event_loop: EventLoop<()>) -> Result<(), EventLoopError> {
    use winit::platform::web::EventLoopExtWebSys;
    console_error_panic_hook::set_once();
    event_loop.spawn_app(app::App::default());
    Ok(())
}
