use statrs::distribution::{Beta, Continuous};

use crate::arithmetic::flush_subnormal;
use crate::expressions::types::CellReferenceIndex;
use crate::functions::lift::{div_error, num_error, number_or_error, value_error};
use crate::functions::statistical::beta_density::beta_density_small_shape;
use crate::functions::statistical::bratio::{
    beta_dist_cdf, beta_search_tails, bfrac_shapes_absorbed, dispatch, first_shape_absorbs,
    l140_overflows, Branch,
};
use crate::functions::statistical::incomplete_beta::beta_density;
use crate::functions::statistical::inverse_beta::{
    beta_inverse_cdf, beta_lower_crossing, beta_quantile_crossing,
};
use crate::functions::statistical::loader::{beta_density_counts_vanish, excel_beta_density};
use crate::functions::statistical::special::{ln_gamma, nearest_crossing_from, refine_increasing};
use crate::functions::Function;
use crate::{calc_result::CalcResult, expressions::parser::Node, model::Model};

/// `statrs`' `inverse_cdf` panics from p about 1E-200, does not finish for BETA.INV(1E-100,1000,1)
/// nor for a shape of 1E8 or more, and for the rest of the sweep (T-1543, 9x9 shapes up to 1E15 by
/// 8 probabilities) is bounded and last-digit exact for shapes up to 1E5 and p of 1E-15 or more.
/// Inside that range its search (the capped port of `inverse_beta.rs`, T-1848, which also ends the
/// one hang found there, `BETA.INV(1-2^-53,1.0024,12177)`) is the start of the refinement. Above
/// that shape the quantile is the crossing of our own incomplete beta in Excel's AS 109 structure
/// (`inverse_beta::beta_quantile_crossing`, T-1538; statrs' cdf misses by 13 ulp at
/// `BETAINV(1E-20,1E10,0.5)`, Excel 0.9999999956419133, gen-m9/0a9226016e8f5f33 A358).
const BETA_QUANTILE_STATRS_SHAPE_MAX: f64 = 1e5;
const BETA_QUANTILE_STATRS_P_MIN: f64 = 1e-15;

/// The quantile of `p` in (0, 1): the crossing of BETA.DIST's own cdf (`special::refine_increasing`
/// on `bratio::beta_search_tails`, the kernel BETA.DIST returns where it is ported), from statrs'
/// search as a start where it is reliable. Excel's BETA.INV solves its own cdf to the crossing
/// (docs/excel-semantics/statistics.md "BETA.INV solves Excel's own CDF to the crossing
/// (T-1745)"); until T-3398 the search crossed the older Cephes-based tails, which BETA.DIST no
/// longer returns. Of the final bracket the even-significand end fits Excel better than the end
/// whose cdf is nearer p (T-3398: 11 visibly-off cells fixed and none new, against 16 fixed and
/// 1 new). Above p = 0.5 the target is the exact complement 1 - p on the upper tail,
/// as for F.INV (T-0974). At most 512 Newton steps and 129 cdfs, or 65 cdfs without a start. A
/// shape above 1E5 takes T-1538's crossing instead (above).
fn beta_quantile(alpha: f64, beta: f64, p: f64) -> f64 {
    if first_shape_square_overflows(alpha, beta, p) || newton_density_vanishes(alpha, beta, p) {
        return f64::NAN;
    }
    if alpha > BETA_QUANTILE_STATRS_SHAPE_MAX || beta > BETA_QUANTILE_STATRS_SHAPE_MAX {
        return beta_quantile_crossing(alpha, beta, p);
    }
    if p <= 0.5 && lower_quantile_underflows(alpha, beta, p) {
        return 0.0;
    }
    if let Some(x) = beta_quantile_shape_one(alpha, beta, p) {
        return x;
    }
    let estimate = if p >= BETA_QUANTILE_STATRS_P_MIN {
        beta_inverse_cdf(alpha, beta, p)
    } else {
        f64::NAN
    };
    // A subnormal crossing is the underflow the caller reports as #NUM!, as the old search's 0 was
    // (`BETAINV(0.5,1E-5,1)`, probes/t2435-dist-extremes S!A71).
    let t = if p > 0.5 {
        beta_crossing(
            |t| -beta_search_tails(t, 1.0 - t, alpha, beta).1,
            -(1.0 - p),
            estimate,
            (alpha, beta),
        )
    } else {
        beta_crossing(
            |t| beta_search_tails(t, 1.0 - t, alpha, beta).0,
            p,
            estimate,
            (alpha, beta),
        )
    };
    flush_subnormal(t)
}

