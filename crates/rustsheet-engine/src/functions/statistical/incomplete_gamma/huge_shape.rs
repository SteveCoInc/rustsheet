//! Excel's gamma quantile at a large shape (GAMMA.INV, GAMMAINV, CHISQ.INV, CHISQ.INV.RT and
//! CHIINV from a shape of 1E5; T-2982, T-3070, T-3409; docs/excel-semantics/statistics.md
//! "GAMMA.INV and CHISQ.INV at a huge shape").
//!
//! Excel searches from its Wilson-Hilferty start x0 ([`wilson_hilferty_quantile`]) and its value
//! is one of three things: x0 itself where the density at x0 is not a normal double (its first
//! step cannot move); the true quantile where its search converges within its step limit; and
//! `#NUM!` where it does not. The search behaves as Newton's method on the odds P/Q
//! ([`search_converges`], T-3409): one model in place of T-3070's and T-3350's three fitted cuts
//! of the band (probes/t3070-gamma-inv-band, probes/t3350-gamma-inv-band-lower-edge,
//! probes/t3396-gamma-inv-band-huge, probes-numeric/fit-audit-t3070-*).

use super::{gamma_density, gamma_quantile, temme_pair, Tail, QUANTILE_TEMME_SHAPE_MIN};

/// From this shape on the quantile is [`huge_shape_quantile`]: Excel's start and step limit
/// matter only at a large shape (the step count passes [`SEARCH_STEP_LIMIT`] from about 4E8, the
/// start's density underflows from about 2.5E6, `CHISQ.INV.RT(1E-300,5E6)`, probes/
/// t3350-chisq-inv-huge-df A205), and the search's cdfs are [`temme_pair`] from here. Not a fit:
/// below it neither rule changes a probed cell (T-3409).
pub(crate) const HUGE_SHAPE_MIN: f64 = QUANTILE_TEMME_SHAPE_MIN;

/// The offset Excel adds to the Wilson-Hilferty cube root: (1.0002658235505755)^(1/3) - 1 is
/// 0.0000886 to 13 digits (probes/t2982-gamma-inv-huge-shape A7-A23).
/// FITTED(T-2982, 17 cells, shape 2E10..1E300, p 0.5)
const WILSON_HILFERTY_OFFSET: f64 = 0.000_088_6;

/// The effective factor on the z term, fitted (T-3070) to Excel's values with the three terms
/// below over every closed-form cell from shape 1.5E10 to 1E300 (167 cells of
/// probes/t2982-gamma-inv-huge-shape and probes/t3070-gamma-inv-band). That fit's residual is at
/// most 3.6 ulps with a reference z (Python's `NormalDist`); with the engine's own z
/// (`normal_quantile`) and doubles the output is up to 9 ulps off from 1.5E10 (t3070 A110; 5 to 7
/// at A112, A152, A179, t2982 A63) and 11 to 33 at p = 1E-100 in the right tail from 5E9 to 2E10
/// (A229, A214, A371). T-2982's 1 - 2.2106E-5, fitted at 1E13 alone, was 100 to 12,459 ulps off at
/// 1E11.
/// FITTED(T-3070, 167 cells, shape 1.5E10..1E300, closed-form cells only)
const WILSON_HILFERTY_SPREAD: f64 = 0.999_977_894_167;

/// Effective terms in 1/a, z^2/a and z^3/a^(3/2) inside the cube, fitted to Excel's values with
/// [`WILSON_HILFERTY_SPREAD`] (same cells and residuals; UNVERIFIED as a form: the probes pin
/// values, not Excel's formula). Together they are the absolute -2.3E-3 + 2.0E-4 z^2 that T-2982
/// found left at 1E11 (A60-A77).
/// The 1/a term (fit range: [`WILSON_HILFERTY_SPREAD`]).
/// FITTED(T-3070, 167 cells, shape 1.5E10..1E300, closed-form cells only)
const WILSON_HILFERTY_INVERSE_SHAPE: f64 = -7.586_532e-4;
/// The z^2/a term (fit range: [`WILSON_HILFERTY_SPREAD`]).
/// FITTED(T-3070, 167 cells, shape 1.5E10..1E300, closed-form cells only)
const WILSON_HILFERTY_SQUARE: f64 = 6.740_481e-5;
/// The z^3/a^(3/2) term (fit range: [`WILSON_HILFERTY_SPREAD`]).
/// FITTED(T-3070, 167 cells, shape 1.5E10..1E300, closed-form cells only)
const WILSON_HILFERTY_CUBE: f64 = -3.086_760e-3;

/// Excel's Wilson-Hilferty start for a scale of 1, where `z` is the standard normal quantile of
/// the lower tail's probability: a (1 - 1/(9a) + z/(3 sqrt a))^3 with Excel's offset, spread and
/// higher terms inside the cube (probes/t2982-gamma-inv-huge-shape, probes/t3070-gamma-inv-band).
/// Where its density underflows it is Excel's quantile. Below a huge shape the refinement starts
/// from the textbook form instead (`super::wilson_hilferty_start`, T-3131).
pub(super) fn wilson_hilferty_quantile(shape: f64, z: f64) -> f64 {
    let root = shape.sqrt();
    let cube_root = 1.0 + WILSON_HILFERTY_OFFSET - 1.0 / (9.0 * shape)
        + z * WILSON_HILFERTY_SPREAD / (3.0 * root)
        + (WILSON_HILFERTY_INVERSE_SHAPE + WILSON_HILFERTY_SQUARE * z * z) / shape
        + WILSON_HILFERTY_CUBE * z * z * z / (shape * root);
    shape * cube_root * cube_root * cube_root
}

