// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![cfg_attr(test, expect(clippy::float_cmp, clippy::unreadable_literal))]
// LINT-DEBT-END
use super::{
    cauchy_two_tail_quantile, df_one_quantile, t_centre_quantile, t_quantile, t_upper_quantile,
};
use crate::test::util::within_fifteen_digits;
use statrs::distribution::StudentsT;

// Excel (probes-numeric t1531-t-tiny-p): the Student-t quantile is finite, never an error, for
// a tiny p, and statrs' own inverse_cdf panics there. Expected values are Excel's cached values.

fn dist(df: f64) -> StudentsT {
    StudentsT::new(0.0, 1.0, df).expect("valid df")
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual / expected - 1.0).abs() < 1e-11,
        "actual {actual:e}, expected {expected:e}"
    );
}

#[test]
fn tail_asymptotic_matches_excel() {
    // x = df / (df + t^2) is tiny: the closed-form branch.
    assert_close(
        t_quantile(&dist(1.0), 1.0, 1e-300),
        -3.183_098_861_837_907e299,
    );
    assert_close(t_quantile(&dist(1.0), 1.0, 1e-10), -3_183_098_861.837_906_4);
    assert_close(
        t_quantile(&dist(5.0), 5.0, 1e-300),
        -1.568_392_559_099_332e60,
    );
    assert_close(t_quantile(&dist(5.0), 5.0, 1e-50), -15_683_925_590.993_399);
    // The upper tail is the mirror image (T.INV.2T(1E-300,1) = 2 * 3.18E299).
    assert_close(
        t_upper_quantile(&dist(1.0), 1.0, 5e-301),
        6.366_197_723_675_814e299,
    );
    // CONFIDENCE.T(1E-300,1,2) = t / sqrt(2).
    assert_close(
        t_upper_quantile(&dist(1.0), 1.0, 5e-301) / 2.0_f64.sqrt(),
        4.501_581_580_785_53e299,
    );
}

#[test]
fn rounded_y_quantile_is_finite_and_ordered_on_a_sweep() {
    // Every df the worksheet accepts from 2 up, and p from the smallest to the largest double
    // below 1: a finite quantile of the right sign (bounded: at most about 81 tails per call,
    // the poor-seed case).
    for df in [2.0, 3.0, 30.0, 99_999.0, 1e5, 1e10] {
        for p in [
            1e-300,
            1e-100,
            1e-15,
            1e-5,
            0.01,
            0.3,
            0.7,
            0.99,
            1.0 - 1e-10,
            1.0 - 1e-16,
        ] {
            let t = t_quantile(&dist(df), df, p);
            assert!(t.is_finite(), "T.INV({p},{df}) = {t}");
            assert_eq!(t < 0.0, p < 0.5, "T.INV({p},{df}) = {t}");
        }
    }
}

#[test]
fn rounded_y_quantile_matches_excel() {
    // Excel's bits (excel-graph), each visibly off through statrs' search before T-2511.
    // gen-m9/17c110a53924beb8 A51 `T.INV(0.001,6)`.
    assert_eq!(t_quantile(&dist(6.0), 6.0, 0.001), -5.207_626_238_725_363);
    // gen-m9/775d053de498b8c7 A114 `T.INV(0.3,4)`.
    assert_eq!(t_quantile(&dist(4.0), 4.0, 0.3), -0.568_649_063_049_705_3);
    // gen-m9/11eb3ab4f4f8104c A32 `T.INV.2T(0.5,10)`.
    assert_eq!(
        t_upper_quantile(&dist(10.0), 10.0, 0.25),
        0.699_812_061_312_431_7
    );
    // gen-m9/2138b909d8f810ea A241 `TINV(0.001,10000000000.5)`: Excel is 7E-8 off the exact
    // quantile at this df, and the rounded y gives its digits.
    assert_eq!(
        t_upper_quantile(&dist(1e10), 1e10, 0.0005),
        3.290_526_968_632_448_3
    );
}

