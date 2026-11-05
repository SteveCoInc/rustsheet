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

#[cfg(test)]
mod tests {
    use super::super::gamma_ur;
    use super::super::tests::assert_near_mode;

    /// 60-digit (x, P, Q) at shape 21, x from 0.6a to 1.4a by 0.1a (mpmath `gammainc`, T-4000).
    const NEAR_MODE_21: [(f64, f64, f64); 9] = [
        (12.6, 0.018_677_090_738_518_077, 0.981_322_909_261_481_9),
        (14.7, 0.071_050_236_332_737_9, 0.928_949_763_667_262_1),
        (16.8, 0.180_935_180_858_221_6, 0.819_064_819_141_778_4),
        (18.9, 0.344_192_476_260_255_34, 0.655_807_523_739_744_7),
        (21.0, 0.529_025_636_132_290_3, 0.470_974_363_867_709_7),
        (23.1, 0.697_090_767_859_555_6, 0.302_909_232_140_444_4),
        (25.2, 0.824_684_121_821_466_4, 0.175_315_878_178_533_53),
        (27.3, 0.907_980_177_142_409_4, 0.092_019_822_857_590_56),
        (29.4, 0.955_820_171_179_228_9, 0.044_179_828_820_771_14),
    ];

    /// The same at shape 101.
    const NEAR_MODE_101: [(f64, f64, f64); 9] = [
        (60.6, 1.319_938_999_836_672_4e-6, 0.999_998_680_061_000_1),
        (70.7, 0.000_404_867_197_387_234_9, 0.999_595_132_802_612_8),
        (80.8, 0.016_651_615_142_057_444, 0.983_348_384_857_942_5),
        (90.9, 0.156_936_749_296_821_17, 0.843_063_250_703_178_8),
        (101.0, 0.513_232_791_918_545_1, 0.486_767_208_081_454_85),
        (111.1, 0.842_846_013_606_726_7, 0.157_153_986_393_273_27),
        (121.2, 0.972_720_774_514_629_1, 0.027_279_225_485_370_86),
        (131.3, 0.997_361_861_657_837_6, 0.002_638_138_342_162_398),
        (141.4, 0.999_849_507_692_163_2, 0.000_150_492_307_836_775_58),
    ];

    // Below shape 1E3 the band is the double-double pair (`near_mode.rs`, T-4000): the double kernel
    // was 1.5 ulp off at Q(101, 100) and 7.5 at Q(21, 25), where Excel is within 1.5 of the truth.
    #[test]
    fn near_mode_pair_is_within_an_ulp_from_shape_21() {
        assert_near_mode(21.0, &NEAR_MODE_21, 1.0);
        assert_near_mode(101.0, &NEAR_MODE_101, 1.0);
        // `POISSON.DIST(100,100,TRUE)`: Excel's 0.5265621985299984, the correctly rounded Q
        // (ironcalc-calc STATISTICAL/POISSON.xlsx D19).
        assert_eq!(gamma_ur(101.0, 100.0), 0.526_562_198_529_998_4);
        // `POISSON(20,25,TRUE)`: the correctly rounded Q, 1.5 ulp below Excel's 0.18549230269414177
        // (probes-numeric/t1048-poisson A9, B9), within half a unit of its 15th digit.
        assert_eq!(gamma_ur(21.0, 25.0), 0.185_492_302_694_141_71);
    }
}
