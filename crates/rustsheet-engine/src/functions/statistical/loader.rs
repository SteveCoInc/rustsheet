//! Catherine Loader's saddle point ("Fast and Accurate Computation of Binomial Probabilities",
//! 2000) as Excel's beta density computes it for min(a, b) > 1.
//!
//! Excel's beta density for min(a, b) > 1 is the same saddle point in Excel's own arithmetic
//! (statistics.md "Beta density for min(a,b) > 1 is Loader's saddle point", T-2464, T-2513): x87
//! stored operations on Excel's LN, a 0.2 cut in its bd0 ([`excel_bd0`]), the exact Stirling error
//! ([`stirlerr_exact`]) and an exponent held in an x87 register into Excel's EXP
//! ([`excel_dbinom_exponent`], [`excel_beta_density`]). The model's shape constant carries an
//! unexplained residual of 2-4 units of 2^-53 and the series branch 3-41 units of noise (T-2586),
//! so it is not bit-exact everywhere.
//!
//! `poisson.rs` is a second consumer: Excel's POISSON.DIST pmf is this saddle point too, on
//! [`excel_bd0`] and [`stirlerr_exact`] (T-2055), so retuning either moves POISSON cells as well.
//! The Student t density is a third ([`excel_t_density`], T-2585).

use crate::arithmetic::{extended_product, extended_quotient, extended_sqrt, extended_sum};
use crate::x87_exp::{
    dd_add, dd_div, dd_div_small, dd_mul, excel_exp, excel_exp_extended, two_sum,
};
use crate::x87_log10::{excel_ln, ln_double_double_pair};
use crate::x87_register::Register;

use super::gammaln::STIRLING;
use super::incomplete_beta::FRAC_1_SQRT_2PI;

/// A double-double `hi + lo`.
type Dd = (f64, f64);

/// ln sqrt(2 pi) = 0.918938533204672741780..., the correctly rounded double
/// (`0x3FED_67F1_C864_BEB5`). Deliberately not `gammaln.rs`'s `LN_SQRT_2PI`, which is one ulp
/// lower because it reproduces the constant Excel's GAMMALN Stirling sum adds; these pieces are
/// accurate forms, not Excel's, so they take the exact value.
const LN_SQRT_2PI_CORRECTLY_ROUNDED: f64 = 0.918_938_533_204_672_8;

/// The bound on [`excel_bd0`]'s series terms; it converges in far fewer (|v| < 0.2, so v^2 < 0.04).
const BD0_MAX_TERMS: u32 = 1000;

/// The Stirling series terms for k = 11 and 12, `B_2k / (2k (2k - 1))`, after `gammaln::STIRLING`'s
/// k = 1..=10. At n = 15 the next term is under 1e-26, far below the 2^-64 of the exponent
/// [`stirlerr_exact`] goes into.
const STIRLING_EXTRA: [f64; 2] = [77_683.0 / 5_796.0, -236_364_091.0 / 1_506_960.0];

/// From this n on [`stirlerr_exact`] sums the Stirling series; below it the recurrence first
/// steps n up past it.
const STIRLING_SERIES_MIN: f64 = 15.0;

/// The exact Stirling error `ln Gamma(n + 1) - (n + 1/2) ln n + n - ln sqrt(2 pi)` of a double
/// n > 0 as a double-double (about 1e-25 absolute), the stirlerr of Excel's beta density
/// (statistics.md "Beta density for min(a,b) > 1 ...": stirlerr is accurate, a GAMMALN or a
/// double-rounded lgamma form misses by rms 15-26 units). Not the F density's model of
/// Excel's rounded stirlerr (`f_density`'s error table), which that density keeps. Below 15 the recurrence
/// stirlerr(m) = stirlerr(m + 1) + (m + 1/2) (ln(m + 1) - ln m) - 1 steps n up (at most 15 steps,
/// every m = n + i exact as a double-double), then the series in 1/m.
pub(crate) fn stirlerr_exact(n: f64) -> Dd {
    let mut sum = (0.0, 0.0);
    let mut step = 0.0;
    let mut ln_here = ln_double_double_pair(two_sum(n, 0.0));
    while n + step < STIRLING_SERIES_MIN {
        let ln_next = ln_double_double_pair(two_sum(n, step + 1.0));
        let factor = two_sum(n, step + 0.5);
        let term = dd_mul(factor, dd_add(ln_next, (-ln_here.0, -ln_here.1)));
        sum = dd_add(sum, dd_add(term, (-1.0, 0.0)));
        ln_here = ln_next;
        step += 1.0;
    }
    dd_add(sum, stirling_series(two_sum(n, step)))
}

