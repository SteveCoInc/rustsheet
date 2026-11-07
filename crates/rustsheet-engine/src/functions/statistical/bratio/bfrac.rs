//! TOMS 708 `bfrac`, the continued fraction for `I_x(a, b)` with a, b > 1 and b0 >= 40 (SPEC-0025's
//! `bfrac.rs`), with Excel's beta density times x·y as its prefactor. On the region T-3571 probed
//! ([`lentz_region`]) the fraction is evaluated as Excel does, by Numerical Recipes' modified Lentz
//! loop on closed-form terms ([`lentz_fraction`]); elsewhere by TOMS's own recurrence.

use super::super::loader::excel_beta_density;
use super::Dispatch;

/// The most terms [`bfrac`] takes before giving the point back to today's kernel (TOMS's own
/// bound, directive 5).
const BFRAC_MAX_TERMS: f64 = 1_000.0;

/// The Lentz loop's stop: |C·D - 1| <= 10·`DBL_EPSILON` (T-3571's scan of stop thresholds brackets
/// it in (2.1e-15, 2.5e-15) on the tune probe; `docs/research/t2464/r5-bfrac/model.py` `EPS`).
/// FITTED(T-3571, 1536 cells, a, b 40..5000, unswapped call (x below the mean), z -8..-0.2 sd)
const LENTZ_EPS: f64 = 10.0 * f64::EPSILON;

/// Numerical Recipes' FPMIN: the Lentz loop's start f = C = TINY (`model.py` `TINY`; 1e-300 scores
/// lower, T-3571 `score_loop_tune.out`), and the floor NR puts under |D| and |C| so the loop never
/// divides by zero (directive 5; `model.py` omits the guards and no probed point reaches them).
const TINY: f64 = 1e-30;

/// The shapes T-3571's probes cover (statistics.md "Incomplete beta bfrac is TOMS's fraction in a
/// Lentz loop"): a and b in [40, 5000].
const LENTZ_SHAPES: std::ops::RangeInclusive<f64> = 40.0..=5000.0;

/// The x the probes cover, in standard deviations from the beta mean: z in [-8, -0.2].
const LENTZ_Z: std::ops::RangeInclusive<f64> = -8.0..=-0.2;

/// Whether bfrac's a0 is past the shapes T-3571's probes cover ([`LENTZ_SHAPES`]), where no probe shows
/// Excel's bfrac arithmetic: the huge-a0 calls that take the large-shape expansion instead (T-4435;
/// `bratio::ported_tails_with`, `fisher.rs` `f_tail`).
pub(crate) fn past_evidenced_shapes(a0: f64) -> bool {
    a0 > *LENTZ_SHAPES.end()
}

/// Whether a bfrac point lies where T-3571's model is evidenced: the unswapped call (bratio's λ >= 0,
/// so a0 = a and x0 = x) with a and b in [40, 5000] and x from 8 to 0.2 standard deviations below the
/// beta mean (`mkprobe.py`'s mu + z·sd). Everywhere else in bfrac's region (a0 < 40 with b0 >= 40,
/// a shape above 5000, the swapped call, within 0.2 sd of the mean or below -8 sd) no probe shows
/// Excel's arithmetic, so the point keeps TOMS's recurrence, the accurate path.
pub(super) fn lentz_region(d: &Dispatch) -> bool {
    let (a, b, x) = (d.a0, d.b0, d.x0);
    if d.swapped || !LENTZ_SHAPES.contains(&a) || !LENTZ_SHAPES.contains(&b) {
        return false;
    }
    let mu = a / (a + b);
    let sd = (mu * (1.0 - mu) / (a + b + 1.0)).sqrt();
    LENTZ_Z.contains(&((x - mu) / sd))
}

