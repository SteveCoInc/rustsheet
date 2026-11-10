use super::{f_beta_arguments, f_bup_sum_overflows, f_cdf, f_quantile, f_tail, fisher, fisher_inv};
use crate::test::util::{ulps, within_fifteen_digits};

// F.INV maps Excel's AS 109 stopping point back by the four-log ratio (T-4975, addendum C):
// probes-numeric/t1538-f-quantile-switchover A1, A8 and A14, each off before, bit for bit.
#[test]
fn f_quantile_is_the_mapped_beta_stopping_point() {
    for (df1, df2, p, right_tail, excel) in [
        (99_999.0, 5.0, 1e-12, false, 0.076_617_852_264_873_51_f64),
        (5.0, 99_999.0, 1.0 - 1e-12, false, 13.051_797_727_478_313),
        (5.0, 99_999.0, 1e-12, true, 13.051_788_459_730_34),
    ] {
        let ours = f_quantile(df1, df2, p, right_tail);
        assert_eq!(
            ours.to_bits(),
            excel.to_bits(),
            "F({df1},{df2}) at {p}: {ours}"
        );
    }
}

// Deep-tail F.INV (excel-graph): a stop at AS 109's 1E-300 accuracy floor is Excel's value
// (`FINV(1E-300,30,3)`, probes-numeric/t4857-f-classes A2242), and a search that runs out of
// steps short of the root is #NUM! (NaN here) while its neighbours converge
// (`F.INV(1E-300,11,3)`, probes/t4974-f-inv-tiny-tail-neighbourhood E7; E6 at df2 2).
#[test]
fn deep_tail_f_quantile_keeps_floor_stops_and_unconverged_num() {
    assert_eq!(
        f_quantile(30.0, 3.0, 1e-300, true),
        3.434_681_060_930_738e223
    );
    assert!(f_quantile(11.0, 3.0, 1e-300, false).is_nan());
    assert!(f_quantile(11.0, 3.0, 1e-250, false).is_nan());
    // Where the root underflows, the floor stop above it is Excel's (gen-m9 A64, A247, A175).
    for (df2, p, excel) in [
        (1.0, 1e-200, 2.705_579_217_114_786e-151),
        (9.0, 1e-300, 2.435_001_815_283_923_6e-150),
        (10.0, 1e-300, 2.705_554_866_974_867e-150),
    ] {
        let ours = f_quantile(1.0, df2, p, false);
        assert!(
            (ours / excel - 1.0).abs() < 1e-12,
            "F.INV({p},1,{df2}): {ours}"
        );
    }
    let e6 = f_quantile(11.0, 2.0, 1e-300, false);
    assert!((e6 / 5.178_247_033_519_641e-56 - 1.0).abs() < 1e-14, "{e6}");
}

// FISHER and FISHERINV round their quotient twice, like Excel's `/` (excel-graph; T-2437).
#[test]
fn fisher_rounds_its_quotient_twice() {
    // gen-m9/4ab2ba98d26c8a5e Calc!A395 `FISHER(Data!E4/Data!$D2)`, x = 1E-15/3.14159.
    assert_eq!(
        fisher(3.183_101_550_488_765_5e-16),
        2.220_446_049_250_312_6e-16
    );
    // probes/t2437-fit-separators A1, A4, A8: Excel's cancellation, not atanh.
    assert_eq!(
        fisher(1.491_723_674_723_48e-11),
        1.491_717_860_324_600_7e-11
    );
    assert_eq!(fisher(1.027_594_709_980_55e-16), 0.0);
    assert_eq!(
        fisher(-6.605_891_778_770_15e-16),
        -6.661_338_147_750_943e-16
    );
}

// A shape of 1 outside the closed forms forms w1 as (0.5 - w) + 0.5 (T-3007): y0^b0 is 3 ulp
// low at probes-numeric/t2220-f-dist-sweep-cdf A718 and 1 ulp low at A318.
#[test]
fn shape_one_other_tail_is_the_complement() {
    assert_eq!(f_cdf(20.0, 2.0, 5.0), 0.820_348_299_875_155_2);
    assert_eq!(f_cdf(4.0, 2.0, 5.0), 0.826_446_280_991_735_6);
}

