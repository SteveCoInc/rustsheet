//! Excel's inverse incomplete beta search: AS 109 (Majumder and Bhattacharjee 1973, with Cran,
//! Martin and Thomas's Remark AS R19 and Berry's AS R83) as Excel runs it, one bounded loop
//! shared by its callers through [`As109Profile`] (T-4975; docs/excel-semantics/statistics.md
//! "BETA.INV returns bounded AS 109 Newton stopping points (T-4975)").
//!
//! Excel's quantiles are AS 109's Newton *stopping points*, not the refined crossing of the cdf:
//! - ironcalc-calc `BETADIST_BETAINV.xlsx` H5, `BETA.INV(0.6734553797493481,3,600,2000,5489582)`,
//!   is 33587.000000000226 where the crossing is 33586.99999999992, and H26 is
//!   0.2339999999999406 where the root is 0.23400000000293818;
//! - gen-m9/4e55500bb8ad3cd8 A360 `BETAINV(1E-200,3.14159,TIME(4,2,1))` is the first estimate,
//!   5.115610690476489E-64: on the cdf's log-quantized plateau no later iterate has a smaller
//!   residual.
//!
//! The profile that fits ([`As109Profile::BetaInv`]): an accurate normal start where AS 109 has
//! Hastings' approximation; a first estimate replaced only at the ends of [0, 1] (by 1E-6 or
//! 1 - 1E-6, not clamped to [1E-4, 0.9999]); the residual divided by Excel's beta density; the
//! absolute accuracy `10^(-5/min(a,b)^2 - p^(-0.2) - 13)`, not truncated to a whole exponent and
//! with the smaller shape (probes/t3862-beta-inv-sqrt-dbl-max H1, H3), and a 1E-300 floor on the
//! squared-step comparison; on an early stop the iterate before the step (AS 109 as published,
//! without the later "value = tx" change); on a plateau the first iterate with the least residual.
//!
//! The absolute accuracy is floored at [`STEP_FLOOR`], the same 1E-300 as the squared-step floor
//! (AS 109's one `SAE` exponent behind both its FPU and its ACU): where the accuracy underflows,
//! the search stops once the squared step is below 1E-300, not only at a zero step
//! ([`FloorStopped`](As109Outcome::FloorStopped)).
//!
//! The loop is bounded (directive 5): at most [`super::MAX_NEWTON_STEPS`] Newton steps, and a
//! search whose cumulative one-third step contractions reach [`MAX_CONTRACTIONS`] (where the
//! cumulative scale 3^-n falls below the smallest normal double) is [`As109Outcome::Exhausted`]:
//! Excel's numeric endpoint results take 11 to 597 contractions and its `#NUM!` ones 672 to 1,667
//! (accuracy/BETA.INV/s1 A89, A94, A184), a mechanism rather than an input cutoff
//! (UNVERIFIED, probe group G21 of docs/research/probe-suggestions-2026-11-07). An exhausted
//! search is never reported as a root: the caller falls back to its endpoint-aware crossing.

use super::super::normal::standard_normal_quantile_value;
use super::super::special::ln_gamma;
use super::MAX_NEWTON_STEPS;

/// The floor of the squared step AS 109 compares new steps with (its FPU), 1E-300 in Excel's
/// search rather than AS 109's 1E-30: at the AS 109 floor `BETA.INV(1E-200,3.14159,0.168...)`
/// stops before the plateau's first estimate is kept (gen-m9/4e55500bb8ad3cd8 A360).
///
/// It floors the squared absolute accuracy too, as AS 109's `IEX = MAX(..., SAE)` floors its ACU
/// with the same exponent (T-4975 follow-up). Where the accuracy itself underflows (a tiny p or
/// shape), Excel's search stops at the first squared Newton step at or below 1E-300 and returns
/// the iterate before it, far above the root:
/// - `BETA.INV(1E-200,0.5,0.5)` = `BETA.INV(1E-300,0.5,0.5)` = 2.7055792171147168E-151, whatever
///   p (probes/t4974-beta-f-inv-endpoint-saturation A4, A5; F.INV(1E-200,1,1) is its
///   back-transform, L4): the 1E-6 end seed divided by 3 at each of 303 steps (the slope's step
///   2x is cut to a third to stay in [0, 1]) until the step 2x/3 squared is below 1E-300;
///   2.7053113740149512E-151 at beta 50 (I4, I5). Ours is within 2E-13 there (the 303 steps
///   carry the rounding of each slope).
/// - `BETA.INV(1E-300,1.5,300)` and the rest of column J of probes/t4974-beta-inv-dbl-min-residual,
///   `BETA.INV(1E-300,2,5E9)` (endpoint-saturation K5) and `FINV(1E-300,30,3)` (probes-numeric
///   t4857-f-classes A2242 and its neighbours), bit for bit.
const STEP_FLOOR: f64 = 1e-300;

