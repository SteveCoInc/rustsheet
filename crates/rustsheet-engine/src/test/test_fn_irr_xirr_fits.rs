#![allow(
    clippy::panic,
    reason = "a test fails by panicking on a non-number result"
)]

//! IRR and XIRR as fitted against the excel-graph solver traces (T-0916; fit lab `irr`, `xirr`;
//! docs/excel-semantics/financial.md "IRR is a secant in d = 1 - v" and "XIRR is a bracketing bisection").
//! Each value is the oracle's; the guess changes Excel's last digits, so each one pins the path.

use crate::cell::CellValue;
use crate::model::Model;
use crate::test::util::new_empty_model;

/// A model with `formulas` in A1, A2, ... of Sheet1, evaluated.
fn evaluated(formulas: &[&str]) -> Model {
    let mut model = new_empty_model();
    for (i, formula) in formulas.iter().enumerate() {
        model._set(&format!("A{}", i + 1), formula);
    }
    model.evaluate();
    model
}

fn results(formulas: &[&str]) -> Vec<u64> {
    let model = evaluated(formulas);
    (1..=formulas.len())
        .map(
            |row| match model.get_cell_value_by_ref(&format!("Sheet1!A{row}")) {
                Ok(CellValue::Number(n)) => n.to_bits(),
                other => panic!("A{row}: expected a number, got {other:?}"),
            },
        )
        .collect()
}

#[test]
fn irr_is_a_secant_in_the_discount_factor() {
    // probes/solver-iteration-traces I!A1, I!A10
    let got = results(&["=IRR({-20,5,5,5,5,5},0.05)", "=IRR({-20,5,5,5,5,5},0.0725)"]);
    let expected = [0.079_308_261_158_238_75_f64, 0.079_308_261_160_528_69];
    assert_eq!(got, expected.map(f64::to_bits));
}

#[test]
fn irr_far_roots_sit_on_the_discount_grid() {
    // probes/irr-fallback-roots P!A25, A28, A39 and t2565-irr-guess-sweeps P!A107 (a zero step
    // that stops on the residual relative to the largest flow); T-3120
    let got = results(&[
        "=IRR({-1,1E+4})",
        "=IRR({-1,3E+6})",
        "=IRR({-1,0,1E+14})",
        "=IRR({-2500000,-1000000,123456789123000})",
    ]);
    let expected = [
        9_999.000_000_001_1_f64,
        2_999_998.999_580_666,
        9_999_998.994_161_328,
        7_026.083_663_633_989,
    ];
    assert_eq!(got, expected.map(f64::to_bits));
}

#[test]
fn irr_drops_leading_zero_flows() {
    // probes/irr-guess-edges P!A33, probes-numeric/m9-fin-irr-rate Calc!A173, A175; T-3120
    let got = results(&[
        "=IRR({0,-100,110},0.05)",
        "=IRR({0,0,-100,110},-0.5)",
        "=IRR({0,0,-100,110},10)",
    ]);
    let expected = [
        0.099_999_999_999_999_87_f64,
        0.100_000_000_000_000_09,
        0.099_999_999_999_999_87,
    ];
    assert_eq!(got, expected.map(f64::to_bits));
}

#[test]
fn xirr_bisects_the_walked_bracket() {
    // probes/solver-iteration-traces X!A1, X!A10
    let got = results(&[
        "=XIRR({-1,1.013},{45000,45365},1)",
        "=XIRR({-1,1.3613},{45000,45365},1)",
    ]);
    let expected = [0.013_000_000_268_220_901_f64, 0.361_299_999_058_246_6];
    assert_eq!(got, expected.map(f64::to_bits));
}

#[test]
fn xirr_bounds_xnpv_on_a_negative_root_only() {
    // probes/xirr-back-dated-stop A1 (positive root: the first step-test pass, at
    // |XNPV| 2.55e-7·max|value|) and A4 (negative root: one step past it); T-3268
    let got = results(&[
        "=XIRR({-1000,1100,-1000,45000},{45000,45365,1100,45365})",
        "=XIRR({-1000,1100,-1000},{45000,45365,1100})",
    ]);
    let expected = [0.031_899_645_924_568_16_f64, -0.017_501_763_999_462_12];
    assert_eq!(got, expected.map(f64::to_bits));
}