// There w is -expm1(b0 ln1p(-x0)) with Excel's own expm1 and log1p (T-3170): libm's put
// gen-m9/146b46e74f2dac3b A329 `F.DIST.RT(10,6,2)` and abc2939ebd1e2365 A318 `FDIST(40,8,2)`
// 1 ulp low.
#[test]
fn shape_one_tail_takes_excels_expm1_and_log1p() {
    assert_eq!(f_tail(6.0, 2.0, 10.0, true), 0.093_686_012_554_127_09);
    assert_eq!(f_tail(8.0, 2.0, 40.0, true), 0.024_614_204_935_833_432);
}

// Excel's r >= 1 arguments (T-3567): the larger is fl(r/fl(1 + r)), not 1/(1 + 1/r).
#[test]
fn f_beta_arguments_keep_r_at_least_one() {
    let r = 298.0 / 15.586;
    assert_eq!(
        f_beta_arguments(1.0, 298.0, 15.586),
        (1.0 / (1.0 + r), r / (1.0 + r))
    );
    let r = 15.586 * 298.0 / 3.0;
    assert_eq!(
        f_beta_arguments(298.0, 3.0, 15.586),
        (r / (1.0 + r), 1.0 / (1.0 + r))
    );
    assert_eq!(f_beta_arguments(4.0, 7.0, 0.0), (0.0, 1.0));
}

// A small df1 with a huge df2 takes L140 bup+bgrat up to a0 = df2/2 = 1E7 (T-3754): the fallback
// kernel was 1.4E-10 off at probes-numeric t3354-f-dist-sweep S!C168 and 8.3E-11 at S!A168.
// Excel is correctly rounded there; ours is 1 ulp from it on each tail.
#[test]
fn huge_df2_small_df1_takes_l140_bup_bgrat() {
    let x = 1.090_808_853_953_79;
    let rt = f_tail(8.0, 5_567_623.0, x, true);
    let cdf = f_tail(8.0, 5_567_623.0, x, false);
    assert!(within_fifteen_digits(0.365_893_631_290_974_9, rt), "{rt}");
    assert!(within_fifteen_digits(0.634_106_368_709_025_1, cdf), "{cdf}");
    assert!(ulps(0.365_893_631_290_974_9, rt) <= 1, "{rt}");
    assert!(ulps(0.634_106_368_709_025_1, cdf) <= 1, "{cdf}");
}

// bratio's L140 bup sum overflows from df1 77 to 79 at df2 1E+10 (T-4044): Excel is `#NUM!` at
// probes-numeric t4017-f-huge-df2-above-mean-1e10 A209 `F.DIST.RT(1.5,79,1E+10)` and A223
// `F.DIST.RT(50,79,1E+10)`, and a number at df1 76 and 80 (t4030-f-dist-rt-df1-boundary).
#[test]
fn bup_sum_overflows_in_excels_df1_band() {
    assert!(f_bup_sum_overflows(79.0, 1E10, 1.5));
    assert!(f_bup_sum_overflows(79.0, 1E10, 50.0));
    assert!(!f_bup_sum_overflows(76.0, 1E10, 1.5));
    assert!(!f_bup_sum_overflows(80.0, 1E10, 1.5));
    assert!(!f_bup_sum_overflows(79.0, 1E9, 1.5));
}

