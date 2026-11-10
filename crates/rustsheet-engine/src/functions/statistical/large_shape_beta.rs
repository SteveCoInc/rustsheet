//! The regularized incomplete beta `I_x(a, b)` at a huge first shape and a small second one
//! (T-2528): `DiDonato` and Morris' asymptotic expansion in the incomplete gamma, the series of
//! TOMS 708 `bgrat` (their section 9), in accurate double arithmetic rather than Excel's. An
//! accuracy stand-in like `incomplete_beta.rs` `half_integer_tail`: the bratio port's bgrat
//! branches (SPEC-0025, T-2510) replace it, or absorb it, when the old kernel goes.
//!
//! Cephes' continued fraction (`incomplete_beta.rs`) needs about 42,000 steps for
//! `F.DIST(2,5,1E10,TRUE)` = 1 - `I_y(5E9, 2.5)` at y = 1 - 1E-9 and is about 1E-7 relative off at
//! every cap from 300 to 300,000, where Excel's 0.9247647537543169 is the true value
//! (probes-numeric/t1530-f-df-cut A8). With nu = a + (b - 1)/2 and z = -nu ln x,
//!
//! `I_x(a, b) = Gamma(a+b) / (Gamma(a) nu^b) * (Q(b, z) + r * sum_n d_n J_n)`,
//!
//! r = z^b e^-z / Gamma(b), the `d_n` and `J_n` bgrat's recurrences from `J_0` = Q(b, z) / r. Each
//! term is about ((b + 2n)/(2 nu))^2 of the one before. At a of [`MIN_SHAPE`] or more and b of at
//! most [`MAX_SECOND_SHAPE`], b/nu is at most 0.1, so the series converges in a few terms.
//!
//! The expansion used to stop at b/a of 1E-6. That cut sent F tails at df2 of 2E6 to 5E7 and df1 of
//! 40 to 79 to Cephes' fraction, 1E-11 to 7E-10 off the math. Those are the calls T-3754's L140
//! bup+bgrat port does not take (more than 9 bup terms, or a0 above 1E7).
//! T-4119's grid (docs/research/t4119: 640 F tails at df1 3 to 79, df2 2.1E6 to 5E7, against 60-digit
//! mpmath) puts the expansion's worst error over the whole region at 2.1E-14, so the cut is gone
//! (decision 0267, accuracy).
//!
//! T-4435 widened the bounds from a >= 1E6 and b < 40 to a >= 2000 and b <= 200 ([`MIN_SHAPE`]). The
//! calls past the old bounds took Cephes' fraction (b0 < 40 past the L140 port's 9 bup terms) or bfrac
//! (b0 >= 40), 1E-13 to 5E-8 off the math. T-4434's sweep (probes-numeric/
//! t4434-huge-shape-outside-large-shape-beta) found Excel within a few units of |ln tail| ulp there at
//! b0 < 40. In that new range the expansion runs in [`Arithmetic::Accurate`].
//!
//! On T-4435's grid (docs/research/t4435: 600 F tails, 200 BETA.DIST, 60-digit mpmath):
//! - the worst error goes from 5.2E-8 to 1.7E-13, and no point gets worse;
//! - every point past 1E-14 is on a kernel this change does not touch: L140's port in Excel's
//!   arithmetic (up to 3.7 units of |ln tail| ulp), or a lower tail with a moderate a0 (up to 4.3E-14);
//! - the points the expansion takes are within 1.8E-15.
// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::cast_precision_loss, clippy::many_single_char_names)]
#![cfg_attr(test, expect(clippy::float_cmp, clippy::unreadable_literal))]
// LINT-DEBT-END

use super::bratio::half_complement;
use super::incomplete_beta::{rlog1, stirling_correction};
use super::incomplete_gamma::{accurate_gamma_density, gamma_ur};
use super::loader::stirlerr_exact;
use crate::x87_exp::{dd_add, dd_div, dd_exp, dd_mul, dd_neg, two_sum};
use crate::x87_log10::{ln_double_double, ln_double_double_pair};

