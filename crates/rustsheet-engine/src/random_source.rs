//! Where a workbook's random functions get their draws from (T-5100).
//!
//! RAND, RANDBETWEEN and RANDARRAY each take uniform draws in `[0, 1)` from the model's
//! [`RandomSource`], the way NOW and TODAY read the model's [`Clock`](crate::clock::Clock). The
//! default is the host's generator, which re-rolls on every recalc as Excel does. A seeded source
//! gives the same sequence of draws on every run, and a fixed one returns the same draw every
//! time, so a harness can evaluate one workbook under several sources and see which values
//! depend on the draws. Excel's own draws cannot be reproduced (its seed is not exposed), so no
//! source aims to match them.

/// The source of the draws a workbook's random functions take.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct RandomSource(Kind);

#[derive(Copy, Clone, Debug, Default, PartialEq)]
enum Kind {
    #[default]
    System,
    Seeded(u64),
    /// A draw in `[0, 1)`, checked by [`RandomSource::fixed`].
    Fixed(f64),
}

/// A fixed draw outside `[0, 1)`.
#[derive(thiserror::Error, Debug, Clone, PartialEq)]
#[error("a fixed draw must be a number in [0, 1), not {0}")]
pub struct RandomSourceError(pub f64);

/// The largest double below 1: the top of the draw range.
pub const LARGEST_DRAW: f64 = 1.0 - f64::EPSILON / 2.0;
const _: () = assert!(LARGEST_DRAW < 1.0);

impl RandomSource {
    /// The host's generator: different draws on every run (the default).
    #[must_use]
    pub const fn system() -> Self {
        Self(Kind::System)
    }

    /// A deterministic generator: the same `seed` gives the same sequence of draws, from the
    /// start of the sequence each time the source is set on a model.
    #[must_use]
    pub const fn seeded(seed: u64) -> Self {
        Self(Kind::Seeded(seed))
    }

    /// Every draw is `draw`.
    ///
    /// # Errors
    /// [`RandomSourceError`] when `draw` is not a number in `[0, 1)`.
    pub fn fixed(draw: f64) -> Result<Self, RandomSourceError> {
        if (0.0..1.0).contains(&draw) {
            // `+ 0.0` turns -0 into 0, so RAND never shows a negative zero.
            Ok(Self(Kind::Fixed(draw + 0.0)))
        } else {
            Err(RandomSourceError(draw))
        }
    }

    /// The state a model starts drawing from with this source.
    pub(crate) fn initial_state(self) -> u64 {
        match self.0 {
            Kind::Seeded(seed) => seed,
            Kind::System | Kind::Fixed(_) => 0,
        }
    }

    /// The next draw in `[0, 1)`, advancing `state` (used by the seeded source only).
    pub(crate) fn draw(self, state: &mut u64) -> f64 {
        match self.0 {
            Kind::System => system_draw(),
            Kind::Seeded(_) => unit_interval(splitmix64(state)),
            Kind::Fixed(draw) => draw,
        }
    }
}

/// `SplitMix64` (Steele, Lea and Flood 2014): a small, well-mixed 64-bit generator whose state is
/// a counter, so any seed, zero included, is valid.
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// The top 52 bits of `bits` as a double in `[0, 1)`: they fill the mantissa of a number in
/// `[1, 2)`, and 1 is taken off, which is exact.
fn unit_interval(bits: u64) -> f64 {
    f64::from_bits(0x3FF0_0000_0000_0000 | (bits >> 12)) - 1.0
}

#[cfg(not(target_arch = "wasm32"))]
fn system_draw() -> f64 {
    rand::random()
}

#[cfg(target_arch = "wasm32")]
fn system_draw() -> f64 {
    js_sys::Math::random()
}

impl crate::Model {
    /// The source RAND, RANDBETWEEN and RANDARRAY take their draws from (T-5100).
    #[must_use]
    pub fn random_source(&self) -> RandomSource {
        self.random
    }

    /// Sets the random source, starting a seeded one at the beginning of its sequence. Like
    /// [`Model::set_clock`] it does not evaluate: the next recalculation draws from it.
    pub fn set_random_source(&mut self, random: RandomSource) {
        self.random = random;
        self.random_state = random.initial_state();
    }

    /// The next draw in `[0, 1)` from the model's random source. Every random function draws
    /// through here.
    pub(crate) fn draw(&mut self) -> f64 {
        self.random.draw(&mut self.random_state)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn draws(source: RandomSource, n: usize) -> Vec<f64> {
        let mut state = source.initial_state();
        (0..n).map(|_| source.draw(&mut state)).collect()
    }

    #[test]
    fn a_seed_gives_the_same_draws_in_range_and_another_seed_others() {
        let a = draws(RandomSource::seeded(7), 1000);
        assert_eq!(a, draws(RandomSource::seeded(7), 1000));
        assert_ne!(a, draws(RandomSource::seeded(8), 1000));
        assert!(a.iter().all(|r| (0.0..1.0).contains(r)));
        // Not stuck: the draws spread over the interval.
        assert!(a.iter().any(|&r| r < 0.1) && a.iter().any(|&r| r > 0.9));
        let zero = draws(RandomSource::seeded(0), 2);
        assert_ne!(zero[0].to_bits(), zero[1].to_bits());
    }

    #[test]
    fn a_fixed_source_repeats_its_draw_and_refuses_one_outside_the_interval() {
        let top = RandomSource::fixed(LARGEST_DRAW).unwrap();
        assert_eq!(draws(top, 3), [LARGEST_DRAW; 3]);
        assert_eq!(draws(RandomSource::fixed(0.0).unwrap(), 2), [0.0, 0.0]);
        for bad in [1.0, -1e-300, f64::NAN, f64::INFINITY] {
            assert!(RandomSource::fixed(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn the_system_source_is_the_default_and_draws_in_range() {
        assert_eq!(RandomSource::default(), RandomSource::system());
        assert!(draws(RandomSource::system(), 100)
            .iter()
            .all(|r| (0.0..1.0).contains(r)));
    }

    #[test]
    fn the_unit_interval_reaches_both_ends() {
        assert_eq!(unit_interval(0).to_bits(), 0.0_f64.to_bits());
        assert_eq!(
            unit_interval(u64::MAX).to_bits(),
            (1.0 - f64::EPSILON).to_bits()
        );
    }
}
