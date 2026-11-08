// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::float_cmp)]
// LINT-DEBT-END
#![allow(clippy::unwrap_used)]

use crate::test::util::{new_empty_model, round_trip_value};

#[test]
fn test_fn_f_test_sanity() {
    let mut model = new_empty_model();

    // Valid call
    model.set_a1("A1", "=F.TEST(A2:A7, B2:B7)");
    model.set_a1("A2", "9");
    model.set_a1("A3", "12");
    model.set_a1("A4", "14");
    model.set_a1("A5", "16");
    model.set_a1("A6", "18");
    model.set_a1("A7", "20");
    model.set_a1("B2", "11");
    model.set_a1("B3", "10");
    model.set_a1("B4", "15");
    model.set_a1("B5", "17");
    model.set_a1("B6", "19");
    model.set_a1("B7", "21");

    // Too few args
    model.set_a1("A8", "=F.TEST(A2:A7)");

    // Too many args
    model.set_a1("A9", "=F.TEST(A2:A7, B2:B7, C2:C7)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"0.859284302");
    assert_eq!(model.get_text_a1("A8"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A9"), *"#ERROR!");
}

// A single value is a sample of one (`#DIV/0!`) whatever its type, but a one-cell reference to a
// blank cell is `#VALUE!`; the samples are checked in order (probes/t2455-transpose-single-value
// A8:A79, T-2455).
#[test]
fn f_test_of_a_single_value() {
    let mut model = new_empty_model();
    model.set_a1("B1", "5");
    model.set_a1("A1", "=FTEST(TRANSPOSE(5),{1;2;3})");
    model.set_a1("A2", r#"=FTEST("abc",{1;2;3})"#);
    model.set_a1("A3", "=FTEST({1;2;3},B1)");
    model.set_a1("A4", "=FTEST(B10,{1;2;3})");
    model.set_a1("A5", "=FTEST(1/0,B10:B10)");
    model.set_a1("A7", "=FTEST(5,B10)");
    model.set_a1("A6", "=FTEST(B10:B11,{1;2;3})");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#DIV/0!");
    assert_eq!(model.get_text_a1("A2"), "#DIV/0!");
    assert_eq!(model.get_text_a1("A3"), "#DIV/0!");
    assert_eq!(model.get_text_a1("A4"), "#VALUE!");
    assert_eq!(model.get_text_a1("A5"), "#DIV/0!");
    assert_eq!(model.get_text_a1("A7"), "#VALUE!");
    assert_eq!(model.get_text_a1("A6"), "#DIV/0!");
}

// F.TEST divides the larger variance by the smaller and so is exactly symmetric in its samples
// (probes/ftest-decomposition A7 = E7); its tail carries exp(a ln x)'s rounding but an accurate
// remainder: within 15 digits of Excel's 0.7947858780819458 (probes/stat-argument-modes H250) and
// of the deep tail 7.189542390448293E-56 (gen-m9/06b4e8f5de409d5e A317), which the exact p misses
// visibly (docs/excel-semantics/statistics.md, T-2361).
#[test]
fn f_test_is_symmetric_and_keeps_the_rounded_powers() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=FTEST({1,2,4,3,7,5,8,6},{2,3,5,4,9,6,9,8})");
    model.set_a1("A2", "=FTEST({2,3,5,4,9,6,9,8},{1,2,4,3,7,5,8,6})");
    model.set_a1(
        "B1",
        "=FTEST({123456789.123,1E+15,1.005,0.1,2,7,1.005,1,1,123456789.123,1E-15},{0.1,2,7,7,1.005})",
    );
    // An overflowing variance is `#NUM!` (gen-m9/7674c89c824aca43 A39), and so, in either order,
    // is a ratio that overflows (unprobed; T-2571).
    model.set_a1("C1", "=FTEST({1E-160,2E-160},{1E+150,2E+150})");
    model.set_a1("C2", "=FTEST({1E+150,2E+150},{1E-160,2E-160})");
    model.set_a1("C3", "=F.TEST({1E+200,1E+200,-1E+200},{1,2,4})");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), "#NUM!");
    assert_eq!(model.get_text_a1("C2"), "#NUM!");
    assert_eq!(model.get_text_a1("C3"), "#NUM!");
    let number = |cell: &str| round_trip_value(&model, cell).parse::<f64>().unwrap();
    assert_eq!(number("A1").to_bits(), number("A2").to_bits());
    // Both shapes 3.5: bratio's L140 (f_test_tail.rs, T-2681) is Excel's value bit for bit
    // (probes/ftest-decomposition A7), where the remainder model was 2 ulp off.
    assert_eq!(number("A1"), 0.794_785_878_081_945_8);
    assert!((number("B1") / 7.189_542_390_448_293e-56 - 1.0).abs() < 5e-16);
}

#[test]
fn f_test_an_error_element_is_that_error() {
    // gen-m9/165db58d1540f014 A337, 9cca17438a7e6f8a A43 (excel-graph; T-2692): the element's
    // error, not #VALUE!, and before the one-point #DIV/0!.
    let mut model = new_empty_model();
    model.set_a1("A1", "=F.TEST({-1,0,1},{1,#N/A,3})");
    model.set_a1("A2", "=FTEST({1,#N/A,3},AVERAGE(0.2))");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"#N/A");
    assert_eq!(model.get_text_a1("A2"), *"#N/A");
}

// One degree of freedom on each side is Excel's closed form 2 asin(sqrt x)/pi, bit for bit where
// the series is 14 ulps off (gen-m9/8f2ac25945af87da A241 and gen-m9/c468f5046aadfbbc A192,
// excel-graph; T-2571).
#[test]
fn f_test_of_one_degree_of_freedom_each_is_the_asin_form() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=F.TEST({0.02,0.027},{1.005,1E+15})");
    model.set_a1("A2", "=F.TEST({7,0.3},{-0.5,123456789.123})");
    model.evaluate();
    let number = |cell: &str| round_trip_value(&model, cell).parse::<f64>().unwrap();
    assert_eq!(
        number("A1").to_bits(),
        8.912_676_813_146_147e-18_f64.to_bits()
    );
    assert_eq!(
        number("A2").to_bits(),
        6.909_871_037_288_267e-8_f64.to_bits()
    );
}
// Both shapes above 1 and one above 8: the balanced gamma-ratio ln B with bpser's split EXP
// finish (T-4975, addendum section D) is Excel's value bit for bit at
// probes/t2829-ftest-ln-beta-grid A4 and A5, each off before.
#[test]
fn f_test_balanced_ln_beta_with_the_split_finish() {
    let mut model = new_empty_model();
    let second = "1,-0.0001,123456789.123,0,1,0.3,7,100,0,1.005,1e-15,-0.0001,3.14159,1.005,3.14159,0.5,1,0.2,0.1,3.14159,100";
    model.set_a1(
        "A1",
        &format!("=F.TEST({{1.005,0.5,1,1,0.2,0.1}},{{{second}}})"),
    );
    model.set_a1(
        "A2",
        &format!("=F.TEST({{1.005,0.5,1,1,0.2,0.1}},{{{second},1}})"),
    );
    model.evaluate();
    let number = |cell: &str| round_trip_value(&model, cell).parse::<f64>().unwrap();
    assert_eq!(number("A1"), 6.812_701_809_757_666e-39);
    assert_eq!(number("A2"), 7.592_664_900_262_27e-39);
}
