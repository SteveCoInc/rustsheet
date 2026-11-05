//! The gamma density, on the prefactor P and Q share ([`gamma_prefactor`]), and Excel's own form
//! of it above the mode below shape 20 (T-3147).

use super::super::loader::stirlerr_exact;
use super::{
    excel_ratio_exponent, gamma_prefactor, stirling_prefactor, Exponent, DIRECT_SHAPE_MAX,
    NEAR_MODE, RATIO_FORM_MIN,
};
use crate::x87_exp::{dd_div, two_sum};
use crate::x87_log10::ln_double_double_pair;

/// The smallest shape below [`DIRECT_SHAPE_MAX`] at which the density above the mode takes Excel's
/// ratio form ([`density_with`]). A fit inside its evidence bracket (2.5, 3] (T-3147; ulp from
/// Excel, the ratio form against the direct one): at 3 `CHISQ.DIST(37.5,6,FALSE)` is -1 against -6
/// (gen-m9/984cba67bbe6972b A42), at 5 `CHISQ.DIST(40,10,FALSE)` 0 against -8 (probes-numeric/
/// m9-chisq-dist A50); at 2.5 `CHISQ.DIST(50,5,FALSE)` is 23 against 5 (probes-numeric/
/// t1049-chisqdist A46).
/// FITTED(T-3147, 3 cells, shape 2.5..5, y above the mode)
const SMALL_RATIO_SHAPE_MIN: f64 = 3.0;

/// The gamma density x^(a-1) e^(-x/scale) / (scale^a Gamma(a)) for a > 0, x > 0 and scale > 0,
/// as [`gamma_prefactor`]`(a, x / scale) / x`: the prefactor P and Q share, so the density keeps
/// its digits where statrs' `exp(ln pdf)` loses the absolute error of its log (T-0971). `None` at
/// x = 0 or where x / scale is not normal, which keep statrs' closed forms.
pub(crate) fn gamma_density(a: f64, x: f64, scale: f64) -> Option<f64> {
    density_with(a, x, scale, Exponent::Excel)
}

/// [`gamma_density`] with the accurate exponent throughout, for the large-shape beta paths, where
/// the ratio form is unprobed (T-3252).
pub(crate) fn accurate_gamma_density(a: f64, x: f64, scale: f64) -> Option<f64> {
    density_with(a, x, scale, Exponent::Accurate)
}

fn density_with(a: f64, x: f64, scale: f64, form: Exponent) -> Option<f64> {
    let y = x / scale;
    if x <= 0.0 || !y.is_normal() {
        return None;
    }
    // The exponential density e^(-x/scale) / scale, as statrs forms it: one rounding fewer than
    // the prefactor's y e^-y / x (gen-m9/ecc87f80077959e0 A261 `CHISQ.DIST(1.5,2,FALSE)`).
    if a == 1.0 {
        return Some((-y).exp() / scale);
    }
    // Below shape 20, above the mode, Excel's density takes its large-shape ratio form too, with
    // the exact Stirling error (the asymptotic series of `stirling_correction` is 2E-13 off at a
    // shape of 5): `CHISQ.DIST(40,10,FALSE)` is Excel's 6.870512074795199E-6, 7 ulp above the
    // true value, where the direct form is 8 ulp below Excel (T-3147). Its cdf does not follow:
    // the same prefactor moves `CHIDIST(38.5,6)` from Excel's value to 8 ulp off, so P and Q keep
    // the direct form (T-3219). Below the mode the direct form is as close or closer (the 1-ulp
    // `GAMMA.DIST(10,9,2,FALSE)` of ironcalc-calc GAMMA.xlsx E3 at 0.56a).
    if form == Exponent::Excel
        && (SMALL_RATIO_SHAPE_MIN..DIRECT_SHAPE_MAX).contains(&a)
        // Above the mode only; the 1.4a bound is unbracketed below 20 (UNVERIFIED, T-3219).
        && y - a > a * NEAR_MODE
    {
        return Some(stirling_prefactor(a, excel_ratio_exponent(a, y), stirlerr_exact(a).0) / x);
    }
    if form == Exponent::Excel && a >= DIRECT_SHAPE_MAX {
        // Excel's large-shape density is one chain, Loader's dgamma: [`loader_density`] on bd0's
        // series inside the window and its direct deviance outside it (T-4308, T-4319). Below
        // a/200 Excel is neither (T-3219), so the prefactor's `rcomp` stays there.
        let deviance = if in_window(a, y) {
            Some(window_exponent(a, y))
        } else if y >= a * RATIO_FORM_MIN {
            Some(direct_deviance(a, y))
        } else {
            None
        };
        if let Some(density) = deviance.and_then(|b| loader_density(a, x, b)) {
            return Some(density);
        }
    }
    Some(gamma_prefactor(a, y, form) / x)
}

