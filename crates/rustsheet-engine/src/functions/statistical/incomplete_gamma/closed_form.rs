//! Excel's closed-form gamma quantiles in the far lower tail (GAMMA.INV, GAMMAINV and CHISQ.INV;
//! T-3013 below shape 1, T-2561 above it; docs/excel-semantics/statistics.md, T-3013 bullet and
//! "Deep lower tail at a shape above 1"): below shape 1 the root of the cdf's leading factor, with
//! or without one Newton step; above it, Newton steps from that root on Excel's cdf (T-3311,
//! T-3403). They moved out of `mod.rs` unchanged (T-3313).

use super::{excel_form_lower, gamma_density, DIRECT_SHAPE_MAX};
use crate::functions::statistical::gammaln::excel_gammaln;
use crate::x87_exp::excel_exp;
use crate::x87_log10::excel_ln;

/// Below this quantile (at scale 1) GAMMA.INV at a shape below 1 takes the closed form
/// [`excel_small_shape_quantile`] and not the search (T-3013). This is a choice inside the
/// evidence bracket (4.15E-22, 6.75E-21), not a measured Excel constant. A28
/// `GAMMA.INV(1E-16,0.75,1)` = 4.147567013992107E-22 is the closed form, while A18 and B18
/// `GAMMA.INV(1E-5,0.25,..)` (6.75E-21 at scale 1) are neither it nor one Newton step from it on
/// the engine's model of Excel's cdf; so are A34 `(1E-16,0.9,1)` (1.6E-18) and
/// gen-m9/60b0b6e69ad35a45 A210 (shape 1/60, 4.9E-19). The cut is on x, not on p or a·LN(x): A28
/// and A34 share p = 1E-16 and nearly the same a·LN(x) (about -36.9), and only A28 is the closed
/// form. Below it the series' correction (about x / (a + 1)) is far below an ulp, so the closed
/// form is the quantile of the leading factor alone. T-3265 probes the gap.
/// FITTED(T-3013, 5 cells, x 4.15E-22..1.6E-18, shape 1/60..0.9)
const CLOSED_FORM_QUANTILE_MAX: f64 = 1e-21;

/// GAMMA.INV takes one Newton step from the closed form where that step is more than this
/// relative part of x, and keeps the closed form otherwise (T-3013). The step is (p - F)/(a F) of
/// x to first order, so a test on the cdf's distance from p alone does not fit. This value is a
/// point inside the evidence bracket (2.78E-14, 4.1E-14), not an Excel constant: 2^-45
/// (2.84E-14) fits too. The step is kept at 2.78E-14 (gen-m9/146b46e74f2dac3b A396
/// `GAMMA.INV(0.05,0.01,EOMONTH(58733,-18))`, where F at the closed form is 2.2E-16 below p) and at
/// 1.33E-14 (A6 `GAMMA.INV(1E-5,0.05,1)`). It is taken at 4.1E-14 (A26 `GAMMA.INV(1E-100,0.75,1)`:
/// F is 3.09E-14 below p at the closed form, and 2.60E-14 above p at the result, Excel's
/// 4.1475670139922064E-134) and at 4.4E-14 (gen-m9/3a25a03eb94fcfbd A254 `GAMMAINV(0.5,0.01,100)`:
/// F is 4.4E-16 below p at the closed form; the result is Excel's 4.4655350189103716E-29). F is
/// [`excel_form_lower`], the leading-factor series form, even where a·LN(x) is above
/// [`EXCEL_EXPONENT_MAX`](super::lower_form::EXCEL_EXPONENT_MAX) and the engine's GAMMA.DIST is the pair. A254 and A396 need that: on
/// GAMMA.DIST's value their steps would be 2.2E-14 (kept) and 4.2E-14 (taken), both against Excel.
/// So either Excel's GAMMA.INV steps on an internal form, or the engine's GAMMA.DIST is off there
/// (T-3265 probes it). One step only: at A26 a second would be 3.46E-14 of x and Excel takes none.
/// A loop under the same test, with the threshold in [3.46E-14, 4.1E-14), fits as well. T-3265
/// probes steps from 2.9E-14 to 4.0E-14.
/// FITTED(T-3013, 4 cells, step 1.33E-14..4.4E-14 of x, shape 0.01..0.75)
const CLOSED_FORM_STEP_MIN: f64 = 3.5e-14;