/// The smallest first shape the expansion is used at, from the math alone (T-4435's boundary table,
/// docs/research/t4435/boundary.py, reproducible through the ignored test `boundary_table`). The table
/// has `BETA.DIST` at a non-dyadic a of 523.71 to 987654.3 and b of 1.5 to 200, from the mean down to
/// deep lower tails (z = nu |ln x| up to 600), against 60-digit mpmath. No Excel value is read.
/// - With the accurate arithmetic ([`Arithmetic::Accurate`]), the forced expansion's worst relative
///   error per (a, b) is 2.2E-14 from a = 1517.93, and 4.9E-15 from a = 2049.9.
/// - Main's kernels (Cephes' fraction past L140's 9 bup terms, and bfrac's density prefactor) are up to
///   3.3E-13 off at a = 2049.9, 1.2E-12 at 12345.67 and 7.5E-11 at 987654.3.
/// - The expansion is at least as accurate as those kernels at every b of the table from a = 1517.93.
///   At 523.71 and 1033.37 it loses at some b (39.5, 150.7 and 200).
///
/// 2000 sits between 1517.93 and 2049.9, where the expansion's worst falls to 4.9E-15.
///
/// A tiny first shape is out: there the terms in 1/nu are not small (`BETA.DIST(0.997,3.366,
/// 3.43E-18,TRUE)` is 1.43E-17 in Excel, probes-numeric/t2464-r2-bpser-small-tiny A10, and 4E-13 by
/// it).
const MIN_SHAPE: f64 = 2000.0;

/// The expansion's lower bound on a before T-4435 (T-2528, T-4119). The F tails from it up keep the
/// double ratio and the double density: T-4017's sweep has Excel's bits there on that arithmetic.
pub(super) const T4119_MIN_SHAPE: f64 = 1e6;

/// The expansion's upper bound on b before T-4435: from it up the expansion is new, so it takes the
/// double-double r at every a.
const T4119_MAX_SECOND_SHAPE: f64 = 40.0;

/// The largest second shape the expansion is used at: T-4435's region 2 (b0 = df1/2 of 40 to 200,
/// df1 up to 400), the range T-4434's sweep and T-4435's grids cover. Past it TOMS 708's other
/// branches keep the point, `#NUM!` included: `BETA.DIST(0.1,1E+200,1E+100,TRUE)` is `#NUM!`
/// (probes/t3007-beta-dist-ln-x-overflow S!A23), where the expansion's tail is 0.
const MAX_SECOND_SHAPE: f64 = 200.0;

/// bgrat's cap on the series' terms.
const MAX_SERIES_TERMS: usize = 30;

/// The deepest cut of [`legendre_fraction`]: it needs about 10 cbrt(b) partial quotients at its
/// slowest, z = b + 1 (`incomplete_gamma`'s budget), and b is at most [`MAX_SECOND_SHAPE`] here.
const MAX_FRACTION_DEPTH: u32 = 1 << 14;

/// The relative size of the last series term at which the series stops.
const SERIES_TOLERANCE: f64 = 1e-17;

/// `I_x(a, b)` with y = 1 - x given, for a of [`MIN_SHAPE`] or more, b up to [`MAX_SECOND_SHAPE`]
/// and 0 < x, y < 1; `None` elsewhere or where a step is not finite, and the caller keeps its
/// kernel.
pub(super) fn large_shape_lower(a: f64, b: f64, x: f64, y: f64) -> Option<f64> {
    expansion(a, b, x, (y, 0.0), Arithmetic::for_shapes(a, b))
}

/// [`large_shape_lower`] with y = 1 - x given as a double-double (head, tail), for a caller that
/// knows the small side past a double: the F tails' ratio df1 x/(df1 x + df2) (`fisher.rs`
/// `f_large_shape_tails`, T-4435). Near x = 1 the expansion reads only y, and nu times y's rounding
/// is a relative error of about |ln tail| ulp in the result.
pub(super) fn large_shape_lower_pair(a: f64, b: f64, x: f64, y: (f64, f64)) -> Option<f64> {
    expansion(a, b, x, y, Arithmetic::Accurate)
}

/// How the expansion forms nu, z and r.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arithmetic {
    /// T-2528's and T-4119's: nu and r in doubles, the arithmetic their cells were measured on
    /// (t4017's bit-exact F cells track it). The range a >= [`T4119_MIN_SHAPE`], b <
    /// [`T4119_MAX_SECOND_SHAPE`] keeps it.
    T4119,
    /// T-4435's: nu = a + (b - 1)/2 and z = -nu ln x in double-double, and r = z^b e^-z / Gamma(b)
    /// with its exponent in double-double ([`gamma_power_ratio`]). nu rounded to a double is up to
    /// ulp(a)/2 off at a shape a that is not a multiple of 1/2 (2049.9, 2537.31), and z = nu |ln x|
    /// of 600 multiplies that into 6E-14 on a deep BETA.DIST tail (probes-numeric
    /// t3568-r5-bup-fresh B117 `BETA.DIST(0.751433992054,2049.9,..)`, 3.4E-14 off the math with nu
    /// in doubles). The double density rounds the deviance z - b - b ln(z/b), several hundred at a
    /// deep tail, to an |ln tail| ulp error of its own.
    Accurate,
}

