#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// ── WRAPCOLS ──────────────────────────────────────────────────────────────────

#[test]
fn test_wrapcols_basic() {
    let mut model = new_empty_model();
    // Column vector 1..6, wrap every 2 → 3 columns of 2 rows each
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("A3", "3");
    model.set_a1("A4", "4");
    model.set_a1("A5", "5");
    model.set_a1("A6", "6");
    model.set_a1("C1", "=WRAPCOLS(A1:A6,2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), "1");
    assert_eq!(model.get_text_a1("C2"), "2");
    assert_eq!(model.get_text_a1("D1"), "3");
    assert_eq!(model.get_text_a1("D2"), "4");
    assert_eq!(model.get_text_a1("E1"), "5");
    assert_eq!(model.get_text_a1("E2"), "6");
}

#[test]
fn test_wrapcols_with_pad() {
    let mut model = new_empty_model();
    // 5 elements, wrap=2 → 3 columns, last column needs 1 pad
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("A3", "3");
    model.set_a1("A4", "4");
    model.set_a1("A5", "5");
    model.set_a1("C1", "=WRAPCOLS(A1:A5,2,0)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), "1");
    assert_eq!(model.get_text_a1("C2"), "2");
    assert_eq!(model.get_text_a1("D1"), "3");
    assert_eq!(model.get_text_a1("D2"), "4");
    assert_eq!(model.get_text_a1("E1"), "5");
    assert_eq!(model.get_text_a1("E2"), "0");
}

// ── WRAPROWS ──────────────────────────────────────────────────────────────────

#[test]
fn test_wraprows_basic() {
    let mut model = new_empty_model();
    // Column vector 1..6, wrap every 3 → 2 rows of 3 cols each
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("A3", "3");
    model.set_a1("A4", "4");
    model.set_a1("A5", "5");
    model.set_a1("A6", "6");
    model.set_a1("C1", "=WRAPROWS(A1:A6,3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), "1");
    assert_eq!(model.get_text_a1("D1"), "2");
    assert_eq!(model.get_text_a1("E1"), "3");
    assert_eq!(model.get_text_a1("C2"), "4");
    assert_eq!(model.get_text_a1("D2"), "5");
    assert_eq!(model.get_text_a1("E2"), "6");
}

#[test]
fn test_wraprows_with_pad() {
    let mut model = new_empty_model();
    // 5 elements, wrap=3 → 2 rows, last row padded with 0
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("A3", "3");
    model.set_a1("A4", "4");
    model.set_a1("A5", "5");
    model.set_a1("C1", "=WRAPROWS(A1:A5,3,0)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), "1");
    assert_eq!(model.get_text_a1("D1"), "2");
    assert_eq!(model.get_text_a1("E1"), "3");
    assert_eq!(model.get_text_a1("C2"), "4");
    assert_eq!(model.get_text_a1("D2"), "5");
    assert_eq!(model.get_text_a1("E2"), "0");
}

#[test]
fn test_wraprows_2d_input_error() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("B1", "3");
    model.set_a1("B2", "4");
    model.set_a1("D1", "=WRAPROWS(A1:B2,2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("D1"), "#VALUE!");
}

/// A `wrap_count` of 2^31 or more is `#NUM!` rather than clamped to the vector's
/// length; one just below still wraps (probes/t2312-wrap-count-threshold).
#[test]
fn wrap_count_of_2_pow_31_is_num() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=COLUMNS(WRAPROWS(SEQUENCE(3),2^31))");
    model.set_a1("A2", "=ROWS(WRAPCOLS(SEQUENCE(3),1E+15))");
    model.set_a1("A3", "=COLUMNS(WRAPROWS(SEQUENCE(3),2^31-0.5))");
    model.set_a1("A4", "=ROWS(WRAPCOLS(SEQUENCE(3),2^20))");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#NUM!");
    assert_eq!(model.get_text_a1("A2"), "#NUM!");
    assert_eq!(model.get_text_a1("A3"), "3");
    assert_eq!(model.get_text_a1("A4"), "3");
}