#[test]
fn xirr_with_a_zero_first_value_needs_no_later_date() {
    // probes/xirr-span-zero-first A28 (first date after the others); T-2329
    let got = results(&["=XIRR({0,-1000,1100},{46000,45000,45365})"]);
    let expected = [2.980_232_238_769_531_4e-9_f64];
    assert_eq!(got, expected.map(f64::to_bits));
}

#[test]
fn xirr_with_a_zero_first_value_brackets_at_the_first_chain_point() {
    // probes/xirr-zero-first-bracket A1, A3, A8, A9, A10: the flows do not matter, the guess does
    let got = results(&[
        "=XIRR({0,-1000,2000},{45000,45000,45365})",
        "=XIRR({0,-1000,2000},{45000,45000,45365},0.3)",
        "=XIRR({0,-100,110},{45000,45000,45000})",
        "=XIRR({0,-100,110},{45000,45000,45000},0)",
        "=XIRR({0,-100,110},{45000,45000,45000},2)",
    ]);
    let expected = [
        2.980_232_238_769_531_4e-9_f64,
        4.470_348_358_154_297e-9,
        2.980_232_238_769_531_4e-9,
        4.882_812_5e-9,
        3.725_290_298_461_914e-9,
    ];
    assert_eq!(got, expected.map(f64::to_bits));
}

fn assert_num_errors(formulas: &[&str]) {
    let cases: Vec<(&str, &str)> = formulas.iter().map(|formula| (*formula, "#NUM!")).collect();
    assert_errors(&cases);
}

#[test]
fn irr_is_div_zero_where_its_secant_cannot_evaluate_npv() {
    // T-3407: a secant step whose NPV leaves the double range (a discount power that overflows)
    // ends the call with worksheet NPV's #DIV/0!, without the retry from 0.1.
    // probes/t3406-irr-num-boundary P!A69 (40 flows) and P!A84 (60 flows) are #DIV/0! in Excel;
    // P!A83, the same 60-flow annuity with a smaller outlay, stays #NUM! (its steps stay in range).
    assert_errors(&[
        ("=IRR({-6912433510.59,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1})", "#DIV/0!"),
        ("=IRR({-59016464.79,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1})", "#DIV/0!"),
        ("=IRR({-13548805.62,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1})", "#NUM!"),
    ]);
}

#[test]
fn irr_is_div_zero_on_a_flat_secant_after_a_far_step() {
    // T-3347: the secant from 0.1 jumps far toward r = -1 (|NPV| past 3E+226), comes back, takes
    // a zero step and goes flat (0/0): Excel is #DIV/0! (probes-numeric/
    // t3347-irr-conventional-holdout P!A138; probes/t3406-irr-num-boundary P!A66). P!A65, the
    // same 40-flow annuity whose run peaks at 1.6E+226, stays #NUM! in Excel and here.
    assert_errors(&[
        ("=IRR({-11890081760.62,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03,13.03})", "#DIV/0!"),
        ("=IRR({-157216647.9,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1})", "#DIV/0!"),
        ("=IRR({-48614255.03,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1})", "#NUM!"),
    ]);
}

#[test]
fn xirr_with_a_zero_first_value_and_a_negative_guess_is_num() {
    // probes/xirr-zero-first-bracket A6, A7, A11 (A6 has a root on the downward chain)
    assert_num_errors(&[
        "=XIRR({0,-1000,900},{45000,45000,45365},-0.2)",
        "=XIRR({0,-1000,1100},{45000,45000,45365},-0.5)",
        "=XIRR({0,-100,110},{45000,45000,45000},-0.5)",
    ]);
}

#[test]
fn xirr_with_a_nonzero_first_value_still_needs_a_later_date() {
    // probes/xirr-mismatched-empties X9
    assert_num_errors(&["=XIRR({-1000,1100},{45365,45000})"]);
}