/// Whether y lies in Excel's large-shape density window |y - a| <= 0.2 (y + a), 2a/3 <= y <= 3a/2,
/// tested in doubles as written. Both ends are inclusive and fall where Excel's do on the ladders
/// across them: at shape 3072, y = 2048 = 2a/3 (and 2a/3 + 1.9E-9) is the window and every y below
/// it the direct deviance (probes/t4290-gamma-window-large-pow2 E1:E13); at shape 2048, y = 3072 =
/// 3a/2 is the window and 3072 + 2.8E-9 the direct deviance (F1:F13); at shape 100 the two give
/// the same double at y = 1.5a (probes/t4255-gamma-window-exact-ratio D58; T-4308).
fn in_window(a: f64, y: f64) -> bool {
    (y - a).abs() <= 0.2 * (y + a)
}

/// Excel's large-shape density exp(-c(a) - B) / sqrt(2 pi a) times (a / x) for the deviance
/// B = a ln(a/y) + y - a: one exponential of the summed exponent, the Stirling error c taken exact,
/// and the shape over the raw argument last (T-4296). On Excel's own B ([`window_exponent`] inside
/// the window, [`direct_deviance`] outside it) it is Excel's bit for bit at 1,333 of 1,351 probed
/// window densities away from the mode, all 138 at it, 156 of 160 above the window and 119 of 155
/// below it, where the prefactor's sqrt(a / 2 pi) exp(-B - c) / x is at 107, 72, 43 and 65
/// (docs/research/t4308/score.py part 4, excel-graph 2026-11-05).
///
/// `None` where 2 pi a overflows (a above about 2.9E307): the chain would give 0 there, where the
/// density is about 1E-154, so the caller keeps the prefactor's sqrt(a / 2 pi) form (unprobed).
fn loader_density(a: f64, x: f64, deviance: f64) -> Option<f64> {
    let two_pi_a = 2.0 * std::f64::consts::PI * a;
    if !two_pi_a.is_finite() {
        return None;
    }
    let exponent = -deviance - stirlerr_exact(a).0;
    Some(exponent.exp() / two_pi_a.sqrt() * (a / x))
}

/// Loader's direct deviance a ln(a/y) + (y - a) outside the window: the log of a/y (below 1 above
/// the mode), and y - a taken before the sum. Above the window it is Excel's bit for bit at all
/// 120 points of T-4302 (probes/t4302-poisson-cdf-beside-pmf rows 7:12 and 31:36, probes-numeric/
/// t4302-poisson-cdf-r4-fresh rows 7:12), where the ratio form a ln(y/a) + (a - y) is not; below
/// the mode the two are the same double. Excel takes the ratio prefactor at x/a = 1.5078125
/// instead (probes/t4290-gamma-window-large-pow2 A223:C223; decision 0290, T-4339).
fn direct_deviance(a: f64, y: f64) -> f64 {
    a * (a / y).ln() + (y - a)
}

/// Excel's window exponent B = a ln(a/y) + y - a, as Loader's bd0 splits it into a leading term and
/// an odd series, B = (y - a) v - (2a atanh(v) - 2a v) with v = (y - a)/(y + a), every operation a
/// double rounding as written and atanh(v) correctly rounded ([`atanh_rounded`]). Read against
/// Excel's B as an exact double, it is Excel's at 551 of 551 fit cells (probes/
/// t4290-gamma-window-large-pow2 A:C) and 193 of 195 hold-out cells (D), the split and margin
/// fixed before scoring, where the correctly rounded B is at 74 and 26 (T-4308). Rejected: the libm
/// atanh (427 and 152, one ulp of atanh off), Loader's series summed term by term (80 and 27),
/// (y - a) v - 2a(atanh(v) - v) (the same at power-of-two shapes, 10 of 77 on the t3252 windows
/// against 77) and every form T-4255 and T-4290 tried (statistics.md).
fn window_exponent(a: f64, y: f64) -> f64 {
    let d = y - a;
    let v = d / (y + a);
    d * v - (2.0 * a * atanh_rounded(v) - 2.0 * a * v)
}

/// atanh(v) for |v| <= 0.2, rounded once from the double-double 0.5 ln((1 + v)/(1 - v)) (about
/// 2^-104 relative, so correctly rounded but within about 2^-50 ulp of a tie): Excel's window
/// exponent takes the correctly rounded value, where libm's is an ulp off at about a fifth of
/// the probed points (T-4308).
fn atanh_rounded(v: f64) -> f64 {
    let (hi, lo) = ln_double_double_pair(dd_div(two_sum(1.0, v), two_sum(1.0, -v)));
    0.5 * (hi + lo)
}

