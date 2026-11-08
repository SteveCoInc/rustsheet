#![allow(clippy::unwrap_used)]

// Covered cells of a merged range that hold content, as a workbook Excel wrote can have them
// (docs/excel-semantics/xlsx-storage.md "A merged range keeps the values of its covered cells",
// fixtures/merged-covered-values and fixtures/merged-covered-formula; docs/specs/SPEC-0036).
//
// * Formulas read covered content, and a covered formula is calculated; a user edit of a covered
//   cell is still refused.
// * Unmerge keeps it (it simply shows again); clear over the merged range clears it.
// * Copy- and cut-paste of a merged block carry it: the recreated merge keeps the covered cells of
//   the pasted block and clears only what it swallows outside that block.
// * Autofill copies a covered source that holds content (a formula shifted), never as a
//   progression term; an empty covered source still clears its target.
// Each operation undoes and redoes.

use crate::expressions::types::Area;
use crate::test::user_model::util::new_empty_user_model;
use crate::types::MergedCell;
use crate::UserModel;

fn area(row: i32, column: i32, width: i32, height: i32) -> Area {
    Area {
        sheet: 0,
        row,
        column,
        width,
        height,
    }
}

fn merged(row: i32, column: i32, width: i32, height: i32) -> MergedCell {
    MergedCell {
        row,
        column,
        width,
        height,
    }
}

fn value(model: &UserModel, row: i32, column: i32) -> String {
    model.get_formatted_cell_value(0, row, column).unwrap()
}

fn content(model: &UserModel, row: i32, column: i32) -> String {
    model.get_cell_content(0, row, column).unwrap()
}

fn merges_sorted(model: &UserModel) -> Vec<MergedCell> {
    let mut merges = model.get_merged_cells(0).unwrap();
    merges.sort_by_key(|m| (m.row, m.column));
    merges
}

// B2:C3 merged over B2 "anchor", a covered C2 7 and a covered B3 `=C2*2`, as an imported workbook
// has it (the user-model merge clears covered cells, so the range is registered directly).
fn model_with_covered_content() -> UserModel {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 2, 2, "anchor").unwrap();
    model.set_user_input(0, 2, 3, "7").unwrap();
    model.set_user_input(0, 3, 2, "=C2*2").unwrap();
    model
        .model
        .workbook
        .worksheet_mut(0)
        .unwrap()
        .merged_cells
        .push(merged(2, 2, 2, 2));
    model.model.evaluate();
    model
}

#[test]
fn formulas_read_covered_content_and_covered_edits_are_refused() {
    let mut model = model_with_covered_content();
    model.set_user_input(0, 6, 1, "=C2+B3").unwrap();
    model.set_user_input(0, 6, 2, "=COUNTA(B2:C3)").unwrap();
    model.set_user_input(0, 6, 3, "=ISFORMULA(B3)").unwrap();
    assert_eq!(value(&model, 3, 2), "14");
    assert_eq!(value(&model, 6, 1), "21");
    assert_eq!(value(&model, 6, 2), "3");
    assert_eq!(value(&model, 6, 3), "TRUE");
    assert!(model.set_user_input(0, 2, 3, "8").is_err());
    assert_eq!(content(&model, 2, 3), "7");
}

#[test]
fn unmerge_shows_covered_content_again() {
    let mut model = model_with_covered_content();
    model.unmerge_cells(&area(2, 2, 2, 2)).unwrap();
    assert!(model.get_merged_cells(0).unwrap().is_empty());
    assert_eq!(value(&model, 2, 3), "7");
    assert_eq!(value(&model, 3, 2), "14");
    model.undo().unwrap();
    assert_eq!(merges_sorted(&model), vec![merged(2, 2, 2, 2)]);
    assert_eq!(value(&model, 2, 3), "7");
    model.redo().unwrap();
    assert!(model.get_merged_cells(0).unwrap().is_empty());
    assert_eq!(value(&model, 3, 2), "14");
}