/// Asserts each formula's number is within `relative` of Excel's value.
fn assert_numbers_near(cases: &[(&str, f64)], relative: f64) {
    let formulas: Vec<&str> = cases.iter().map(|case| case.0).collect();
    let model = evaluated(&formulas);
    for (row, (_, want)) in (1..).zip(cases) {
        let value = model.get_cell_value_by_ref(&format!("Sheet1!A{row}"));
        assert!(
            matches!(value, Ok(CellValue::Number(n)) if (n - want).abs() <= relative * want.abs()),
            "A{row}: {value:?}, want {want}"
        );
    }
}

#[test]
fn irr_is_num_at_a_far_root() {
    // probes/irr-fallback-roots A29, A39 and irr-guess-edges-2 A15, A31: 1 + r at or past 1E+7
    assert_num_errors(&["=IRR({-1,1E+7})", "=IRR({-1,1E+13})", "=IRR({-1E-15,1})"]);
    // A28, A39: 3E+6 and r = 9999998.994 still return (their digits are T-2131's)
    assert_numbers_near(
        &[
            ("=IRR({-1,3E+6})", 2_999_998.999_580_666),
            ("=IRR({-1,0,1E+14})", 9_999_998.994_161_328),
        ],
        1e-9,
    );
}

#[test]
fn irr_is_div0_where_the_secant_reaches_a_zero_discount_factor() {
    // probes/irr-guess-edges A39 (T-2802): from the guess 0.1 the secant's second step lands on
    // d = 1 exactly (v = 0), and Excel's 1/v is #DIV/0!.
    assert_errors(&[("=IRR({-1,1E+300})", "#DIV/0!")]);
    // A far root the secant stalls at, short of d = 1, stays #NUM! (irr-guess-edges-2 A19, A21).
    assert_num_errors(&["=IRR({-1,9E+14})", "=IRR({-1,2E+15})"]);
}

#[test]
fn irr_where_the_secant_fails_is_num_at_a_root_near_minus_one() {
    // probes/irr-fallback-roots A1, A5, A8, A10: the NPV polynomial's root nearest v = 0 is negative
    assert_num_errors(&[
        "=IRR({1,7,-1})",
        "=IRR({1,7,-1},1)",
        "=IRR({1,20,-1})",
        "=IRR({1,3,-1})",
    ]);
}

#[test]
fn irr_where_the_secant_fails_finds_the_root_nearest_v_zero() {
    // probes/irr-fallback-roots A12, A15, A16, A22: within 1E-14 relative of Excel's
    assert_numbers_near(
        &[
            ("=IRR({42,1,1,-1},0.5)", -0.746_704_476_653_737_1),
            ("=IRR({42,1,1,-1},0)", -0.746_704_476_653_737_1),
            ("=IRR({0.3,-2.5,1.005},0.01)", 6.909_808_543_634_611),
            ("=IRR({-400,1,1,1,1,1},0.5)", -0.673_736_675_180_840_5),
        ],
        1e-14,
    );
}

#[test]
fn irr_last_attempt_keeps_the_former_residual_cut_cells() {
    // T-3411 round 4, now through the d = -0.1 attempt (T-4975 audit 3): where every secant attempt
    // fails Excel is #NUM!; these cells pinned the removed Newton fallback's residual cut, |NPV|
    // one step past the stop is 1E-4 or more. probes-numeric/t3478-irr-r4-wide A90 (1.22E-4 there)
    // is #NUM!; A79 and A292 return although at the stop iterate itself their residuals are 8.8E-4
    // and 1.3E-4 (their last digits are T-3408's, so only the type is pinned here).
    // probes-numeric/t3411-irr-residual-scale A5 and A6: t3411-irr-r4-fresh A37 returns, and the same
    // flows times 10 are #NUM! in Excel, so the cut is on the absolute residual.
    assert_num_errors(&[
        "=IRR({7839274018045.82,-1800700392574.26,-1932978417380.95,-628102382859.986,-771459031037.155,-1833829809207.7,-190607277255.53})",
        "=IRR({3538681482050.93,-959839357236.378,-695426025690.099,-1342006103306.3,-220806582262.62,-330645052475.425},0.00162399739616426)",
    ]);
    assert_numbers_near(
        &[
            (
                "=IRR({-20348062620.1183,4488889137.16682,2126791179.52687,322827138.632301,314093761.795836,3018827684.70375,1928928663.74205,818467822.296312,481991283.63873,3290162008.36438,1171706288.69735,1301641841.0064,2072846658.16491,366670370.47232,2006947896.47083,1595280758.06325,1440048477.42067},-0.0340669405751921)",
                0.038_101_815_872_697_12,
            ),
            (
                "=IRR({145997625287.114,-23243466498.3165,-24930632662.067,-13155199863.4897,-11546268184.9697,-22070569337.2782,-27101092776.3016,-20207638082.3591,-6392695211.92509},666.601763607955)",
                0.004_302_898_815_059_297,
            ),
        ],
        1e-6,
    );
}

