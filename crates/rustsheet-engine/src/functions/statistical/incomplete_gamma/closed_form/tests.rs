#![allow(clippy::unwrap_used)]
use super::super::tests::within_ulps;
use super::super::{gamma_quantile, Tail};
use super::excel_newton_quantile;

/// T-2561: at a shape above 1 a tiny lower-tail quantile is Excel's leading-factor root, bit for bit
/// (gen-m9, excel-graph; docs/excel-semantics/statistics.md "Deep lower tail at a shape above 1").
#[test]
fn a_tiny_quantile_above_shape_one_is_the_leading_factor_root() {
    // (shape, scale, p, Excel): CHISQ.INV(1E-300,4), (1E-300,5), (1E-200,7), (1E-100,4),
    // (1E-200,9), (1E-100,6), GAMMAINV(1E-300,2,1925) and GAMMAINV(1E-100,1.500000000000001,0.3).
    for (shape, scale, p, excel) in [
        (2.0, 2.0, 1e-300, 2.828427124746146e-150),
        (2.5, 2.0, 1e-300, 3.233_407_780_583_180_5e-120),
        (3.5, 2.0, 1e-200, 2.90163405242589e-57),
        (2.0, 2.0, 1e-100, 2.828_427_124_746_177_2e-50),
        (4.5, 2.0, 1e-200, 1.732023266119695e-44),
        (3.0, 2.0, 1e-100, 1.686865330603492e-33),
        (2.0, 1925.0, 1e-300, 2.722_361_107_568_165_7e-147),
        (1.500_000_000_000_001, 0.3, 1e-100, 7.814095618017905e-68),
    ] {
        assert_eq!(
            gamma_quantile(shape, scale, p, Tail::Lower),
            excel,
            "shape {shape} p {p}"
        );
    }
    // The root is the steps' iterate 0, kept where no step comes nearer p (T-3403); at shape 1
    // and from shape 20 the steps do not apply.
    assert_eq!(excel_newton_quantile(1.0, 1e-300), None);
    assert_eq!(excel_newton_quantile(25.0, 1e-300), None);
}

/// Within half a unit of the 15th significant digit of `expected` (decision 0083).
fn within_15_digits(actual: f64, expected: f64) -> bool {
    let unit = 10f64.powf(expected.abs().log10().floor() - 14.0);
    (actual - expected).abs() <= unit / 2.0
}

#[test]
fn newton_steps_from_the_leading_factor_root_reach_excel_deep_lower_tail() {
    // Excel's values, gen-m9 (excel-graph); T-3311. The first step from the root.
    assert_eq!(
        gamma_quantile(5.0, 2.0, 1e-200, Tail::Lower),
        5.210342169394733e-40 // d8266bc15533702a A212 `CHISQ.INV(1E-200,10)`
    );
    assert_eq!(
        gamma_quantile(1.5, 2.0, 1.0 - 1e-16, Tail::Upper),
        5.5854857570344876e-11 // ecc87f80077959e0 A301 `CHISQ.INV.RT(1-1E-16,3)`
    );
    // One ulp from Excel, the cdf form's own residue (T-3242), within 15 digits:
    // 06b4e8f5de409d5e A128 `CHISQ.INV(1E-200,5)`.
    let x = gamma_quantile(2.5, 2.0, 1e-200, Tail::Lower);
    assert!(within_ulps(x, 3.233407780583136e-80, 3.0), "{x:e}");
    assert!(within_15_digits(x, 3.233407780583136e-80), "{x:e}");
    // Where the root is the answer the steps keep it: `CHISQ.INV(1E-100,6)` is twice the unit
    // quantile (T-3311 sheet W).
    assert_eq!(
        excel_newton_quantile(3.0, 1e-100).map(|x| x * 2.0),
        Some(1.686865330603492e-33)
    );
    // The second step: a5828d3925575713 A338 `GAMMAINV(1E-16,2.5,TRUE)`, and within 15 digits
    // a612e89321a6df9c A169 `CHISQ.INV(1E-20,6)`.
    assert_eq!(
        gamma_quantile(2.5, 1.0, 1e-16, Tail::Lower),
        6.436_215_297_435_573e-7
    );
    let x = gamma_quantile(3.0, 2.0, 1e-20, Tail::Lower);
    assert!(within_15_digits(x, 7.829_736_048_647_269e-7), "{x:e}");
    // The cut's cost (decision 0210): c18dbf21f11093f3 A107 `CHISQ.INV(POISSON.DIST(9,100,FALSE),8)`
    // is 1 ulp from Excel's 7.92099982566666E-8, within 15 digits (decision 0083's allowance).
    let x = gamma_quantile(4.0, 2.0, 1.025_153_212_086_874e-31, Tail::Lower);
    assert!(within_ulps(x, 7.920_999_825_666_66e-8, 1.0), "{x:e}");
    assert!(within_15_digits(x, 7.920_999_825_666_66e-8), "{x:e}");
    // T-3403: without T-3311's gates the steps reach Excel where the gated rule was 5 to 485 ulp
    // off: gen-m9/c2974c061384fdd9 A355 `CHISQ.INV(1E-20,10)`, and below p = 1E-200 and shape 1.5
    // probes-numeric/t3311-fresh-sweep-2 A43 `GAMMA.INV(7.37E-197,1.155,1)` and t3311-fresh-sweep
    // A68 `GAMMA.INV(1.33E-243,2.024,1)`.
    for (shape, scale, p, excel) in [
        (5.0, 2.0, 1e-20, 0.000_521_056_841_397_433_4_f64),
        (1.155, 1.0, 7.37e-197, 1.643_414_925_805_473_7e-170),
        (2.024, 1.0, 1.33e-243, 1.430_242_391_437_823_3e-120),
    ] {
        assert_eq!(
            gamma_quantile(shape, scale, p, Tail::Lower).to_bits(),
            excel.to_bits(),
            "GAMMA.INV({p:e},{shape},{scale})"
        );
    }
    // Above the measured probabilities the search applies (`GAMMA.INV(1E-8,1.5,1)`, T-3403), at
    // shape 1 and from shape 20 too.
    assert_eq!(excel_newton_quantile(1.5, 1e-8), None);
    assert_eq!(excel_newton_quantile(1.5, 1e-9), None);
    assert_eq!(excel_newton_quantile(1.0, 1e-30), None);
    assert_eq!(excel_newton_quantile(20.0, 1e-100), None);
    // Where the steps leave the form's domain before they settle, the search applies:
    // `CHISQ.INV(6.29E-19,39)`, whose root's cdf is 0.59 off p.
    assert_eq!(excel_newton_quantile(19.5, 6.29e-19), None);
}
