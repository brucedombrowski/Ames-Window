//! The `winit` application: owns the window, the scene and the clock.

use std::num::NonZeroU32;
use std::rc::Rc;
use std::time::{Duration, Instant};

use ames_core::{Angle, Camera, Rod, Scene, Window};
use softbuffer::{Context, Surface};
use tiny_skia::Pixmap;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, KeyEvent, MouseButton, TouchPhase, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowId;

use crate::gesture::{Gesture, Pointer};
use crate::paint::{Viewport, paint, splash};

/// How long the greeting stays fully opaque.
const SPLASH_HOLD: Duration = Duration::from_millis(1800);
/// How long it takes to dissolve into the window.
const SPLASH_FADE: Duration = Duration::from_millis(700);
/// Cursor blink period.
const BLINK: Duration = Duration::from_millis(530);

/// Everything that only exists while we have a window to draw into.
///
/// iOS may tear the window down when the app is suspended, so this is kept
/// separate from the state that must survive, like the scene.
struct Graphics {
    window: Rc<winit::window::Window>,
    surface: Surface<Rc<winit::window::Window>, Rc<winit::window::Window>>,
}

/// The application.
pub struct App {
    graphics: Option<Graphics>,
    scene: Scene,
    pointer: Pointer,
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
            scene,
            pointer: Pointer::default(),
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
            self.scene.advance(elapsed.min(0.1));
        }
    }

    /// Draws one frame and asks for the next.
    fn redraw(&mut self) {
        self.tick();
        let Some(graphics) = self.graphics.as_mut() else {
            return;
        };
        let size = graphics.window.inner_size();
        let (Some(width), Some(height)) =
            (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
        else {
            return;
        };
        let Some(mut pixmap) = Pixmap::new(width.get(), height.get()) else {
            return;
        };
        paint(&mut pixmap, &self.scene);
        let since_launch = self.launched.elapsed();
        let cursor_on = (since_launch.as_millis() / BLINK.as_millis()).is_multiple_of(2);
        splash(&mut pixmap, Self::splash_opacity(since_launch), cursor_on);

        if graphics.surface.resize(width, height).is_err() {
            return;
        }
        let Ok(mut buffer) = graphics.surface.buffer_mut() else {
            return;
        };
        for (dst, src) in buffer.iter_mut().zip(pixmap.pixels()) {
            // softbuffer wants 0RGB; the backdrop is opaque so premultiplied
            // RGBA is already straight RGB.
            *dst = (u32::from(src.red()) << 16)
                | (u32::from(src.green()) << 8)
                | u32::from(src.blue());
        }
        let _ = buffer.present();
        graphics.window.request_redraw();
    }

    /// Fully opaque for a moment, then a linear fade, then gone.
    fn splash_opacity(since_launch: Duration) -> f32 {
        let fading = since_launch.saturating_sub(SPLASH_HOLD);
        1.0 - fading.as_secs_f32() / SPLASH_FADE.as_secs_f32()
    }

    fn handle(&mut self, gesture: Gesture) {
        match gesture {
            Gesture::Tap => self.toggle_rod(),
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
            Key::Named(NamedKey::Space) => self.toggle_rod(),
            Key::Character(c) if c.eq_ignore_ascii_case("r") => self.scene.spin = -self.scene.spin,
            _ => {}
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.graphics.is_some() {
            return;
        }
        let attributes = winit::window::Window::default_attributes().with_title("Ames Window");
        let Ok(window) = event_loop.create_window(attributes) else {
            event_loop.exit();
            return;
        };
        let window = Rc::new(window);
        let Ok(context) = Context::new(window.clone()) else {
            event_loop.exit();
            return;
        };
        let Ok(surface) = Surface::new(&context, window.clone()) else {
            event_loop.exit();
            return;
        };
        self.last_tick = Instant::now();
        self.graphics = Some(Graphics { window, surface });
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.graphics = None;
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
