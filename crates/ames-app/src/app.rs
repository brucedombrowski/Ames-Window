//! The `winit` application: owns the window, the scene and the clock.

use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

use ames_core::{Angle, Camera, Rod, Scene, Window};
use tiny_skia::Pixmap;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, KeyEvent, MouseButton, TouchPhase, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowId;

use crate::budget::Budget;
use crate::gesture::{Gesture, Pointer};
use crate::paint::{Viewport, backdrop, paint_scene, splash};
use crate::present::Presenter;

/// How long the greeting stays fully opaque.
const SPLASH_HOLD: Duration = Duration::from_millis(1800);
/// How long it takes to dissolve into the window.
const SPLASH_FADE: Duration = Duration::from_millis(700);
/// Cursor blink period.
const BLINK: Duration = Duration::from_millis(530);
/// The longest step the simulation will take in one frame. See `tick`.
const MAX_STEP: f32 = 0.25;

/// Everything that only exists while we have a window to draw into.
///
/// iOS may tear the window down when the app is suspended, so this is kept
/// separate from the state that must survive, like the scene.
struct Graphics {
    window: Rc<winit::window::Window>,
    presenter: Presenter,
}

/// The picture being drawn, kept between frames so that nothing is allocated
/// or repainted that does not change.
struct Canvas {
    /// The frame, drawn afresh each time.
    frame: Pixmap,
    /// The backdrop alone. It is a gradient over every pixel, the most
    /// expensive thing in the picture and the only part that never moves, so
    /// it is painted once per size and copied in thereafter.
    backdrop: Pixmap,
}

impl Canvas {
    fn new(width: u32, height: u32) -> Option<Self> {
        let mut painted = Pixmap::new(width, height)?;
        backdrop(&mut painted);
        Some(Self {
            frame: painted.clone(),
            backdrop: painted,
        })
    }

    fn is(&self, width: u32, height: u32) -> bool {
        self.frame.width() == width && self.frame.height() == height
    }

    /// Wipes the frame back to the bare backdrop.
    fn clear(&mut self) {
        self.frame.data_mut().copy_from_slice(self.backdrop.data());
    }
}

/// How many frames each published mean covers: one second at 60 a second.
const TIMING_FRAMES: u32 = 60;

/// Where a frame's time goes, averaged, so a feedback report can say whether
/// the app is slow or something around it is.
///
/// A frame has two halves. Painting is this app computing pixels.
/// Presenting is getting those pixels to the screen. A report that gives
/// both, beside the frame rate, shows which half a slow machine is slow at,
/// or that neither is and the time is going elsewhere.
#[derive(Debug, Default)]
struct Timing {
    frames: u32,
    painting: Duration,
    presenting: Duration,
}

/// Mean milliseconds per frame spent painting and presenting.
type Means = (f32, f32);

impl Timing {
    /// Adds one frame. Every [`TIMING_FRAMES`] frames, yields the means and
    /// starts again.
    fn record(&mut self, painting: Duration, presenting: Duration) -> Option<Means> {
        self.frames += 1;
        self.painting += painting;
        self.presenting += presenting;
        if self.frames < TIMING_FRAMES {
            return None;
        }
        #[allow(clippy::cast_precision_loss)]
        let mean = |total: Duration| total.as_secs_f32() * 1000.0 / self.frames as f32;
        let means = (mean(self.painting), mean(self.presenting));
        *self = Self::default();
        Some(means)
    }
}

/// Leaves the means on the canvas element, where the page's script can read
/// them for a feedback report. Nothing is sent anywhere.
#[cfg(target_arch = "wasm32")]
fn publish(window: &winit::window::Window, (painting, presenting): Means) {
    use winit::platform::web::WindowExtWebSys;
    if let Some(canvas) = window.canvas() {
        let _ = canvas.set_attribute("data-paint-ms", &format!("{painting:.1}"));
        let _ = canvas.set_attribute("data-present-ms", &format!("{presenting:.1}"));
    }
}

/// Only the browser build has a page to tell.
#[cfg(not(target_arch = "wasm32"))]
fn publish(_window: &winit::window::Window, _means: Means) {}

/// The application.
pub struct App {
    graphics: Option<Graphics>,
    canvas: Option<Canvas>,
    budget: Budget,
    timing: Timing,
    scene: Scene,
    pointer: Pointer,
    /// Whether the dial that gives the game away is showing.
    reveal: bool,
    cursor: PhysicalPosition<f64>,
    last_tick: Instant,
    launched: Instant,
}

impl Default for App {
    fn default() -> Self {
        let mut scene = Scene::new(Window::classic(), Camera::default());
        scene.rod = Some(Rod::default());
        Self {
            graphics: None,
            canvas: None,
            budget: Budget::default(),
            timing: Timing::default(),
            scene,
            pointer: Pointer::default(),
            reveal: false,
            cursor: PhysicalPosition::default(),
            last_tick: Instant::now(),
            launched: Instant::now(),
        }
    }
}

