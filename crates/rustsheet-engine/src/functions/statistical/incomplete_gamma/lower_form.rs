//! Excel's lower-region forms of the incomplete gamma (T-3060, T-3412; docs/excel-semantics/
//! statistics.md, "Lower region at a shape above 1 (T-3412)"): the leading-factor series form of
//! the cdf up to x = 1, its (0.5 - P) + 0.5 complement below x = a + 1, and the density's leading
//! factor. They moved out of `mod.rs` to keep it under the file-size gate.

use super::{lower_series, pair_with, Exponent, Terms, DIRECT_SHAPE_MAX, MAX_TERMS};
use crate::functions::statistical::gammaln::excel_gammaln;
use crate::x87_exp::excel_exp;
use crate::x87_log10::excel_ln;

/// The largest a·LN(x) at which [`excel_small_shape_lower`] takes Excel's form: from |a ln x| = 16
/// on the double exponent's ulp is 2^-48 or more, so its rounding moves P by up to 1.8E-15, the 15th
/// digit (probes/t3012-gamma-dist-at-quantile A3-A5, A16-A18, A21, A24: up to 102 ulps between
/// the forms). Nearer 0 Excel still takes the form (its value is the exact cdf with the double
/// exponent to within 0.1 ulp of that exponent at A20, A25, A30 and A31), but the forms are a few
/// ulps apart and the engine's EXP, LN and GAMMALN leave a 1-ulp residue either way: A31
/// `GAMMA.DIST(1E-5,0.25,1,TRUE)` and C25 are bit-exact on the pair and 1 ulp off with the form,
/// so the pair is kept there (T-3060). No probe cell pins -16 itself: the in-region cells
/// nearest the cut are at about -23.05 (rows 1, 19, 24) and the excluded ones at -11.54 to
/// -11.61 (rows 2, 6, 10, 20, 25), so any cut in between scores the same; -16 is the ulp
/// argument above (T-3242 probes the gap).
/// FITTED(T-3060, 136 cells, shape 0.05..0.9, a ln x -230..-11.5)
pub(super) const EXCEL_EXPONENT_MAX: f64 = -16.0;

/// GAMMA.DIST's cdf P(a, x) at a shape below 1 and a normal x below 1 where a·LN(x) is at most
/// [`EXCEL_EXPONENT_MAX`] (so the pair takes P from the series, not as 1 - Q): Excel's
/// leading factor `EXP(a*LN(x))/EXP(GAMMALN(a+1))`, with a·LN(x) rounded to a double, times e^-x
/// and the series (T-3060). That rounding is Excel's error, up to 1.2E-14 of P where |a ln x| is
/// near 230: `GAMMA.DIST(4.147567013992206E-134,0.75,1,TRUE)` (its literal read as
/// 4.14756701399220E-134) is Excel's 1.000000000000026E-100 against the true
/// 1.0000000000000131E-100 (probes/t3012-gamma-dist-at-quantile A16; docs/excel-semantics/
/// statistics.md, T-3060 bullet). `None` elsewhere, and where the series runs out of terms.
pub(crate) fn excel_small_shape_lower(a: f64, x: f64) -> Option<f64> {
    if !(a > 0.0 && a < 1.0 && (f64::MIN_POSITIVE..1.0).contains(&x)) {
        return None;
    }
    let exponent = a * excel_ln(x);
    if exponent > EXCEL_EXPONENT_MAX {
        return None;
    }
    // P is then about e^-16 / Gamma(a + 1), under 2E-7, so the pair takes it from the series (it
    // takes 1 - Q only where Q is at most 1/2).
    excel_form_lower(a, x, exponent)
}

/// GAMMA.DIST's and CHISQ.DIST's cdf P(a, x) wherever Excel takes its leading-factor series form
/// ([`excel_form_lower`]): below shape 1 where [`excel_small_shape_lower`] takes it, and above
/// shape 1 at a normal x up to 1 ([`excel_lower_above_shape_one`]). `None` elsewhere.
pub(crate) fn excel_lower(a: f64, x: f64) -> Option<f64> {
    excel_small_shape_lower(a, x).or_else(|| excel_lower_above_shape_one(a, x))
}

