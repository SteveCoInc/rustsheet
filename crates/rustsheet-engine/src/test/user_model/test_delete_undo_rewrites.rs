#![allow(clippy::unwrap_used)]
//! Undo of a row or column delete puts back the conditional formats and defined names the delete
//! rewrote, and redo rewrites them again (T-4702). Excel rewrites names as it rewrites cell
//! references (probes/sheet-ops/t4702-names-*, t4702-name-deleted-cell-*, excel-graph).

use crate::cf_types::{CfRule, CfRuleInput};
use crate::types::{Dxf, Workbook};
use crate::user_model::history::{Diff, DiffType, QueueDiffs};
use crate::UserModel;

fn rule(formula: &str) -> CfRuleInput {
    CfRuleInput::Formula {
        formula: formula.to_string(),
        format: Dxf::default(),
        stop_if_true: false,
    }
}

/// Sheet `S` with values in C..F (or rows 3..6), two conditional formats a delete of column D
/// (row 4) shrinks or rewrites, and three names over it.
fn model(by_column: bool) -> UserModel {
    let mut model = UserModel::new_empty("model", "en", "UTC", "en").unwrap();
    model.rename_sheet(0, "S").unwrap();
    let (cells, names, formats) = if by_column {
        (
            [(1, 3), (1, 4), (1, 5), (1, 6)],
            ["S!$D$1", "S!$C$1:$D$1", "S!$E$1"],
            [("D1:D9", "=$D1>0"), ("B1:F1", "=B$1>5")],
        )
    } else {
        (
            [(3, 1), (4, 1), (5, 1), (6, 1)],
            ["S!$A$4", "S!$A$3:$A$4", "S!$A$5"],
            [("A4:I4", "=A$4>0"), ("A2:A6", "=$A2>5")],
        )
    };
    for (i, (row, column)) in cells.into_iter().enumerate() {
        model
            .set_user_input(0, row, column, &format!("{}", 10_i32.pow(i as u32)))
            .unwrap();
    }
    for (name, formula) in ["Gone", "Edge", "Pt"].into_iter().zip(names) {
        model.new_defined_name(name, None, formula).unwrap();
    }
    for (range, formula) in formats {
        model
            .add_conditional_formatting(0, range, rule(formula))
            .unwrap();
    }
    let (row, column) = total(by_column);
    model
        .set_user_input(0, row, column, "=SUM(Edge)+Pt")
        .unwrap();
    model
}

/// The cell that reads the names, away from the deleted line.
fn total(by_column: bool) -> (i32, i32) {
    if by_column {
        (20, 1)
    } else {
        (1, 10)
    }
}

/// The sheet's conditional formats and the workbook's names, to compare states.
fn state(model: &UserModel) -> String {
    format!(
        "{:?} {:?}",
        model.model.workbook.worksheets[0].conditional_formatting,
        model.get_defined_name_list()
    )
}

fn delete_undo_redo(by_column: bool, names_after: [&str; 3]) {
    let mut model = model(by_column);
    let before = state(&model);
    let formats = model.model.workbook.worksheets[0]
        .conditional_formatting
        .clone();
    let (row, column) = total(by_column);
    let sum = model.get_formatted_cell_value(0, row, column).unwrap();
    if by_column {
        model.delete_columns(0, 4, 1).unwrap();
    } else {
        model.delete_rows(0, 4, 1).unwrap();
    }
    let after = state(&model);
    let formulas: Vec<String> = model
        .get_defined_name_list()
        .into_iter()
        .map(|(_, _, formula)| formula)
        .collect();
    assert_eq!(formulas, names_after);
    assert_ne!(after, before);
    let now = &model.model.workbook.worksheets[0].conditional_formatting;
    assert!(
        formats.iter().zip(now).all(|(a, b)| a != b),
        "every format was rewritten"
    );
    // Edge kept its first cell (1) and Pt moved with its cell (100).
    assert_eq!(
        model.get_formatted_cell_value(0, row, column).unwrap(),
        "101"
    );
    model.undo().unwrap();
    assert_eq!(state(&model), before);
    assert_eq!(model.get_formatted_cell_value(0, row, column).unwrap(), sum);
    model.redo().unwrap();
    assert_eq!(state(&model), after);
}

#[test]
fn undo_of_a_column_delete_restores_formats_and_names() {
    delete_undo_redo(true, ["#REF!", "S!$C$1:$C$1", "S!$D$1"]);
}

#[test]
fn undo_of_a_row_delete_restores_formats_and_names() {
    delete_undo_redo(false, ["#REF!", "S!$A$3:$A$3", "S!$A$4"]);
}

/// A peer's undo diff with a conditional format at no index or over no cell, or for a name that
/// does not exist, is refused and changes nothing (CLAUDE.md directive 5).
#[test]
fn a_restored_format_or_name_is_checked() {
    let mut model = model(true);
    let good = CfRule::Formula {
        formula: "=1".to_string(),
        dxf_id: 0,
        stop_if_true: false,
    };
    let format = |index: u32, range: &str| Diff::UpdateConditionalFormatting {
        sheet: 0,
        index,
        old_range: range.to_string(),
        old_rule: Box::new(good.clone()),
        old_priority: 1,
        new_range: range.to_string(),
        new_rule: Box::new(good.clone()),
    };
    let bads = [
        format(9, "A1"),
        format(0, "not a range"),
        Diff::UpdateDefinedName {
            name: "Nobody".to_string(),
            scope: None,
            old_formula: "S!$A$1".to_string(),
            new_name: "Nobody".to_string(),
            new_scope: None,
            new_formula: "S!$A$1".to_string(),
        },
    ];
    for (what, bad) in ["format index", "format range", "name"]
        .into_iter()
        .zip(bads)
    {
        for r#type in [DiffType::Undo, DiffType::Redo] {
            let bytes = bitcode::encode(&vec![QueueDiffs {
                r#type,
                list: vec![bad.clone()],
            }]);
            let before: Workbook = model.model.workbook.clone();
            assert!(model.apply_external_diffs(&bytes).is_err(), "{what}");
            assert!(
                model.model.workbook == before,
                "{what}: a refused list changed the model"
            );
        }
    }
}