/// BETA.INV's quantile for p above 1/2 where the upper-tail search of [`beta_quantile`] rounds to
/// 1: Excel's AS 109 flip, `1 - y` with y the lower quantile of Beta(beta, alpha) at the exact
/// `1 - p` (statistics.md "BETA.INV at a huge shape is AS 109's Newton search (T-1538)"), so 1
/// wherever y is a normal double below half an ulp of 1 (`BETA.INV(1-1E-8,2,0.1)`, y = 3.9E-81).
/// A y of 0 or a subnormal one stays `None` (`#NUM!`), as before T-3351: Excel is `#NUM!` at most
/// such points and 1 at a few that no cut found so far separates, so no threshold is chosen. The
/// evidence and its counts are in statistics.md "BETA.INV where the quantile rounds to 1 (T-3351)".
///
/// y is the crossing of the search's own lower tail (`bratio::beta_search_tails`) over [0, 1], at
/// most 65 cdfs and no statrs start (lift/kinds/statistical.rs `QUANTILE_WEIGHT` charges it); at a
/// shape above 1E5 it is T-1538's crossing (`inverse_beta::beta_lower_crossing`), as
/// [`beta_quantile`] takes there (no evidenced cell reaches that branch yet: TODO(T-4865)).
/// `lower_quantile_underflows` finds a y below the normal doubles without a search.
fn upper_quantile_near_one(alpha: f64, beta: f64, p: f64) -> Option<f64> {
    // 1 - p is exact for p in (1/2, 1).
    let q = 1.0 - p;
    let y = if alpha > BETA_QUANTILE_STATRS_SHAPE_MAX || beta > BETA_QUANTILE_STATRS_SHAPE_MAX {
        beta_lower_crossing(beta, alpha, q)
    } else if lower_quantile_underflows(beta, alpha, q) {
        0.0
    } else {
        refine_increasing(
            |x| beta_search_tails(x, 1.0 - x, beta, alpha).0,
            q,
            f64::NAN,
            0.0,
            1.0,
        )
    };
    (f64::MIN_POSITIVE..0.5).contains(&y).then_some(1.0 - y)
}

/// Whether the square of AS 109's first shape overflows a double: alpha for p up to 1/2, and beta
/// above it, where AS 109 solves `I_x(beta, alpha) = 1 - p` instead (the flip of
/// docs/excel-semantics/statistics.md "BETA.INV at a huge shape is AS 109's Newton search
/// (T-1538)"). Excel's BETA.INV and BETAINV are `#NUM!` exactly from there, whatever p and the
/// other shape: `BETA.INV(1E-10,a,1E-100)` is 0.999999 at a = 1.34078079299425E+154 (a*a finite)
/// and `#NUM!` at 1.34078079299426E+154 (a*a = inf), as are `BETA.INV(0.4,a,0.5)`,
/// `BETA.INV(1E-100,a,1E-100)` and the flipped `BETA.INV(0.6,0.5,a)`, while
/// `BETA.INV(0.1,0.5,a)`, where a is the second shape, stays a number
/// (probes/t3862-beta-inv-sqrt-dbl-max; statistics.md "BETA.INV is #NUM! once the square of AS
/// 109's first shape overflows (T-3862)"). AS 109 as revised (Cran, Martin and Thomas 1977, AS R19,
/// and the statrs port's `5/a/a`) has `pp^2` in its accuracy exponent; which of Excel's
/// expressions squares it is not established, only that the cut is the double's own overflow.
fn first_shape_square_overflows(alpha: f64, beta: f64, p: f64) -> bool {
    let first = if p > 0.5 { beta } else { alpha };
    (first * first).is_infinite()
}

