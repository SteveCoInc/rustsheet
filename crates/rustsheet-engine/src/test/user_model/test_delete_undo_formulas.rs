#![allow(clippy::unwrap_used)]
//! Undo of a row or column delete puts back the formulas the delete rewrote past what
//! re-inserting the lines undoes: a reference made `#REF!`, a range that lost an edge (T-4668).

use std::collections::HashMap;

use crate::formula_rewrite::RewrittenCell;
use crate::types::{Cell, Workbook};
use crate::user_model::history::{ColumnData, Diff, DiffType, QueueDiffs};
use crate::UserModel;

/// Formulas on `S` (sheet 0) and `F` (sheet 1) around column D (or row 4) of `S`, with the
/// texts a delete of column D (`by_column`) or of row 4 gives them.
fn cases(by_column: bool) -> Vec<(u32, i32, i32, &'static str, &'static str)> {
    if by_column {
        vec![
            (1, 1, 1, "=SUM(S!D4:D7)", "=SUM(S!#REF!)"),
            (1, 2, 1, "=S!D1", "=#REF!"),
            (1, 3, 1, "=SUM(S!D1:E1)", "=SUM(S!D1:D1)"),
            (1, 4, 1, "=SUM(S!C1:D1)", "=SUM(S!C1:C1)"),
            (1, 5, 1, "=SUM(S!C1:E1)", "=SUM(S!C1:D1)"),
            (0, 9, 1, "=D1+E1", "=#REF!+D1"),
            (0, 9, 6, "=D1+E1", "=#REF!+D1"),
        ]
    } else {
        vec![
            (1, 1, 1, "=SUM(S!A4:D4)", "=SUM(S!#REF!)"),
            (1, 2, 1, "=SUM(S!A4:A5)", "=SUM(S!A4:A4)"),
            (1, 3, 1, "=SUM(S!A3:A4)", "=SUM(S!A3:A3)"),
            (0, 1, 9, "=A4+A5", "=#REF!+A4"),
            (0, 9, 9, "=A4+A5", "=#REF!+A4"),
        ]
    }
}

fn model(by_column: bool) -> UserModel {
    let mut model = UserModel::new_empty("model", "en", "UTC", "en").unwrap();
    model.rename_sheet(0, "S").unwrap();
    model.new_sheet().unwrap();
    model.rename_sheet(1, "F").unwrap();
    for row in 1..=7 {
        for column in 1..=5 {
            model
                .set_user_input(0, row, column, &(row * 10 + column).to_string())
                .unwrap();
        }
    }
    for (sheet, row, column, formula, _) in cases(by_column) {
        model.set_user_input(sheet, row, column, formula).unwrap();
    }
    model
}

/// Where a cell before the delete is after it.
fn after(by_column: bool, (sheet, row, column): (u32, i32, i32)) -> (u32, i32, i32) {
    match (sheet, by_column) {
        (0, true) if column > 4 => (0, row, column - 1),
        (0, false) if row > 4 => (0, row - 1, column),
        _ => (sheet, row, column),
    }
}

fn assert_formulas(model: &UserModel, by_column: bool, deleted: bool) {
    for (sheet, row, column, original, rewritten) in cases(by_column) {
        let (expected, (s, r, c)) = if deleted {
            (rewritten, after(by_column, (sheet, row, column)))
        } else {
            (original, (sheet, row, column))
        };
        let content = model.get_cell_content(s, r, c).unwrap();
        assert_eq!(content, expected, "{original} at {sheet}!R{row}C{column}");
    }
}

/// Undo of a column delete restores every formula it rewrote, on another sheet and on the edited
/// one, moved or not; redo rewrites them again.
#[test]
fn undo_of_a_column_delete_restores_the_rewritten_formulas() {
    let mut model = model(true);
    let values = model.get_cell_content(1, 1, 1).unwrap();
    model.delete_columns(0, 4, 1).unwrap();
    assert_formulas(&model, true, true);
    model.undo().unwrap();
    assert_formulas(&model, true, false);
    assert_eq!(model.get_cell_content(1, 1, 1).unwrap(), values);
    assert_eq!(model.get_formatted_cell_value(1, 1, 1).unwrap(), "236");
    model.redo().unwrap();
    assert_formulas(&model, true, true);
    model.undo().unwrap();
    assert_formulas(&model, true, false);
}

