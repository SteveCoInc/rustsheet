#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// ── CHOOSECOLS ────────────────────────────────────────────────────────────────

#[test]
fn test_choosecols_basic() {
    let mut model = new_empty_model();
    // Build a 2×3 array in A1:C2
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("C1", "3");
    model._set("A2", "4");
    model._set("B2", "5");
    model._set("C2", "6");
    // Select columns 1 and 3
    model._set("E1", "=CHOOSECOLS(A1:C2,1,3)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "1");
    assert_eq!(model._get_text("F1"), "3");
    assert_eq!(model._get_text("E2"), "4");
    assert_eq!(model._get_text("F2"), "6");
}

#[test]
fn test_choosecols_negative_index() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("C1", "3");
    // -1 selects last column
    model._set("E1", "=CHOOSECOLS(A1:C1,-1)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "3");
}

#[test]
fn test_choosecols_out_of_range() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("E1", "=CHOOSECOLS(A1:B1,5)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "#VALUE!");
}

// ── CHOOSEROWS ────────────────────────────────────────────────────────────────

#[test]
fn test_chooserows_basic() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    // Select rows 1 and 3
    model._set("C1", "=CHOOSEROWS(A1:A3,1,3)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "1");
    assert_eq!(model._get_text("C2"), "3");
}

#[test]
fn test_chooserows_negative_index() {
    let mut model = new_empty_model();
    model._set("A1", "10");
    model._set("A2", "20");
    model._set("A3", "30");
    // -1 selects last row
    model._set("C1", "=CHOOSEROWS(A1:A3,-1)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "30");
}

#[test]
fn test_chooserows_out_of_range() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("C1", "=CHOOSEROWS(A1:A2,5)");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "#VALUE!");
}

#[test]
fn test_choose_error_argument_beats_out_of_range_index() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "2");
    model._set("A2", "3");
    model._set("B2", "4");
    model._set("E1", "=CHOOSEROWS(A1:B2,9,NA())");
    model._set("E2", "=CHOOSECOLS(A1:B2,0,1/0)");
    model._set("E3", "=CHOOSEROWS(A1:B2,9,1)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "#N/A");
    assert_eq!(model._get_text("E2"), "#DIV/0!");
    assert_eq!(model._get_text("E3"), "#VALUE!");
}

#[test]
fn test_choose_huge_negative_index_is_out_of_range() {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("E1", "=CHOOSEROWS(A1:A2,-1E+300)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "#VALUE!");
}

/// probes/t4718-choose-index-error-order H1, H2, H7, H8, H5, H29: first problem in argument order wins,
/// and a 2-D index array is `#VALUE!` whatever it holds.
#[test]
fn test_choose_index_list_problem_order_and_2d_index() {
    let mut model = new_empty_model();
    model._set("E1", "=CHOOSECOLS(A1:C2,1/0,NA())");
    model._set("E2", "=CHOOSECOLS(A1:C2,NA(),1/0)");
    model._set("E3", "=CHOOSEROWS(A1:C2,\"x\",NA())");
    model._set("E4", "=CHOOSEROWS(A1:C2,NA(),\"x\")");
    model._set("E5", "=CHOOSECOLS(A1:C2,{1,#N/A;#DIV/0!,2})");
    model._set("E6", "=CHOOSEROWS(A1:C2,{1,2;2,1})");
    model.evaluate();
    for (cell, want) in [
        ("E1", "#DIV/0!"),
        ("E2", "#N/A"),
        ("E3", "#VALUE!"),
        ("E4", "#N/A"),
        ("E5", "#VALUE!"),
        ("E6", "#VALUE!"),
    ] {
        assert_eq!(model._get_text(cell), want, "{cell}");
    }
}