#[test]
fn clear_over_the_merged_range_clears_covered_content() {
    let mut model = model_with_covered_content();
    model.range_clear_contents(&area(2, 2, 2, 2)).unwrap();
    for (row, column) in [(2, 2), (2, 3), (3, 2)] {
        assert_eq!(content(&model, row, column), "", "({row}, {column})");
    }
    assert_eq!(merges_sorted(&model), vec![merged(2, 2, 2, 2)]);
    model.undo().unwrap();
    assert_eq!(content(&model, 2, 3), "7");
    assert_eq!(content(&model, 3, 2), "=C2*2");
    assert_eq!(value(&model, 3, 2), "14");
    model.redo().unwrap();
    assert_eq!(content(&model, 2, 3), "");
    assert_eq!(content(&model, 3, 2), "");
}

#[test]
fn copy_paste_keeps_the_covered_content_of_the_pasted_block() {
    let mut model = model_with_covered_content();
    model.set_selected_cell(2, 2).unwrap();
    model.set_selected_range(2, 2, 3, 3).unwrap();
    let clipboard = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(6, 5).unwrap();
    model
        .paste_from_clipboard(0, clipboard.range, &clipboard.data, false)
        .unwrap();
    assert_eq!(
        merges_sorted(&model),
        vec![merged(2, 2, 2, 2), merged(6, 5, 2, 2)]
    );
    assert_eq!(value(&model, 6, 5), "anchor");
    assert_eq!(value(&model, 6, 6), "7");
    // the covered formula is shifted with the paste: `=F6*2`
    assert_eq!(content(&model, 7, 5), "=F6*2");
    assert_eq!(value(&model, 7, 5), "14");
    // the source keeps its covered content
    assert_eq!(value(&model, 2, 3), "7");

    model.undo().unwrap();
    assert_eq!(merges_sorted(&model), vec![merged(2, 2, 2, 2)]);
    for (row, column) in [(6, 5), (6, 6), (7, 5)] {
        assert_eq!(content(&model, row, column), "", "({row}, {column})");
    }
    model.redo().unwrap();
    assert_eq!(
        merges_sorted(&model),
        vec![merged(2, 2, 2, 2), merged(6, 5, 2, 2)]
    );
    assert_eq!(value(&model, 6, 6), "7");
    assert_eq!(value(&model, 7, 5), "14");
}

#[test]
fn cut_paste_moves_the_covered_content() {
    let mut model = model_with_covered_content();
    model.set_selected_cell(2, 2).unwrap();
    model.set_selected_range(2, 2, 3, 3).unwrap();
    let clipboard = model.copy_to_clipboard().unwrap();
    model.set_selected_cell(6, 5).unwrap();
    model
        .paste_from_clipboard(0, clipboard.range, &clipboard.data, true)
        .unwrap();
    assert_eq!(merges_sorted(&model), vec![merged(6, 5, 2, 2)]);
    assert_eq!(value(&model, 6, 5), "anchor");
    assert_eq!(value(&model, 6, 6), "7");
    // a cut moves the reference with the cell it names
    assert_eq!(content(&model, 7, 5), "=F6*2");
    assert_eq!(value(&model, 7, 5), "14");
    for (row, column) in [(2, 2), (2, 3), (3, 2)] {
        assert_eq!(content(&model, row, column), "", "({row}, {column})");
    }

    model.undo().unwrap();
    assert_eq!(merges_sorted(&model), vec![merged(2, 2, 2, 2)]);
    assert_eq!(value(&model, 2, 3), "7");
    assert_eq!(content(&model, 3, 2), "=C2*2");
    assert_eq!(content(&model, 6, 6), "");
    model.redo().unwrap();
    assert_eq!(merges_sorted(&model), vec![merged(6, 5, 2, 2)]);
    assert_eq!(value(&model, 6, 6), "7");
    assert_eq!(value(&model, 7, 5), "14");
}