/// Excel's GAMMA.INV(p, a, 1) at a shape below 1 other than 1/2, where its quantile is tiny
/// (T-3013; docs/excel-semantics/statistics.md, T-3013 bullet): the closed form
/// `EXP((LN(p)+GAMMALN(a)+LN(a))/a)`, the root of the leading factor p = x^a / Gamma(a + 1),
/// summed in that order (probes/t3012-gamma-inv-small-shape: bit-exact at 14 cells of column A,
/// A9 `GAMMA.INV(1E-30,0.1,1)` = 6.073048362407713E-301 and A16 `(1E-16,0.25,1)` =
/// 6.749697893111757E-65 among them, where `GAMMALN(a+1)` in place of the last two terms is one
/// ulp of the exponent off). Where the Newton step on the leading-factor series form
/// ([`excel_form_lower`]) is more than [`CLOSED_FORM_STEP_MIN`] of x, it takes that one step
/// (A26). The value may be 0 or subnormal (Excel's `#DIV/0!`, the caller's to decide). `None` outside [`CLOSED_FORM_QUANTILE_MAX`], at a
/// p outside (0, 1), and where a cdf or density fails: the search applies there.
pub(crate) fn excel_small_shape_quantile(a: f64, p: f64) -> Option<f64> {
    if !(a > 0.0 && a < 1.0 && a != 0.5 && p > 0.0 && p < 1.0) {
        return None;
    }
    let mut x = leading_factor_root(a, p);
    if x.is_nan() || x >= CLOSED_FORM_QUANTILE_MAX {
        return None;
    }
    if x < f64::MIN_POSITIVE {
        return Some(x);
    }
    let (_, step) = form_step(a, x, p)?;
    // A subnormal step is no step. The only evidence is one cell (A9, and B9 at scale 1E+15):
    // `GAMMA.INV(1E-30,0.1,1)` = 6.073048362407713E-301 is the closed form though its step, 8.5E-14
    // of x, is about 5E-314, as if Excel flushed it to zero. A NaN step fails both tests, so it
    // is no step either.
    if step.abs() >= f64::MIN_POSITIVE && step.abs() > CLOSED_FORM_STEP_MIN * x {
        x -= step;
    }
    (x > 0.0 && x.is_finite()).then_some(x)
}

/// The root of the cdf's leading factor, p = x^a / Gamma(a + 1), as Excel forms it:
/// `EXP((LN(p)+GAMMALN(a)+LN(a))/a)`, summed in that order (T-3013 at a shape below 1, T-2561
/// above it). NaN where a logarithm is.
fn leading_factor_root(a: f64, p: f64) -> f64 {
    excel_exp((excel_ln(p) + excel_gammaln(a) + excel_ln(a)) / a)
}

/// The leading-factor series form ([`excel_form_lower`]) at x and the Newton step on it toward p,
/// (F - p) / f with f the density, for [`excel_small_shape_quantile`] and
/// [`excel_newton_quantile`]. `None` where the form or the density fails.
fn form_step(a: f64, x: f64, p: f64) -> Option<(f64, f64)> {
    let cdf = excel_form_lower(a, x, a * excel_ln(x))?;
    Some((cdf, (cdf - p) / gamma_density(a, x, 1.0)?))
}

/// The largest lower-tail probability at which [`excel_newton_quantile`] applies: the top of the
/// range T-3311 measured the steps on, `CHISQ.INV.RT(0.9999999999,3)`, whose lower tail 1 - p is
/// 1.000000082740371E-10 (probes/m9-chisq-inv-rt A21). Above it, to p = 1E-3, the steps are
/// bit-exact more often than the search but put 4 probe cells 5 to 16 ulp off where the search is
/// within 4 (`GAMMA.INV(1E-8,1.5,1)` 11 ulp against 4; T-3403), so the search keeps them.
/// FITTED(T-3311, 23 cells, shape 1.5..7.5, p 1E-200..1.0000000827E-10)
const NEWTON_P_MAX: f64 = 1.000_000_082_740_371e-10;

