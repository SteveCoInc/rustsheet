#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// A squared difference past the double range is #NUM!, not a p of 0 (probes/t4098-tests-prob-domains
// A44, A45; statistics.md "CHISQ.TEST, Z.TEST and PROB: Gate A domains (T-4098)").
#[test]
fn test_chisq_test_overflowing_square_is_num() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=CHISQ.TEST({1E+200,1,1},{1,1,1})");
    model.set_a1("A2", "=CHISQ.TEST({1E+160,1,1},{1,1,1})");
    model.set_a1("A3", "=CHITEST({1E+160,1,1},{1,1,1})");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#NUM!");
    assert_eq!(model.get_text_a1("A2"), *"#NUM!");
    assert_eq!(model.get_text_a1("A3"), *"#NUM!");
}

#[test]
fn test_fn_chisq_test_smoke() {
    let mut model = new_empty_model();
    model.set_a1("A2", "48");
    model.set_a1("A3", "32");
    model.set_a1("A4", "12");
    model.set_a1("A5", "1");
    model.set_a1("A6", "'13");
    model.set_a1("A7", "TRUE");
    model.set_a1("A8", "1");
    model.set_a1("A9", "13");
    model.set_a1("A10", "15");

    model.set_a1("B2", "55");
    model.set_a1("B3", "34");
    model.set_a1("B4", "13");
    model.set_a1("B5", "blah");
    model.set_a1("B6", "13");
    model.set_a1("B7", "1");
    model.set_a1("B8", "TRUE");
    model.set_a1("B9", "'14");
    model.set_a1("B10", "16");

    model.set_a1("C1", "=CHISQ.TEST(A2:A10, B2:B10)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), *"0.997129538");
}

#[test]
fn arrays() {
    let mut model = new_empty_model();
    model.set_a1("A2", "TRUE");
    model.set_a1("A3", "4");
    model.set_a1("A4", "'3");
    model.set_a1("B2", "2");
    model.set_a1("B3", "2");
    model.set_a1("B4", "2");
    model.set_a1("C1", "=CHISQ.TEST(A2:A4, B2:B4)");

    model.set_a1("G5", "=CHISQ.TEST({TRUE,4,\"3\"}, {2,2,2})");

    // 1D arrays with different shapes
    model.set_a1("G6", "=CHISQ.TEST({1,2,3}, {3;3;4})");

    // 2D array
    model.set_a1("G7", "=CHISQ.TEST({1,2;3,4},{2,3;2,2})");

    // 1D arrays with same shape
    model.set_a1("G8", "=CHISQ.TEST({1,2,3,4}, {2,3,4,5})");

    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), *"0.367879441");
    assert_eq!(model.get_text_a1("G5"), *"0.367879441");

    assert_eq!(model.get_text_a1("G6"), *"0.383531573");

    assert_eq!(model.get_text_a1("G7"), *"0.067889155");

    assert_eq!(model.get_text_a1("G8"), *"0.733094495");
}

#[test]
fn more_arrays() {
    let mut model = new_empty_model();
    model.set_a1("V20", "2");
    model.set_a1("V21", "4");
    model.set_a1("W20", "3");
    model.set_a1("W21", "5");
    model.set_a1("C1", "=CHISQ.TEST({1,2;3,4},V20:W21)");
    model.set_a1("C2", "=CHISQ.TEST({1,2;3,4}, {2,3;4,5})");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), *"0.257280177");
    assert_eq!(model.get_text_a1("C2"), *"0.257280177");
}

#[test]
fn array_ranges() {
    let mut model = new_empty_model();
    model.set_a1("A2", "TRUE");
    model.set_a1("A3", "4");
    model.set_a1("A4", "'3");
    model.set_a1("B2", "2");
    model.set_a1("B3", "2");
    model.set_a1("B4", "2");
    model.set_a1("C1", "=CHISQ.TEST(A2:A4, {2;2;2})");

    model.set_a1("G5", "=CHISQ.TEST({TRUE;4;\"3\"}, B2:B4)");
    model.evaluate();

    assert_eq!(model.get_text_a1("C1"), *"0.367879441");
    assert_eq!(model.get_text_a1("G5"), *"0.367879441");
}