#[test]
fn bisection_branch_is_bounded() {
    // x is not tiny and q < 1E-8: bisection on the cdf (T.INV(1E-10,5), 1 ulp from Excel).
    assert_close(t_quantile(&dist(5.0), 5.0, 1e-10), -156.825_592_708_894_3);
    // x is not tiny and q < 1E-100: statrs panics here. Excel gives -79.06351062982456; ours is
    // wrong (the cdf of statrs saturates), so only the bound is pinned (TODO(T-1542)).
    let t = t_quantile(&dist(99999.0), 99999.0, 1e-300);
    assert!(t.is_finite() && t < -30.0 && t > -80.0, "{t}");
}

#[test]
fn centre_quantiles_keep_their_digits() {
    // T-2724: two-tailed quantiles with p near 1, against the true quantile at the double p
    // (mpmath, 40 digits, `I_w(1/2, df/2) = 1 - p`), within 1E-15 relative; statrs' 1 - y lost
    // most digits (`TINV(1-1E-10,5)` was 2.36E-8) and gave 0 at 1-1E-16.
    // df 1 is Excel's 1/TAN form, not the true quantile: `t_inv_of` takes it before this path
    // (`df_one_quantile_is_one_over_excel_tan`, T-2858).
    for (df, p, exact) in [
        (2.0, 0.999_999, 1.414_213_562_414_469e-6),
        (2.0, 1.0 - 1e-10, 1.414_213_679_385_65e-10),
        (2.0, 1.0 - 1e-16, 1.570_092_458_683_775_2e-16),
        (5.0, 0.999_999, 1.317_152_762_108_468_7e-6),
        (5.0, 1.0 - 1e-10, 1.317_152_871_051_844_3e-10),
        (5.0, 1.0 - 1e-16, 1.462_333_323_398_885_8e-16),
        (1000.0, 0.999_999, 1.253_627_505_003_303e-6),
        (1000.0, 1.0 - 1e-10, 1.253_627_608_692_530_5e-10),
        (1000.0, 1.0 - 1e-16, 1.391_806_120_317_668_7e-16),
        (1e6, 0.999_999, 1.253_314_450_680_441_7e-6),
        (1e6, 1.0 - 1e-10, 1.253_314_554_343_776_3e-10),
        (1e6, 1.0 - 1e-16, 1.391_458_560_200_48e-16),
    ] {
        let t = t_upper_quantile(&dist(df), df, p / 2.0);
        assert!(
            (t / exact - 1.0).abs() < 1e-15,
            "T.INV.2T({p},{df}) = {t:e}, exact {exact:e}"
        );
    }
    // Excel's values (gen-m9 b9a32ce9a1fd0d71 A58, 334f3dc9ca937f10 A128, probes-numeric
    // m9-t-inv-2t A117 and A130; T-3349: probes/t3349-t-inv-2t-near-one A1, decision 0188
    // item 3's input, once 1.49E-8, and A15, A29) to half a unit of the 15th digit.
    for (df, p, excel) in [
        (2.0, 0.999_999_999_989_522, 1.481_806_159_731_988e-11),
        (2.0, 0.999_999_999_999_999, 1.413_083_212_815_401_9e-15),
        (1000.0, 0.999_999_999_999, 1.253_599_772_570_125_8e-12),
        (5.0, 1.0 - 1e-10, 1.317_152_871_051_845_8e-10),
        (7.0, 0.999_999, 1.298_730_137_860_588_3e-6),
        (1000.0, 0.999_999_999_9, 1.253_627_608_692_531_2e-10),
        (1e6, 0.999_999_999_9, 1.253_314_554_343_774_5e-10),
    ] {
        let t = t_upper_quantile(&dist(df), df, p / 2.0);
        assert!(
            within_fifteen_digits(excel, t),
            "T.INV.2T({p},{df}) = {t:e}, Excel {excel:e}"
        );
    }
    // T.INV.2T's negative branch keeps the centre to c = p - 1 = 0.1, bit-exact at
    // `T.INV.2T(LN(3),7)` (gen-m9 c2684066d8df263e A64).
    assert_eq!(
        -t_centre_quantile(7.0, 3f64.ln() - 1.0),
        -0.128_473_231_950_879_78
    );
    // T.INV at p = 1/2 +- 1E-10: the same centre at c = |2p - 1|, odd in p.
    let exact = 2.634_305_742_103_688_5e-10;
    let above = t_quantile(&dist(5.0), 5.0, 0.5 + 1e-10);
    assert!((above / exact - 1.0).abs() < 1e-15, "{above:e}");
    assert_eq!(t_quantile(&dist(5.0), 5.0, 0.5 - 1e-10), -above);
    assert_eq!(t_quantile(&dist(5.0), 5.0, 0.5), 0.0);
}