/// Whether AS 109's Newton step has no density: Excel's BETA.DIST density at the shapes after the
/// flip (alpha, beta for p up to 1/2; beta, alpha above it) is `#NUM!` at every x
/// ([`beta_density_counts_vanish`]), and BETA.INV and BETAINV are `#NUM!` on exactly those
/// shapes, whatever p: `BETA.INV(0.1,1E+16,2)` is `#NUM!` while `BETA.INV(0.1,9.44060876285923E+15,2)`
/// and `BETA.INV(0.1,1.81970085860999E+16,2)` around it are numbers, and the flipped
/// `BETA.INV(0.9,a,2)` is a number at every a from 1E+15 to 1E+16, on 469 of the 469 BETA.INV cells
/// with both shapes above 1 of probes/t4713-beta-inv-cut-pieces, t4713-beta-inv-beta-above-1-cut
/// and the two t3862 cases (docs/excel-semantics/statistics.md "BETA.INV and BETA.DIST's density
/// are #NUM! where Loader's n - k is not positive (T-4713)").
fn newton_density_vanishes(alpha: f64, beta: f64, p: f64) -> bool {
    if p > 0.5 {
        beta_density_counts_vanish(beta, alpha)
    } else {
        beta_density_counts_vanish(alpha, beta)
    }
}

/// Whether the x with `I_x(alpha, beta) = p` lies below the smallest normal double, where the
/// quantile underflows: `I_x` at x = `f64::MIN_POSITIVE` is at least p. There `I_x` is its
/// series' leading term `x^alpha/(alpha B(alpha, beta))` to the double (the next term is
/// `alpha (1 - beta) x/(alpha + 1)` relative, far below an ulp at shapes up to 1E5), taken in logs so a
/// tiny alpha does not overflow it. Excel's BETA.INV and BETAINV are `#NUM!` there for p up to 1/2
/// (docs/excel-semantics/statistics.md "BETAINV is #NUM! where its quantile underflows, but 0.5 at
/// a tiny alpha and a high p (T-2435)"); the crossing search cannot see it at alpha 1E-300, where
/// every cdf it evaluates rounds to 1 and it stopped at 3.333333333333333E-302
/// (`BETAINV(1E-20,1E-300,30)`, probes/t2435-dist-extremes A68).
fn lower_quantile_underflows(alpha: f64, beta: f64, p: f64) -> bool {
    let ln_beta = ln_gamma(alpha) + ln_gamma(beta) - ln_gamma(alpha + beta);
    alpha * f64::MIN_POSITIVE.ln() - alpha.ln() - ln_beta >= p.ln()
}

/// The crossing of the search's tail `f` with `target` (`refine_increasing`'s even-significand end
/// of the final bracket), except where bratio takes basym at that point (both shapes above 100,
/// near the mean): there the end of the bracket whose tail is nearer `target`
/// (`special::nearest_crossing_from`). On the same kernel, that end is Excel's on 15 in-sample
/// BETA.INV cells the even end misses and misses 5 it hits (probes-numeric/t3443-basym-a; +11
/// exact on the hold-out, +5 on the fresh sweep), and it keeps
/// `BETA.INV(G29,1000,1000,0.1,1)` at Excel's 0.555 (ironcalc-calc `BETADIST_BETAINV.xlsx` H29: the
/// root's upper tail is one ulp above fl(1 - p), so the even end is the next double). Off basym the
/// nearest end regresses gated cells at small shapes (`BETADIST_BETAINV.xlsx` H16, H60, H61), so
/// it stays the even end there (docs/excel-semantics/statistics.md "BETA.DIST and F.DIST at basym
/// points; BETA.INV takes the nearest end there (T-3443)").
fn beta_crossing(f: impl Fn(f64) -> f64, target: f64, estimate: f64, shapes: (f64, f64)) -> f64 {
    let t = refine_increasing(&f, target, estimate, 0.0, 1.0);
    let at_basym =
        dispatch(shapes.0, shapes.1, t, 1.0 - t).is_some_and(|d| d.branch == Branch::Basym);
    if at_basym {
        nearest_crossing_from(f, target, t, 0.0, 1.0)
    } else {
        t
    }
}