// A recreated merge that sticks out of the pasted block still clears what it swallows there: the
// copied range is clamped to the sheet's dimension, the merge travels whole.
#[test]
fn a_pasted_merge_clears_what_it_swallows_outside_the_pasted_block() {
    let mut model = new_empty_user_model();
    model.set_user_input(0, 1, 1, "a").unwrap();
    // A1:A3 merged; the sheet's content ends at row 1, so the copy of A1:A3 is clamped to A1
    model
        .model
        .workbook
        .worksheet_mut(0)
        .unwrap()
        .merged_cells
        .push(merged(1, 1, 1, 3));
    model.set_selected_range(1, 1, 3, 1).unwrap();
    let clipboard = model.copy_to_clipboard().unwrap();
    assert_eq!(clipboard.range, (1, 1, 1, 1));
    // the target sheet holds content where the merge lands outside the pasted block C5
    model.new_sheet().unwrap();
    model.set_user_input(1, 6, 3, "outside").unwrap();
    model.set_selected_sheet(1).unwrap();
    model.set_selected_cell(5, 3).unwrap();
    model
        .paste_from_clipboard(0, clipboard.range, &clipboard.data, false)
        .unwrap();
    let in_sheet = |model: &UserModel, row, column| model.get_cell_content(1, row, column).unwrap();
    assert_eq!(model.get_merged_cells(1).unwrap(), vec![merged(5, 3, 1, 3)]);
    assert_eq!(in_sheet(&model, 5, 3), "a");
    assert_eq!(in_sheet(&model, 6, 3), "");
    model.undo().unwrap();
    assert!(model.get_merged_cells(1).unwrap().is_empty());
    assert_eq!(in_sheet(&model, 6, 3), "outside");
    model.redo().unwrap();
    assert_eq!(in_sheet(&model, 6, 3), "");
}

#[test]
fn autofill_copies_covered_content_and_clears_from_empty_covered_cells() {
    let mut model = model_with_covered_content();
    // fill B2:C3 down to row 7: tiles B4:C5 and B6:C7
    model.auto_fill_rows(&area(2, 2, 2, 2), 7).unwrap();
    assert_eq!(
        merges_sorted(&model),
        vec![merged(2, 2, 2, 2), merged(4, 2, 2, 2), merged(6, 2, 2, 2)]
    );
    for tile in [4, 6] {
        assert_eq!(value(&model, tile, 2), "anchor");
        // the covered number is copied, not continued as a progression
        assert_eq!(value(&model, tile, 3), "7");
        // the covered formula is shifted with the fill
        assert_eq!(content(&model, tile + 1, 2), format!("=C{tile}*2"));
        assert_eq!(value(&model, tile + 1, 2), "14");
        // C3 is empty, so are its copies
        assert_eq!(content(&model, tile + 1, 3), "");
    }

    model.undo().unwrap();
    assert_eq!(merges_sorted(&model), vec![merged(2, 2, 2, 2)]);
    for row in 4..=7 {
        for column in 2..=3 {
            assert_eq!(content(&model, row, column), "", "({row}, {column})");
        }
    }
    model.redo().unwrap();
    assert_eq!(value(&model, 6, 3), "7");
    assert_eq!(value(&model, 7, 2), "14");
}

// An empty covered source still clears its target, which held content before the fill.
#[test]
fn autofill_from_an_empty_covered_cell_clears_the_target() {
    let mut model = model_with_covered_content();
    model.set_user_input(0, 5, 3, "old").unwrap();
    model.auto_fill_rows(&area(2, 2, 2, 2), 5).unwrap();
    assert_eq!(content(&model, 5, 3), "");
    model.undo().unwrap();
    assert_eq!(content(&model, 5, 3), "old");
    model.redo().unwrap();
    assert_eq!(content(&model, 5, 3), "");
}