impl App {
    /// Advances the simulation by however long it has been since last time.
    fn tick(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_tick).as_secs_f32();
        self.last_tick = now;
        // While a finger is down the user is in charge of the angle.
        if !self.pointer.is_down() {
            // The clamp stops the window lurching after the tab has been in the
            // background. It must stay well above a plausible frame time: a
            // frame slower than the clamp advances the scene by less than the
            // time that really passed, and the window visibly spins slow.
            self.scene.advance(elapsed.min(MAX_STEP));
        }
    }

    /// Draws one frame and asks for the next.
    fn redraw(&mut self) {
        self.tick();
        let Some(graphics) = self.graphics.as_mut() else {
            return;
        };
        let started = Instant::now();
        let size = render_size(graphics.window.inner_size(), &self.budget);
        let (Some(width), Some(height)) =
            (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
        else {
            return;
        };
        let resized = !self
            .canvas
            .as_ref()
            .is_some_and(|c| c.is(width.get(), height.get()));
        if resized {
            self.canvas = Canvas::new(width.get(), height.get());
        }
        let Some(canvas) = self.canvas.as_mut() else {
            return;
        };
        canvas.clear();
        let pixmap = &mut canvas.frame;
        paint_scene(pixmap, &self.scene, self.reveal);
        let since_launch = self.launched.elapsed();
        let cursor_on = (since_launch.as_millis() / BLINK.as_millis()).is_multiple_of(2);
        let splash_opacity = Self::splash_opacity(since_launch);
        splash(pixmap, splash_opacity, cursor_on);

        let painted = started.elapsed();

        if !graphics.presenter.present(pixmap) {
            return;
        }
        // Time only ordinary frames of the window. The splash is a different,
        // heavier picture, and a frame that repainted the backdrop did work
        // the next one will not.
        if splash_opacity <= 0.0 && !resized {
            let total = started.elapsed();
            self.budget
                .record(total.as_secs_f32(), width.max(height).get());
            let presenting = total.saturating_sub(painted);
            if let Some(means) = self.timing.record(painted, presenting) {
                publish(&graphics.window, means);
            }
        }
        graphics.window.request_redraw();
    }

    /// Fully opaque for a moment, then a linear fade, then gone.
    fn splash_opacity(since_launch: Duration) -> f32 {
        let fading = since_launch.saturating_sub(SPLASH_HOLD);
        1.0 - fading.as_secs_f32() / SPLASH_FADE.as_secs_f32()
    }

    fn handle(&mut self, gesture: Gesture) {
        match gesture {
            Gesture::Tap => self.reveal = !self.reveal,
            Gesture::Drag(dx) => self.scrub(dx),
        }
    }

    fn toggle_rod(&mut self) {
        self.scene.rod = match self.scene.rod {
            Some(_) => None,
            None => Some(Rod::default()),
        };
    }

    /// Turns the window by hand: one view unit of drag is one radian.
    fn scrub(&mut self, dx: f32) {
        let Some(graphics) = &self.graphics else {
            return;
        };
        let size = graphics.window.inner_size();
        let scale = Viewport::fit(size.width, size.height).scale();
        let delta = Angle::radians(dx / scale);
        self.scene.set_angle(self.scene.angle() + delta);
    }

    fn key(&mut self, event: &KeyEvent) {
        if event.state != ElementState::Pressed {
            return;
        }
        match &event.logical_key {
            Key::Named(NamedKey::Space) => self.reveal = !self.reveal,
            Key::Character(c) if c.eq_ignore_ascii_case("x") => self.toggle_rod(),
            Key::Character(c) if c.eq_ignore_ascii_case("r") => self.scene.spin = -self.scene.spin,
            _ => {}
        }
    }
}

/// How many pixels to rasterise for a surface of `size`.
///
/// Frames are drawn on the CPU, so the cost of one is linear in its area. A
/// laptop canvas at a high device-pixel ratio is several megapixels, which is
/// more than WebAssembly rasterises in a frame — and once a frame takes longer
/// than `MAX_STEP` the window appears to spin slowly as well as stutter. So
/// the browser build draws no more than the [`Budget`] allows and lets the
/// browser scale the canvas up. Native rasterisation keeps up with any window
/// a person can open, and is left alone.
fn render_size(size: PhysicalSize<u32>, budget: &Budget) -> PhysicalSize<u32> {
    capped(size, CAPPED.then(|| budget.edge()))
}

/// Whether this build limits how many pixels it draws: only in the browser.
const CAPPED: bool = cfg!(target_arch = "wasm32");

/// `size` with its longest edge brought down to `max_edge`, aspect preserved.
///
/// Idempotent: capping a size that is already within the limit returns it
/// unchanged.
fn capped(size: PhysicalSize<u32>, max_edge: Option<u32>) -> PhysicalSize<u32> {
    let Some(max_edge) = max_edge else {
        return size;
    };
    let longest = size.width.max(size.height);
    if longest <= max_edge || longest == 0 {
        return size;
    }
    // `u64` so the multiplication cannot overflow on a very wide surface.
    let scaled = |n: u32| {
        u32::try_from(u64::from(n) * u64::from(max_edge) / u64::from(longest))
            .unwrap_or(max_edge)
            .max(1)
    };
    PhysicalSize::new(scaled(size.width), scaled(size.height))
}

