//! Every colour in the app, in one place.

use tiny_skia::Color;

/// A colour the palette can hold in a `const`; `tiny_skia::Color` cannot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Swatch(u8, u8, u8, u8);

impl From<Swatch> for Color {
    fn from(Swatch(r, g, b, a): Swatch) -> Self {
        Self::from_rgba8(r, g, b, a)
    }
}

/// Deep slate at the top of the backdrop.
pub const SKY_TOP: Swatch = rgb(0x16, 0x1b, 0x24);
/// Warmer slate at the bottom of the backdrop.
pub const SKY_BOTTOM: Swatch = rgb(0x2a, 0x2f, 0x3a);
/// Painted wood for the frame.
pub const FRAME: Swatch = rgb(0xe8, 0xd9, 0xb5);
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