/// The first estimate that replaces one at an end of [0, 1]: 1E-6 from 0 and 1 - 1E-6 from 1.
/// `BETA.INV(0.6,0.5,1.3E+154)` is 1 - fl(1 - 1E-6) = 1.0000000000287557E-06 and
/// `BETA.INV(1E-10,1.3E+154,1E-100)` is 0.999999, the replaced estimate itself
/// (probes/t3862-beta-inv-sqrt-dbl-max A1, D1).
const ENDPOINT_SEED: f64 = 1e-6;

/// The cumulative one-third contractions at which the search counts as exhausted: 3^-645 is the
/// first power of 1/3 below `f64::MIN_POSITIVE` (3^-644 = 2.6E-308). Excel's numeric endpoint
/// results need 11 to 597 and its `#NUM!` ones 672 to 1,667 (accuracy/BETA.INV/s1; UNVERIFIED
/// as a rule, probe group G21).
const MAX_CONTRACTIONS: u32 = 645;

/// Which caller's search [`as109_search`] runs. Each profile fixes the start, the accuracy test
/// and the stop; the cdf and density are the caller's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum As109Profile {
    /// BETA.INV and BETAINV, and the beta iterate behind F.INV (T-4975 sections A and C): AS 109's
    /// step contractions, and its accuracy test on the step or the previous step stops the search
    /// (the iterate before the step).
    BetaInv,
    /// The Student-t inverse's power-law tail (T-3363, `inverse_beta::beta_small_quantile`): plain
    /// Newton steps, stopping where ln p no longer moves with the relative residual, where that
    /// residual changes sign, where the squared step underflows, where the residual is below the
    /// smallest normal double or where an iterate leaves (0, 1); always settled, never exhausted.
    SmallBetaTail,
    /// T.INV.2T, TINV and CONFIDENCE.T away from the centre (T-4975 section B): AS 109's steps,
    /// stopping only when both the absolute (AS R19) and the relative accuracy tests hold.
    StudentT,
}

/// How [`as109_search`] ended.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum As109Outcome {
    /// The accuracy test held: the iterate before the step.
    Stopped(f64),
    /// The accuracy test held only through [`STEP_FLOOR`] (the profile's own accuracy squared is
    /// below it): the iterate before the step, which can be far above the root in a tiny tail
    /// (see [`STEP_FLOOR`]). Callers that return the math where Excel is grossly wrong
    /// (decision 0188 R3) keep this stop: it is Excel's value, bit for bit where measured.
    FloorStopped(f64),
    /// BETA.INV's search (not Student's t's, which keeps [`Exhausted`](Self::Exhausted)) could
    /// not take its first step: the density at the first estimate is below the normal doubles (its
    /// reciprocal, AS 109's slope factor, overflows) or the step is not finite. Excel returns the
    /// first estimate there, bit for bit wherever measured: `BETA.INV(2^-1022,30,3)` =
    /// 1.5861167524067332E-17 and `BETA.INV(2^-1022,2,2)` = 5.312217141104403E-168
    /// (probes/t4974-beta-inv-dbl-min-residual H7, C7), `BETA.INV(1E-200,1000,1)` = 0.2064 (the
    /// estimate 1 - 2/(t + 1); probes-numeric/t1543 A86),
    /// `BETA.INV(6.07658893047835E-282,14.043817035397,2.81602340373877)` =
    /// 2.657957500039701E-25 (accuracy/BETA.INV/s1 A169), and the R3-marked
    /// `F.INV(2.22507385850721E-308,39,3)` = 2.0587119279917463E-36 (accuracy/F.INV/s1 A126),
    /// as are Student's t's R3-marked tiny-p cells (t3349, t1531) with this outcome. In the tiny
    /// tails the callers' R3 check still returns the math wherever this stop is more than 1E-4
    /// off it: most such points are R3-marked (decision 0188), and the same rule cannot tell the
    /// unmarked ones (column H and C of dbl-min-residual) from the marked ones (A86).
    SlopeStopped(f64),
    /// The step no longer moved the iterate, or the steps ran out on a bracketed plateau: the
    /// first iterate with the least residual.
    Settled(f64),
    /// The first estimate was replaced at an end of [0, 1] and its step is not finite.
    EndSeed(f64),
    /// The search ran out without a bracket, or its step was not finite away from an end seed,
    /// or AS 109's normal approximation overflowed (see [`first_estimate`]): not a root
    /// (contract 2 of the addendum).
    Exhausted,
    /// The steps ran out with the last residual still larger than the target itself: the search
    /// was still walking towards a root it never reached. Excel's quantile is `#NUM!` there:
    /// `F.INV(1E-300,11,3)` and `F.INV(1E-250,11,3)` (probes/t4974-f-inv-tiny-tail-neighbourhood
    /// E7, E11), whose first estimate lands below the root where the cdf underflows, so the
    /// first step jumps to 0.32 and the iterate then shrinks by (1 - 1/5.5) a step, short of the
    /// root 3E-56 after [`super::MAX_NEWTON_STEPS`] steps, while df2 2 and 4 around it converge
    /// (E6, E8, E10, E12).
    Unconverged,
}