/// A titled window on the desktop; a canvas appended to the page on the web.
#[cfg(not(target_arch = "wasm32"))]
fn window_attributes() -> winit::window::WindowAttributes {
    winit::window::Window::default_attributes().with_title("Ames Window")
}

#[cfg(target_arch = "wasm32")]
fn window_attributes() -> winit::window::WindowAttributes {
    use winit::platform::web::WindowAttributesExtWebSys;
    winit::window::Window::default_attributes().with_append(true)
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.graphics.is_some() {
            return;
        }
        let attributes = window_attributes();
        let Ok(window) = event_loop.create_window(attributes) else {
            event_loop.exit();
            return;
        };
        let window = Rc::new(window);
        let Some(presenter) = Presenter::new(&window) else {
            event_loop.exit();
            return;
        };
        self.last_tick = Instant::now();
        self.graphics = Some(Graphics { window, presenter });

        // winit hands us a window that belongs to no scene, which on iOS means
        // it is never displayed. Give it one, or we render into the void.
        #[cfg(target_os = "ios")]
        if let Some(g) = &self.graphics {
            crate::ios_scene::adopt(&g.window);
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.graphics = None;
        self.canvas = None;
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => self.redraw(),
            WindowEvent::Resized(_) => {
                if let Some(g) = &self.graphics {
                    g.window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => self.key(&event),

            // Mouse, for the desktop.
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = position;
                #[allow(clippy::cast_possible_truncation)]
                if let Some(gesture) = self.pointer.moved(position.x as f32) {
                    self.handle(gesture);
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } =>
            {
                #[allow(clippy::cast_possible_truncation)]
                match state {
                    ElementState::Pressed => self.pointer.down(self.cursor.x as f32),
                    ElementState::Released => {
                        if let Some(gesture) = self.pointer.up() {
                            self.handle(gesture);
                        }
                    }
                }
            }

            // Touch, for the phone.
            WindowEvent::Touch(touch) => {
                #[allow(clippy::cast_possible_truncation)]
                let x = touch.location.x as f32;
                let gesture = match touch.phase {
                    TouchPhase::Started => {
                        self.pointer.down(x);
                        None
                    }
                    TouchPhase::Moved => self.pointer.moved(x),
                    TouchPhase::Ended | TouchPhase::Cancelled => self.pointer.up(),
                };
                if let Some(gesture) = gesture {
                    self.handle(gesture);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(g) = &self.graphics {
            g.window.request_redraw();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Duration, PhysicalSize, TIMING_FRAMES, Timing, capped};

    #[test]
    fn a_small_surface_is_left_alone() {
        let size = PhysicalSize::new(800, 600);
        assert_eq!(capped(size, Some(1440)), size);
    }

    #[test]
    fn no_limit_means_every_pixel() {
        let size = PhysicalSize::new(3840, 2160);
        assert_eq!(capped(size, None), size);
    }

    #[test]
    fn the_longest_edge_lands_on_the_limit() {
        assert_eq!(
            capped(PhysicalSize::new(3840, 2160), Some(1440)).width,
            1440
        );
        assert_eq!(
            capped(PhysicalSize::new(1200, 3600), Some(1440)).height,
            1440
        );
    }

    #[test]
    fn the_aspect_ratio_survives() {
        let before = PhysicalSize::new(3840u32, 2160u32);
        let after = capped(before, Some(1440));
        let ratio = |s: PhysicalSize<u32>| f64::from(s.width) / f64::from(s.height);
        assert!((ratio(before) - ratio(after)).abs() < 0.01);
    }

    #[test]
    fn capping_twice_changes_nothing() {
        let once = capped(PhysicalSize::new(3840, 2160), Some(1440));
        assert_eq!(capped(once, Some(1440)), once);
    }

    #[test]
    fn a_degenerate_surface_does_not_panic() {
        assert_eq!(capped(PhysicalSize::new(0, 0), Some(1440)).width, 0);
        assert!(capped(PhysicalSize::new(4000, 1), Some(1440)).height >= 1);
    }

    #[test]
    fn timing_yields_means_once_a_batch_and_starts_again() {
        let mut timing = Timing::default();
        let (paint, present) = (Duration::from_millis(4), Duration::from_millis(2));
        for _ in 0..TIMING_FRAMES - 1 {
            assert!(timing.record(paint, present).is_none());
        }
        let (painting, presenting) = timing.record(paint, present).unwrap();
        assert!((painting - 4.0).abs() < 1e-3, "{painting}");
        assert!((presenting - 2.0).abs() < 1e-3, "{presenting}");
        assert!(timing.record(paint, present).is_none());
    }
}