/// The step limit of Excel's search ([`search_converges`]): it converges where Newton's method on
/// the odds P/Q from the start takes a step below [`SEARCH_STEP_TOLERANCE`] of x within this many
/// steps (T-3409). A fit (UNVERIFIED as Excel's own count and tolerance; the probes pin where the
/// search fails, not how Excel counts): replayed with 40-digit Temme cdfs and the tolerance 1E-8,
/// the last values take 207 steps (`GAMMA.INV(0.9,5E9,1)`, `GAMMA.INV(5.47E-6,8000000000,1)`,
/// `CHISQ.INV.RT(0.1,1E10)`) and the first `#NUM!` 208 (`GAMMA.INV(1E-180,1E9,1)`,
/// `GAMMA.INV(1E-80,9E09,1)`, `GAMMA.INV(1E-85,8E09,1)`); it separates 804 of the 805 probed
/// cells from shape 1E8 that are not the start (the miss, `GAMMA.INV(1.11E-10,9000000000,1)`, is
/// a value at 208). Each step from the inner side of a tail
/// moves the start about 1/|z| standard deviations, so the count is about the fall in ln density
/// from the start to the quantile, through the mode when the start lies past it: T-3350's lower
/// edge (ln P(a, x0) - ln p < 205.3), T-3070's upper edge and T-3396's band closing at 1.15E10.
/// FITTED(T-3409, 805 cells, shape 1E8..1.2E10, every probed inversion that is not the start)
const SEARCH_STEP_LIMIT: u32 = 207;

/// The relative step at which [`search_converges`] counts the search as converged. Not a fit:
/// the count changes by at most one between 1E-6 and 3E-16 (quadratic convergence), and 1E-8 is
/// well above the noise of a double cdf.
const SEARCH_STEP_TOLERANCE: f64 = 1e-8;

/// Excel's quantile of the gamma distribution of `shape` (at least [`HUGE_SHAPE_MIN`]) and
/// `scale` where the probability of `tail` is `p` in (0, 1), and `z` the standard normal quantile
/// of the lower tail's probability (for the upper tail, minus that of `p`, so a tiny `p` keeps its
/// digits: `CHISQ.INV.RT(1E-100,1E10)` = 10005667524.015223, probes/t3070-gamma-inv-band A371).
/// NaN where Excel is `#NUM!`.
///
/// - Where the density at the start ([`wilson_hilferty_quantile`]) is below the smallest normal
///   double, the start times the scale: the closed form from shape 2E10 at p = 0.5, and
///   `CHISQ.INV.RT(1E-300,df)` from df 5E6 (probes/t3350-chisq-inv-huge-df A205-A289; at 2E6,
///   A193, the density is normal and Excel is the true quantile). The cut holds on all 197
///   probed start cells from shape 1E5 and no other cell (ln density -708.54 at
///   `GAMMA.INV(0.999999,1.5E10,1)`, t3070 A110, the start; -701.60 at `GAMMA.INV(0.5,1.95E10,1)`,
///   A210, `#NUM!`; ln of the smallest normal double is -708.40).
/// - Else, where [`search_converges`], the true quantile ([`gamma_quantile`]):
///   `GAMMA.INV(0.5,5E9,1)` = 4999999999.666667, `CHISQ.INV(1E-100,6E9)` = 5997669910.951994.
/// - Else NaN: `GAMMA.INV(0.5,6E9,1)` (A65), `GAMMA.INV(1E-300,6E+8,1)` (probes-numeric/
///   fit-audit-t3070-shape-min A19), `GAMMA.INV(p,1.2E10,1)` at every probed p (A85-A98).
///
/// FITTED(T-3070, T-3409, 197 start cells and the cells either side, shape 1E5..1E300, ln density -708.54..-701.60)
pub(crate) fn huge_shape_quantile(shape: f64, scale: f64, p: f64, tail: Tail, z: f64) -> f64 {
    let start = wilson_hilferty_quantile(shape, z);
    if gamma_density(shape, start, 1.0).is_none_or(|density| density < f64::MIN_POSITIVE) {
        return start * scale;
    }
    if search_converges(shape, start, p, tail) {
        gamma_quantile(shape, scale, p, tail)
    } else {
        f64::NAN
    }
}

/// Whether Excel's search from `start` (at scale 1) converges within [`SEARCH_STEP_LIMIT`] steps:
/// Newton's method on the odds P/Q = r, where r is p/(1 - p) for the lower tail and (1 - p)/p for
/// the upper, with P and Q Temme's ([`temme_pair`]). Its step (PQ - r Q^2)/f is about P/f in the
/// lower tail and Q/f past the mode, so from the start it never overshoots (the odds are convex
/// and increasing, and Excel's start lies above the quantile). A step that leaves x positive and
/// finite, a density that is not a positive double, or a step limit reached is a failure.
fn search_converges(shape: f64, start: f64, p: f64, tail: Tail) -> bool {
    let odds = match tail {
        Tail::Lower => p / (1.0 - p),
        Tail::Upper => (1.0 - p) / p,
    };
    let mut x = start;
    for _ in 0..=SEARCH_STEP_LIMIT {
        let density = match gamma_density(shape, x, 1.0) {
            Some(density) if density > 0.0 => density,
            _ => return false,
        };
        let (lower, upper) = temme_pair(shape, x);
        let step = (lower * upper - odds * upper * upper) / density;
        if !step.is_finite() {
            return false;
        }
        if step.abs() < SEARCH_STEP_TOLERANCE * x {
            return true;
        }
        x -= step;
        if !(x > 0.0 && x.is_finite()) {
            return false;
        }
    }
    false
}
