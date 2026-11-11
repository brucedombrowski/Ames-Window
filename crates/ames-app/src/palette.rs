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

/// Dark maroon at the top of the backdrop, like the demonstration's curtain.
pub const SKY_TOP: Swatch = rgb(0x24, 0x11, 0x13);
/// A touch lighter at the bottom.
pub const SKY_BOTTOM: Swatch = rgb(0x3c, 0x1c, 0x1e);
/// Splash screen backdrop, a terminal black.
pub const SPLASH: Swatch = rgb(0x0c, 0x0e, 0x12);
/// Splash screen lettering, phosphor green.
pub const SPLASH_TEXT: Swatch = rgb(0x8c, 0xf0, 0x9a);
/// Whitewashed board for the frame.
pub const FRAME: Swatch = rgb(0xf1, 0xea, 0xdc);
/// The painted reveals, the red that sells the depth.
pub const REVEAL: Swatch = rgb(0xd8, 0x44, 0x26);
/// The frame's edge line, a touch darker so bars read as separate pieces.
pub const FRAME_EDGE: Swatch = rgb(0xb9, 0xae, 0x9b);
/// Mullions, the same board as the frame.
pub const MULLION: Swatch = rgb(0xf1, 0xea, 0xdc);
/// The blue pen.
pub const ROD: Swatch = rgb(0x3d, 0x7c, 0xe6);
/// Ring of the plan‑view dial.
pub const DIAL: Swatch = rgba(0xff, 0xff, 0xff, 0x30);
/// The window's true orientation on the dial.
pub const ACTUAL: Swatch = rgb(0xf1, 0xea, 0xdc);
/// The orientation the viewer is tricked into seeing. Amber, and nothing
/// like the pen's blue: on the dial the two must not be mistaken.
pub const PERCEIVED: Swatch = rgb(0xf2, 0xa6, 0x3a);

const fn rgb(r: u8, g: u8, b: u8) -> Swatch {
    rgba(r, g, b, 0xff)
}

const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Swatch {
    Swatch(r, g, b, a)
}