/// Stirling's series `sum_k B_2k / (2k (2k - 1) m^(2k - 1))` for m >= 15, the two leading terms in
/// double-double and the rest (`STIRLING` from k = 3, then [`STIRLING_EXTRA`]) in double (it is under
/// 1e-9 there).
fn stirling_series(m: Dd) -> Dd {
    let z = dd_div((1.0, 0.0), m);
    let z2 = dd_mul(z, z);
    let tail = STIRLING[2..]
        .iter()
        .chain(&STIRLING_EXTRA)
        .rev()
        .fold(0.0, |acc, &coefficient| coefficient + z2.0 * acc);
    let second = dd_add(dd_div_small((-1.0, 0.0), 360.0), dd_mul(z2, (tail, 0.0)));
    let first = dd_add(dd_div_small((1.0, 0.0), 12.0), dd_mul(z2, second));
    dd_mul(z, first)
}

/// The ratio cut of Excel's bd0: its series runs while |k - M| < 0.2 (k + M) (statistics.md "Beta
/// density for min(a,b) > 1 ...": 0.25 drops (5.5, 12.5) from 113 to 13 of 120 exact cells; R's
/// is 0.1).
/// FITTED(T-2464, 3072 cells, BETA.DIST a 2.5, b 3.5, v 0.130..0.250)
const EXCEL_BD0_CUT: f64 = 0.2;

/// Excel's bd0(k, M) = k ln(k / M) + M - k, as the double its beta density sums (model M4 of
/// docs/research/t2464/r3-loader-const, with r3-loader-bd0's knobs), in x87 stored operations:
/// d = fl(k - M) and s = fl(k + M); then
/// - the log form when |d| >= RN64(0.2 s): fl(t + fl(M - k)) with t = fl(k LN(fl(k / M))) and LN
///   Excel's (M - k stored: beyond a ratio of 2 it rounds, and the stored form fits Excel);
/// - below the cut Loader's series in v = fl(d / s) (stored), in extended precision, which carries
///   +1 times the rounding error of fl(2 k v): Excel's arithmetic there is not identified, and this
///   phenomenological term (r3-loader-bd0's `ph` knob) is what fits (T-2586 owns the rest).
///
/// For k > 0 and M > 0; the series has at most [`BD0_MAX_TERMS`] terms.
pub(crate) fn excel_bd0(k: f64, m: f64) -> f64 {
    match bd0_branch(k, m) {
        Bd0Branch::Series(sum) => sum.store(),
        Bd0Branch::Log(t) => extended_sum(t, extended_sum(m, -k)),
    }
}

/// [`excel_bd0`] as Excel's t density uses it, left in an x87 register: the series sum unstored,
/// and the log form `t + (M - k)` summed in the register rather than stored (r3-loader-bd0's
/// `ld = 'e'`; docs/research/t2464/r3-tdensity model LT1). Storing it first moves
/// `T.DIST(2.2014...,11,FALSE)` (t2513-r3-tdensity-hold2) by 3 ulp off Excel's (T-2585).
fn excel_bd0_unstored(k: f64, m: f64) -> Register {
    match bd0_branch(k, m) {
        Bd0Branch::Series(sum) => sum,
        Bd0Branch::Log(t) => Register::load(t).add(Register::load(m).add(Register::load(-k))),
    }
}

/// Which form [`excel_bd0`] takes, with what both callers share of it.
enum Bd0Branch {
    /// The series sum, unstored.
    Series(Register),
    /// The log form's t = fl(k LN(fl(k / M))).
    Log(f64),
}

/// [`excel_bd0`]'s cut between its series and its log form.
fn bd0_branch(k: f64, m: f64) -> Bd0Branch {
    let d = extended_sum(k, -m);
    let s = extended_sum(k, m);
    let cut = Register::load(EXCEL_BD0_CUT).mul(Register::load(s));
    if Register::load(d.abs()).add(cut.neg()).is_negative() {
        return Bd0Branch::Series(excel_bd0_series(k, d, s));
    }
    Bd0Branch::Log(extended_product(k, excel_ln(extended_quotient(k, m))))
}