#[test]
fn xirr_at_an_exact_root_of_zero_brackets_at_the_first_chain_point() {
    // probes/irr-guess-edges A41-A47 (T-2728): the flows sum to 0, so XNPV(0) is exactly 0 and
    // the first point of the upward chain brackets: the result is the guess halved down to 0
    let got = results(&[
        "=XIRR({-100,10,20,30,40},{0,1,59,60,61})",
        "=XIRR({-100,10,20,30,40},{0,1,59,60,61},1000)",
        "=XIRR({-100,10,20,30,40},{0,1,59,60,61},123456789.123)",
        "=XIRR({-100,10,20,30,40},{0,1,59,60,61},1E+10)",
        // probes/t2728-irr-guess-ref-mark A73-A81: a negative guess halves toward 0 from below
        "=XIRR({-100,10,20,30,40},{0,1,59,60,61},-0.5)",
        "=XIRR({-100,10,20,30,40},{0,1,59,60,61},0)",
        "=XIRR({-100,10,20,30,40},{0,1,59,60,61},1E+20)",
        "=XIRR({100,-10,-20,-30,-40},{0,1,59,60,61})",
        "=XIRR({-100,10,20,30,40},{0,1,59,60,61},1E+15)",
    ]);
    let expected = [
        2.980_232_238_769_531_4e-9_f64,
        3.637_978_807_091_713e-9,
        3.426_614_245_766_179_6e-9,
        4.336_808_689_942_018e-9,
        -3.725_290_298_461_914e-9,
        4.882_812_5e-9,
        2.524_354_896_707_237_8e-9,
        2.980_232_238_769_531_4e-9,
        3.308_722_450_212_110_7e-9,
    ];
    assert_eq!(got, expected.map(f64::to_bits));
    assert_num_errors(&["=XIRR({-100,10,20,30,40},{0,1,59,60,61},1E+300)"]);
}

/// Asserts each formula's error.
fn assert_errors(cases: &[(&str, &str)]) {
    let formulas: Vec<&str> = cases.iter().map(|case| case.0).collect();
    let model = evaluated(&formulas);
    for (row, (formula, want)) in (1..).zip(cases) {
        assert_eq!(
            model.get_cell_value_by_ref(&format!("Sheet1!A{row}")),
            Ok(CellValue::String((*want).to_string())),
            "A{row}: {formula}"
        );
    }
}

#[test]
fn irr_rejects_a_number_straight_from_cell_or_error_type() {
    // probes/irr-guess-edges-2, probes/t2728-irr-guess-ref-mark A51, A63 (T-2728)
    assert_errors(&[
        ("=IRR(CELL(\"row\",D2))", "#VALUE!"),
        ("=IRR(CELL(\"col\",D2),0.5)", "#VALUE!"),
        ("=IRR(ERROR.TYPE(#REF!))", "#VALUE!"),
        ("=IRR(CELL(\"row\",D2)+0)", "#NUM!"),
        ("=IRR(@CELL(\"row\",D2))", "#NUM!"),
        ("=IRR(ROW(D2))", "#NUM!"),
    ]);
}

