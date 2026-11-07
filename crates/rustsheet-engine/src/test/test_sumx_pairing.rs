#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// Pairing rules: docs/excel-semantics/coercion.md "SUMX2MY2 / SUMX2PY2 / SUMXMY2 pairing".

#[test]
fn skips_pairs_with_a_non_number() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "text");
    model.set_a1("A3", "3");
    model.set_a1("B1", "2");
    model.set_a1("B2", "5");
    // B3 left blank
    model.set_a1("C1", "=SUMX2MY2(A1:A3,B1:B3)");
    model.set_a1("C2", "=SUMX2PY2(A1:A3,B1:B3)");
    model.set_a1("C3", "=SUMXMY2(A1:A3,B1:B3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), *"-3");
    assert_eq!(model.get_text_a1("C2"), *"5");
    assert_eq!(model.get_text_a1("C3"), *"1");
}

#[test]
fn no_numeric_pair_is_div_zero() {
    let mut model = new_empty_model();
    model.set_a1("A1", "text");
    model.set_a1("B1", "1");
    model.set_a1("C1", "=SUMX2MY2(A1:A2,B1:B2)");
    model.set_a1("C2", "=SUMXMY2(D1:D2,E1:E2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("C2"), *"#DIV/0!");
}

#[test]
fn different_counts_are_na_same_count_different_shape_pairs_row_major() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("B1", "2");
    model.set_a1("A2", "3");
    model.set_a1("B2", "4");
    model.set_a1("D1", "1");
    model.set_a1("D2", "1");
    model.set_a1("D3", "1");
    model.set_a1("D4", "1");
    model.set_a1("F1", "=SUMXMY2(A1:A2,D1:D3)");
    // (1,1) (2,1) (3,1) (4,1): 0 + 1 + 4 + 9
    model.set_a1("F2", "=SUMXMY2(A1:B2,D1:D4)");
    model.evaluate();
    assert_eq!(model.get_text_a1("F1"), *"#N/A");
    assert_eq!(model.get_text_a1("F2"), *"14");
}

#[test]
fn scalar_arguments() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=SUMXMY2(2,4)");
    model.set_a1("A2", "=SUMX2PY2(2,3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"4");
    assert_eq!(model.get_text_a1("A2"), *"13");
}

// Order of checks: probes/sumx-blank-text-error-order (excel-graph, T-0961).
#[test]
fn blank_one_cell_argument_and_errors_in_order() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "x");
    model.set_a1("C1", "5");
    model.set_a1("C2", "6");
    model.set_a1("C3", "7");
    model.set_a1("E1", "1");
    model.set_a1("E2", "=1/0");
    model.set_a1("E3", "2");
    // A4 left blank
    model.set_a1("F1", "=SUMXMY2(A4,NA())");
    model.set_a1("F2", "=SUMXMY2(NA(),A4)");
    model.set_a1("F3", "=SUMXMY2(A4:A4,C1:C2)");
    model.set_a1("F4", "=SUMXMY2(,1)");
    model.set_a1("F5", "=SUMXMY2(E1:E2,A4)");
    model.set_a1("F6", "=SUMXMY2(E2,C1:C3)");
    model.set_a1("F7", "=SUMXMY2(C1:C2,E1:E3)");
    model.set_a1("F8", "=SUMXMY2(5,1/{1,0})");
    model.set_a1("F9", "=SUMXMY2(E1:E3,C1:C3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("F1"), *"#VALUE!");
    assert_eq!(model.get_text_a1("F2"), *"#N/A");
    assert_eq!(model.get_text_a1("F3"), *"#VALUE!");
    assert_eq!(model.get_text_a1("F4"), *"#VALUE!");
    assert_eq!(model.get_text_a1("F5"), *"#VALUE!");
    assert_eq!(model.get_text_a1("F6"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("F7"), *"#N/A");
    assert_eq!(model.get_text_a1("F8"), *"#N/A");
    assert_eq!(model.get_text_a1("F9"), *"#DIV/0!");
}

#[test]
fn no_numeric_pair_is_zero_when_both_sides_hold_a_number() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "x");
    model.set_a1("F1", "=SUMXMY2(A1:A2,{\"a\",5})");
    model.set_a1("F2", "=SUMX2PY2({1,\"a\"},{\"b\",2})");
    model.set_a1("F3", "=SUMXMY2(A2,2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("F1"), *"0");
    assert_eq!(model.get_text_a1("F2"), *"0");
    assert_eq!(model.get_text_a1("F3"), *"#DIV/0!");
}