impl Arithmetic {
    /// T-4435's range takes [`Arithmetic::Accurate`]; the old range keeps T-4119's.
    fn for_shapes(a: f64, b: f64) -> Self {
        if a < T4119_MIN_SHAPE || b >= T4119_MAX_SECOND_SHAPE {
            Self::Accurate
        } else {
            Self::T4119
        }
    }
}

/// (lower, upper) F tails above the mean in T-4435's regions: the upper tail `I_v(df2/2, df1/2)` by the
/// large-shape expansion ([`large_shape_lower_pair`]; `fisher.rs` `f_tail` routes it), with its small side
/// u = df1 x/(df1 x + df2) formed in double-double, since the double u's rounding is multiplied by
/// about |ln tail| in the tail. On T-4435's grid (docs/research/t4435; 60-digit mpmath at the exact
/// ratio):
/// - region 1 (df1 21 to 79, df2 1E4 to 2E6, past L140's 9 bup terms; tails to 1E-200) is within
///   3.7E-16, against 5.6E-12 by Cephes' fraction and 1.4E-13 by the expansion on the double ratio;
/// - region 2 (df1 80 to 400, df2 1E5 to 1E10) is within 1.3E-15, against 5.2E-8 by bfrac.
///
/// Excel is not this (T-4434's sweep, probes-numeric/t4434-huge-shape-outside-large-shape-beta: a few
/// units of |ln tail| ulp off the math in region 1, up to 8.5E-9 in region 2, T-4214); this is
/// decision 0114's `accuracy` stage. `None` below the mean, where the lower tail is the one formed
/// directly (today's kernel), or outside the expansion's bounds.
pub(super) fn f_large_shape_tails(df1: f64, df2: f64, x: f64) -> Option<(f64, f64)> {
    let (a, b) = (df1 / 2.0, df2 / 2.0);
    let scaled = dd_mul((df1, 0.0), (x, 0.0));
    let total = dd_add(scaled, (df2, 0.0));
    let u = dd_div(scaled, total);
    if u.0.is_nan() || u.0 <= a / (a + b) {
        return None;
    }
    let upper = large_shape_lower_pair(b, a, df2 / total.0, u)?;
    Some((half_complement(upper), upper))
}

/// The expansion in the given [`Arithmetic`].
fn expansion(a: f64, b: f64, x: f64, y: (f64, f64), arithmetic: Arithmetic) -> Option<f64> {
    if !(a >= MIN_SHAPE && b <= MAX_SECOND_SHAPE) {
        return None;
    }
    expand(a, b, x, y, arithmetic)
}

/// [`expansion`] without its bounds on a and b (T-4435's boundary table scores it below them).
fn expand(a: f64, b: f64, x: f64, y: (f64, f64), arithmetic: Arithmetic) -> Option<f64> {
    if !(a.is_finite() && a > 0.0 && b > 0.0 && x > 0.0 && y.0 > 0.0) {
        return None;
    }
    let bm1 = b - 1.0;
    let nu = a + 0.5 * bm1;
    // nu - a = (b - 1)/2 and nu itself, exactly, for the accurate arithmetic.
    let (half_bm1, nu_pair) = match arithmetic {
        Arithmetic::T4119 => ((nu - a, 0.0), (nu, 0.0)),
        Arithmetic::Accurate => {
            let bm1_pair = two_sum(b, -1.0);
            let half = (0.5 * bm1_pair.0, 0.5 * bm1_pair.1);
            (half, dd_add((a, 0.0), half))
        }
    };
    // ln x in double-double, from 1 - y exactly when x is near 1: the rounding of a given x near 1
    // is what nu multiplies into z (each unit of z is a factor e on the result).
    let x_pair = if x > 0.5 {
        dd_add(two_sum(1.0, -y.0), (-y.1, 0.0))
    } else {
        (x, 0.0)
    };
    let ln_x = ln_double_double_pair(x_pair);
    let (z, z_lo) = dd_mul(dd_neg(nu_pair), ln_x);
    if z.is_nan() || z <= 0.0 {
        return None;
    }
    let scale = shape_ratio_factor(a, b, half_bm1.0 + half_bm1.1);
    if !z.is_finite() {
        return Some(0.0);
    }
    // r = z^b e^-z / Gamma(b) = z times the gamma density at z.
    let r = match arithmetic {
        Arithmetic::Accurate => gamma_power_ratio(b, z),
        Arithmetic::T4119 => accurate_gamma_density(b, z, 1.0).map_or(0.0, |density| density * z),
    };
    if !(r.is_finite() && scale.is_finite()) {
        return None;
    }
    if r == 0.0 {
        // e^-z underflows: so does the tail, Q(b, z) at most a few times r.
        let upper = gamma_ur(b, z);
        return upper.is_finite().then_some(scale * upper);
    }
    let j0 = upper_over_prefactor(b, z, r)?;
    // Q(b, z + z_lo) = Q(b, z) - z_lo r / z to first order: the low part of z.
    let series = correction_series(b, z, ln_x.0, nu, j0)?;
    let value = scale * r * ((j0 - z_lo / z) + series);
    (value.is_finite() && value >= 0.0).then_some(value.min(1.0))
}

