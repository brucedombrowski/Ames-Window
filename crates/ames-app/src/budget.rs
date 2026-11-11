//! Decides how many pixels the browser build can afford to draw.
//!
//! Frames are rasterised on the CPU, so the cost of one grows with its area,
//! and the same page has to run on a fast desktop and a slow laptop. Rather
//! than guess one size for both, [`Budget`] times the work each frame takes
//! and, when frames are too slow to animate smoothly, draws fewer pixels and
//! lets the browser scale the canvas up. The picture is flat colour on flat
//! colour, so it survives that well; a stuttering window does not survive at
//! all, because the illusion is in the motion.

/// The longest edge drawn to begin with, in pixels.
pub const START_EDGE: u32 = 1440;
/// The longest edge is never brought below this, however slow the machine.
pub const MIN_EDGE: u32 = 480;
/// The time one frame's work may take, in seconds. A 60 Hz display allows
/// 16.7 ms between frames; this leaves the browser the rest.
const TARGET: f32 = 0.011;
/// How many frames are timed before deciding.
const SAMPLE: u16 = 30;
/// Frames ignored at the start, while the browser is still compiling the
/// module and the first frames are not representative.
const WARM_UP: u16 = 20;
/// No single decision shrinks the edge by more than this factor, so one bad
/// second cannot take the picture straight to the minimum.
const LARGEST_CUT: f32 = 0.6;

/// Tracks how long frames take and lowers the resolution when they overrun.
///
/// It only ever lowers. Raising the resolution again when frames turn out
/// cheap would oscillate around the target, and each change of size is itself
/// a visible flicker.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Budget {
    edge: u32,
    warm_up: u16,
    frames: u16,
    spent: f32,
}

impl Default for Budget {
    fn default() -> Self {
        Self {
            edge: START_EDGE,
            warm_up: WARM_UP,
            frames: 0,
            spent: 0.0,
        }
    }
}

impl Budget {
    /// The longest edge to rasterise right now, in pixels.
    #[must_use]
    pub fn edge(&self) -> u32 {
        self.edge
    }

    /// Records that drawing a frame took `seconds`, with `drawn` pixels on
    /// its longest edge.
    ///
    /// `drawn` matters because a small window is drawn below the limit: the
    /// limit has to come down from what was really drawn, not from where it
    /// stood, or a slow machine with a small window would never be helped.
    pub fn record(&mut self, seconds: f32, drawn: u32) {
        if self.warm_up > 0 {
            self.warm_up -= 1;
            return;
        }
        self.spent += seconds;
        self.frames += 1;
        if self.frames < SAMPLE {
            return;
        }
        let mean = self.spent / f32::from(self.frames);
        self.frames = 0;
        self.spent = 0.0;
        if mean <= TARGET {
            return;
        }
        // Cost is proportional to area, so the edge scales with the square
        // root of the time to be saved.
        let cut = (TARGET / mean).sqrt().max(LARGEST_CUT);
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let edge = (drawn.min(self.edge) as f32 * cut) as u32;
        self.edge = edge.max(MIN_EDGE).min(self.edge);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Feeds enough identical frames to get past the warm-up and reach one
    /// decision.
    fn run(budget: &mut Budget, seconds: f32, drawn: u32) {
        for _ in 0..WARM_UP + SAMPLE {
            budget.record(seconds, drawn);
        }
    }

    #[test]
    fn fast_frames_keep_full_resolution() {
        let mut budget = Budget::default();
        run(&mut budget, 0.004, START_EDGE);
        assert_eq!(budget.edge(), START_EDGE);
    }

    #[test]
    fn slow_frames_lower_it() {
        let mut budget = Budget::default();
        run(&mut budget, 0.022, START_EDGE);
        // Twice over budget: about 1/sqrt(2) of the edge, half the pixels.
        assert!((1000..1040).contains(&budget.edge()), "{}", budget.edge());
    }

    #[test]
    fn the_warm_up_is_not_counted() {
        let mut budget = Budget::default();
        for _ in 0..WARM_UP {
            budget.record(1.0, START_EDGE);
        }
        for _ in 0..SAMPLE {
            budget.record(0.004, START_EDGE);
        }
        assert_eq!(budget.edge(), START_EDGE);
    }

    #[test]
    fn one_decision_never_cuts_too_deep() {
        let mut budget = Budget::default();
        run(&mut budget, 5.0, START_EDGE);
        assert!(budget.edge() >= 860, "{}", budget.edge());
    }

    #[test]
    fn it_stops_at_the_minimum_and_never_rises() {
        let mut budget = Budget::default();
        for _ in 0..20 {
            let drawn = budget.edge();
            run(&mut budget, 5.0, drawn);
        }
        assert_eq!(budget.edge(), MIN_EDGE);
        run(&mut budget, 0.001, MIN_EDGE);
        assert_eq!(budget.edge(), MIN_EDGE);
    }

    #[test]
    fn a_small_window_is_cut_from_what_was_drawn() {
        let mut budget = Budget::default();
        run(&mut budget, 0.022, 800);
        assert!(budget.edge() < 800, "{}", budget.edge());
    }
}