/// [`excel_bd0`]'s series branch: R's loop `s = d v + 2k sum_j v^(2j+1) / (2j + 1)` in x87 registers
/// with v stored, less the rounding error of fl(2 k v), left unstored.
fn excel_bd0_series(k: f64, d: f64, s: f64) -> Register {
    let v = Register::load(extended_quotient(d, s));
    let mut ej = Register::load(2.0 * k).mul(v);
    let rounding = Register::load(ej.store()).add(ej.neg());
    let mut sum = Register::load(d).mul(v).add(rounding.neg());
    let v2 = v.mul(v);
    for j in 1..BD0_MAX_TERMS {
        ej = ej.mul(v2);
        let next = sum.add(ej.div(Register::load(f64::from(2 * j + 1))));
        if next.same_value(sum) {
            return next;
        }
        sum = next;
    }
    sum
}

/// ln(2 pi), the double Excel's lf starts from (R's `M_LN_2PI`, correctly rounded:
/// `0x1.d67f1c864beb5p+0`, exactly twice [`LN_SQRT_2PI_CORRECTLY_ROUNDED`]). Bit-exact counts are
/// very sensitive to it (statistics.md "Beta density for min(a,b) > 1 ...").
const LN_2PI: f64 = 2.0 * LN_SQRT_2PI_CORRECTLY_ROUNDED;

/// The exponent of Excel's Loader binomial density `dbinom_raw(k, n, p, q)` = EXP(E) for
/// 0 < k < n, as the exact double-double sum
/// E = stirlerr(n) - stirlerr(k) - stirlerr(nk) - lf/2 + X, which Excel holds in an x87 register
/// (statistics.md "Beta density for min(a,b) > 1 ...", model M4):
/// - k, n and nk = n - k are the caller's stored doubles (Excel's own, not recomputed: single-ulp
///   neighbours of a shape jump by up to 20 units with the rounding of nk);
/// - the stirlerr terms exact ([`stirlerr_exact`]);
/// - lf = RN64(RN64(RN64(ln 2 pi + LN k) + LN nk) - LN n), Excel's LN, extended sums;
/// - X = RN64(-bd0(k, fl(n p)) - bd0(nk, fl(n q))) with [`excel_bd0`].
///
/// `None` where a piece is not finite (an n p or n q of 0, a ratio past the double range). Exposed
/// for the binomial pmf, which shares it (T-2146).
pub(crate) fn excel_dbinom_exponent(k: f64, n: f64, nk: f64, p: f64, q: f64) -> Option<Dd> {
    let np = extended_product(n, p);
    let nq = extended_product(n, q);
    if !(k > 0.0 && nk > 0.0 && np > 0.0 && nq > 0.0 && n.is_finite()) {
        return None;
    }
    let bd0_p = excel_bd0(k, np);
    let bd0_q = excel_bd0(nk, nq);
    // A register loads a NaN or an infinity as 0, so a ratio past the double range (k / M for a
    // subnormal M) must stop here.
    if !(bd0_p.is_finite() && bd0_q.is_finite()) {
        return None;
    }
    let x_part = Register::load(-bd0_p).add(Register::load(-bd0_q));
    let lf = Register::load(LN_2PI)
        .add(Register::load(excel_ln(k)))
        .add(Register::load(excel_ln(nk)))
        .add(Register::load(-excel_ln(n)));
    let (lf_hi, lf_lo) = lf.to_double_double();
    let parts = dd_add(stirlerr_exact(k), stirlerr_exact(nk));
    let stirling = dd_add(stirlerr_exact(n), (-parts.0, -parts.1));
    let exponent = dd_add(
        dd_add(stirling, (-0.5 * lf_hi, -0.5 * lf_lo)),
        x_part.to_double_double(),
    );
    (exponent.0.is_finite() && exponent.1.is_finite()).then_some(exponent)
}