/// The quantile at a shape of exactly 1 and p of 1E-15 or more, in closed form: p when both
/// shapes are 1, `-expm1(log1p(-p)/beta)` when alpha is 1 and `exp(ln(p)/alpha)` when beta is 1.
/// The crossing search cannot be used there: BETA.DIST's cdf at a shape of 1 forms its small tail
/// as (0.5 - w) + 0.5 (T-3007), so it is flat over a band tens of doubles wide and the search
/// stops anywhere in it (probes-numeric/m9-beta-inv A69 `BETA.INV(0.999,1,100)`: 40 ulp off by
/// the search, Excel within an ulp of the 50-digit root). Over the 127 literal shape-1 cells in
/// eval/ with 1E-15 <= p < 1 this form is within 15 digits of Excel at every one (80 bit-exact);
/// `1 - (1-p)^(1/beta)` is not (A51 `BETA.INV(0.001,1,2.5)`, 640 ulp). Below 1E-15 Excel leaves
/// the closed form (probes-numeric/t1543-inverse-cdf-tiny-p A73:A75, A86), so the search stays
/// there. docs/excel-semantics/statistics.md "BETA.INV at a shape of 1 is the closed form to 15
/// digits for p of 1E-15 or more (T-3664)".
fn beta_quantile_shape_one(alpha: f64, beta: f64, p: f64) -> Option<f64> {
    if p < BETA_QUANTILE_STATRS_P_MIN {
        return None;
    }
    match (alpha == 1.0, beta == 1.0) {
        (true, true) => Some(p),
        (true, false) => Some(-((-p).ln_1p() / beta).exp_m1()),
        (false, true) => Some((p.ln() / alpha).exp()),
        (false, false) => None,
    }
}

/// BETA.DIST's `cumulative`: a boolean, a number (TRUE unless 0), "TRUE" or "FALSE" in any case,
/// empty as FALSE, an error as itself; anything else is `#VALUE!`. A one-cell range is its value
/// (probes/stat-later-argument-lift L11, L12). This is `Model::cast_bool_ref` but for a LAMBDA
/// value, which is `#VALUE!` here as in BETA.DIST's scalar code before the lift, where
/// `cast_bool_ref` gives `#N/IMPL!`; it stays separate so the scalar result does not change.
fn beta_cumulative(value: &CalcResult, cell: CellReferenceIndex) -> Result<bool, CalcResult> {
    match value {
        CalcResult::Boolean(b) => Ok(*b),
        CalcResult::Number(n) => Ok(*n != 0.0),
        CalcResult::EmptyArg | CalcResult::EmptyCell => Ok(false),
        CalcResult::String(s) => match s.to_uppercase().as_str() {
            "TRUE" => Ok(true),
            "FALSE" => Ok(false),
            _ => Err(value_error(cell)),
        },
        error @ CalcResult::Error { .. } => Err(error.clone()),
        _ => Err(value_error(cell)),
    }
}