impl As109Outcome {
    /// The iterate, `None` for an exhausted search; an end seed only when `keep_end_seed`.
    pub(crate) fn value(self, keep_end_seed: bool) -> Option<f64> {
        match self {
            Self::Stopped(x) | Self::FloorStopped(x) | Self::SlopeStopped(x) | Self::Settled(x) => {
                Some(x)
            }
            Self::EndSeed(x) => keep_end_seed.then_some(x),
            Self::Exhausted | Self::Unconverged => None,
        }
    }
}

/// One oriented AS 109 problem: solve `cdf(x) = target` for `I_x(pp, qq)`, `target` in (0, 1/2]
/// after the caller's flip, with `ln_beta` = ln B(pp, qq) for the first estimate.
pub(crate) struct As109Problem<C, D> {
    pub profile: As109Profile,
    pub pp: f64,
    pub qq: f64,
    pub target: f64,
    pub ln_beta: f64,
    /// Excel's `I_x(pp, qq)` at the oriented (x, y = 1 - x).
    pub cdf: C,
    /// Excel's beta density for (pp, qq) at the same (x, y), the Newton slope.
    pub density: D,
}

/// AS 109's first estimate of the root of `I_x(pp, qq) = target` from the normal deviate of the
/// upper tail `target`, with an estimate at an end of [0, 1] replaced by [`ENDPOINT_SEED`].
///
/// `None` where the normal approximation's `exp(2w)` overflows (both shapes above 1 and a target
/// near the smallest normal double): Excel's search does not start from the 1E-6 end seed there.
/// `BETA.INV(2.22507385850721E-308,1.12049449852741,2.10175380918621)` is 9.192367895421942E-151
/// in Excel (accuracy/BETA.INV/s1 A131, a stop at the 1E-300 floor); from the 1E-6 seed the same
/// stop is 5.04E-151, from a seed of 0.5 it is 9.192367895408647E-151 (1.4E-12 off), so Excel
/// seems to start at 0.5, but that iterate is not Excel's to 15 digits, and the caller's converged
/// crossing (the math, as before) is kept.
fn first_estimate(pp: f64, qq: f64, target: f64, ln_beta: f64) -> Option<f64> {
    let y = -standard_normal_quantile_value(target);
    let estimate = if pp > 1.0 && qq > 1.0 {
        let r = (y * y - 3.0) / 6.0;
        let s = 1.0 / (pp + pp - 1.0);
        let t = 1.0 / (qq + qq - 1.0);
        let h = 2.0 / (s + t);
        let w = y * (h + r).sqrt() / h - (t - s) * (r + 5.0 / 6.0 - 2.0 / (3.0 * h));
        let growth = (w + w).exp();
        if growth.is_infinite() {
            return None;
        }
        pp / (pp + qq * growth)
    } else {
        let r = qq + qq;
        let mut t = 1.0 / (9.0 * qq);
        t = r * (1.0 - t + y * t.sqrt()).powi(3);
        if t <= 0.0 {
            1.0 - ((((1.0 - target) * qq).ln() + ln_beta) / qq).exp()
        } else {
            t = (4.0 * pp + r - 2.0) / t;
            if t <= 1.0 {
                (((target * pp).ln() + ln_beta) / pp).exp()
            } else {
                1.0 - 2.0 / (t + 1.0)
            }
        }
    };
    Some(if estimate <= 0.0 {
        ENDPOINT_SEED
    } else if estimate >= 1.0 {
        1.0 - ENDPOINT_SEED
    } else {
        estimate
    })
}