/// How far, relative to p, the nearest iterate's cdf may be from p for [`excel_newton_quantile`]
/// to return it: a convergence check, not a fit. Where the steps settle on Excel's staircase the
/// nearest level is within 1E-13 of p at every probed point (its levels are up to 3E-14 apart);
/// where they leave the form's domain first (`CHISQ.INV(6.29E-19,39)`, whose root's cdf is 0.59
/// off) it is far above, and the search applies.
const NEWTON_CONVERGED: f64 = 1e-12;

/// The most Newton steps [`excel_newton_quantile`] takes: a bound, not a fit. The value at every
/// probed point is the same for any count from 15 to 50, because the steps soon cycle between a
/// few levels of Excel's staircase cdf (below) and no later iterate is nearer p; 50 leaves that
/// margin, and each step is one series of a shape below 7.5 at x below 1.
const NEWTON_STEPS: u32 = 50;

/// Excel's lower-tail quantile at a shape above 1 and below [`DIRECT_SHAPE_MAX`] and a tiny p
/// (T-3311, T-3403; docs/excel-semantics/statistics.md, "Deep lower tail at a shape above 1"):
/// Newton steps from [`leading_factor_root`] on the leading-factor series form
/// ([`excel_form_lower`], Excel's GAMMA.DIST there) with [`gamma_density`], keeping the first
/// iterate whose cdf is nearest p. The root itself is iterate 0, kept where no step comes nearer.
///
/// Excel's GAMMA.DIST there is that form: within an ulp of it at 304 of 330 probed cells
/// (probes-numeric t3311-gamma-inv-lower-tail-cdf), where the engine's accurate pair is within
/// an ulp at 46. Since LN(x) moves by an ulp only every few ulps of x (every 128 at x = 1.6E-80),
/// the form is a staircase, flat across 3 to 9 ulps of x around Excel's quantiles, and its levels
/// are up to 3E-14 of p apart. Excel's quantile sits on a level that is not the nearest to p from
/// the true quantile: `CHISQ.INV(1E-200,5)` is Excel's 3.233407780583136E-80, 20 ulp above the
/// crossing, on the level 2.29E-14 above p. The steps soon cycle between two to nine levels, and
/// Excel's value is one of the iterates at most points; which one Excel returns is not known, and
/// the nearest-to-p rule is the best of the rules tried (T-3311 Outcome; T-3403: a fixed step
/// count, a step or residual threshold, and the root first when its cdf is near p all score lower).
///
/// T-3311 applied the steps only at shapes 1.5 to 7.5, p from 1E-200, and where the root's cdf was
/// within 1E-6 of p, after T-2561's root (taken where our pair at it was within 5E-14 of p). T-3403
/// drops those gates: over the 805 literal-argument quantile cells of eval/ at shapes 1 to 20 and
/// lower tails up to [`NEWTON_P_MAX`], the steps alone are bit-exact at 543 and over 16 ulp off at
/// 3, against 412 and 113 for the gated rule; on T-3311's own fresh sweeps (t3311-fresh-sweep,
/// -sweep-2), 414 of 511 bit-exact and 16 over 4 ulp off, against 272 and 170.
///
/// `None` outside those shapes and probabilities, where the root is not normal or not below 1,
/// where a cdf or density fails, and where the steps do not converge ([`NEWTON_CONVERGED`]): the
/// search applies there.
pub(super) fn excel_newton_quantile(a: f64, p: f64) -> Option<f64> {
    if !(a > 1.0 && a < DIRECT_SHAPE_MAX && p > 0.0 && p <= NEWTON_P_MAX) {
        return None;
    }
    let mut x = leading_factor_root(a, p);
    if !(f64::MIN_POSITIVE..1.0).contains(&x) {
        return None;
    }
    let (mut cdf, mut step) = form_step(a, x, p)?;
    let miss = (cdf - p).abs();
    if miss.is_nan() {
        return None;
    }
    let mut best = (x, miss);
    for _ in 0..NEWTON_STEPS {
        x -= step;
        // The cdf is convex below the mode a - 1, so the steps stay near the root; this only
        // guards the form's domain.
        if !(f64::MIN_POSITIVE..1.0).contains(&x) {
            break;
        }
        (cdf, step) = form_step(a, x, p)?;
        let miss = (cdf - p).abs();
        if miss < best.1 {
            best = (x, miss);
        }
        if miss == 0.0 {
            break;
        }
    }
    (best.1 <= NEWTON_CONVERGED * p).then_some(best.0)
}
