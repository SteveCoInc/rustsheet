#![allow(clippy::unwrap_used)]

//! A legacy (CSE) array formula over a range larger than its result (arrays.md "Legacy (CSE) array
//! formula over a larger range"; poi/MatrixFormulaEvalTestData.xlsx Sheet1 H46:J136, Excel-cached).

use crate::{model::Model, test::util::new_empty_model};

/// The displayed text of each cell of the `width` x `height` block at A`row`, row by row.
fn block(model: &Model, row: i32, width: i32, height: i32) -> Vec<Vec<String>> {
    (row..row + height)
        .map(|r| (1..=width).map(|c| model._get_text_at(0, r, c)).collect())
        .collect()
}

fn texts(rows: &[&[&str]]) -> Vec<Vec<String>> {
    rows.iter()
        .map(|row| row.iter().map(|t| (*t).to_string()).collect())
        .collect()
}

#[test]
fn a_one_by_one_result_fills_every_cell() {
    let mut model = new_empty_model();
    // TRANSPOSE(B46) over H46:J48 is 1 in all nine cells.
    model.set_user_array_formula(0, 5, 1, 3, 3, "={7}").unwrap();
    model.evaluate();
    assert_eq!(
        block(&model, 5, 3, 3),
        texts(&[&["7", "7", "7"], &["7", "7", "7"], &["7", "7", "7"]])
    );
}

#[test]
fn a_one_row_result_repeats_down_and_is_na_past_its_width() {
    let mut model = new_empty_model();
    // B62:C62+E62:F62 over H62:J64 repeats the row down, J62:J64 #N/A.
    model
        .set_user_array_formula(0, 5, 1, 3, 3, "={1,2}")
        .unwrap();
    model.evaluate();
    assert_eq!(
        block(&model, 5, 3, 3),
        texts(&[
            &["1", "2", "#N/A"],
            &["1", "2", "#N/A"],
            &["1", "2", "#N/A"]
        ])
    );
}

#[test]
fn a_one_column_result_repeats_across_and_is_na_past_its_height() {
    let mut model = new_empty_model();
    // TRANSPOSE(B66:C66) over H66:J68 is the column repeated across, row 68 #N/A.
    model
        .set_user_array_formula(0, 5, 1, 3, 3, "={1;2}")
        .unwrap();
    model.evaluate();
    assert_eq!(
        block(&model, 5, 3, 3),
        texts(&[
            &["1", "1", "1"],
            &["2", "2", "2"],
            &["#N/A", "#N/A", "#N/A"]
        ])
    );
}

#[test]
fn a_two_dimensional_result_is_na_outside_it() {
    let mut model = new_empty_model();
    model
        .set_user_array_formula(0, 5, 1, 3, 3, "={1,2;3,4}")
        .unwrap();
    model.evaluate();
    assert_eq!(
        block(&model, 5, 3, 3),
        texts(&[
            &["1", "2", "#N/A"],
            &["3", "4", "#N/A"],
            &["#N/A", "#N/A", "#N/A"]
        ])
    );
}

#[test]
fn the_fill_is_not_an_error_iferror_catches() {
    let mut model = new_empty_model();
    // corpus-bench/13894 F14:F25: an IFERROR-wrapped 6x1 result over 12 rows leaves F20:F25 #N/A.
    model
        .set_user_array_formula(0, 5, 1, 1, 3, "=IFERROR({1;2},\"x\")")
        .unwrap();
    model.evaluate();
    assert_eq!(block(&model, 5, 1, 3), texts(&[&["1"], &["2"], &["#N/A"]]));
}