#[test]
fn irr_rejects_a_marked_number_through_its_wrappers() {
    // probes/t2728-irr-guess-ref-mark A53, A55, A57, A59, A61; probes/irr-guess-edges-2 A37 (T-2766)
    assert_errors(&[
        ("=IRR(IF(TRUE,CELL(\"row\",D2)))", "#VALUE!"),
        ("=IRR(CHOOSE(1,CELL(\"row\",D2)))", "#VALUE!"),
        ("=IRR(INDEX(CELL(\"row\",D2),1))", "#VALUE!"),
        ("=IRR(IFERROR(CELL(\"row\",D2),0))", "#VALUE!"),
        ("=IRR(LAMBDA(x,x)(CELL(\"row\",D2)))", "#VALUE!"),
        ("=IRR(LET(x,CELL(\"row\",D2),x))", "#VALUE!"),
        ("=IRR(LET(x,CELL(\"row\",D2),x*1))", "#NUM!"),
    ]);
}

#[test]
fn irr_reads_the_guess_between_a_flows_error_and_an_element_error() {
    // probes/t2728-irr-guess-ref-mark A1, A39, A41 and -2 A1, A3, A5 (T-2728)
    assert_errors(&[
        ("=IRR(1/0,NA())", "#DIV/0!"),
        ("=IRR(1/0,1E+300)", "#DIV/0!"),
        ("=IRR({1,\"a\"}+0,1/0)", "#DIV/0!"),
        ("=IRR({1,\"a\"}+0,0.1)", "#VALUE!"),
        ("=IRR(CELL(\"row\",D2),1/0)", "#DIV/0!"),
        ("=IRR(ERROR.TYPE(#REF!),NA())", "#N/A"),
        ("=IRR({1,\"a\"}+0,INDEX(B3:C9,2))", "#REF!"),
    ]);
}

#[test]
fn xnpv_discounts_with_excels_caret_at_an_exact_root() {
    // probes/xirr-exact-root-sign: a few ulps from 0, with Excel's sign (financial.md "XNPV
    // discounts with Excel's `^`").
    let got = results(&["=XNPV(0.1,{-1,6.727499949325611},{45000,52300})"]);
    assert_eq!(got, [2.220_446_049_250_313e-16_f64.to_bits()]);
}

#[test]
fn xnpv_keeps_a_flow_whose_power_overflows_as_zero() {
    // Unprobed: unchanged from the `powf` days (value/inf = 0), not Excel evidence.
    let got = results(&["=XNPV(1E+300,{-100,100},{39448,40000})"]);
    assert_eq!(got, [(-100.0_f64).to_bits()]);
}

#[test]
fn irr_secant_runs_up_to_200_steps_on_the_evidenced_flows() {
    // T-3408: Excel returns a secant run's own digits after more than 50 steps.
    // probes-numeric/t3411-irr-r4-fresh A53 stops at its 90th step, t3411-irr-r4-fresh-3 A53 at its
    // 200th; with a 50-step cap both fell to the retry from 0.1, 9 and 11 digits from Excel.
    // t3411-irr-r4-fresh-4 A70's run from its guess would stop at its 208th step: Excel returns the
    // retry's digits there, so the run is cut before it.
    let got = results(&[
        "=IRR({-5170.93099296084,288.865145373819,570.218662187279,565.457499144911,610.517931952135,231.187511687411,527.73694495784,523.592480523921,396.954768103593,30.931160753107,302.758564145744,296.839518608567,335.470634439476,485.006931605991},-0.991141596755925)",
        "=IRR({-5419.4566980255,541.157630544974,453.161192371308,501.313367619903,179.560456356541,219.651507558475,748.165593608538,475.983383783222,453.532531891375,415.276123775752,204.050971572924,587.376200777869,269.367533912702,676.494160107977,468.392546259015,221.821037515441,174.308048353656,341.477410565068,688.641976177886,685.57480555384,743.851200755666,473.330527831559,552.574820085196},-0.998251375322924)",
        "=IRR({-1.41053204859511,0.0734811591985414,0.0876809772190229,0.0412299828542401,0.093324708773603,0.0773116289158734,0.0218076354237776,0.00731120902409104,0.0648079334193454,0.0443983631322261,0.082530036288092,0.0765035879197722,0.0682983617375568,0.0596183385481602,0.0365402052878083,0.0759080563699283,0.037306440759054,0.09764840318161,0.00640354609700719,0.0675777796671609,0.0317312445994466,0.0138573845664404,0.0526333999410834,0.0957794340181701,0.0263417663149393,0.0575019039921351,0.0193894786196443},-0.996502177548583)",
    ]);
    let expected = [
        -0.000_160_231_940_793_065_24_f64,
        0.059_134_610_468_991_6,
        0.000_363_049_032_130_691_2,
    ];
    assert_eq!(got, expected.map(f64::to_bits));
}