// Excel's upper tail is exactly 0 once bup's prefactor in L140 leaves the normal range (T-4045):
// probes-numeric t4045-f-dist-rt-zero-cut B18 `F.DIST.RT(71.5,20,1E7)` is 4.320911961488639E-291
// and B19 `F.DIST.RT(71.6,20,1E7)` is 0; A17 `F.DIST.RT(47.6,30,1E8)` is a number, A18 0.
#[test]
fn upper_tail_is_zero_where_bups_prefactor_underflows() {
    assert!(f_tail(20.0, 1E7, 71.5, true) > 0.0);
    assert_eq!(f_tail(20.0, 1E7, 71.6, true), 0.0);
    assert!(f_tail(30.0, 1E8, 47.6, true) > 0.0);
    assert_eq!(f_tail(30.0, 1E8, 47.666_666_666_7, true), 0.0);
}

// A df of 1 takes bgrat at b = 1/2 on those arguments (T-3580): probes-numeric
// t3567-r5-fdf1-hold A1/B1 `F.DIST.RT(15.586,1,298)` and `F.DIST(15.586,1,298,TRUE)`.
#[test]
fn df_of_one_takes_bgrat_at_one_half() {
    assert_eq!(f_tail(1.0, 298.0, 15.586, true), 9.842_252_247_713_118e-5);
    assert_eq!(f_tail(1.0, 298.0, 15.586, false), 0.999_901_577_477_522_8);
}

#[test]
fn fisher_inv_rounds_its_quotient_twice() {
    // gen-m9/ee2497a5e683c5d9 Calc!A30 `FISHERINV(TAN(PI()))`, TAN(PI()) = -1.22514845490862E-16.
    assert_eq!(
        fisher_inv(-1.225_148_454_908_62e-16),
        -1.110_223_024_625_156_5e-16
    );
    // probes/t2437-fit-separators A23, A26.
    assert_eq!(
        fisher_inv(-8.011_504_958_898_52e-15),
        -7.993_605_777_301_19e-15
    );
    assert_eq!(
        fisher_inv(-6.552_011_261_371_12e-16),
        -6.661_338_147_750_943e-16
    );
    // FISHER_FISHERINV.xlsx C38: EXP(2y) overflows to 1.
    assert_eq!(fisher_inv(1000.0), 1.0);
}

/// F tails at a huge df2 that T-3754's L140 bup+bgrat port leaves to the fallback kernel: df1 of
/// 20 and more (past `MAX_BUP_TERMS`) and df2 above 2E7 (a0 above its 1E7 cap). The expansion's
/// b/a cut used to send these to Cephes' fraction, 1E-11 to 7E-10 off; T-4119 removed it. The
/// df1 = 3 rows were already accurate and guard that path.
/// References: docs/research/t4017/references.json, at the double arguments Excel sees. The
/// bound is T-4119's target, 2.5E-14 relative; the grid's worst point is 2.1E-14.
#[test]
fn f_tails_past_the_bup_cap_take_the_large_shape_expansion() {
    for (df1, df2, x, reference) in [
        (3.0, 1e7, 2.0, 0.11161029807296603),
        (3.0, 1e7, 5.0, 0.001816657939558678),
        (3.0, 3e7, 3.0, 0.029290902489543566),
        (40.0, 3e7, 2.0, 0.0001763055857205162),
        (60.0, 3e7, 10.0, 4.432803223201799e-90),
        (79.0, 3e7, 1.5, 0.002685050433156616),
    ] {
        let got = f_tail(df1, df2, x, true);
        assert!(
            (got / reference - 1.0).abs() < 2.5e-14,
            "F.DIST.RT({x},{df1},{df2}) = {got} against {reference}"
        );
    }
}

// Excel's F quantile at a huge df is its BETA.INV mapped to F (probes-numeric, excel-graph).
#[test]
fn huge_df_quantiles_go_through_excels_beta_inv() {
    // t1538-f-inv-huge-df A3: y = BETA.INV(0.5,5E9,2.5) near 1, bit for bit.
    assert_eq!(
        f_quantile(1e10, 5.0, 0.5, false).to_bits(),
        1.149_039_552_113_033_7_f64.to_bits()
    );
    // A48: F.INV.RT(0.975,1E10,5) = 1/F.INV(0.975,5,1E10), through the flipped root.
    let rt = f_quantile(1e10, 5.0, 0.975, true);
    assert!(
        (rt - 0.389_635_637_664_716_87).abs() <= 4.0 * f64::EPSILON * rt,
        "{rt}"
    );
    // t1538-f-quantile-switchover A9: the flipped root mapped as it is (2E-15 relative of Excel's;
    // through 1 - (1 - s) it would be 9E-8 off).
    let upper = f_quantile(100_000.0, 5.0, 1.0 - 1e-12, false);
    assert!(
        (upper - 97_570.089_441_084_84).abs() <= 2e-15 * upper,
        "{upper}"
    );
}

