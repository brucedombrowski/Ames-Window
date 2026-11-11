//! # ames-core
//!
//! The geometry behind the **Ames window**, a classic illusion devised by
//! Adelbert Ames Jr. in 1951.
//!
//! A flat trapezoid, painted to look like a rectangular window seen in
//! perspective, is spun slowly about its vertical axis. Because the brain
//! insists the taller edge must be the nearer one, the window appears to
//! swing back and forth through roughly 180° rather than turn full circle.
//! A rod threaded through a pane seems to bend and pass through the frame.
//!
//! This crate has no dependencies and no notion of pixels. It models the
//! window, spins it, projects it onto a flat picture plane, and hands back
//! simple 2‑D shapes sorted back to front. Drawing them is someone else's job.
//!
//! ```
//! use ames_core::{Camera, Scene, Window};
//!
//! let mut scene = Scene::new(Window::classic(), Camera::default());
//! scene.advance(0.5);
//! let shapes = scene.render();
//! assert!(!shapes.is_empty());
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod angle;
mod camera;
mod illusion;
mod math;
mod rod;
mod scene;
mod window;

pub use angle::Angle;
pub use camera::Camera;
pub use illusion::perceived_angle;
pub use math::{Vec2, Vec3};
pub use rod::Rod;
pub use scene::{Scene, Shape, Style};
pub use window::{Face, Trapezoid, Window};
