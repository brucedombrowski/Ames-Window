//! Turns raw pointer events into the two gestures the app understands.
//!
//! Mouse and touch arrive as different `winit` events but mean the same
//! thing here, so both are funnelled through [`Pointer`].

/// Something the user did.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Gesture {
    /// A press and release with little movement in between.
    Tap,
    /// The pointer moved horizontally by this many pixels while pressed.
    Drag(f32),
}

/// Movement under this many pixels still counts as a tap.
const TAP_SLOP: f32 = 12.0;

/// Tracks a single pressed pointer.
#[derive(Debug, Default)]
pub struct Pointer {
    press: Option<Press>,
}

#[derive(Debug)]
struct Press {
    last_x: f32,
    travelled: f32,
}

impl Pointer {
    /// The pointer went down at `x`.
    pub fn down(&mut self, x: f32) {
        self.press = Some(Press {
            last_x: x,
            travelled: 0.0,
        });
    }

    /// The pointer moved to `x`. Yields a drag once it is clearly not a tap.
    pub fn moved(&mut self, x: f32) -> Option<Gesture> {
        let press = self.press.as_mut()?;
        let dx = x - press.last_x;
        press.last_x = x;
        press.travelled += dx.abs();
        (press.travelled > TAP_SLOP).then_some(Gesture::Drag(dx))
    }

    /// The pointer lifted. Yields a tap if it never became a drag.
    pub fn up(&mut self) -> Option<Gesture> {
        let press = self.press.take()?;
        (press.travelled <= TAP_SLOP).then_some(Gesture::Tap)
    }

    /// Whether a pointer is currently held down.
    pub fn is_down(&self) -> bool {
        self.press.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_still_press_is_a_tap() {
        let mut p = Pointer::default();
        p.down(100.0);
        assert_eq!(p.moved(104.0), None);
        assert_eq!(p.up(), Some(Gesture::Tap));
    }

    #[test]
    fn a_long_move_is_a_drag_and_not_a_tap() {
        let mut p = Pointer::default();
        p.down(0.0);
        assert_eq!(p.moved(10.0), None);
        assert_eq!(p.moved(30.0), Some(Gesture::Drag(20.0)));
        assert_eq!(p.moved(25.0), Some(Gesture::Drag(-5.0)));
        assert_eq!(p.up(), None);
        assert!(!p.is_down());
    }

    #[test]
    fn moves_without_a_press_are_ignored() {
        let mut p = Pointer::default();
        assert_eq!(p.moved(50.0), None);
        assert_eq!(p.up(), None);
    }
}
