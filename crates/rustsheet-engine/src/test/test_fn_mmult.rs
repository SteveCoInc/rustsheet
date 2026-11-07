#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn mmult_2x2() {
    let mut model = new_empty_model();
    // A = [[1, 2],
    //      [3, 4]]
    model.set_a1("A1", "1");
    model.set_a1("B1", "2");
    model.set_a1("A2", "3");
    model.set_a1("B2", "4");
    // B = [[5, 6],
    //      [7, 8]]
    model.set_a1("D1", "5");
    model.set_a1("E1", "6");
    model.set_a1("D2", "7");
    model.set_a1("E2", "8");

    model.set_a1("G1", "=MMULT(A1:B2, D1:E2)");
    model.evaluate();
    // A*B = [[19, 22],
    //        [43, 50]]
    assert_eq!(model.get_text_a1("G1"), "19");
    assert_eq!(model.get_text_a1("H1"), "22");
    assert_eq!(model.get_text_a1("G2"), "43");
    assert_eq!(model.get_text_a1("H2"), "50");
}

#[test]
fn mmult_2x3_times_3x2() {
    let mut model = new_empty_model();
    // A = [[1, 2, 3],
    //      [4, 5, 6]]
    model.set_a1("A1", "1");
    model.set_a1("B1", "2");
    model.set_a1("C1", "3");
    model.set_a1("A2", "4");
    model.set_a1("B2", "5");
    model.set_a1("C2", "6");
    // B = [[7,  8],
    //      [9,  10],
    //      [11, 12]]
    model.set_a1("E1", "7");
    model.set_a1("F1", "8");
    model.set_a1("E2", "9");
    model.set_a1("F2", "10");
    model.set_a1("E3", "11");
    model.set_a1("F3", "12");

    model.set_a1("H1", "=MMULT(A1:C2, E1:F3)");
    model.evaluate();
    // A*B = [[58,  64],
    //        [139, 154]]
    assert_eq!(model.get_text_a1("H1"), "58");
    assert_eq!(model.get_text_a1("I1"), "64");
    assert_eq!(model.get_text_a1("H2"), "139");
    assert_eq!(model.get_text_a1("I2"), "154");
}

#[test]
fn mmult_dimension_mismatch() {
    let mut model = new_empty_model();
    // 1x2 times 3x1 -> mismatch
    model.set_a1("A1", "1");
    model.set_a1("B1", "2");
    model.set_a1("D1", "3");
    model.set_a1("D2", "4");
    model.set_a1("D3", "5");
    model.set_a1("F1", "=MMULT(A1:B1, D1:D3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("F1"), "#VALUE!");
}

#[test]
fn mmult_wrong_arg_count() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=MMULT(1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#ERROR!");
}

#[test]
fn mmult_non_numeric() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("B1", "\"x\"");
    model.set_a1("D1", "1");
    model.set_a1("D2", "2");
    model.set_a1("F1", "=MMULT(A1:B1, D1:D2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("F1"), "#VALUE!");
}

#[test]
fn mmult_empty_cell_in_input() {
    // An empty cell inside either matrix must yield #VALUE!
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    // B1 left empty
    model.set_a1("A2", "3");
    model.set_a1("B2", "4");
    model.set_a1("D1", "5");
    model.set_a1("E1", "6");
    model.set_a1("D2", "7");
    model.set_a1("E2", "8");
    model.set_a1("G1", "=MMULT(A1:B2, D1:E2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("G1"), "#VALUE!");
}

#[test]
fn mmult_boolean_in_input() {
    // Booleans are not coerced for MMULT — they yield #VALUE!.
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("B1", "TRUE");
    model.set_a1("A2", "3");
    model.set_a1("B2", "FALSE");
    model.set_a1("D1", "5");
    model.set_a1("E1", "6");
    model.set_a1("D2", "7");
    model.set_a1("E2", "8");
    model.set_a1("G1", "=MMULT(A1:B2, D1:E2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("G1"), "#VALUE!");
}

#[test]
fn mmult_omitted_arguments() {
    let mut model = new_empty_model();
    model.set_a1("C1", "1");
    model.set_a1("D1", "2");
    model.set_a1("C2", "3");
    model.set_a1("D2", "4");
    model.set_a1("A1", "=MMULT(,C1:D2)");
    model.set_a1("A2", "=MMULT(,)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#VALUE!");
    assert_eq!(model.get_text_a1("A2"), "#VALUE!");
}

#[test]
fn mmult_overflow_returns_num() {
    // Two 1x1 matrices whose product overflows f64.
    let mut model = new_empty_model();
    model.set_a1("A1", "1E300");
    model.set_a1("C1", "1E300");
    model.set_a1("E1", "=MMULT(A1, C1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("E1"), "#NUM!");
}

#[test]
fn mmult_propagates_input_error() {
    // An error in either argument must be propagated (not turned into #VALUE!).
    let mut model = new_empty_model();
    model.set_a1("A1", "=NA()");
    model.set_a1("B1", "2");
    model.set_a1("A2", "3");
    model.set_a1("B2", "4");
    model.set_a1("D1", "5");
    model.set_a1("E1", "6");
    model.set_a1("D2", "7");
    model.set_a1("E2", "8");
    model.set_a1("G1", "=MMULT(A1:B2, D1:E2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("G1"), "#N/A");
}