/// The lowest exponent [`gamma_power_ratio`] hands `dd_exp`, which is documented for |a| <= 650. Below
/// it r is the square of e^(E/2); below twice it r is far below the double range and the double density
/// answers.
const MIN_RATIO_EXPONENT: f64 = -650.0;

/// r = z^b e^-z / Gamma(b), z times the gamma density at z, with its exponent in double-double:
/// exp(b ln(z/b) - (z - b) - stirlerr(b)) sqrt(b / 2 pi), from ln Gamma(b) = (b - 1/2) ln b - b +
/// ln sqrt(2 pi) + stirlerr(b). The double density rounds the deviance z - b - b ln(z/b), several
/// hundred at a deep tail, and that is a relative error of about |ln tail| ulp in the result. On
/// T-4435's grid (docs/research/t4435) the F tails at df1 20 to 79, df2 1E4 to 2E6 and tails near
/// 1E-200 go from 2 such units off the 60-digit math to 0.1. Past [`MIN_RATIO_EXPONENT`] the double
/// density.
fn gamma_power_ratio(b: f64, z: f64) -> f64 {
    let ln_ratio = dd_add(ln_double_double(z), dd_neg(ln_double_double(b)));
    let exponent = dd_add(dd_mul((b, 0.0), ln_ratio), dd_neg(two_sum(z, -b)));
    let exponent = dd_add(exponent, dd_neg(stirlerr_exact(b)));
    if exponent.0.is_nan() || exponent.0 <= 2.0 * MIN_RATIO_EXPONENT {
        return accurate_gamma_density(b, z, 1.0).map_or(0.0, |density| density * z);
    }
    let root = (b / (2.0 * std::f64::consts::PI)).sqrt();
    if exponent.0 > MIN_RATIO_EXPONENT {
        let power = dd_exp(exponent);
        return (power.0 + power.1) * root;
    }
    // Past dd_exp's range, e^E = (e^(E/2))^2, multiplied in an order that stays normal as long as r
    // does: the deepest F tails before Excel's zero cut (probes/t4045-f-dist-rt-zero-cut-fresh C3,
    // z about 717, E about -656).
    let half = dd_exp((0.5 * exponent.0, 0.5 * exponent.1));
    let half = half.0 + half.1;
    (half * root) * half
}

/// Q(b, z) / r with r = z^b e^-z / Gamma(b): for z of b + 1 or more Legendre's continued fraction
/// 1/(z + 1 - b - 1(1 - b)/(z + 3 - b - 2(2 - b)/(z + 5 - b - ...))), evaluated from the bottom up
/// at a depth doubled until two depths agree, which keeps it within about an ulp where the forward
/// (Lentz) product of `incomplete_gamma` carries up to 7 (Q(2.5, 4.9999999825), T-2528); below,
/// that module's Q(b, z) over r. `None` past [`MAX_FRACTION_DEPTH`].
fn upper_over_prefactor(b: f64, z: f64, r: f64) -> Option<f64> {
    if z < b + 1.0 {
        let upper = gamma_ur(b, z);
        return upper.is_finite().then_some(upper / r);
    }
    let mut depth = 16_u32;
    let mut previous = legendre_fraction(b, z, depth);
    while depth < MAX_FRACTION_DEPTH {
        depth *= 2;
        let next = legendre_fraction(b, z, depth);
        if (next - previous).abs() <= f64::EPSILON * next.abs() {
            return next.is_finite().then_some(next);
        }
        previous = next;
    }
    None
}