// T-1871: deep-tail quantiles within 1E-14 relative of Excel (Excel is the 50-digit math at the
// first three; they are 50, 59 and 12 ulp from Excel's bits). statrs' estimate at a tiny p can be
// no root (A168: 1.87E-14 against 1.7E-40), and the crossing of the Cephes tail was 8.4E-5
// (A154) and 1.8% (A155) off until `large_prefactor` took brcomp's far branch. The last two
// are 53 and 9 ulp off Excel's bits (Excel's own tiny-p arithmetic, TODO(T-4894)); before,
// 1E16 and 2.5E-12 relative.
#[test]
fn deep_tail_quantiles_are_within_fourteen_digits() {
    let cases = [
        // accuracy/F.INV/s1 A154, A168; accuracy/F.INV.RT/s1 A155.
        (
            4.566_228_541_406_27e-172,
            29.0,
            7719.0,
            false,
            6.547_980_469_317_685e-13,
        ),
        (
            1.041_227_060_375_08e-20,
            1.0,
            161.0,
            false,
            1.708_281_752_414_819_5e-40,
        ),
        (
            4.695_097_943_991_8e-162,
            3216.0,
            23.0,
            true,
            241_639_249_736_357.75,
        ),
        // gen-m9/c6cfda6bd10cae93 A114, gen-m9/3a25a03eb94fcfbd A44.
        (1e-16, 1.0, 2.0, false, 1.999_999_999_999_986e-32),
        (1e-20, 4.0, 6.0, false, 6.123_724_357_374_616e-11),
    ];
    for (p, df1, df2, right_tail, excel) in cases {
        let x = f_quantile(df1, df2, p, right_tail);
        assert!((x / excel - 1.0).abs() < 1e-14, "{p:e} {df1} {df2}: {x:e}");
    }
}

/// T-4435's regions, against 40-digit mpmath at the exact ratio (docs/research/t4434/mathref.py):
/// region 2 (df1 80 to 400, bfrac's calls at a huge df2) was 4.7E-8 to 8.8E-8 off by bfrac
/// (`FDIST(1.5,80,1E10)`, probes-numeric/t4045-f-dist-rt-zero-cut E3), region 1 (df1 21 to 79 at
/// df2 1E5 to 1E6, past L140's 9 bup terms) 2E-13 to 1.2E-11 off by Cephes' fraction.
#[test]
fn f_tails_at_a_huge_shape_outside_the_old_bounds_follow_the_math() {
    for (df1, df2, x, reference) in [
        (80.0, 1e10, 1.5, 0.0025481924241211686),
        (80.0, 1e10, 1.4, 0.010567980346381957),
        (200.0, 1e10, 1.5, 5.9245419296008984e-06),
        (24.0, 1e6, 5.0, 9.72630506010578e-15),
        (24.0, 1e6, 20.0, 2.4748557825776378e-86),
        (40.0, 1e6, 1.5, 0.021876418947230438),
        (24.0, 1e5, 5.0, 9.939500327850076e-15),
    ] {
        let got = f_tail(df1, df2, x, true);
        assert!(
            (got / reference - 1.0).abs() < 2e-15,
            "F.DIST.RT({x},{df1},{df2}) = {got} against {reference}"
        );
    }
    let lower = f_tail(80.0, 1e10, 1.5, false);
    assert!((lower / 0.9974518075758788 - 1.0).abs() < 2e-16, "{lower}");
}
