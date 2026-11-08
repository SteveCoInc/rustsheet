#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_covariance_smoke() {
    let mut model = new_empty_model();
    model.set_a1("A1", "3");
    model.set_a1("A2", "9");
    model.set_a1("A3", "2");
    model.set_a1("A4", "7");
    model.set_a1("A5", "4");
    model.set_a1("A6", "12");

    model.set_a1("B1", "5");
    model.set_a1("B2", "15");
    model.set_a1("B3", "6");
    model.set_a1("B4", "17");
    model.set_a1("B5", "8");
    model.set_a1("B6", "20");

    model.set_a1("C1", "=COVARIANCE.P(A1:A6, B1:B6)");
    model.set_a1("C2", "=COVARIANCE.S(A1:A6, B1:B6)");
    model.evaluate();

    assert_eq!(model.get_text_a1("C1"), *"19.19444444");
    assert_eq!(model.get_text_a1("C2"), *"23.03333333");
}

#[test]
fn arrays_mixed() {
    let mut model = new_empty_model();

    model.set_a1("A2", "2");
    model.set_a1("A3", "4");
    model.set_a1("A4", "6");
    model.set_a1("A5", "8");

    model.set_a1("B2", "1");
    model.set_a1("B3", "3");
    model.set_a1("B4", "5");
    model.set_a1("B5", "7");

    model.set_a1("C1", "=COVARIANCE.P(A2:A5, {1,3,5,7})");
    model.set_a1("C2", "=COVARIANCE.S(A2:A5, {1,3,5,7})");
    model.set_a1("C3", "=COVARIANCE.P(A2:A5, B2:B5)");
    model.set_a1("C4", "=COVARIANCE.S(A2:A5, B2:B5)");
    model.set_a1("C5", "=COVARIANCE.P({2,4,6,8}, B2:B5)");
    model.set_a1("C6", "=COVARIANCE.S({2,4,6,8}, B2:B5)");
    model.set_a1("C7", "=COVARIANCE.P({2,4,6,8}, {1,3,5,7})");
    model.set_a1("C8", "=COVARIANCE.S({2,4,6,8}, {1,3,5,7})");

    model.evaluate();

    assert_eq!(model.get_text_a1("C1"), *"5");
    assert_eq!(model.get_text_a1("C2"), *"6.666666667");
}

#[test]
fn arguments() {
    let mut model = new_empty_model();

    model.set_a1("A1", "=COVARIANCE.P()");
    model.set_a1("A2", "=COVARIANCE.S()");
    model.set_a1("A3", "=COVARIANCE.P(1)");
    model.set_a1("A4", "=COVARIANCE.S(1)");
    model.set_a1("A5", "=COVARIANCE.P({1}, {2})");
    model.set_a1("A6", "=COVARIANCE.S({1}, {2})");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A3"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A4"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A5"), *"0");
    assert_eq!(model.get_text_a1("A6"), *"#DIV/0!");
}