/// bfrac's fraction r = 1/(b0 + K(`α_n`/`β_n`)) as Excel evaluates it (T-3571, `model.py` `fraction`):
/// Numerical Recipes' modified Lentz loop from f = C = TINY, D = 0, over the partial (1, b0) and
/// then TOMS's terms in closed form, each rounding in the model's order:
/// - λ = fl(a - fl((a+b)·x)) also for a > b (TOMS takes (a+b)·y - b there; T-3571 `lamab.out`);
/// - b0 = c·(a/(a+1)) with c = λ + 1;
/// - `α_n` = v/s/s·u·n·(b-n)·x·x left to right, u = a+n-1, v = a+b+n-1, s = a+2n-1;
/// - `β_n` = n + fl(n·x·(b-n))/s + ((a+n)/(a+2n+1))·(c + n·(y+1)).
///
/// `None` if the loop has not settled within [`BFRAC_MAX_TERMS`] terms (directive 5).
fn lentz_fraction(a: f64, b: f64, x: f64, y: f64) -> Option<f64> {
    let floor = |v: f64| if v.abs() < TINY { TINY } else { v };
    let lambda = a - (a + b) * x;
    let c = lambda + 1.0;
    let b0 = c * (a / (a + 1.0));
    let mut f = TINY;
    let mut cc = floor(b0 + 1.0 / f);
    let mut dd = 1.0 / floor(b0);
    f *= cc * dd;
    let yp1 = y + 1.0;
    let mut n = 0.0;
    loop {
        if n >= BFRAC_MAX_TERMS {
            return None;
        }
        n += 1.0;
        let u = a + n - 1.0;
        let v = a + b + n - 1.0;
        let s = a + 2.0 * n - 1.0;
        let alpha = v / s / s * u * n * (b - n) * x * x;
        let beta = n + n * x * (b - n) / s + (a + n) / (a + 2.0 * n + 1.0) * (c + n * yp1);
        dd = 1.0 / floor(beta + alpha * dd);
        cc = floor(beta + alpha / cc);
        let delta = cc * dd;
        f *= delta;
        if (delta - 1.0).abs() <= LENTZ_EPS {
            return Some(f);
        }
    }
}

/// TOMS 708 `bfrac(a, b, x, y, λ, eps)` as Excel computes it (`bratio_excel.py` `bfrac`, BEST:
/// `brcomp = "pdf"`, `bgrat = "orig"`; statistics.md "Incomplete beta L140 and bfrac: the prefactor
/// is Excel's own density times x·y"):
/// - brcomp = fl(P·fl(x·y)) with P Excel's BETA.DIST density at x (T-2507's Loader form);
/// - the fraction is the 1992 Fortran's, w = n·(b - n)·x and β = n + w/s + e·(c + n·(y + 1)), in
///   doubles (its own arithmetic is not found, T-3571).
///
/// On [`lentz_region`] the fraction is [`lentz_fraction`] instead, cdf = fl(brcomp·f) (T-3812).
///
/// λ is bratio's |λ| after its swap. `None` where the density is not formed or the fraction has
/// not settled within [`BFRAC_MAX_TERMS`] terms.
pub(super) fn bfrac(d: &Dispatch, eps: f64) -> Option<f64> {
    let (a, b, x, y, lambda) = (d.a0, d.b0, d.x0, d.y0, d.lambda);
    let brc = excel_beta_density(x, a, b)? * (x * y);
    if brc == 0.0 {
        return Some(0.0);
    }
    if lentz_region(d) {
        let w = brc * lentz_fraction(a, b, x, y)?;
        return w.is_finite().then_some(w);
    }
    let c = lambda + 1.0;
    let c0 = b / a;
    let c1 = 1.0 / a + 1.0;
    let yp1 = y + 1.0;
    let (mut n, mut p, mut s) = (0.0, 1.0, a + 1.0);
    let (mut an, mut bn, mut anp1) = (0.0, 1.0, 1.0);
    let mut bnp1 = c / c1;
    let mut r = c1 / c;
    loop {
        if n >= BFRAC_MAX_TERMS {
            return None;
        }
        n += 1.0;
        let t = n / a;
        let w = n * (b - n) * x;
        let e = a / s;
        let alpha = p * (p + c0) * e * e * (w * x);
        let e = (t + 1.0) / (c1 + t + t);
        let beta = n + w / s + e * (c + n * yp1);
        p = t + 1.0;
        s += 2.0;
        let next_a = alpha * an + beta * anp1;
        an = anp1;
        anp1 = next_a;
        let next_b = alpha * bn + beta * bnp1;
        bn = bnp1;
        bnp1 = next_b;
        let r0 = r;
        r = anp1 / bnp1;
        if (r - r0).abs() <= eps * r {
            break;
        }
        an /= bnp1;
        bn /= bnp1;
        anp1 = r;
        bnp1 = 1.0;
    }
    let w = brc * r;
    w.is_finite().then_some(w)
}
