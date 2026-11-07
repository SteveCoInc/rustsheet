#![allow(clippy::unwrap_used)]
use super::super::{gamma_sf, poisson_upper};
use super::{excel_lower, excel_lower_density};
use statrs::distribution::Gamma;

/// Units in the last place between two positive doubles.
fn ulp_distance(a: f64, b: f64) -> u64 {
    a.to_bits().abs_diff(b.to_bits())
}

/// Excel's lower tail above shape 1 at x up to 1 is its leading-factor series form, at any shape
/// (T-3412): bit for bit at these cells (probes-numeric/m9-gamma-dist-a9 A1, fit-audit-t2638-
/// ratio-min S!A60, t3252-gamma-prefactor-window A86), where the pair was 13, 361 and 165 ulp
/// off. Above x = 1 and at shape 1 the pair keeps the value.
#[test]
fn lower_tail_above_shape_one_is_the_leading_factor_form() {
    for (a, x, excel) in [
        (9.0, 0.9, 4.765_843_251_833_135e-7_f64),
        (100.0, 0.45, 1.437_985_210_971_763_8e-193),
        (50.0, 0.15, 1.809_768_622_493_298_2e-106),
    ] {
        assert_eq!(
            excel_lower(a, x).unwrap().to_bits(),
            excel.to_bits(),
            "P({a}, {x})"
        );
    }
    assert_eq!(excel_lower(9.0, 1.5), None);
    assert_eq!(excel_lower(1.0, 0.5), None);
}

/// Below x = a + 1 at a shape from 1 to 20, Excel's upper tail is (0.5 - P) + 0.5 (T-3412):
/// CHISQ.DIST.RT(1,5) and (1.5,3) (probes-numeric/t1049-chisqdistrt A11, m9-chisq-dist-rt A12;
/// P from the leading-factor form), CHISQ.DIST.RT(9.5924,21) (t3269-fresh-sweep A126; P from the
/// series above x = 1) and POISSON(5,2.36035,TRUE) and (7,1.5,TRUE) (t3269-fresh-sweep A145,
/// gen-m9/d8f44d556674c2fd A343), each 1 ulp off with 1 - P.
#[test]
fn lower_region_upper_tail_takes_excel_s_complement() {
    for (x, df, excel) in [
        (1.0, 5.0, 0.962_565_773_247_296_3_f64),
        (1.5, 3.0, 0.682_270_330_336_212_5),
        (9.5924, 21.0, 0.983_734_419_275_696),
    ] {
        let dist = Gamma::new(df / 2.0, 0.5).unwrap();
        assert_eq!(
            gamma_sf(&dist, x).to_bits(),
            excel.to_bits(),
            "CHISQ.DIST.RT({x},{df})"
        );
    }
    for (k, mean, excel) in [
        (5.0, 2.36035, 0.966_663_198_794_583_3_f64),
        (7.0, 1.5, 0.999_830_434_271_130_4),
    ] {
        let q = poisson_upper(k + 1.0, mean);
        assert_eq!(q.to_bits(), excel.to_bits(), "POISSON({k},{mean},TRUE)");
    }
}

/// Excel's density above shape 1 at x/scale up to 1 is the cdf's leading factor times e^-y a/x
/// (T-3412): bit for bit at probes-numeric/fit-audit-t2638-ratio-min S!A9 `GAMMA.DIST(0.75,25,1,
/// FALSE)` and S!A14 `GAMMA.DIST(0.1782,33,1,FALSE)` (the ratio prefactor 118 and 135 ulp off), 2
/// ulp from gen-m9/984cba67bbe6972b A49 `CHISQ.DIST(1E-10,8,FALSE)` (the log form was visibly off),
/// and 0 where the factor is subnormal (probes-numeric/t3412-density-sweep A69).
#[test]
fn density_below_unit_ratio_is_the_leading_factor_times_a_over_x() {
    for (a, x, excel) in [
        (25.0, 0.75, 7.639_128_048_428_017e-28_f64),
        (33.0, 0.1782, 3.400_034_835_394_816e-60),
    ] {
        let ours = excel_lower_density(a, x, x).unwrap();
        assert_eq!(
            ours.to_bits(),
            excel.to_bits(),
            "GAMMA.DIST({x},{a},1,FALSE)"
        );
    }
    let chi = excel_lower_density(4.0, 1e-10, 5e-11).unwrap();
    assert!(
        ulp_distance(chi, 1.041_666_666_614_588_4e-32) <= 2,
        "{chi:e}"
    );
    let flushed = excel_lower_density(24.55, 4.57398e-12, 4.57398e-12 / 2.0);
    assert_eq!(flushed, Some(0.0));
    assert_eq!(excel_lower_density(1.0, 0.5, 0.5), None);
    assert_eq!(excel_lower_density(5.0, 3.0, 1.5), None);
}
