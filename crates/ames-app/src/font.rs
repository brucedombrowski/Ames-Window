//! A five‑by‑seven bitmap font, just big enough to say hello.
//!
//! `tiny-skia` draws shapes, not text, and shipping a font file for one
//! sentence felt heavy. So the splash screen is lettered the way the first
//! terminals were: a grid of dots per character.

use tiny_skia::{Paint, Pixmap, Rect, Transform};

use crate::palette::Swatch;

/// Width of a glyph in cells.
pub const COLUMNS: u32 = 5;
/// Height of a glyph in cells.
pub const ROWS: u32 = 7;
/// Empty cells between glyphs.
const GAP: u32 = 1;

/// One glyph, as seven rows of five bits, most significant bit on the left.
pub type Glyph = [u8; ROWS as usize];

/// The dot pattern for a character, if the font knows it.
#[must_use]
pub const fn glyph(c: char) -> Option<Glyph> {
    Some(match c {
        ' ' => [0b00000; 7],
        '!' => [
            0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100,
        ],
        ',' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00110, 0b00100, 0b01000,
        ],
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'd' => [
            0b00001, 0b00001, 0b01101, 0b10011, 0b10001, 0b10001, 0b01111,
        ],
        'e' => [
            0b00000, 0b00000, 0b01110, 0b10001, 0b11111, 0b10000, 0b01110,
        ],
        'l' => [
            0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        'o' => [
            0b00000, 0b00000, 0b01110, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'r' => [
            0b00000, 0b00000, 0b10110, 0b11001, 0b10000, 0b10000, 0b10000,
        ],
        'w' => [
            0b00000, 0b00000, 0b10001, 0b10001, 0b10101, 0b10101, 0b01010,
        ],
        _ => return None,
    })
}

/// Width of `text` in cells, including the gaps between glyphs.
#[must_use]
pub fn width_in_cells(text: &str) -> u32 {
    let glyphs = u32::try_from(text.chars().count()).unwrap_or(u32::MAX);
    glyphs * (COLUMNS + GAP) - GAP.min(glyphs)
}

/// Draws `text` with its top‑left corner at `(x, y)`, each cell `cell`
/// pixels square. Characters the font lacks are skipped silently.
pub fn draw(pixmap: &mut Pixmap, text: &str, x: f32, y: f32, cell: f32, color: Swatch) {
    let mut paint = Paint::default();
    paint.set_color(color.into());
    // Leave a sliver of background between dots so they read as dots.
    let dot = cell * 0.82;

    let mut pen = x;
    for c in text.chars() {
        if let Some(rows) = glyph(c) {
            for (row, bits) in rows.iter().enumerate() {
                for col in 0..COLUMNS {
                    if bits >> (COLUMNS - 1 - col) & 1 == 1 {
                        #[allow(clippy::cast_precision_loss)]
                        let (dx, dy) = (col as f32 * cell, row as f32 * cell);
                        if let Some(rect) = Rect::from_xywh(pen + dx, y + dy, dot, dot) {
                            pixmap.fill_rect(rect, &paint, Transform::identity(), None);
                        }
                    }
                }
            }
        }
        #[allow(clippy::cast_precision_loss)]
        {
            pen += (COLUMNS + GAP) as f32 * cell;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_glyph_fits_its_five_columns() {
        for c in "Hello, world!".chars() {
            let rows = glyph(c).unwrap_or_else(|| panic!("missing glyph {c:?}"));
            assert!(rows.iter().all(|row| *row < 1 << COLUMNS), "{c:?}");
        }
    }

    #[test]
    fn unknown_characters_are_none() {
        assert!(glyph('Z').is_none());
    }

    #[test]
    fn width_counts_gaps_between_glyphs_only() {
        assert_eq!(width_in_cells(""), 0);
        assert_eq!(width_in_cells("H"), 5);
        assert_eq!(width_in_cells("He"), 11);
    }
}