/// The absolute accuracy of the profile's stop: `10^(-5/m^2 - target^(-0.2) - 13)` with m the
/// smaller shape and the exponent not truncated (0 once it underflows, where only a zero step
/// stops the search).
fn accuracy(profile: As109Profile, pp: f64, qq: f64, target: f64) -> f64 {
    match profile {
        As109Profile::SmallBetaTail => 0.0,
        // AS R19's accuracy on the first shape, df/2: with the smaller shape (1/2) the exponent
        // is below -33 and the search never stops early, which takes 6 of T.INV.2T's 6 controls
        // at p = 1/2 off Excel (addendum section B: df 5, 7, 8, 9, 28, 123456789).
        As109Profile::StudentT => 10f64.powf(-5.0 / pp / pp - 1.0 / target.powf(0.2) - 13.0),
        As109Profile::BetaInv => {
            let m = pp.min(qq);
            10f64.powf(-5.0 / m / m - 1.0 / target.powf(0.2) - 13.0)
        }
    }
}

/// Excel's AS 109 search on `problem` (see the module documentation): AS 109's first estimate,
/// then [`newton_search`] under the problem's profile, at most [`MAX_NEWTON_STEPS`] cdf and
/// density evaluations.
pub(crate) fn as109_search<C, D>(problem: &As109Problem<C, D>) -> As109Outcome
where
    C: Fn(f64, f64) -> f64,
    D: Fn(f64, f64) -> f64,
{
    let As109Problem {
        profile,
        pp,
        qq,
        target,
        ln_beta,
        ..
    } = *problem;
    if !(target > 0.0 && target <= 0.5) {
        return As109Outcome::Exhausted;
    }
    let Some(seed) = first_estimate(pp, qq, target, ln_beta) else {
        return As109Outcome::Exhausted;
    };
    if !(seed > 0.0 && seed < 1.0) {
        return As109Outcome::Exhausted;
    }
    let accuracy = accuracy(profile, pp, qq, target);
    newton_search(
        profile,
        NewtonStart {
            seed,
            target,
            accuracy,
            max_steps: MAX_NEWTON_STEPS,
        },
        &problem.cdf,
        &problem.density,
    )
}

/// Where [`newton_search`] starts and what it solves for.
#[derive(Clone, Copy, Debug)]
pub(crate) struct NewtonStart {
    /// The first iterate.
    pub seed: f64,
    /// The probability solved for.
    pub target: f64,
    /// The profile's absolute accuracy (AS 109's acu; unused by [`As109Profile::SmallBetaTail`]).
    pub accuracy: f64,
    /// The most cdf evaluations.
    pub max_steps: u32,
}