#[test]
fn irr_checks_the_discount_domain_at_convergence() {
    // T-4975 item 11: an intermediate iterate past v = 0 (d > 1) no longer fails the run; only the
    // discount it converges to must be positive. probes/t2565-irr-guess-sweeps P!C157 (guess
    // 2697.5242098806502) and gen-m9/a91bad4e84f49739 Calc!A369 (`IRR(Data!C5:E5)`), both
    // visibly off before.
    let got = results(&[
        "=IRR({2.5,1,-123456789.123},2697.5242098806502)",
        "=IRR({0.3,-2.5,1.005})",
    ]);
    let expected = [7_026.083_663_321_499_f64, 6.909_808_543_634_389];
    assert_eq!(got, expected.map(f64::to_bits));
    // probes/irr-guess-edges P!A28 and A29: the numbers of D!A1:D12 in row order, from the
    // default guess and from 0.5, are #DIV/0! in Excel (#NUM! before).
    assert_errors(&[
        (
            "=IRR({0,1.005,1E+15,2,42,1E+15,-0.0001,2,-0.5,1E-15,-1,1E+15,-1,0,0.3,123456789.123,1.005,0,-1,0,1E-15,0.3,1})",
            "#DIV/0!",
        ),
        (
            "=IRR({0,1.005,1E+15,2,42,1E+15,-0.0001,2,-0.5,1E-15,-1,1E+15,-1,0,0.3,123456789.123,1.005,0,-1,0,1E-15,0.3,1},0.5)",
            "#DIV/0!",
        ),
    ]);
}

#[test]
fn irr_secant_cap_is_the_same_for_every_call() {
    // T-4975 audit 3: probes-numeric/t3478-irr-r4-wide Calc!A206, whose largest flow (4.7E+8) was
    // outside the long cap's training box, so its run stopped at 50 steps and was #NUM!.
    let got = results(&[
        "=IRR({-466383321.854056,3602903.77027555,11883179.3894947,10170065.7109031,6380517.86458303,1450414.65489934,12053622.5684458,9761577.23690692,4723702.18738595,13954374.168184,1948909.77982285,1521967.75462305,1903606.31559473,6219742.45447484,5021479.79737767,11247062.8492215,12565660.4212703,975394.321243717,12643033.4354266,11881291.9671628,14396710.0095129,3754495.40967869,5823106.16523961,10920824.4648419,15739809.7660733,6240870.52051412,6828064.55051699,5715666.44535026,10074110.7706943,808257.810304779,14265901.8345943,6905324.43894212,13005472.6427097},16396220.611551)",
    ]);
    assert_eq!(got, [(-0.031_819_409_296_013_46_f64).to_bits()]);
    // A trailing zero flow leaves NPV unchanged, and so the root: it used to move the call out of
    // the box (54 to 55 flows) and turn it #NUM!.
    let got = results(&[
        "=IRR(VSTACK(-50,SEQUENCE(53,1,1,0)),-0.9)",
        "=IRR(VSTACK(-50,SEQUENCE(53,1,1,0),0),-0.9)",
    ]);
    assert_eq!(got[0], got[1]);
}

#[test]
fn irr_last_attempt_starts_at_d_minus_a_tenth() {
    // T-4975 audit 3 (addendum 3, section H): where the guess's and the default guess's attempts
    // fail, the attempt from d = -0.1 finds Excel's digits (probes/irr-fallback-roots P!A12 and
    // P!A22; ironcalc-calc IRR.xlsx General!L17 at update 125), which the Newton fallback missed by
    // 37 and 17 ulps; its signed start offset gives the negated flows their own digits
    // (probes-numeric/t2927-irr-fallback-sweep A80).
    let got = results(&[
        "=IRR({42,1,1,-1})",
        "=IRR({42,1,1,-1},0.5)",
        "=IRR({-42,-1,-1,1})",
        "=IRR({-400,1,1,1,1,1})",
        "=IRR({-400,1,1,1,1,1},0.01)",
    ]);
    let expected = [
        -0.746_704_476_653_737_1_f64,
        -0.746_704_476_653_737_1,
        -0.746_704_476_653_740_7,
        -0.673_736_675_180_840_5,
        -0.673_736_675_180_840_5,
    ];
    assert_eq!(got, expected.map(f64::to_bits));
}

