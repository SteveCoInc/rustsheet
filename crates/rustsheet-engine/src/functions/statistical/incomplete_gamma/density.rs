//! The gamma density, on the prefactor P and Q share ([`gamma_prefactor`]), and Excel's own form
//! of it above the mode below shape 20 (T-3147).

use super::super::loader::stirlerr_exact;
use super::{
    excel_away_from_mode, excel_ratio_exponent, gamma_prefactor, rlog, stirling_prefactor,
    Exponent, DIRECT_SHAPE_MAX, NEAR_MODE,
};

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
    if form == Exponent::Excel && a >= DIRECT_SHAPE_MAX && !excel_away_from_mode(a, y, form) {
        if let Some(density) = mode_density(a, x, y) {
            return Some(density);
        }
    }
    Some(gamma_prefactor(a, y, form) / x)
}

/// The large-shape density near the mode, in Excel's chain: exp(-c(a) - a rlog(y/a)) / sqrt(2 pi a)
/// times (a / x), one exponential of the summed exponent, the Stirling error c taken exact, and the
/// shape over the raw argument last (T-4296). At x = a (exponent 0) it is Excel's bit for bit at
/// 40 fit shapes from 20 to 1E7, the 40 hold-out shapes and the fit shapes at scale 3
/// (probes/t4296-gamma-density-at-mode A1:C40, excel-graph 2026-11-05), where the prefactor's
/// sqrt(a / 2 pi) exp(-c) / x is 0 to 1 ulp off at 18 of 40. Away from x = a the exponent still
/// differs from Excel's by about half an ulp of ln(x/a) per unit of shape (T-4290), so most
/// cells there stay a few ulp off; the chain is bit-exact at 288 of 1,281 probed window densities
/// against 162 for the prefactor's (docs/research/t4296/score.py).
///
/// `None` where 2 pi a overflows (a above about 2.9E307): the chain would give 0 there, where the
/// density is about 1E-154, so the caller keeps the prefactor's sqrt(a / 2 pi) form (unprobed).
fn mode_density(a: f64, x: f64, y: f64) -> Option<f64> {
    let two_pi_a = 2.0 * std::f64::consts::PI * a;
    if !two_pi_a.is_finite() {
        return None;
    }
    let exponent = -a * rlog(a, y) - stirlerr_exact(a).0;
    Some(exponent.exp() / two_pi_a.sqrt() * (a / x))
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
    fn a_shape_past_the_two_pi_overflow_keeps_a_nonzero_density() {
        // 2 pi a is infinite here, so the window chain would give 0 (T-4296 review).
        let density = gamma_density(1e308, 1e308, 1.0).unwrap_or(0.0);
        assert!(density > 0.0 && density.is_finite(), "{density}");
    }
}