/// The one bounded Newton loop behind Excel's inverse beta searches: from `start.seed`, solve
/// `cdf(x, 1 - x) = start.target` with the slope `density(x, 1 - x)`, stopping by `profile`'s
/// rules (see [`As109Profile`]), keeping the first iterate of least absolute residual.
pub(crate) fn newton_search<C, D>(
    profile: As109Profile,
    start: NewtonStart,
    cdf: &C,
    density: &D,
) -> As109Outcome
where
    C: Fn(f64, f64) -> f64,
    D: Fn(f64, f64) -> f64,
{
    let NewtonStart {
        seed,
        target,
        accuracy,
        max_steps,
    } = start;
    let end_seed = seed == ENDPOINT_SEED || seed == 1.0 - ENDPOINT_SEED;
    // The accuracy is floored with the squared step (see `STEP_FLOOR`); the power-law tail's
    // search has no accuracy test.
    let profile_sq = accuracy * accuracy;
    let floored = profile != As109Profile::SmallBetaTail && profile_sq < STEP_FLOOR;
    let acu_sq = if floored { STEP_FLOOR } else { profile_sq };
    let ln_target = target.ln();
    let mut value = seed;
    let mut yprev = 0.0;
    let mut sq: f64 = 1.0;
    let mut prev = 1.0;
    let mut contractions = 0u32;
    let mut best = (f64::INFINITY, value);
    let (mut below, mut above) = (false, false);
    let mut previous_relative: Option<f64> = None;
    let mut last_residual = 0.0;
    for step in 0..max_steps {
        let (x, y_complement) = (value, 1.0 - value);
        let residual = cdf(x, y_complement) - target;
        last_residual = residual;
        if profile == As109Profile::SmallBetaTail {
            if !residual.is_finite() {
                break;
            }
        } else if residual.is_nan() {
            return As109Outcome::Exhausted;
        }
        if residual.abs() < best.0 {
            best = (residual.abs(), value);
        }
        if profile == As109Profile::SmallBetaTail {
            // T-3363's stops: ln p no longer moves with the relative residual, the residual
            // changes sign, or it is below the smallest normal double.
            let relative = residual / target;
            let settled = previous_relative
                .is_some_and(|r| ln_target + relative == ln_target || r * relative < 0.0);
            if settled || residual.abs() < f64::MIN_POSITIVE {
                break;
            }
            previous_relative = Some(relative);
            let y = residual / density(x, y_complement);
            if !y.is_finite() || y * y == 0.0 {
                break;
            }
            value -= y;
            if !(value > 0.0 && value < 1.0) {
                break;
            }
            continue;
        }
        below |= residual < 0.0;
        above |= residual > 0.0;
        if residual == 0.0 {
            return As109Outcome::Stopped(value);
        }
        let slope = density(x, y_complement);
        let y = residual / slope;
        // AS 109 divides by the density as EXP of its logarithm's negation, which overflows below
        // the normal doubles: there, as where the step is not finite, there is no step to take.
        if !y.is_finite() || !slope.recip().is_finite() {
            // An end seed is the caller's to keep or not (Excel keeps it at t3862 A1, 0.999999,
            // and not at accuracy/BETA.INV/s1 A182, 0.5); any other iterate is Excel's stop.
            return if step == 0 && end_seed {
                As109Outcome::EndSeed(value)
            } else if profile == As109Profile::BetaInv && step == 0 {
                As109Outcome::SlopeStopped(value)
            } else {
                As109Outcome::Exhausted
            };
        }
        if y * yprev <= 0.0 {
            prev = sq.max(STEP_FLOOR);
        }
        let mut g = 1.0;
        let tx = loop {
            let adj = g * y;
            sq = adj * adj;
            if sq < prev {
                let tx = value - adj;
                if (0.0..=1.0).contains(&tx) {
                    let absolute = prev <= acu_sq || y * y <= acu_sq;
                    // Student's t stops only when the relative test holds too: the step is within
                    // an epsilon of the iterate (addendum section B).
                    let stop = match profile {
                        As109Profile::StudentT => absolute && y.abs() <= f64::EPSILON * value,
                        _ => absolute,
                    };
                    if stop {
                        return if floored {
                            As109Outcome::FloorStopped(value)
                        } else {
                            As109Outcome::Stopped(value)
                        };
                    }
                    if tx != 0.0 && tx != 1.0 {
                        break tx;
                    }
                }
            }
            g /= 3.0;
            contractions += 1;
            if contractions >= MAX_CONTRACTIONS {
                return if below && above {
                    As109Outcome::Settled(best.1)
                } else {
                    As109Outcome::Exhausted
                };
            }
        };
        if tx == value {
            return As109Outcome::Settled(best.1);
        }
        value = tx;
        yprev = y;
    }
    if profile == As109Profile::SmallBetaTail {
        As109Outcome::Settled(best.1)
    } else if last_residual.abs() > target {
        As109Outcome::Unconverged
    } else if below && above {
        As109Outcome::Settled(best.1)
    } else {
        As109Outcome::Exhausted
    }
}

/// ln B(pp, qq) for AS 109's first estimate, the one form BETA.INV's and F.INV's callers hand
/// [`As109Problem::ln_beta`]: correctly rounded GAMMALNs summed in argument order. The ordered
/// form with stored GAMMALN differences (`bratio::bpser`'s) moves 4 more of the 2,384 literal
/// BETA.INV cells visibly off and fixes 1 of the 2,076 F.INV ones (T-4975), so it is not taken.
pub(crate) fn inverse_ln_beta(pp: f64, qq: f64) -> f64 {
    ln_gamma(pp) + ln_gamma(qq) - ln_gamma(pp + qq)
}