#[test]
fn irr_runs_two_hundred_updates_for_every_input() {
    // Addendum 3, section H: gen-m9/31474c20fc1dafd6 Calc!A173 (`IRR(Data!C1:E7)`, a 1E+15 flow,
    // Excel after update 51) and ba4ddb817a8e700e Calc!A373 (`IRR(Data!C1:E11,Data!A8*Data!D6)`,
    // update 68), the numbers of their ranges in row order; probes-numeric/
    // t3347-irr-conventional-holdout P!A25 (returned at update 200 under the 1e-4 test) and P!A118
    // (the twice-rounded 1 - d); t3347-irr-r4-fresh A52 (a root the error classifier pre-empted).
    let got = results(&[
        "=IRR({-0.5,42,1.005,123456789.123,-0.5,0.3,1E+15,0.3,42,1,0.3,1,0.5,-1})",
        "=IRR({-2.5,100,42,0.1,-1,0.2,0.3,123456789.123,-2.5,2,7,2.5,-1,0.5,2.5,1E-15,1E+15,100,123456789.123,1.005},-0.5)",
        "=IRR({-3059548204.29,521255991.15,945998930.48,996766465.97,595003477.29})",
        "=IRR({-4644417054.85,1623802700.37,1743057871.25,1275367027.61})",
        "=IRR({5269092526.48884,-118527.360339241,-17182.2159040934,-88912.399751738,-35803.7486524801,-68708.7537575748,-107014.260861857,-21219.4170843404,-58236.3903095811,-75317.1992186433,-73521.1582838103,-78264.948216601,-95601.7612564427,-20004.8379456876,-59614.8055900756,-74062.3546290729,-97283.3914088112,-56129.0503653067,-58145.8255470416,-48409.3532170003,-108090.158585088,-45997.9318202708,-82924.7421268438,-101200.667800715,-41772.6685472847,-105413.239889478,-98553.098519149,-68569.2483676826,-71791.9560824583,-42447.9076549358,-36765.1911876846,-83375.1900307704,-94787.7252673916,-40639.5229742157,-80558.5784522469,-27717.2344386984,-105699.720411514,-105617.163696477,-70476.7499653863,-83168.7929195298,-27640.5447601861,-91856.4988985311,-24406.4093294585,-89850.9757171176,-117269.72585201,-82518.7550353443,-34096.4975927237,-108777.801358874,-28523.4697883074,-110926.061182294,-53382.79917888,-53307.0761355066,-79383.8284492618,-55281.6599406716,-47144.5323918044,-113927.080482745,-29801.1090065886,-109438.395331364,-39439.0385560273,-12302.2295282068,-54378.50934114,-70093.3876903242,-65168.6387331095,-16411.9046245493,-107064.04835217,-108617.700161449,-76046.5078982186,-20292.8797531886,-24074.7263918106,-94623.8331158653,-12844.4873074928,-12965.3916524786,-66770.9335877795,-118521.93093073,-96674.1135094589,-71059.4908587591,-7154.82148693787,-21139.4469608767,-105361.705483859,-30816.8471759608,-64283.4027977171,-57448.2712778636,-59790.9597607717,-65830.6080500559,-54058.2006948844,-90658.4735698973,-77739.4062014091,-50702.6344094987})",
    ]);
    let expected = [
        661.407_639_121_985_5_f64,
        39.426_895_723_824_34,
        -6.722_756_777_233_219e-5,
        -0.000_244_917_238_009_900_73,
        -0.098_593_648_815_817_34,
    ];
    assert_eq!(got, expected.map(f64::to_bits));
}