/// P(a, x) above shape 1 at a normal x of at most 1: Excel's leading-factor series form, with no
/// a·LN(x) gate and no upper shape (T-3412). Excel's cdf there is a staircase in x, since LN(x)
/// moves by an ulp only every few ulps of x; the accurate pair is smooth.
///
/// Evidence, on cells with Excel's own arguments (literals, or column-A quantiles substituted from
/// the oracle): below shape 20 (probes-numeric/t3311-gamma-inv-lower-tail-cdf, t1049-chisqdist,
/// m9-gamma-dist-a2, -a9, m9-chisq-dist), split 30% by a hash of (case, cell) before scoring, the
/// form is bit-exact at 123 of 251 train cells and 60 of 124 test cells and within 4 ulp at all 370
/// normal results, where the pair is bit-exact at 20 and 10 and over 16 ulp off at 134
/// (`GAMMA.DIST(0.9,9,1,TRUE)` 13 ulp to 0, `CHISQ.DIST(0.5,20,TRUE)` 14 to 1). At shapes 40 to
/// 100 (fit-audit-t2638-ratio-min, t3252-gamma-prefactor-window) it is within 7 ulp at all 18
/// cells, where the ratio and `rcomp` prefactors are 6 to 552 off. Fresh sweeps taken after each
/// freeze (decision 0188 R4.2; `docs/research/t3412/fresh_sweep.py`): probes-numeric/t3412-fresh-sweep,
/// shapes 1 to 20, GAMMA.DIST 90 of 90 within 4 ulp (42 bit-exact; the pair 1 and 56 over 16 ulp)
/// and CHISQ.DIST 50 of 50 (27; the pair 2); t3412-fresh-sweep-2, shapes 20 to 170, GAMMA.DIST 55 of
/// 60 within 4 ulp and CHISQ.DIST 27 of 30, every other cell within 16 (the pair: 42 and 19 cells
/// over 16 ulp), no cell farther. Above x = 1 the form is not Excel's (6 of 51 bit-exact, the pair 10;
/// neither is Excel's there, TODO(T-4959)), and the quantiles' lower tail takes the same cut at 1
/// (`quantile_tail`). Shape 1 is e^-x's closed pair. From about shape 170 Gamma(a + 1) overflows and
/// the form is 0, which is not a normal double, so the pair keeps it (0 too, as Excel at shapes
/// 1E9 to 1E200: t2435-dist-extremes, fit-audit-t3271-cdf-cut, t3316-gamma-cdf-huge-shape).
/// `None` elsewhere, where the result is not a normal double, or where the series runs out of
/// terms.
fn excel_lower_above_shape_one(a: f64, x: f64) -> Option<f64> {
    if !(a > 1.0 && (f64::MIN_POSITIVE..=1.0).contains(&x)) {
        return None;
    }
    excel_form_lower(a, x, a * excel_ln(x)).filter(|lower| lower.is_normal())
}

/// Q(a, x) from shape 1 to [`DIRECT_SHAPE_MAX`](super::DIRECT_SHAPE_MAX) below x = a + 1, where the pair takes Q as the
/// complement of the series' P: Excel's complement is (0.5 - P) + 0.5, not 1 - P (the two differ
/// for P below 1/4), with P its leading-factor form up to x = 1 ([`excel_lower_above_shape_one`])
/// and the pair's series above. On the CHISQ.DIST.RT and POISSON cdf cells with Excel's own
/// arguments (the hash split above), it is bit-exact up to x = 1 at 51 of 51 train and 32 of 33 test
/// cells, where 1 - P of the pair is at 43 and 27, and above it at 69 of 98 and 20 of 31, against 58
/// and 16 (T-3412). On the fresh sweep (probes-numeric/t3412-fresh-sweep) CHISQ.DIST.RT is
/// bit-exact at 53 of 60 cells (1 - P: 47) and POISSON's cdf at 43 of 50 (38). From shape 20 it is
/// not Excel's (POISSON: 45 of 86 against 71 for the callers' kernels), so they keep their kernels
/// there. `form` is the pair's large-shape exponent (unused below shape 20). `None` elsewhere, or
/// past `terms`.
pub(super) fn excel_lower_region_upper(
    a: f64,
    x: f64,
    form: Exponent,
    terms: &mut Terms,
) -> Option<f64> {
    if !(a > 1.0 && a < DIRECT_SHAPE_MAX && x > 0.0 && x < a + 1.0) {
        return None;
    }
    let lower = match excel_lower_above_shape_one(a, x) {
        Some(lower) => lower,
        None => pair_with(a, x, terms, form).0,
    };
    let upper = (0.5 - lower) + 0.5;
    (!upper.is_nan()).then_some(upper)
}