/// Legendre's fraction for Q(b, z) / r cut at `depth` partial quotients, from the bottom up.
fn legendre_fraction(b: f64, z: f64, depth: u32) -> f64 {
    let mut tail = 0.0;
    for i in (1..=depth).rev() {
        let i = f64::from(i);
        tail = -i * (i - b) / (z + 2.0 * i + 1.0 - b + tail);
    }
    1.0 / (z + 1.0 - b + tail)
}

/// Gamma(a + b) / (Gamma(a) nu^b) for a of 8 or more, as exp of
/// -a rlog1(b/a) + (b - 1/2) ln(1 + b/a) - b ln(1 + (b - 1)/(2a)) and the Stirling corrections:
/// the logs of Gamma(a + b) / Gamma(a) and of nu^b each near b ln a, cancelled before they are
/// formed (their difference is about b(b - 1)/(2a)).
fn shape_ratio_factor(a: f64, b: f64, nu_minus_a: f64) -> f64 {
    let t = b / a;
    let exponent = -a * rlog1(t) + (b - 0.5) * t.ln_1p() - b * (nu_minus_a / a).ln_1p()
        + stirling_correction(a + b)
        - stirling_correction(a);
    exponent.exp()
}

/// bgrat's sum of `d_n J_n` for n = 1, 2, ... ([`MAX_SERIES_TERMS`] at most), from
/// `J_0` = Q(b, z) / r: `J_n` = ((b + 2n - 2)(b + 2n - 1) `J_{n-1}` + (z + b + 2n - 1) t^(n-1)) v
/// with v = 1/(4 nu^2), t = (ln x)^2 / 4, and `d_n` from the coefficients `c_n` = 1/(2n+1)!. `None`
/// if the terms have not fallen below [`SERIES_TOLERANCE`] by then.
fn correction_series(b: f64, z: f64, ln_x: f64, nu: f64, j0: f64) -> Option<f64> {
    let bm1 = b - 1.0;
    let v = 0.25 / (nu * nu);
    let t2 = 0.25 * ln_x * ln_x;
    let mut c = [0.0; MAX_SERIES_TERMS];
    let mut d = [0.0; MAX_SERIES_TERMS];
    let (mut j, mut sum, mut t, mut cn, mut n2) = (j0, 0.0, 1.0, 1.0, 0.0);
    for n in 1..=MAX_SERIES_TERMS {
        let bp2n = b + n2;
        j = (bp2n * (bp2n + 1.0) * j + (z + bp2n + 1.0) * t) * v;
        n2 += 2.0;
        t *= t2;
        cn /= n2 * (n2 + 1.0);
        let nm1 = n - 1;
        c[nm1] = cn;
        let mut s = 0.0;
        let mut coef = b - n as f64;
        for i in 1..n {
            s += coef * c[i - 1] * d[nm1 - i];
            coef += b;
        }
        d[nm1] = bm1 * cn + s / n as f64;
        let dj = d[nm1] * j;
        sum += dj;
        if dj.abs() <= SERIES_TOLERANCE * (j0 + sum).abs() {
            return Some(sum);
        }
    }
    // Not settled within bgrat's cap: the caller keeps its kernel.
    None
}

#[cfg(test)]
mod tests {
    use super::{expand, gamma_power_ratio, large_shape_lower, Arithmetic, MIN_SHAPE};