impl Model {
    /// `BETA.DIST(x, alpha, beta, cumulative, [A], [B])`: every argument lifts over an array and
    /// a range (SPEC-0011; probes/stat-argument-modes H84-H89, probes/stat-later-argument-lift
    /// H1-H5, K3-K6). The cdf is `bratio::beta_dist_cdf` (`bratio::beta_dist_lower_tail` plus
    /// T-4045's L140 zero, T-3764): Excel's closed forms on bratio's oriented arguments at a shape
    /// of 1 (T-2502), the ported bpser branches of TOMS 708 bratio (T-2505), and the T-0970
    /// incomplete beta (T-2119) for the branches not ported yet;
    /// the density is `beta_density::beta_density_small_shape` for min(alpha, beta) <= 1 (T-2503)
    /// and `incomplete_beta::beta_density` where it applies above.
    pub(crate) fn fn_beta_dist(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::BetaDist, args, cell, &mut |model, values| {
            let [x, alpha, beta, cumulative, bounds @ ..] = values else {
                return CalcResult::new_args_number_error(cell);
            };
            number_or_error(model.beta_dist_of(x, alpha, beta, Some(cumulative), bounds, cell))
        })
    }

    /// The BETA.DIST kernel; `cumulative` is `None` for BETADIST, always cumulative, and
    /// `bounds` holds the given `[A]` and `[B]`.
    fn beta_dist_of(
        &self,
        x: &CalcResult,
        alpha: &CalcResult,
        beta: &CalcResult,
        cumulative: Option<&CalcResult>,
        bounds: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        // Excel reads a TRUE as 1 in x (probes/stat-later-argument-lift L1).
        let x = self.cast_number_ref(x, cell)?;
        let alpha = self.cast_number_ref(alpha, cell)?;
        let beta_param = self.cast_number_ref(beta, cell)?;
        let cumulative = match cumulative {
            Some(value) => beta_cumulative(value, cell)?,
            None => true,
        };
        let a = self.cast_number_or(bounds.first(), 0.0, cell)?;
        let b = self.cast_number_or(bounds.get(1), 1.0, cell)?;

        // Excel: alpha <= 0 or beta <= 0 → #NUM!; x < A, x > B, or A = B → #NUM!
        if alpha <= 0.0 || beta_param <= 0.0 || b == a || x < a || x > b {
            return Err(num_error(cell));
        }

        // Transform to standard Beta(0,1)
        let width = b - a;
        // B - A past f64::MAX: Excel's cdf is 0 at x = A and 1 at x = B, and everything else is
        // #DIV/0! (`BETA.DIST(0,2,3,TRUE,-1E308,1E308)`, the pdf at x = B; probes/
        // stat-overflow-arguments A7-A10, E9). statrs' incomplete beta would unwrap a NaN `t`.
        if !width.is_finite() {
            return match (cumulative, x == a, x == b) {
                (true, true, _) => Ok(0.0),
                (true, _, true) => Ok(1.0),
                _ => Err(div_error(cell)),
            };
        }
        // The density is #NUM! at an x of exactly 0 with alpha <= 1, and at an x of exactly 1 with
        // beta <= 1: Excel checks the given x against 0 and 1 rather than the interval's own ends,
        // so `BETA.DIST(0,0.5,2,FALSE,-1,1)` and `BETA.DIST(1,2,0.5,FALSE,0,5)` are #NUM! at an
        // interior point while `BETA.DIST(2,0.5,2,FALSE,2,5)` is 0 at the interval's own lower end
        // (statistics.md "BETA.DIST's density is #NUM! at an x of 0 or 1 (T-2435)";
        // probes/t2435-dist-extremes A59:A67, A103:A106, A126:A128, A132:A140).
        if !cumulative && ((x == 0.0 && alpha <= 1.0) || (x == 1.0 && beta_param <= 1.0)) {
            return Err(num_error(cell));
        }
        let t = (x - a) / width;
        let dist = Beta::new(alpha, beta_param).map_err(|_| num_error(cell))?;
        // At the interval's own ends Excel gives 0 rather than the #NUM! an x of 0 or 1 takes
        // above, at a shape of exactly 1 as well as a diverging one, so the density steps down to
        // 0 at the end: `BETA.DIST(2,0.5,2,FALSE,2,5)`, `BETA.DIST(5,2,0.5,FALSE,2,5)`,
        // `BETA.DIST(2,1,2,FALSE,2,5)` and `BETA.DIST(5,2,1,FALSE,2,5)` are 0 where
        // `BETA.DIST(2.0000001,1,2,FALSE,2,5)` is 0.6666666444444445
        // (probes/t2435-dist-extremes A126, A127, A143:A146).
        if !cumulative && ((t == 0.0 && alpha <= 1.0) || (t == 1.0 && beta_param <= 1.0)) {
            return Ok(0.0);
        }
        let result = if cumulative {
            // A subnormal cdf is kept only at beta = 1, the closed form t^alpha, and is 0 at any
            // other beta, 1.0000001 included (`(x)=0` FALSE at B2, B3, B6, B7, B11, B12 of
            // probes/t2737-subnormal-cdf-shapes, TRUE at B1, B4, B5, B8:B10; T-2737).
            if l140_overflows(alpha, beta_param, t, 1.0 - t)
                || bfrac_shapes_absorbed(alpha, beta_param, t, 1.0 - t)
            {
                return Err(num_error(cell));
            }
            let cdf = beta_dist_cdf(t, 1.0 - t, alpha, beta_param);
            if beta_param == 1.0 {
                cdf
            } else {
                flush_subnormal(cdf)
            }
        } else {
            // On the Loader branch (both shapes above 1) the density is #NUM! where alpha absorbs
            // beta, as the cdf is on bfrac: `BETA.DIST(0.32..,6E+17,41,FALSE)`
            // and `BETA.DIST(0.1,1E+200,2,FALSE)` are #NUM! (probes/t4238-negbinom-bfrac-huge-s,
            // t3007-beta-dist-ln-x-overflow A20; T-4238), while `BETA.DIST(0.1,1E+290,0.5,FALSE)`,
            // off that branch, is 0 (t3007 A6).
            // It is #NUM! as well wherever Loader's count n - k is not positive, at every x
            // (`BETA.DIST(1-2^-51,1E+16,2,FALSE)`, probes/t4713-beta-inv-cut-pieces H11; T-4713).
            if alpha > 1.0
                && beta_param > 1.0
                && (first_shape_absorbs(alpha, beta_param)
                    || beta_density_counts_vanish(alpha, beta_param))
            {
                return Err(num_error(cell));
            }
            // general-interval beta pdf: f_X(x) = f_T(t) / (B - A), t=(x-A)/(B-A). A subnormal
            // density is 0, as in Excel (`(x)=0` TRUE: probes-numeric/t2513-r3-m1-residual-hold E863,
            // E875, E957; docs/excel-semantics/numbers.md "Distribution densities"). Both shapes
            // above 1: Excel's Loader saddle point (T-2507; decision 0181 accepts the cells it
            // leaves visibly off, which T-3220 owns).
            flush_subnormal(
                beta_density_small_shape(t, alpha, beta_param)
                    .or_else(|| {
                        (alpha > 1.0 && beta_param > 1.0)
                            .then(|| excel_beta_density(t, alpha, beta_param))
                            .flatten()
                    })
                    .or_else(|| beta_density(t, alpha, beta_param))
                    .unwrap_or_else(|| dist.pdf(t))
                    / width,
            )
        };
        if result.is_nan() || result.is_infinite() {
            return Err(num_error(cell));
        }
        Ok(result)
    }

    /// `BETA.INV(probability, alpha, beta, [A], [B])`, and BETAINV: every argument lifts over an
    /// array and a range (SPEC-0011; probes/stat-argument-modes H90-H99,
    /// probes/stat-quantile-lift). The quantile is the crossing of BETA.DIST's cdf
    /// (`beta_quantile`), which the lifting table charges (lift/kinds/statistical.rs
    /// `QUANTILE_WEIGHT`).
    pub(crate) fn fn_beta_inv(&mut self, args: &[Node], cell: CellReferenceIndex) -> CalcResult {
        self.lift_call(Function::BetaInv, args, cell, &mut |model, values| {
            number_or_error(model.beta_inv_of(values, cell))
        })
    }

    fn beta_inv_of(
        &self,
        values: &[CalcResult],
        cell: CellReferenceIndex,
    ) -> Result<f64, CalcResult> {
        let [p, alpha, beta, bounds @ ..] = values else {
            return Err(CalcResult::new_args_number_error(cell));
        };
        // A boolean reads as 1 or 0 in every argument (probes/stat-quantile-lift L3, gen-m9/165db58d1540f014 A257).
        let p = self.cast_number_ref(p, cell)?;
        let alpha = self.cast_number_ref(alpha, cell)?;
        let beta_param = self.cast_number_ref(beta, cell)?;
        let a = self.cast_number_or(bounds.first(), 0.0, cell)?;
        let b = self.cast_number_or(bounds.get(1), 1.0, cell)?;
        // alpha or beta <= 0, probability outside (0, 1) and A >= B are #NUM!.
        if alpha <= 0.0 || beta_param <= 0.0 || p <= 0.0 || p >= 1.0 || b <= a {
            return Err(num_error(cell));
        }
        // A quantile that rounds to 1 at p above 1/2 is `upper_quantile_near_one`'s: 1, or #NUM!
        // where the flipped tail's y underflows (`BETAINV(1-1E-16,2,0.01)`, T-3351). Otherwise a
        // quantile that underflows to 0 or rounds to 1 is #NUM! (`BETAINV(0.025,1E-15,2.5)`,
        // `BETAINV(0.05,0.002,1)`; statistics.md "Scalar distributions at extreme arguments
        // (T-2255)", probes/t2475-stat-subexpressions P!A127). Excel is #NUM! at some of those and
        // 1 - 2^-53 or 2^-53 at others with no rule found yet (statistics.md "BETA.INV where the
        // quantile rounds to 1 (T-3351)"), so they keep the #NUM!.
        let t = beta_quantile(alpha, beta_param, p);
        let t = if p > 0.5 && t >= 1.0 {
            upper_quantile_near_one(alpha, beta_param, p).ok_or_else(|| num_error(cell))?
        } else if t > 0.0 && t < 1.0 {
            t
        } else {
            return Err(num_error(cell));
        };
        // Map back from [0,1] to [A,B]
        Ok(a + t * (b - a))
    }

    /// `BETADIST(x, alpha, beta, [A], [B])`: BETA.DIST with `cumulative` TRUE; every argument
    /// lifts (probes/stat-argument-modes H100-H104, probes/stat-later-argument-lift H7, H9, K7-K10).
    pub(crate) fn fn_betadist_compat(
        &mut self,
        args: &[Node],
        cell: CellReferenceIndex,
    ) -> CalcResult {
        self.lift_call(
            Function::BetaDistCompat,
            args,
            cell,
            &mut |model, values| {
                let [x, alpha, beta, bounds @ ..] = values else {
                    return CalcResult::new_args_number_error(cell);
                };
                number_or_error(model.beta_dist_of(x, alpha, beta, None, bounds, cell))
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::beta_quantile;
    use crate::functions::statistical::bratio::beta_dist_lower_tail;

    // Excel's BETA.INV is #NUM! exactly where Loader's n - k vanishes at the flipped shapes
    // (probes/t4713-beta-inv-beta-above-1-cut A40, A41, E41; t4713-beta-inv-cut-pieces A1, K37;
    // T-4713): 9.44E+15 and 1.82E+16 keep a number, 1E+16 is #NUM!, and p above 1/2 takes
    // (beta, alpha), whose count is positive.
    #[test]
    fn quantile_is_nan_where_loader_count_vanishes() {
        assert!(beta_quantile(1e16, 2.0, 0.1).is_nan());
        assert!(beta_quantile(9.440_608_762_859_23e15, 2.0, 0.1) < 1.0);
        assert!(beta_quantile(1.819_700_858_609_99e16, 2.0, 0.1) < 1.0);
        assert!(!beta_quantile(1.5e15, 2.0, 0.9).is_nan());
    }

    #[test]
    fn quantile_is_the_crossing_of_beta_dist_own_cdf() {
        // ironcalc-calc STATISTICAL/BETADIST_BETAINV.xlsx (Excel-cached), each off Excel before
        // T-3398 and exact on BETA.DIST's own kernel: `BETA.INV(G,alpha,beta,A,B)` = A + t (B - A).
        // H15: BETA.INV(0.4999999999999999,2,2,0.2,1.2) = 0.7.
        assert_eq!(
            0.2 + beta_quantile(2.0, 2.0, 0.499_999_999_999_999_9) * 1.0,
            0.7
        );
        // H32: BETA.INV(0.5136696247356396,5,5,0.1,1) = 0.555, where the search crosses bup's
        // recurrence on L140 bup+bpser, not the power sum BETA.DIST takes there (T-3198).
        assert_eq!(
            0.1 + beta_quantile(5.0, 5.0, 0.513_669_624_735_639_6) * 0.9,
            0.555
        );
        // H43: BETA.INV(0.5882886668596937,2,2.5,0.15,1.2) = 0.6655623437846115.
        assert_eq!(
            0.15 + beta_quantile(2.0, 2.5, 0.588_288_666_859_693_7) * (1.2 - 0.15),
            0.665_562_343_784_611_5
        );
    }

    #[test]
    fn a_quantile_below_the_smallest_normal_double_is_zero() {
        // probes/t2435-dist-extremes (excel-graph): #NUM! where the quantile underflows, for p up
        // to 1/2. `BETAINV(1E-20,1E-300,30)` (A68, A110; gen-m9/3e98b26cf6713e77 A151) and
        // `BETAINV(0.5,1E-300,30)` (A69), where the search crossed at 3.333333333333333E-302,
        // and the tiny alphas of A70, A71, A74:A76.
        for (p, alpha, beta) in [
            (1e-20, 1e-300, 30.0),
            (0.5, 1e-300, 30.0),
            (0.5, 1e-10, 30.0),
            (0.5, 1e-5, 1.0),
            (1e-20, 0.01, 30.0),
            (1e-20, 0.02, 30.0),
            (1e-20, 0.05, 30.0),
        ] {
            assert_eq!(
                beta_quantile(alpha, beta, p),
                0.0,
                "BETAINV({p},{alpha},{beta})"
            );
        }
        // Just above it, Excel's quantile is a number: A112 `BETAINV(0.5,1E-3,1)` = 0.5^1000,
        // A108 `BETAINV(1E-20,0.1,30)` = 2.05508004608823E-202.
        assert_eq!(beta_quantile(1e-3, 1.0, 0.5), 9.332_636_185_032_827e-302);
        let a108 = beta_quantile(0.1, 30.0, 1e-20);
        assert!(
            (a108 / 2.055_080_046_088_23e-202 - 1.0).abs() < 1e-12,
            "{a108}"
        );
    }

    #[test]
    fn quantile_is_nan_once_the_square_of_as109_first_shape_overflows() {
        // probes/t3862-beta-inv-sqrt-dbl-max (excel-graph): the 15-digit literals around
        // sqrt(DBL_MAX) = 1.3407807929942596E+154. Row 3, a*a finite: a number; row 4, a*a = inf:
        // #NUM! (NaN here, which BETA.INV reports as #NUM!).
        let below: f64 = 1.340_780_792_994_25e154;
        let above: f64 = 1.340_780_792_994_26e154;
        assert!((below * below).is_finite() && (above * above).is_infinite());
        // A3/A4 `BETA.INV(1E-10,a,1E-100)`, B3/B4 `BETA.INV(0.4,a,0.5)`.
        for (p, beta) in [(1e-10, 1e-100), (0.4, 0.5)] {
            let t = beta_quantile(below, beta, p);
            assert!(t > 0.0 && t < 1.0, "BETA.INV({p},{below},{beta}) = {t}");
            assert!(beta_quantile(above, beta, p).is_nan());
        }
        // D3/D4 `BETA.INV(0.6,0.5,a)`: above p = 1/2, AS 109's first shape is beta.
        let t = beta_quantile(0.5, below, 0.6);
        assert!(t > 0.0 && t < 1.0, "{t}");
        assert!(beta_quantile(0.5, above, 0.6).is_nan());
        // E4 `BETA.INV(0.1,0.5,a)` = 5.85776711227727E-157: a huge second shape is not squared.
        // (Ours is the math there; Excel is 0.5% below it: TODO(T-4700).)
        let e4 = beta_quantile(0.5, above, 0.1);
        assert!(e4 > 0.0 && e4 < 1e-150, "{e4}");
    }

    #[test]
    fn quantile_at_basym_points_is_the_nearest_end() {
        // G29: BETA.DIST(0.555,1000,1000,TRUE,0.1,1) = 0.6903551301985376 on basym.
        let t = (0.555 - 0.1) / 0.9;
        assert_eq!(
            beta_dist_lower_tail(t, 1.0 - t, 1000.0, 1000.0),
            0.690_355_130_198_537_6
        );
        // H29: BETA.INV(0.6903551301985376,1000,1000,0.1,1) = 0.555, where BETA.DIST's cdf is
        // basym's and the root's upper tail is one ulp above fl(1 - p) (T-3443): the even end of
        // the final bracket would map to 0.5550000000000002.
        assert_eq!(
            0.1 + beta_quantile(1000.0, 1000.0, 0.690_355_130_198_537_6) * 0.9,
            0.555
        );
    }
}
