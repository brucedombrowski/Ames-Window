//! Every colour in the app, in one place.

use tiny_skia::Color;

/// A colour the palette can hold in a `const`; `tiny_skia::Color` cannot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Swatch(u8, u8, u8, u8);

impl Swatch {
    /// The same colour with its alpha scaled by `opacity` (`0.0..=1.0`).
    #[must_use]
    pub fn with_opacity(self, opacity: f32) -> Self {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let alpha = (f32::from(self.3) * opacity.clamp(0.0, 1.0)).round() as u8;
        Self(self.0, self.1, self.2, alpha)
    }
}

impl From<Swatch> for Color {
    fn from(Swatch(r, g, b, a): Swatch) -> Self {
        Self::from_rgba8(r, g, b, a)
    }
}

/// Deep slate at the top of the backdrop.
pub const SKY_TOP: Swatch = rgb(0x16, 0x1b, 0x24);
/// Warmer slate at the bottom of the backdrop.
pub const SKY_BOTTOM: Swatch = rgb(0x2a, 0x2f, 0x3a);
/// Splash screen backdrop, a terminal black.
pub const SPLASH: Swatch = rgb(0x0c, 0x0e, 0x12);
/// Splash screen lettering, phosphor green.
pub const SPLASH_TEXT: Swatch = rgb(0x8c, 0xf0, 0x9a);
/// Painted wood for the frame.
pub const FRAME: Swatch = rgb(0xe8, 0xd9, 0xb5);
/// Painted shadow beside each bar, the trick that sells the depth.
pub const SHADOW: Swatch = rgb(0x4a, 0x3f, 0x30);
/// The frame's edge line, a touch darker so bars read as separate pieces.
pub const FRAME_EDGE: Swatch = rgb(0x8a, 0x75, 0x4a);
/// Mullions, a shade lighter than the frame so they do not vanish edge‑on.
pub const MULLION: Swatch = rgb(0xf4, 0xec, 0xd6);
/// The steel rod.
pub const ROD: Swatch = rgb(0xff, 0x6b, 0x4a);
/// Ring of the plan‑view dial.
pub const DIAL: Swatch = rgba(0xff, 0xff, 0xff, 0x30);
/// The window's true orientation on the dial.
pub const ACTUAL: Swatch = rgb(0xe8, 0xd9, 0xb5);
/// The orientation the viewer is tricked into seeing.
pub const PERCEIVED: Swatch = rgb(0x5e, 0xc8, 0xff);

const fn rgb(r: u8, g: u8, b: u8) -> Swatch {
    rgba(r, g, b, 0xff)
}

const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Swatch {
    Swatch(r, g, b, a)
}
