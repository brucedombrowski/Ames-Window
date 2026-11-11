//! The Ames window, as an application.
//!
//! One binary runs everywhere `winit` does: iOS, macOS, Windows and Linux.
//! Frames are rasterised on the CPU with `tiny-skia` and presented through
//! `softbuffer`, so there is no GPU API, no shader and no platform code here.

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
    event_loop.run_app(&mut app::App::default())
}