/// The same for a row delete.
#[test]
fn undo_of_a_row_delete_restores_the_rewritten_formulas() {
    let mut model = model(false);
    model.delete_rows(0, 4, 1).unwrap();
    assert_formulas(&model, false, true);
    model.undo().unwrap();
    assert_formulas(&model, false, false);
    assert_eq!(model.get_formatted_cell_value(1, 1, 1).unwrap(), "170");
    model.redo().unwrap();
    assert_formulas(&model, false, true);
    model.undo().unwrap();
    assert_formulas(&model, false, false);
}

/// A formula the delete only shifts is not recorded: re-inserting the column shifts it back.
#[test]
fn a_shifted_formula_is_not_recorded() {
    let mut model = UserModel::new_empty("model", "en", "UTC", "en").unwrap();
    model.set_user_input(0, 1, 1, "=E1+SUM(B1:F1)").unwrap();
    model.set_user_input(0, 1, 7, "=A1").unwrap();
    model.flush_send_queue();
    model.delete_columns(0, 4, 1).unwrap();
    let entries: Vec<QueueDiffs> = bitcode::decode(&model.flush_send_queue()).unwrap();
    let Some(Diff::DeleteColumns { rewritten, .. }) = entries[0].list.last() else {
        panic!("no column delete");
    };
    assert!(rewritten.is_empty());
    assert_eq!(model.get_cell_content(0, 1, 1).unwrap(), "=D1+SUM(B1:E1)");
}

/// A peer's undo of a column delete naming a rewritten cell whose place holds no formula, or a
/// cell with no formula, or a place inside the deleted columns, is refused and changes nothing
/// (CLAUDE.md directive 5).
#[test]
fn a_restored_rewritten_cell_is_checked() {
    let mut model = UserModel::new_empty("model", "en", "UTC", "en").unwrap();
    model.set_user_input(0, 1, 1, "=1+1").unwrap();
    model.set_user_input(0, 2, 1, "5").unwrap();
    model.set_user_input(0, 1, 6, "=2+2").unwrap();
    let formula = model.model.workbook.worksheets[0]
        .cell(1, 1)
        .unwrap()
        .clone();
    let entry = |row, column, cell: &Cell| RewrittenCell {
        sheet: 0,
        row,
        column,
        cell: cell.clone(),
    };
    let bads = [
        // A2 holds a number, not a formula.
        entry(2, 1, &formula),
        // Nothing at A3.
        entry(3, 1, &formula),
        // The cell to restore holds no formula.
        entry(1, 1, &Cell::NumberCell { v: 1.0, s: 0 }),
        // Inside the deleted column.
        entry(1, 2, &formula),
        // G1 before the delete is F1 now, which holds a formula, but its formula index is past
        // the sheet's formulas.
        entry(
            1,
            7,
            &Cell::CellFormula {
                f: 99,
                s: 0,
                v: crate::types::FormulaValue::Unevaluated,
            },
        ),
        // On no sheet.
        RewrittenCell {
            sheet: 9,
            ..entry(1, 1, &formula)
        },
    ];
    for bad in bads {
        let what = format!("{bad:?}");
        let diff = Diff::DeleteColumns {
            sheet: 0,
            column: 2,
            count: 1,
            old_data: vec![ColumnData {
                column: None,
                data: HashMap::new(),
            }],
            tables: vec![],
            rewritten: vec![bad],
        };
        let bytes = bitcode::encode(&vec![QueueDiffs {
            r#type: DiffType::Undo,
            list: vec![diff],
        }]);
        let before: Workbook = model.model.workbook.clone();
        assert!(model.apply_external_diffs(&bytes).is_err(), "{what}");
        assert!(
            model.model.workbook == before,
            "{what}: a refused list changed the model"
        );
    }
}