#[test]
fn centre_quantile_rises_with_c_at_every_df() {
    // The search is bounded (`refine_increasing`); its result is finite and rises with c up to
    // the switch back to statrs.
    for df in [1.0, 2.0, 3.0, 30.0, 1e3, 99_999.0, 1e5, 1e8, 1e10] {
        let mut last = 0.0;
        for c in [1.2e-16, 1e-12, 1e-8, 1e-4, 1e-2, 0.05, 0.074] {
            let t = t_upper_quantile(&dist(df), df, (1.0 - c) / 2.0);
            assert!(t.is_finite() && t > last, "df {df} c {c}: {t}");
            last = t;
        }
    }
}

#[test]
fn no_panic_across_df_and_p() {
    for df in [2.0, 10.0, 100.0, 1e3, 1e4, 9e4] {
        for q in [1e-100, 1e-50, 1e-10, 1e-9, 1e-8, 1e-7, 1e-6, 1e-150, 1e-300] {
            let t = t_quantile(&dist(df), df, q);
            assert!(t.is_finite() && t < 0.0, "df {df} q {q}: {t}");
        }
    }
}

// T-2858: Excel's df = 1 doubles (eval cells TINV/T.INV/T.INV.2T), bit for bit.
#[test]
fn df_one_quantile_is_one_over_excel_tan() {
    // T.INV.2T(0.5,1) = 1 and T.INV.2T(1.5,1) = -1 (round points, double-rounded division)
    assert_eq!(df_one_quantile(0.5, true), 1.0);
    assert_eq!(df_one_quantile(1.5, true), -1.0);
    assert_eq!(df_one_quantile(1.0, true), 0.0);
    assert_eq!(df_one_quantile(0.9, false), 3.0776835371752544);
    assert_eq!(df_one_quantile(1e-16, false), -3183098861837907.0);
    assert_eq!(
        df_one_quantile(1.0 + f64::EPSILON, true),
        -5.053466325954936e-16
    );
    assert_eq!(cauchy_two_tail_quantile(0.5), 1.0);
    assert_eq!(-cauchy_two_tail_quantile(2.0 - 1.5), -1.0);
    // TINV(1-1E-10,1)
    assert_eq!(cauchy_two_tail_quantile(1.0 - 1e-10), 1.570797438519081e-10);
    // Near p = 1, ahead of T-2724's centre kernel (probes/t3349-t-inv-2t-near-one A3, A10):
    // Excel is the 1/TAN form, not the true quantile (1.5695408241038844E-15 at A10).
    assert_eq!(
        cauchy_two_tail_quantile(0.999_999_999_989_522),
        1.645_889_555_288_576_6e-11
    );
    assert_eq!(
        cauchy_two_tail_quantile(0.999_999_999_999_999),
        1.615_569_657_220_650_2e-15
    );
    // TINV(1+2^-52,1) = -t2(2 - p)
    assert_eq!(
        -cauchy_two_tail_quantile(2.0 - (1.0 + f64::EPSILON)),
        -5.053466325954936e-16
    );
    // T.INV(1E-16,1) = -t2(2p)
    assert_eq!(-cauchy_two_tail_quantile(2.0 * 1e-16), -3183098861837907.0);
    // T.INV(0.9,1)
    assert_eq!(
        cauchy_two_tail_quantile(2.0 * (1.0 - 0.9)),
        3.0776835371752544
    );
    // CONFIDENCE.T(0.05,1,2)
    assert_eq!(
        cauchy_two_tail_quantile(0.05) * 1.0 / 2.0_f64.sqrt(),
        8.984643532093761
    );
}
