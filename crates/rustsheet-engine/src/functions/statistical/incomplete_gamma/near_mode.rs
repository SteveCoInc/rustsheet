//! The pair (P(a, x), Q(a, x)) near the mode at a shape from [`BAND_SHAPE_MIN`] up to
//! [`TEMME_NEAR_MODE_MIN`], in double-double and rounded once (T-4000), for the accurate-exponent
//! callers only (POISSON's cdf and the quantiles; `pair_with`).
//!
//! Excel is within 1.5 ulp of the true value there (60-digit mpmath `gammainc`):
//! `POISSON.DIST(100,100,TRUE)` = Q(101, 100) is 0.5265621985299984, -0.5 ulp (ironcalc-calc
//! STATISTICAL/POISSON.xlsx D19), and `POISSON(20,25,TRUE)` = Q(21, 25) 0.18549230269414177, +1.5
//! (probes-numeric/t1048-poisson A9, B9). The double kernel was +1.5 and +7.5 ulp off: `rcomp`'s
//! exponent -a rlog(x/a) rounded to a double, and the forward series or Lentz fraction carrying
//! every step's rounding. Temme's 4-term expansion, which takes the band from shape 1E3 (T-3286),
//! is no help below it: its first dropped term is about 3E-13 relative at shape 101.
//!
//! So each factor is taken in double-double:
//! - the prefactor x^a e^-x / Gamma(a) as exp(ln(a)/2 - ln(2 pi)/2 - a phi - mu(a)), with phi =
//!   (x - a)/a - ln(x/a) from [`temme_phi`] and mu Binet's function ([`stirlerr_exact`]);
//! - the power series for P below x = a + 1, and [`accurate_fraction`] for Q from it on;
//! - the other tail as 1 minus it, in double-double.

use super::super::f_density::HALF_LN_TWO_PI;
use super::super::loader::stirlerr_exact;
use super::fraction::{accurate_fraction, Dd};
use super::{temme_phi, Terms, NEAR_MODE, TEMME_NEAR_MODE_MIN};
use crate::x87_exp::{dd_add, dd_div, dd_exp, dd_mul, dd_neg, two_sum};
use crate::x87_log10::ln_double_double;

/// The series stops once a term is below this fraction of the sum: 2^-104, past the
/// double-double sum's own precision, so the terms it leaves (falling geometrically by
/// x/(a + n) < 1) are far below half an ulp of P.
const SERIES_TOLERANCE: f64 = f64::EPSILON * f64::EPSILON;

/// The smallest shape [`near_mode_pair`] takes: the task's range, shapes 21 to 999. At shape 20,
/// where the prefactor first takes Stirling's form, Excel is the double kernel's bits and not the
/// accurate value: `POISSON.DIST(19,20.9,TRUE)` = Q(20, 20.9) is 0.3925598705719857, 2.3 ulp below
/// the truth (gated probes/t3252-gamma-prefactor-window A103), so shape 20 keeps that kernel.
/// (UNVERIFIED, T-4000): no cell between 20 and 21 (a non-integer shape, only the quantiles reach
/// it here) says which side Excel is on.
const BAND_SHAPE_MIN: f64 = 21.0;

/// Whether [`near_mode_pair`] takes (a, x): a shape in [[`BAND_SHAPE_MIN`],
/// [`TEMME_NEAR_MODE_MIN`]) and x within [`NEAR_MODE`] of it, relative to a.
pub(super) fn in_band(a: f64, x: f64) -> bool {
    (BAND_SHAPE_MIN..TEMME_NEAR_MODE_MIN).contains(&a) && (x - a).abs() <= a * NEAR_MODE
}

/// (P, Q) at a point [`in_band`], each rounded once from double-double; `None` past `terms`.
pub(super) fn near_mode_pair(a: f64, x: f64, terms: &mut Terms) -> Option<(f64, f64)> {
    let prefactor = prefactor(a, x);
    let one = (1.0, 0.0);
    if x < a + 1.0 {
        let lower = dd_div(dd_mul(prefactor, lower_series(a, x, terms)?), (a, 0.0));
        Some((lower.0, dd_add(one, dd_neg(lower)).0))
    } else {
        let upper = dd_mul(prefactor, accurate_fraction(a, x, terms)?);
        Some((dd_add(one, dd_neg(upper)).0, upper.0))
    }
}

/// x^a e^-x / Gamma(a) = sqrt(a / 2 pi) exp(-a phi - mu(a)) in double-double. In the band the
/// exponent is at most about 111 in size (a phi below 1E3 · 0.111), well inside [`dd_exp`]'s range.
fn prefactor(a: f64, x: f64) -> Dd {
    let ln_a = ln_double_double(a);
    let half_ln_a = (ln_a.0 / 2.0, ln_a.1 / 2.0);
    let decay = dd_add(dd_mul((a, 0.0), temme_phi(a, x)), stirlerr_exact(a));
    dd_exp(dd_add(
        dd_add(half_ln_a, dd_neg(HALF_LN_TWO_PI)),
        dd_neg(decay),
    ))
}

/// 1 + x/(a + 1) + x^2/((a + 1)(a + 2)) + ... in double-double, for x below a + 1 (so every
/// ratio x/(a + n) is below 1 and the terms fall); `None` past `terms`.
fn lower_series(a: f64, x: f64, terms: &mut Terms) -> Option<Dd> {
    let mut term = (1.0, 0.0);
    let mut sum = (1.0, 0.0);
    let mut n = 0.0;
    loop {
        if !terms.take() {
            return None;
        }
        n += 1.0;
        term = dd_div(dd_mul(term, (x, 0.0)), two_sum(a, n));
        sum = dd_add(sum, term);
        if term.0 <= sum.0 * SERIES_TOLERANCE {
            return Some(sum);
        }
    }
}