#[test]
fn array_2d_ranges() {
    let mut model = new_empty_model();
    model.set_a1("A2", "2");
    model.set_a1("B2", "3");
    model.set_a1("C2", "4");
    model.set_a1("A3", "5");
    model.set_a1("B3", "6");
    model.set_a1("C3", "7");
    model.set_a1("G1", "=CHISQ.TEST({1,2,3;4,2,6}, A2:C3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("G1"), *"0.129195493");
}

#[test]
fn ranges_1d() {
    let mut model = new_empty_model();
    model.set_a1("A2", "1");
    model.set_a1("A3", "2");
    model.set_a1("A4", "3");
    model.set_a1("B2", "4");
    model.set_a1("C2", "5");
    model.set_a1("D2", "6");
    model.set_a1("G1", "=CHISQ.TEST(A2:A4, B2:D2)");
    model.set_a1("G2", "=CHISQ.TEST(B2:D2, A2:A4)");

    model.evaluate();
    assert_eq!(model.get_text_a1("G1"), *"0.062349477");
    assert_eq!(model.get_text_a1("G2"), *"0.000261259");
}

#[test]
fn arguments() {
    let mut model = new_empty_model();

    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("A3", "3");
    model.set_a1("B1", "4");
    model.set_a1("B2", "5");
    model.set_a1("B3", "6");

    model.set_a1("C1", "=CHISQ.TEST()");
    model.set_a1("C2", "=CHISQ.TEST(A1:B3)");
    model.set_a1("C3", "=CHISQ.TEST(A1:A3, B1:B3)");

    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("C2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("C3"), *"0.062349477");
}

// T-2431: a negative actual value is used as is; only a negative summed statistic is #NUM!.
#[test]
fn test_chisq_test_negative_actual_value() {
    let mut model = crate::test::util::new_empty_model();
    model.set_a1("A1", "=CHISQ.TEST({-1;1;1},{1,1,1})");
    model.set_a1("A2", "=CHISQ.TEST({1;1;1},{-1,1,1})");
    model.set_a1("A3", "=CHISQ.TEST({1;5;5},{-1,2,2})");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"0.135335283");
    assert_eq!(model.get_text_a1("A2"), *"#NUM!");
    // a negative expected cell with a positive total is a number (A314 is 0 for a huge statistic)
    assert_eq!(model.get_text_a1("A3"), *"0.082084999");
}

// T-2094: a side with no number at all is #DIV/0!; one text cell among numbers is unchanged.
#[test]
fn test_chisq_test_blank_range() {
    let mut model = crate::test::util::new_empty_model();
    for (cell, v) in [("B1", "1"), ("B2", "2"), ("B3", "3"), ("B4", "4")] {
        model.set_a1(cell, v);
    }
    model.set_a1("C1", "a");
    model.set_a1("C2", "2");
    model.set_a1("C3", "3");
    model.set_a1("C4", "4");
    model.set_a1("A1", "=CHISQ.TEST(B1:B4,Z1:Z4)");
    model.set_a1("A2", "=CHISQ.TEST(Z1:Z4,B1:B4)");
    model.set_a1("A3", "=CHITEST(Z1:Z4,B1:B4)");
    model.set_a1("A4", "=CHISQ.TEST(B1:B4,C1:C4)");
    model.set_a1("A5", "=CHISQ.TEST(5,5)");
    model.set_a1("A6", "=CHISQ.TEST(B2:B4,C2:C4)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("A2"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("A3"), *"#DIV/0!");
    // the pair with the text cell adds nothing to the statistic: same as the pair dropped, df from 4 points
    assert_eq!(model.get_text_a1("A4"), *"1");
    assert_eq!(model.get_text_a1("A6"), *"1");
    assert_eq!(model.get_text_a1("A5"), *"#N/A");
}

// The p-value is the direct right tail, not 1 - P: chi2 = 18 at df 2 is EXP(-9) to the last bit in
// Excel (probes-numeric/m9-legacy-chitest Calc!A23; T-3058).
#[test]
fn test_chitest_small_p_is_the_direct_tail() {
    let mut model = new_empty_model();
    for (cell, v) in [
        ("F1", "5"),
        ("F2", "0"),
        ("F3", "x"),
        ("G1", "1"),
        ("G2", "2"),
        ("G3", "3"),
    ] {
        model.set_a1(cell, v);
    }
    model.set_a1("A1", "=CHITEST(F1:F3,G1:G3)");
    model.evaluate();
    let crate::cell::CellValue::Number(p) = model.get_cell_value_by_ref("Sheet1!A1").unwrap()
    else {
        panic!("expected a number");
    };
    assert_eq!(p.to_bits(), (-9.0_f64).exp().to_bits());
}