    /// T-4435's boundary table: the expansion forced at every point of
    /// docs/research/t4435/boundary-points.tsv (`x a b` per line), below its bounds too, printed as
    /// `row<TAB>value` for `boundary.py score-forced`. Run:
    /// `cargo test -p rustsheet_engine --lib boundary_table -- --ignored --nocapture`.
    #[test]
    #[ignore = "research table (T-4435); reads docs/research/t4435/boundary-points.tsv"]
    fn boundary_table() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/research/t4435/boundary-points.tsv"
        );
        let text = std::fs::read_to_string(path).expect("boundary-points.tsv");
        for (row, line) in text.lines().enumerate() {
            let v: Vec<f64> = line
                .split('\t')
                .map(|t| t.parse().expect("number"))
                .collect();
            let (x, a, b) = (v[0], v[1], v[2]);
            let got = expand(a, b, x, (1.0 - x, 0.0), Arithmetic::Accurate).unwrap_or(f64::NAN);
            println!("{}\t{got:e}", row + 1);
        }
    }

    /// The expansion's lower bound on a holds where it is set.
    #[test]
    fn the_expansion_starts_at_min_shape() {
        let below = MIN_SHAPE * (1.0 - f64::EPSILON);
        assert_eq!(large_shape_lower(below, 10.5, 0.99, 0.01), None);
        assert!(large_shape_lower(MIN_SHAPE, 10.5, 0.99, 0.01).is_some());
    }

    /// r = z^b e^-z / Gamma(b) in double-double against 50-digit mpmath
    /// (`b*log(z) - z - loggamma(b)`, exponentiated).
    #[test]
    fn gamma_power_ratio_matches_mpmath() {
        for (b, z, reference) in [
            (27.675, 600.0, 5.477943728990614e-212),
            (10.5, 40.0, 2.4860738518595877e-07),
            (150.7, 151.0, 4.8932457439315895),
        ] {
            let got = gamma_power_ratio(b, z);
            assert!(
                (got / reference - 1.0).abs() < 4e-16,
                "r({b}, {z}) = {got} against {reference}"
            );
        }
    }
    use crate::functions::statistical::incomplete_beta::beta_reg_tails;

    /// Above the mean of Beta(2.5, 5E9) (5E-10) the reflected side takes the expansion: both tails
    /// stay finite, in [0, 1], add up to 1 and fall monotonically in the upper tail.
    #[test]
    fn a_sweep_above_the_mean_stays_in_range() {
        let mut previous_upper = 1.0;
        for step in 0..=2_000 {
            let x = 5e-10 * 1.01_f64.powi(step) * 1.000_001;
            let (lower, upper) = beta_reg_tails(x, 1.0 - x, 2.5, 5e9);
            assert!(lower.is_finite() && upper.is_finite(), "{x}");
            assert!(
                (0.0..=1.0).contains(&lower) && (0.0..=1.0).contains(&upper),
                "{x}"
            );
            assert!((lower + upper - 1.0).abs() <= f64::EPSILON, "{x}");
            assert!(
                upper <= previous_upper,
                "{x}: {upper} after {previous_upper}"
            );
            previous_upper = upper;
        }
        assert_eq!(previous_upper, 0.0);
    }

    /// 1 - `I_u(5/2, df2/2)` = `I_v(df2/2, 5/2)`, the F tail above x, against a 60-digit mpmath
    /// `betainc(5/2, df2/2, u, 1)` (120 digits: at 60 it is 4E-14 off at x = 50) at the double
    /// u = 5x / (5x + df2) (T-2528).
    #[test]
    fn f_upper_tails_at_a_huge_df2_match_the_reference() {
        for (df2, x, reference) in [
            (1e10, 2.0, 0.07523524624568315),
            (1e10, 10.0, 1.3857974134229093e-09),
            (1e10, 20.0, 5.285149604903511e-20),
            (1e10, 50.0, 5.496930852316323e-52),
            (1e9, 2.0, 0.07523524713822163),
            (1e8, 5.0, 0.0001393339615389781),
            (1e8, 20.0, 5.285272758334189e-20),
        ] {
            let scaled: f64 = 5.0 * x;
            let total = scaled + df2;
            let got = large_shape_lower(df2 / 2.0, 2.5, df2 / total, scaled / total);
            let got = got.unwrap_or(f64::NAN);
            assert!(
                (got / reference - 1.0).abs() < 2e-15,
                "{df2} {x}: {got} against {reference}"
            );
        }
    }

    #[test]
    fn only_a_huge_first_shape_takes_the_expansion() {
        assert_eq!(large_shape_lower(100.0, 2.5, 0.99, 0.01), None);
        assert_eq!(large_shape_lower(1e7, 200.5, 0.99, 0.01), None);
        assert_eq!(large_shape_lower(3.366, 3.43e-18, 0.997, 0.003), None);
        assert_eq!(large_shape_lower(1e200, 1e100, 0.1, 0.9), None);
        assert_eq!(large_shape_lower(5e9, 0.0, 0.99, 0.01), None);
        assert_eq!(large_shape_lower(5e9, 2.5, 0.0, 1.0), None);
        // Far below the mean the tail is 0, not NaN.
        assert_eq!(large_shape_lower(5e9, 2.5, 1e-300, 1.0), Some(0.0));
    }
}
