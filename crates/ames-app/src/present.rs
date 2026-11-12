//! Puts a finished picture on the screen.
//!
//! The picture arrives as a `tiny-skia` pixmap: four bytes a pixel, red,
//! green, blue, alpha. How it reaches the screen depends on where the app is
//! running, and that is the only thing this module knows.
//!
//! Outside the browser, `softbuffer` presents it. `softbuffer` wants each
//! pixel as one `u32` in `0RGB` order, so every pixel is repacked on the way.
//!
//! In the browser there is a shorter road. A canvas takes exactly the bytes
//! the pixmap already holds, so they are handed over as they are and the
//! browser copies them. Going through `softbuffer` there meant repacking
//! every pixel twice inside WebAssembly. Measured in Chrome, that cost about
//! six times what drawing the picture did; handing the bytes over directly
//! costs a small fraction of it. With WebAssembly's optimising compiler
//! switched off, both drawing and repacking slowed about twofold, and the
//! saving held. How it behaves where a browser does not compile WebAssembly
//! at all has not been measured.

use tiny_skia::Pixmap;

#[cfg(not(target_arch = "wasm32"))]
pub use native::Presenter;
#[cfg(target_arch = "wasm32")]
pub use web::Presenter;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::Pixmap;
    use softbuffer::{Context, Surface};
    use std::num::NonZeroU32;
    use std::rc::Rc;
    use winit::window::Window;

    /// Presents through `softbuffer`.
    pub struct Presenter {
        surface: Surface<Rc<Window>, Rc<Window>>,
    }

    impl Presenter {
        /// A presenter for `window`, or `None` if the platform refuses one.
        pub fn new(window: &Rc<Window>) -> Option<Self> {
            let context = Context::new(window.clone()).ok()?;
            let surface = Surface::new(&context, window.clone()).ok()?;
            Some(Self { surface })
        }

        /// Shows `pixmap`. Returns whether it reached the screen.
        pub fn present(&mut self, pixmap: &Pixmap) -> bool {
            let (Some(width), Some(height)) = (
                NonZeroU32::new(pixmap.width()),
                NonZeroU32::new(pixmap.height()),
            ) else {
                return false;
            };
            if self.surface.resize(width, height).is_err() {
                return false;
            }
            let Ok(mut buffer) = self.surface.buffer_mut() else {
                return false;
            };
            for (dst, src) in buffer.iter_mut().zip(pixmap.pixels()) {
                // softbuffer wants 0RGB; the backdrop is opaque so
                // premultiplied RGBA is already straight RGB.
                *dst = (u32::from(src.red()) << 16)
                    | (u32::from(src.green()) << 8)
                    | u32::from(src.blue());
            }
            buffer.present().is_ok()
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use super::Pixmap;
    use std::rc::Rc;
    use wasm_bindgen::{Clamped, JsCast};
    use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};
    use winit::platform::web::WindowExtWebSys;
    use winit::window::Window;

    /// Presents by handing the pixmap's bytes to the window's canvas.
    pub struct Presenter {
        canvas: HtmlCanvasElement,
        context: CanvasRenderingContext2d,
    }

    impl Presenter {
        /// A presenter for `window`, or `None` if its canvas cannot be drawn
        /// on.
        pub fn new(window: &Rc<Window>) -> Option<Self> {
            let canvas = window.canvas()?;
            let context = canvas
                .get_context("2d")
                .ok()??
                .dyn_into::<CanvasRenderingContext2d>()
                .ok()?;
            Some(Self { canvas, context })
        }

        /// Shows `pixmap`. Returns whether it reached the screen.
        pub fn present(&mut self, pixmap: &Pixmap) -> bool {
            let (width, height) = (pixmap.width(), pixmap.height());
            // Setting a canvas's size clears it and may reallocate it, so
            // only when the picture's size has really changed.
            if self.canvas.width() != width {
                self.canvas.set_width(width);
            }
            if self.canvas.height() != height {
                self.canvas.set_height(height);
            }
            // The backdrop is opaque, so the pixmap's premultiplied bytes are
            // already the straight RGBA a canvas expects.
            let Ok(image) =
                ImageData::new_with_u8_clamped_array_and_sh(Clamped(pixmap.data()), width, height)
            else {
                return false;
            };
            self.context.put_image_data(&image, 0.0, 0.0).is_ok()
        }
    }
}