/// Excel's BETA.DIST(x, a, b, FALSE) for min(a, b) > 1 and 0 < x < 1: R dbeta's
/// `(a + b - 1) dbinom_raw(a - 1, a + b - 2, x, fl(1 - x))` (Loader 2000) in Excel's arithmetic,
/// EXP(LN(n + 1) + E) with E [`excel_dbinom_exponent`]'s, the sum held in an x87 register into
/// Excel's EXP (statistics.md "Beta density for min(a,b) > 1 is Loader's saddle point", T-2464,
/// T-2513). k = fl(a - 1), n = fl(fl(a + b) - 2) and nk = fl(n - k) are x87 stored operations. No
/// switch at large shapes (no a0 = 8 or a + b = 171.6 branch). `None` outside that domain or where
/// a piece is not finite, where the caller keeps its other forms.
pub(crate) fn excel_beta_density(x: f64, a: f64, b: f64) -> Option<f64> {
    if !(a > 1.0 && b > 1.0 && x > 0.0 && x < 1.0 && a.is_finite() && b.is_finite()) {
        return None;
    }
    let k = extended_sum(a, -1.0);
    let n = extended_sum(extended_sum(a, b), -2.0);
    let nk = extended_sum(n, -k);
    let q = extended_sum(1.0, -x);
    let exponent = excel_dbinom_exponent(k, n, nk, x, q)?;
    let ln_scale = excel_ln(extended_sum(n, 1.0));
    let total = dd_add((ln_scale, 0.0), exponent);
    if !total.0.is_finite() {
        return None;
    }
    let density = excel_exp_extended(Register::from_double_double(total));
    density.is_finite().then_some(density)
}

/// The bound on [`loader_t_constant`]'s series terms; at n = 3 (ratio 1/3) it reaches 1E-33 in
/// about 70.
const T_CONSTANT_MAX_TERMS: u32 = 400;

/// The exact shape constant of Loader's t density,
/// `-bd0(n/2, (n+1)/2) + stirlerr((n+1)/2) - stirlerr(n/2)`, as a double-double. The bd0 part is
/// `(n/2) ln(1 + 1/n) - 1/2 = sum_{k>=2} (-1)^(k+1) / (2k n^(k-1))`, summed as that series so
/// that it keeps its digits at large n (it is about -1/(4n)). Excel's own constant is 1-2 units
/// of 2^-53 off it per df, unidentified (statistics.md "Student t density is Loader's dt", T-2586).
fn loader_t_constant(n: f64) -> Dd {
    let z = dd_div((1.0, 0.0), (n, 0.0));
    let mut power = z;
    let mut sum = (0.0, 0.0);
    for k in 2..T_CONSTANT_MAX_TERMS {
        let term = dd_div_small(power, f64::from(2 * k));
        sum = if k % 2 == 0 {
            dd_add(sum, (-term.0, -term.1))
        } else {
            dd_add(sum, term)
        };
        if term.0.abs() < 1e-36 * sum.0.abs() {
            break;
        }
        power = dd_mul(power, z);
    }
    let stirling = dd_add(stirlerr_exact(0.5 * (n + 1.0)), {
        let half = stirlerr_exact(0.5 * n);
        (-half.0, -half.1)
    });
    dd_add(sum, stirling)
}

