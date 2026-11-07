#![allow(clippy::unwrap_used)]

use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::test::util::new_empty_model;

#[test]
fn full_column_reference_yields_spill_error() {
    let mut model = new_empty_model();
    model.set_a1("A1", "10");
    model.set_a1("A2", "20");
    model.set_a1("A3", "30");

    model.set_a1("C3", "=A:A");
    model.evaluate();

    assert_eq!(
        model.get_text_a1("C3"),
        "#SPILL!",
        "=A:A must produce #SPILL! because the spill overflows the sheet"
    );
    // No SpillCells should have been written; neighbouring cells stay empty.
    assert_eq!(model.get_text_a1("C4"), "");
    assert_eq!(model.get_text_a1("C5"), "");
}

#[test]
fn full_column_reference_in_empty_column_yields_spill_error() {
    let mut model = new_empty_model();

    model.set_a1("C3", "=A:A");
    model.evaluate();

    assert_eq!(
        model.get_text_a1("C3"),
        "#SPILL!",
        "=A:A on an empty column must also produce #SPILL!"
    );
}

#[test]
fn full_row_reference_yields_spill_error() {
    let mut model = new_empty_model();
    model.set_a1("A1", "100");
    model.set_a1("B1", "200");
    model.set_a1("C1", "300");

    model.set_a1("C3", "=1:1");
    model.evaluate();

    assert_eq!(
        model.get_text_a1("C3"),
        "#SPILL!",
        "=1:1 must produce #SPILL! because the spill overflows the sheet"
    );
    // No SpillCells should be written; adjacent cells in the same row stay empty.
    assert_eq!(model.get_text_a1("D3"), "");
    assert_eq!(model.get_text_a1("E3"), "");
}

#[test]
fn column_spill_overflowing_last_row_yields_spill_error() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("A3", "3");

    let anchor_row = LAST_ROW - 8;
    model
        .set_user_input(0, anchor_row, 1, "=A1:A10".to_string())
        .unwrap();
    model.evaluate();

    assert_eq!(
        model.get_text_by_index(0, anchor_row, 1),
        "#SPILL!",
        "=A1:A10 near the bottom border must produce #SPILL!"
    );
    // The row below the anchor must be empty — no stray SpillCells.
    assert_eq!(model.get_text_by_index(0, anchor_row + 1, 1), "");
}

#[test]
fn column_spill_fitting_exactly_at_last_row_succeeds() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("A3", "3");

    let anchor_row = LAST_ROW - 9;
    model
        .set_user_input(0, anchor_row, 1, "=A1:A10".to_string())
        .unwrap();
    model.evaluate();

    assert_eq!(model.get_text_by_index(0, anchor_row, 1), "1");
    assert_eq!(model.get_text_by_index(0, anchor_row + 1, 1), "2");
    assert_eq!(model.get_text_by_index(0, anchor_row + 2, 1), "3");
}

#[test]
fn row_spill_overflowing_last_column_yields_spill_error() {
    let mut model = new_empty_model();
    // Populate A1:J1 so the range is non-trivially non-empty
    for col in 1..=10 {
        model.set_user_input(0, 1, col, col.to_string()).unwrap();
    }

    let anchor_col = LAST_COLUMN - 8;
    model
        .set_user_input(0, 3, anchor_col, "=A1:J1".to_string())
        .unwrap();
    model.evaluate();

    assert_eq!(
        model.get_text_by_index(0, 3, anchor_col),
        "#SPILL!",
        "=A1:J1 near the right border must produce #SPILL!"
    );
    // The column to the right of the anchor must be empty — no stray SpillCells.
    assert_eq!(model.get_text_by_index(0, 3, anchor_col + 1), "");
}

#[test]
fn row_spill_fitting_exactly_at_last_column_succeeds() {
    let mut model = new_empty_model();
    for col in 1..=10 {
        model.set_user_input(0, 1, col, col.to_string()).unwrap();
    }

    let anchor_col = LAST_COLUMN - 9;
    model
        .set_user_input(0, 3, anchor_col, "=A1:J1".to_string())
        .unwrap();
    model.evaluate();

    assert_eq!(model.get_text_by_index(0, 3, anchor_col), "1");
    assert_eq!(model.get_text_by_index(0, 3, anchor_col + 1), "2");
    assert_eq!(model.get_text_by_index(0, 3, anchor_col + 2), "3");
}

// ── SEQUENCE / RANDARRAY near the sheet border ────────────────────────────────

#[test]
fn sequence_spill_overflowing_last_row_yields_spill_error() {
    let mut model = new_empty_model();

    // SEQUENCE(10) produces a 10×1 array.  Anchored at LAST_ROW-8 the last
    // spill row is (LAST_ROW-8)+10-1 = LAST_ROW+1, which overflows.
    let anchor_row = LAST_ROW - 8;
    model
        .set_user_input(0, anchor_row, 1, "=SEQUENCE(10)".to_string())
        .unwrap();
    model.evaluate();

    assert_eq!(
        model.get_text_by_index(0, anchor_row, 1),
        "#SPILL!",
        "SEQUENCE(10) near the bottom border must produce #SPILL!, not #ERROR!"
    );
    // No partial spill cells should have been written.
    assert_eq!(model.get_text_by_index(0, anchor_row + 1, 1), "");
}

#[test]
fn sequence_spill_fitting_exactly_at_last_row_succeeds() {
    let mut model = new_empty_model();

    // SEQUENCE(10) anchored at LAST_ROW-9: last spill row = LAST_ROW, fits exactly.
    let anchor_row = LAST_ROW - 9;
    model
        .set_user_input(0, anchor_row, 1, "=SEQUENCE(10)".to_string())
        .unwrap();
    model.evaluate();

    assert_eq!(model.get_text_by_index(0, anchor_row, 1), "1");
    assert_eq!(model.get_text_by_index(0, anchor_row + 1, 1), "2");
    assert_eq!(model.get_text_by_index(0, anchor_row + 9, 1), "10");
}

#[test]
fn randarray_spill_overflowing_last_column_yields_spill_error() {
    let mut model = new_empty_model();

    // RANDARRAY(1,10) produces a 1×10 array.  Anchored at column LAST_COLUMN-8
    // the last spill column is (LAST_COLUMN-8)+10-1 = LAST_COLUMN+1, which overflows.
    let anchor_col = LAST_COLUMN - 8;
    model
        .set_user_input(0, 3, anchor_col, "=RANDARRAY(1,10)".to_string())
        .unwrap();
    model.evaluate();

    assert_eq!(
        model.get_text_by_index(0, 3, anchor_col),
        "#SPILL!",
        "RANDARRAY(1,10) near the right border must produce #SPILL!, not #ERROR!"
    );
    // No partial spill cells should have been written.
    assert_eq!(model.get_text_by_index(0, 3, anchor_col + 1), "");
}
