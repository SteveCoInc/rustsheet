//! The continued fraction for Q(a, x) / (x^a e^-x / Gamma(a)) above x = a + 1: the modified Lentz
//! evaluation the double kernel uses ([`continued_fraction`]), and the same fraction in
//! double-double evaluated backward ([`accurate_fraction`], T-3285; any shape since T-4000), whose
//! rounding does not accumulate the way the forward product's does (5.1 ulp at Q(2.5, 5), 16 at
//! Q(1.5, 3)).
// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::many_single_char_names)]
// LINT-DEBT-END

use super::{continued_fraction_budget, Terms};
use crate::x87_exp::{dd_add, dd_div, dd_mul, two_sum};

/// A double-double: an unevaluated sum `hi + lo`.
pub(super) type Dd = (f64, f64);

/// The backward fraction's depth is this many times the forward fraction's term count, plus
/// [`DEPTH_MARGIN`]: the forward one stops once a step changes its value by a quarter of 2^-52,
/// and the omitted tail shrinks geometrically, so twice as deep leaves it far below an ulp.
const DEPTH_FACTOR: u32 = 2;

/// See [`DEPTH_FACTOR`].
const DEPTH_MARGIN: u32 = 8;

/// Q(a, x) / prefactor = 1/(x + 1 - a - 1(1-a)/(x + 3 - a - 2(2-a)/(x + 5 - a - ...))) by the
/// modified Lentz method, and the number of terms it took; `None` past `terms` or the fraction's
/// own [`continued_fraction_budget`].
pub(super) fn continued_fraction(a: f64, x: f64, terms: &mut Terms) -> Option<(f64, u32)> {
    let tiny = f64::MIN_POSITIVE / f64::EPSILON;
    let mut b = x + 1.0 - a;
    let mut c = 1.0 / tiny;
    let mut d = 1.0 / b;
    let mut h = d;
    let mut i = 1.0;
    let mut taken = 0;
    let mut budget = Terms(continued_fraction_budget(a));
    loop {
        if !terms.take() || !budget.take() {
            return None;
        }
        taken += 1;
        let an = -i * (i - a);
        b += 2.0;
        d = an * d + b;
        if d.abs() < tiny {
            d = tiny;
        }
        c = b + an / c;
        if c.abs() < tiny {
            c = tiny;
        }
        d = 1.0 / d;
        let delta = d * c;
        h *= delta;
        if (delta - 1.0).abs() <= f64::EPSILON * 0.25 {
            return Some((h, taken));
        }
        i += 1.0;
    }
}

/// The fraction of [`continued_fraction`] in double-double: the forward fraction sets the depth,
/// from which [`backward_fraction`] evaluates it. `None` past `terms` or the fraction's budget.
pub(super) fn accurate_fraction(a: f64, x: f64, terms: &mut Terms) -> Option<Dd> {
    let (_, taken) = continued_fraction(a, x, terms)?;
    let depth = taken
        .saturating_mul(DEPTH_FACTOR)
        .saturating_add(DEPTH_MARGIN);
    Some(backward_fraction(a, x, depth))
}

/// The continued fraction of [`continued_fraction`] evaluated from depth `depth` up:
/// `t(depth) = 0`, `t(i - 1) = i (a - i) / (x + 1 - a + 2i + t(i))`, then
/// `1 / (x + 1 - a + t(0))`. Each numerator i (a - i) and offset 1 - a + 2i is taken exactly as a
/// double-double (`two_sum`, then the exact product), and x is added to the offset's head by
/// `two_sum`; at a multiple of 1/2 below 20 (the half-integer callers) every low word is 0, so
/// the steps are exact there as before the general shapes of T-4000.
fn backward_fraction(a: f64, x: f64, depth: u32) -> Dd {
    let mut tail: Dd = (0.0, 0.0);
    for i in (1..=depth).rev() {
        let i = f64::from(i);
        let numerator = dd_mul((i, 0.0), two_sum(a, -i));
        let denominator = dd_add(offset_from(x, two_sum(1.0 + 2.0 * i, -a)), tail);
        tail = dd_div(numerator, denominator);
    }
    dd_div((1.0, 0.0), dd_add(offset_from(x, two_sum(1.0, -a)), tail))
}

/// x + offset for a double-double offset: `two_sum` of x and the head, then the low word.
fn offset_from(x: f64, offset: Dd) -> Dd {
    dd_add(two_sum(x, offset.0), (offset.1, 0.0))
}