// A number or single cell is an array of one value (statistics.md "Array arguments given a
// scalar, a single cell or an error").
#[test]
fn paired_functions_take_scalars_as_one_value_arrays() {
    let mut model = new_empty_model();
    model.set_a1("A1", "0");
    model.set_a1("B1", "1");
    model.set_a1("B2", "2");
    model.set_a1("C1", "1");
    model.set_a1("C2", "2");
    model.set_a1("C3", "3");
    model.set_a1("D1", "=COVAR(0,A1)");
    model.set_a1("D2", "=COVAR(0,B1:B2)");
    model.set_a1("D3", "=SLOPE(1,2)");
    model.set_a1("D4", "=FORECAST.LINEAR(1,0.2,1)");
    model.set_a1("D5", "=FORECAST.LINEAR(1,0.2,B1:B2)");
    model.set_a1("D6", "=T.TEST(C1:C3,5,1,1)");
    model.set_a1("D7", "=T.TEST(A1,B1:B2,0.1,3)");
    model.set_a1("D8", "=RSQ(A1,B1:B2)");
    model.set_a1("E1", "x");
    model.set_a1("E2", "TRUE");
    model.set_a1("D9", "=CHISQ.TEST(E1,B1:B2)");
    model.set_a1("D10", "=CORREL(E2,B1:B2)");
    model.set_a1("D12", "=T.TEST(E1,A1:A2,0.1,3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("D9"), *"#N/A");
    assert_eq!(model.get_text_a1("D10"), *"#N/A");
    assert_eq!(model.get_text_a1("D12"), *"#NUM!");
    assert_eq!(model.get_text_a1("D1"), *"0");
    assert_eq!(model.get_text_a1("D2"), *"#N/A");
    assert_eq!(model.get_text_a1("D3"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("D4"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("D5"), *"#N/A");
    assert_eq!(model.get_text_a1("D6"), *"#N/A");
    assert_eq!(model.get_text_a1("D7"), *"#NUM!");
    assert_eq!(model.get_text_a1("D8"), *"#N/A");
}

// A size mismatch is #N/A before an error cell is read; with equal sizes the error wins
// (probes/ftest-decomposition G7:G12, T-1927).
#[test]
fn size_mismatch_beats_an_error_cell() {
    let mut model = new_empty_model();
    model.set_a1("H1", "1");
    model.set_a1("H2", "=1/0");
    model.set_a1("H3", "3");
    for (cell, value) in [("I1", "2"), ("I2", "4"), ("I3", "5"), ("I4", "7")] {
        model.set_a1(cell, value);
    }
    model.set_a1("A1", "=COVARIANCE.S(H1:H3,I1:I4)");
    model.set_a1("A2", "=COVARIANCE.P(I1:I4,H1:H3)");
    model.set_a1("A3", "=COVAR(H1:H3,I1:I4)");
    model.set_a1("A4", "=COVARIANCE.S(H1:H3,I1:I3)");
    model.set_a1("A5", "=T.TEST(H1:H3,I1:I4,1,1)");
    model.set_a1("A6", "=T.TEST(H1:H3,I1:I4,2,2)");
    model.set_a1("A7", "=F.TEST(H1:H3,I1:I4)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#N/A");
    assert_eq!(model.get_text_a1("A2"), "#N/A");
    assert_eq!(model.get_text_a1("A3"), "#N/A");
    assert_eq!(model.get_text_a1("A4"), "#DIV/0!");
    assert_eq!(model.get_text_a1("A5"), "#N/A");
    assert_eq!(model.get_text_a1("A6"), "#DIV/0!");
    assert_eq!(model.get_text_a1("A7"), "#DIV/0!");
}

/// A one-cell reference to an empty cell is `#VALUE!` in the paired functions, before the size
/// check and the other argument's error; one holding a number is one value (statistics.md,
/// T-2255, T-2426).
#[test]
fn test_paired_blank_one_cell_reference_is_value_error() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("A3", "3");
    model.set_a1("B1", "5");
    model.set_a1("A4", "=COVAR(A1:A3,Z1)");
    model.set_a1("A5", "=CORREL(Z1:Z1,{1,2,3})");
    model.set_a1("A6", "=F.TEST(Z1,A1:A3)");
    model.set_a1("A7", "=INTERCEPT(Z1,1/0)");
    model.set_a1("A8", "=COVAR(A1:A3,B1)");
    model.set_a1("A9", "=T.TEST(Z1,A1:A3,2,2)");
    model.set_a1("A10", "=CHISQ.TEST(A1:A3,Z1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A4"), "#VALUE!");
    assert_eq!(model.get_text_a1("A5"), "#VALUE!");
    assert_eq!(model.get_text_a1("A6"), "#VALUE!");
    assert_eq!(model.get_text_a1("A7"), "#VALUE!");
    assert_eq!(model.get_text_a1("A8"), "#N/A");
    assert_eq!(model.get_text_a1("A9"), "#VALUE!");
    assert_eq!(model.get_text_a1("A10"), "#VALUE!");
}

// T-2431: two arrays with the same cell count pair in reading order whatever their shapes.
#[test]
fn test_paired_functions_pair_by_cell_count() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=CORREL({1,2;3,4},{1;2;3;4})");
    model.set_a1("A2", "=INTERCEPT({1;2;3;4},{1,2;3,5})");
    model.set_a1("A3", "=CORREL({1,2;3,4},{1;2;3})");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"1");
    assert_eq!(model.get_text_a1("A2"), *"0.457142857");
    assert_eq!(model.get_text_a1("A3"), *"#N/A");
}

// A typed or computed boolean is one value, so the size check and the one-point rule apply, not
// `#VALUE!` (T-2575; probes/t2436/computed-scalar A35:A37; A3 is the same rule for a computed boolean,
// as gen-m9/899f9555e67da2d6 A369 and f34f66fb7f81e673 A127, no oracle of that exact formula).
#[test]
fn test_covariance_boolean_scalar_is_one_value() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=COVARIANCE.P({1,2,3},TRUE)");
    model.set_a1("A2", "=COVARIANCE.S(-2.5,FALSE)");
    model.set_a1("A3", "=COVARIANCE.S(1,IF(1,TRUE))");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#N/A");
    assert_eq!(model.get_text_a1("A2"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("A3"), *"#DIV/0!");
}
