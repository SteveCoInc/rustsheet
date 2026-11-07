#![allow(clippy::unwrap_used)]

use crate::test::util::{new_empty_model, whole_column_arms};

// ── HSTACK ────────────────────────────────────────────────────────────────────

#[test]
fn test_hstack_two_columns() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("B1", "3");
    model.set_a1("B2", "4");
    model.set_a1("D1", "=HSTACK(A1:A2,B1:B2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("D1"), "1");
    assert_eq!(model.get_text_a1("E1"), "3");
    assert_eq!(model.get_text_a1("D2"), "2");
    assert_eq!(model.get_text_a1("E2"), "4");
}

#[test]
fn test_hstack_unequal_rows() {
    let mut model = new_empty_model();
    // Column A has 2 rows, column B has 1 row → row 2 of B padded with #N/A
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("B1", "3");
    model.set_a1("D1", "=HSTACK(A1:A2,B1:B1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("D1"), "1");
    assert_eq!(model.get_text_a1("E1"), "3");
    assert_eq!(model.get_text_a1("D2"), "2");
    assert_eq!(model.get_text_a1("E2"), "#N/A");
}

// ── VSTACK ────────────────────────────────────────────────────────────────────

#[test]
fn test_vstack_two_rows() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("B1", "2");
    model.set_a1("A2", "3");
    model.set_a1("B2", "4");
    model.set_a1("D1", "=VSTACK(A1:B1,A2:B2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("D1"), "1");
    assert_eq!(model.get_text_a1("E1"), "2");
    assert_eq!(model.get_text_a1("D2"), "3");
    assert_eq!(model.get_text_a1("E2"), "4");
}

#[test]
fn test_vstack_unequal_cols() {
    let mut model = new_empty_model();
    // Row 1 has 2 cols, row 2 has 1 col → col 2 of row 2 padded with #N/A
    model.set_a1("A1", "1");
    model.set_a1("B1", "2");
    model.set_a1("A2", "3");
    model.set_a1("D1", "=VSTACK(A1:B1,A2:A2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("D1"), "1");
    assert_eq!(model.get_text_a1("E1"), "2");
    assert_eq!(model.get_text_a1("D2"), "3");
    assert_eq!(model.get_text_a1("E2"), "#N/A");
}

#[test]
fn test_hstack_refuses_an_oversized_stack_before_copying() {
    // 300 whole columns side by side are past the array-cell limit: refused from the arguments'
    // extents before any column is copied (T-1564), while an error argument still wins.
    let mut model = new_empty_model();
    model.set_a1("A1", &format!("=ROWS(HSTACK({}))", whole_column_arms(300)));
    model.set_a1("A2", "=ROWS(HSTACK(C:C,D:D,1/0))");
    model.set_a1("A3", "=ROWS(VSTACK(C1:XFD2,C1:XFD100))");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#ERROR!");
    assert_eq!(model.get_text_a1("A2"), "#DIV/0!");
    assert_eq!(model.get_text_a1("A3"), "#ERROR!");
}

#[test]
fn test_hstack_reads_its_ranges_before_the_size_check() {
    // A range argument is read before the stack is checked, so a cycle through it is circular
    // (0) rather than the cell-count `#ERROR!`, and so is one beside an error argument
    // as before T-1564 (probes/scalar-lift-open-c3 W1; its oracle 0 is flagged untrusted).
    let mut model = new_empty_model();
    model.set_a1("W1", "=ROWS(HSTACK(A:A,1:1))");
    model.set_a1("G1", "=ROWS(HSTACK(G1:G2,NA()))");
    model.evaluate();
    assert_eq!(model.get_text_a1("W1"), "0");
    assert_eq!(model.get_text_a1("G1"), "0");
    // VSTACK alike: each range is under the range-size limit, the stack past the cell-count one.
    let mut model = new_empty_model();
    model.set_a1("A1", "=ROWS(VSTACK(A1:XFD60,B1:XFD60))");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "0");
}
