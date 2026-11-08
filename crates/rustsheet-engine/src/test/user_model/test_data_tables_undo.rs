#![allow(clippy::unwrap_used)]
//! Undo of a row or column delete, or of a clear, gives back the data tables it made `#REF!` or
//! deleted, and redo changes them again (T-4940; docs/excel-semantics/recalc.md, "Data tables
//! follow row and column edits around them").

use crate::expressions::types::Area;
use crate::types::{DataTable, DataTableInput};
use crate::user_model::history::{Diff, DiffType, QueueDiffs};
use crate::UserModel;

/// Results B4:B6 with the column input D1: the region is A3:B6.
fn table() -> DataTable {
    DataTable {
        range: (4, 2, 6, 2),
        row_input: None,
        column_input: Some(DataTableInput::Cell(1, 4)),
    }
}

fn model() -> UserModel {
    let mut model = UserModel::new_empty("model", "en", "UTC", "en").unwrap();
    model.set_user_input(0, 1, 4, "1").unwrap();
    model.set_user_input(0, 3, 2, "=D1*2").unwrap();
    model.model.add_data_table(0, table()).unwrap();
    model.model.evaluate();
    model
}

fn tables(model: &UserModel) -> Vec<DataTable> {
    model.model.workbook.worksheets[0].data_tables.clone()
}

#[test]
fn undo_of_a_row_delete_restores_a_deleted_input() {
    let mut model = model();
    model.delete_rows(0, 1, 1).unwrap();
    assert_eq!(
        tables(&model)[0].column_input,
        Some(DataTableInput::Deleted)
    );
    model.undo().unwrap();
    assert_eq!(tables(&model), vec![table()]);
    model.redo().unwrap();
    assert_eq!(
        tables(&model)[0].column_input,
        Some(DataTableInput::Deleted)
    );
    assert_eq!(tables(&model)[0].range, (3, 2, 5, 2));
    model.undo().unwrap();
    assert_eq!(tables(&model), vec![table()]);
}

#[test]
fn undo_of_a_column_delete_restores_a_deleted_input() {
    let mut model = model();
    model.delete_columns(0, 4, 1).unwrap();
    assert_eq!(
        tables(&model)[0].column_input,
        Some(DataTableInput::Deleted)
    );
    model.undo().unwrap();
    assert_eq!(tables(&model), vec![table()]);
    model.redo().unwrap();
    assert_eq!(
        tables(&model)[0].column_input,
        Some(DataTableInput::Deleted)
    );
    model.undo().unwrap();
    assert_eq!(tables(&model), vec![table()]);
}

#[test]
fn undo_of_a_delete_of_the_whole_region_restores_the_table() {
    for rows in [true, false] {
        let mut model = model();
        let results = model.get_formatted_cell_value(0, 5, 2).unwrap();
        if rows {
            model.delete_rows(0, 3, 4).unwrap();
        } else {
            model.delete_columns(0, 1, 2).unwrap();
        }
        assert!(tables(&model).is_empty());
        model.undo().unwrap();
        assert_eq!(tables(&model), vec![table()]);
        assert_eq!(model.get_formatted_cell_value(0, 5, 2).unwrap(), results);
        model.redo().unwrap();
        assert!(tables(&model).is_empty());
        model.undo().unwrap();
        assert_eq!(tables(&model), vec![table()]);
    }
}

#[test]
fn undo_of_a_clear_over_every_result_restores_the_table() {
    let area = Area {
        sheet: 0,
        row: 4,
        column: 2,
        width: 1,
        height: 3,
    };
    for all in [true, false] {
        let mut model = model();
        if all {
            model.range_clear_all(&area).unwrap();
        } else {
            model.range_clear_contents(&area).unwrap();
        }
        assert!(tables(&model).is_empty());
        model.undo().unwrap();
        assert_eq!(tables(&model), vec![table()]);
        model.redo().unwrap();
        assert!(tables(&model).is_empty());
        model.undo().unwrap();
        assert_eq!(tables(&model), vec![table()]);
    }
}

#[test]
fn a_diff_with_tables_that_break_the_rules_is_refused() {
    let bad = |range, column_input| DataTable {
        range,
        row_input: None,
        column_input,
    };
    let cases = [
        // Results on row 1: no header row above them.
        vec![bad((1, 2, 2, 2), Some(DataTableInput::Cell(1, 4)))],
        // An input inside the results.
        vec![bad((4, 2, 6, 2), Some(DataTableInput::Cell(5, 2)))],
        // No input at all.
        vec![bad((4, 2, 6, 2), None)],
        // Two tables whose results overlap.
        vec![
            bad((4, 2, 6, 2), Some(DataTableInput::Cell(1, 4))),
            bad((6, 2, 7, 2), Some(DataTableInput::Cell(1, 4))),
        ],
    ];
    for tables in cases {
        for direction in [DiffType::Undo, DiffType::Redo] {
            let mut model = model();
            let before = model.model.workbook.clone();
            let diff = Diff::SetDataTables {
                sheet: 0,
                old_value: tables.clone(),
                new_value: tables.clone(),
            };
            let bytes = bitcode::encode(&vec![QueueDiffs {
                r#type: direction,
                list: vec![diff],
            }]);
            assert!(model.apply_external_diffs(&bytes).is_err());
            assert_eq!(model.model.workbook, before);
        }
    }
}