/// The leading-factor series form of P(a, x) for x below 1 at any shape below 1, ungated (T-3060):
/// `EXP(exponent)/EXP(GAMMALN(a+1))` times e^-x and the series, where `exponent` is the double
/// a·LN(x). GAMMA.DIST takes it only where a·LN(x) is at most [`EXCEL_EXPONENT_MAX`]; above that
/// the engine's GAMMA.DIST is the accurate pair, a few ulps away. `None` where the series runs
/// out of terms.
pub(super) fn excel_form_lower(a: f64, x: f64, exponent: f64) -> Option<f64> {
    let sum = lower_series(a, x, &mut Terms(MAX_TERMS));
    if sum.is_nan() {
        return None;
    }
    Some(excel_leading_factor(a, exponent) * excel_exp(-x) * sum)
}

/// Excel's leading factor x^a / Gamma(a + 1) of the lower tail, `EXP(exponent)/EXP(GAMMALN(a+1))`
/// with `exponent` the double a·LN(x) (T-3060, T-3412), shared by the cdf form
/// ([`excel_form_lower`]) and the density ([`excel_lower_density`]).
fn excel_leading_factor(a: f64, exponent: f64) -> f64 {
    excel_exp(exponent) / excel_exp(excel_gammaln(a + 1.0))
}

/// GAMMA.DIST's and CHISQ.DIST's density at `x` for shape `a` above 1 where y = x / scale (`y`,
/// as the caller divides) is a normal double of at most 1: the cdf's leading factor times e^-y
/// times a / x, Excel's density there (T-3412 follow-up). On the density cells of eval/ with
/// literal arguments, split 30% by a hash of (case, cell) before scoring, it is bit-exact at 70 of
/// 128 train and 32 of 53 test cells below shape 20, within 1 ulp at 112 and 52, and never more
/// than 4 ulp off, where the engine's log forms were bit-exact at 29 and 14 and over 4 ulp off at
/// 33 and 21; from shape 20 (fit-audit-t2638-ratio-min, t3252-gamma-prefactor-window and others)
/// at 128 of 158 against 96, 1 over 4 ulp against 61. The orders tried (the factor times a / x
/// last, or (e^-y a) / x, or y and the scale in place of x) are 47 to 56 of the 128 train cells.
/// On its fresh sweep (probes-numeric/t3412-density-sweep, 120 cells, shapes 1 to 160, taken after
/// the freeze) it was bit-exact at 76 cells and over 4 ulp off at 1, where the log forms were
/// bit-exact at 33 and over 4 ulp off at 67; with the flush below, 77 and 0.
///
/// That one cell is the factor's flush: Excel's `GAMMA.DIST(4.57398E-12,24.55,2,FALSE)` is 0 where
/// the factor is a subnormal 3E-311 and a / x (5E12) would bring the density back to 2.5E-298, so
/// a subnormal factor is 0 here, as a subnormal operand is elsewhere in Excel (numbers.md "A
/// subnormal divisor is zero"). Only that cell shows it (UNVERIFIED, TODO(T-4960)). `None` elsewhere
/// and where the density is not a normal double.
pub(crate) fn excel_lower_density(a: f64, x: f64, y: f64) -> Option<f64> {
    if !(a > 1.0 && (f64::MIN_POSITIVE..=1.0).contains(&y) && x > 0.0) {
        return None;
    }
    let factor = excel_leading_factor(a, a * excel_ln(y));
    if factor < f64::MIN_POSITIVE {
        return Some(0.0);
    }
    let density = factor * excel_exp(-y) * (a / x);
    density.is_normal().then_some(density)
}

#[cfg(test)]
mod tests;