/// Excel's T.DIST(t, n, FALSE) for a truncated df n >= 1 (statistics.md "Student t density is
/// Loader's dt", T-2513, T-2585), every operation an x87 one stored to a double:
/// - df 1: `fl(1/fl(pi fl(1 + fl(t t))))`; df 2: `EXP(fl(-1.5 LN(fl(2 + fl(t t)))))`;
/// - df >= 3, Loader's saddle point: xx = fl(t t), q = fl(xx/n), w = fl(1 + q); when xx > n,
///   u = fl(fl(LN(w) n)/2), otherwise u = fl(-bd0(fl(n/2), fl(fl(n + xx)/2)) + fl(xx/2)) with
///   Excel's bd0 held in a register ([`excel_bd0_unstored`]); then E = fl(C - u) with the exact constant C
///   ([`loader_t_constant`]) held in an x87 register, and
///   `fl(fl(EXP(E)/fl(sqrt(w))) / sqrt(2 pi))`, `1/sqrt(2 pi)` the correctly rounded double.
///
/// On docs/research/t2464/cells.tsv this is within 15 digits on 18,631 of its 18,699 df >= 3
/// cells and bit-exact on all 1,077 df 1 and df 2 cells. `None` for a non-finite piece (t t
/// overflowing), where the caller keeps its other form.
pub(crate) fn excel_t_density(t: f64, n: f64) -> Option<f64> {
    if !(n >= 1.0 && t.is_finite() && n.is_finite()) {
        return None;
    }
    let xx = extended_product(t, t);
    if !xx.is_finite() {
        return None;
    }
    if n == 1.0 {
        let denominator = extended_product(std::f64::consts::PI, extended_sum(1.0, xx));
        let density = extended_quotient(1.0, denominator);
        return density.is_finite().then_some(density);
    }
    if n == 2.0 {
        let exponent = extended_product(-1.5, excel_ln(extended_sum(2.0, xx)));
        let density = excel_exp(exponent);
        return density.is_finite().then_some(density);
    }
    let q = extended_quotient(xx, n);
    let w = extended_sum(1.0, q);
    let u = if xx > n {
        extended_quotient(extended_product(excel_ln(w), n), 2.0)
    } else {
        let mean = extended_quotient(extended_sum(n, xx), 2.0);
        excel_bd0_unstored(extended_quotient(n, 2.0), mean)
            .neg()
            .add(Register::load(extended_quotient(xx, 2.0)))
            .store()
    };
    if !u.is_finite() {
        return None;
    }
    let exponent = Register::from_double_double(loader_t_constant(n))
        .add(Register::load(-u))
        .store();
    let scaled = extended_quotient(excel_exp(exponent), extended_sqrt(w));
    let density = extended_product(scaled, FRAC_1_SQRT_2PI);
    density.is_finite().then_some(density)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        reason = "test: a malformed fixture fails the test"
    )]
    use super::*;
    use crate::test::util::{ulps, within_fifteen_digits};

    #[test]
    fn stirlerr_exact_matches_mpmath() {
        // mpmath (50 digits) ln Gamma(n + 1) - (n + 1/2) ln n + n - ln sqrt(2 pi) at the same
        // doubles, as hi + lo: the recurrence below 15 and the series from 15 on.
        for (n, hi, lo) in [
            (1e-10, 10.593_986_934_110_418, 7.124_858_491_460_29e-16),
            (0.03, 0.952_950_298_497_347_8, -9.226_922_120_730_322e-19),
            (0.5, 0.153_426_409_720_027_36, -1.159_523_406_923_149_8e-17),
            (1.0, 0.081_061_466_795_327_26, -2.850_421_842_770_954_6e-18),
            (4.3, 0.019_345_426_904_332_788, 2.960_181_844_418_298e-19),
            (7.5, 0.011_104_559_758_206_917, -2.038_131_079_125_446e-20),
            (14.99, 0.005_558_438_081_636_329, 2.937_702_605_005_666e-19),
            (15.0, 0.005_554_733_551_962_801, 3.185_347_414_196_856e-19),
            (
                16.0,
                0.005_207_655_919_609_640_4,
                -2.883_608_994_661_908_6e-21,
            ),
            (
                100.0,
                0.000_833_330_555_634_914_7,
                1.246_488_391_106_547_6e-20,
            ),
        ] {
            let (got_hi, got_lo) = stirlerr_exact(n);
            let error = (got_hi - hi) + (got_lo - lo);
            assert!(
                error.abs() < 1e-24 * hi.max(1.0),
                "stirlerr({n}): {error:e}"
            );
        }
    }

    #[test]
    fn excel_beta_density_matches_excel() {
        // Excel's BETA.DIST(x, a, b, FALSE) bit for bit (the arguments are the doubles of each
        // cell's formula; the model is bit-exact on about 20-30% of cells, T-2586): both bd0 sides
        // in the log form (probes-numeric t2513-r3-loader-const-hold1 A342, A369, A3251), a series
        // side (t2513-r3-loader-bd0-a A4814, A5192, A5787; t2513-verify-r3-loader-bd0-hold A649,
        // A1214), integer and half-integer shapes (t2513-r3-loader-bd0-b A2760, A3666), a shape near 1
        // and min >= 8 (t2464-r2-density-large-p1 A9967, A7281) and diverse shapes
        // (t2513-r3-loader-const-diverse1 A252, A1307).
        let cases: [(f64, f64, f64, f64); 14] = [
            (
                0.376_922_732_080_715_4,
                8.3,
                5.899_999_999_999_998,
                0.886_766_852_474_694_8,
            ),
            (
                0.365_288_592_308_167_5,
                8.3,
                5.899_999_999_999_999_5,
                0.772_288_531_152_508_3,
            ),
            (
                0.444_288_270_094_662_3,
                33.976,
                14.85,
                0.009_542_680_147_732_819,
            ),
            (0.523_647_654_910_737_2, 5.5, 12.5, 0.534_367_175_242_143_3),
            (0.344_717_003_125_003_9, 5.5, 12.5, 3.187_884_049_445_971_7),
            (0.192_030_067_200_634_6, 5.5, 12.5, 2.547_970_230_648_507),
            (0.128_688_030_000_000_23, 9.7, 21.3, 0.258_876_334_448_779_7),
            (
                0.707_297_924_512_993_8,
                4.75,
                11.25,
                0.011_149_186_257_390_1,
            ),
            (0.247_103_647_826_088_96, 3.0, 5.0, 2.060_100_389_491_967),
            (0.679_114_979_411_773_7, 4.5, 4.5, 1.439_649_122_004_031),
            (0.099_840_000_000_003_4, 1.1, 1.9, 1.579_203_777_920_108_2),
            (0.165_435_806_451_616, 8.5, 40.0, 7.447_826_760_045_769),
            (
                0.781_164_318_528_283_5,
                7.3,
                7.501,
                0.232_804_337_859_794_24,
            ),
            (0.616_917_685_110_905_7, 8.67, 1.45, 0.428_699_138_085_424_5),
        ];
        for (x, a, b, excel) in cases {
            let got = excel_beta_density(x, a, b);
            assert_eq!(
                got.map(f64::to_bits),
                Some(excel.to_bits()),
                "BETA.DIST({x},{a},{b},FALSE) = {got:?}, Excel {excel}"
            );
        }
    }

    #[test]
    fn excel_beta_density_domain() {
        // Outside min(a, b) > 1 and 0 < x < 1, and where n x underflows to 0, the caller keeps
        // its other forms.
        assert_eq!(excel_beta_density(0.5, 1.0, 3.0), None);
        assert_eq!(excel_beta_density(0.5, 3.0, 0.5), None);
        assert_eq!(excel_beta_density(0.0, 3.0, 3.0), None);
        assert_eq!(excel_beta_density(1.0, 3.0, 3.0), None);
        assert_eq!(excel_beta_density(5e-324, 1.5, 3.0), None);
        assert_eq!(excel_beta_density(f64::NAN, 3.0, 3.0), None);
    }

    /// The two hold-outs of `test/fixtures/t_density_hold.fixture`: (t, n, Excel's T.DIST(t,n,FALSE)).
    fn t_density_holdouts() -> [Vec<(f64, f64, f64)>; 2] {
        let bits = |field: &str| f64::from_bits(u64::from_str_radix(field, 16).unwrap());
        let mut parts = [Vec::new(), Vec::new()];
        let mut part = 0;
        for line in include_str!("../../test/fixtures/t_density_hold.fixture").lines() {
            let fields: Vec<&str> = line.split_whitespace().collect();
            match fields.as_slice() {
                [] => part = 1,
                ["#", ..] => {}
                [t, n, excel] => parts[part].push((bits(t), n.parse().unwrap(), bits(excel))),
                _ => panic!("malformed fixture line {line:?}"),
            }
        }
        parts
    }

    // T-2585 acceptance: the research model LT1 with the exact constant scores 573 exact / 960 within
    // 4 ulp / 960 within 15 digits on t2513-r3-tdensity-hold2 and 505 / 918 / 977 on the verifier's
    // t2513-verify-r3-tdensity-hold (df 13-50); the correctly rounded density gets 266 / 776 / 839 and
    // 93 / 555 / 787 (docs/research/t2464/r3-tdensity RESULTS.md, verify-r3-tdensity).
    #[test]
    fn t_density_holdouts_score_as_the_loader_model() {
        let [hold2, verify] = t_density_holdouts();
        assert_eq!((hold2.len(), verify.len()), (960, 1000));
        for (rows, min_exact, min_ulps, min_fifteen) in
            [(hold2, 573, 960, 960), (verify, 505, 918, 977)]
        {
            let (mut exact, mut close, mut fifteen) = (0, 0, 0);
            for (t, n, excel) in rows {
                let density = excel_t_density(t, n).unwrap_or(f64::NAN);
                exact += usize::from(density == excel);
                close += usize::from(ulps(density, excel) <= 4);
                fifteen += usize::from(within_fifteen_digits(excel, density));
            }
            assert!(
                exact >= min_exact && close >= min_ulps && fifteen >= min_fifteen,
                "{exact} / {close} / {fifteen}"
            );
        }
    }

    #[test]
    fn t_density_closed_forms_at_df_one_and_two() {
        // fl(1/fl(pi fl(1 + t t))) and EXP(fl(-1.5 LN(fl(2 + t t)))); T.DIST(0,1,FALSE) is 1/pi.
        assert_eq!(excel_t_density(0.0, 1.0), Some(1.0 / std::f64::consts::PI));
        assert_eq!(
            excel_t_density(0.0, 2.0),
            Some(excel_exp(-1.5 * excel_ln(2.0)))
        );
        assert_eq!(excel_t_density(1e200, 5.0), None);
        assert_eq!(excel_t_density(f64::NAN, 5.0), None);
    }
}