#[cfg(test)]
mod tests {
    use super::gamma_density;

    #[test]
    fn below_the_mode_the_ratio_form_starts_at_two_thirds_of_the_shape() {
        // Excel's GAMMA.DIST(x,a,1,FALSE) bit for bit on the ratio form below 0.666a, where `rcomp`
        // is 24 to 72 ulp off (probes/t3252-gamma-prefactor-window A3, A22, A23; T-3252).
        for (a, x, excel) in [
            (50.0, 32.5, 0.001_526_191_948_114_739_6),
            (200.0, 120.0, 1.111_402_810_556_919e-11),
            (200.0, 130.0, 4.174_320_755_401_682_5e-9),
        ] {
            assert_eq!(
                gamma_density(a, x, 1.0),
                Some(excel),
                "GAMMA.DIST({x},{a},1,FALSE)"
            );
        }
    }

    #[test]
    fn at_the_mode_the_density_is_excel_s_window_chain() {
        // exp(-c(a)) / sqrt(2 pi a) * (a / x) at x = a, Excel's bits where sqrt(a / 2 pi) exp(-c) / x
        // is 1 ulp off (probes/t4296-gamma-density-at-mode A32, C32, A36, B34; T-4296).
        for (x, a, scale, excel) in [
            (3000.0, 3000.0, 1.0, 0.007_283_453_882_974_526_5),
            (9000.0, 3000.0, 3.0, 0.002_427_817_960_991_508_7),
            (12345.5, 12345.5, 1.0, 0.003_590_482_318_690_912_5),
            (108.422, 108.422, 1.0, 0.038_284_022_269_937_386),
        ] {
            assert_eq!(
                gamma_density(a, x, scale),
                Some(excel),
                "GAMMA.DIST({x},{a},{scale},FALSE)"
            );
        }
    }

    #[test]
    fn inside_the_window_the_density_is_excel_s_bd0_split_exponent() {
        // Excel's bits on the window chain with B = (y - a) v - (2a atanh(v) - 2a v), where the
        // correctly rounded B is 5 to 1,135 ulp off (T-4308): probes/t3252-gamma-prefactor-window
        // A4, t4255-gamma-window-exact-ratio C5, both window ends (t4290-gamma-window-large-pow2
        // E7 at 2a/3 and F7 at 3a/2) and probes-numeric/t4296-gamma-density-r4-fresh A42, A43 and
        // A46 at scales other than 1.
        for (x, a, scale, excel) in [
            (35.0, 50.0, 1.0, 0.004_730_667_654_048_708_5),
            (32.25, 48.0, 1.0, 0.003_035_513_715_728_734_6),
            (2048.0, 3072.0, 1.0, 6.287_537_497_111_510_4e-99),
            (3072.0, 2048.0, 1.0, 4.858_310_149_119_557e-87),
            (
                4_620.000_077_580_01,
                57.240_712_107_224_9,
                71.931_451_611_773_7,
                0.000_439_643_676_330_607_65,
            ),
            (
                6_712.558_165_817_08,
                34_152.171_922_174_6,
                0.176_056_425_630_832,
                5.535_299_812_893_028e-96,
            ),
            (
                61.770_389_938_097_9,
                97.937_187_980_708_1,
                0.510_209_231_828_174,
                0.005_995_354_591_055_000_6,
            ),
        ] {
            assert_eq!(
                gamma_density(a, x, scale),
                Some(excel),
                "GAMMA.DIST({x},{a},{scale},FALSE)"
            );
        }
    }

    #[test]
    fn just_outside_the_window_the_density_takes_the_direct_deviance() {
        // The 15-digit 2047.99999999999, just below 2a/3 at shape 3072
        // (probes/t4290-gamma-window-large-pow2 E6): Excel's chain on the direct deviance, 879 ulp
        // from the window exponent, at a shape where the cdf's bound would still give the accurate
        // exponent (T-4308).
        assert_eq!(
            gamma_density(3072.0, 2_047.999_999_999_99, 1.0),
            Some(6.287_537_497_079_017e-99)
        );
    }

    #[test]
    fn a_shape_past_the_two_pi_overflow_keeps_a_nonzero_density() {
        // 2 pi a is infinite here, so the window chain would give 0 (T-4296 review).
        let density = gamma_density(1e308, 1e308, 1.0).unwrap_or(0.0);
        assert!(density > 0.0 && density.is_finite(), "{density}");
    }
}
