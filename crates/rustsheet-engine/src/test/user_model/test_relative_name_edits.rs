#![allow(clippy::unwrap_used)]
//! A relative reference in a defined name does not move under a row or column insert or delete,
//! whichever sheet uses the name; an absolute one does (T-4760; references.md "Relative references
//! in a name"; probes/sheet-ops/t4747-name-relative-*, t4760-name-relative-same-sheet-*, excel-graph).

use crate::UserModel;

/// Sheet `Data` with `Data!An = n*10` for rows 1..=12, and names `Rel` = `formula`, `Abs` =
/// `Data!$A$5`.
fn model(formula: &str) -> UserModel {
    let mut model = UserModel::new_empty("model", "en", "UTC", "en").unwrap();
    model.rename_sheet(0, "Data").unwrap();
    for row in 1..=12 {
        model
            .set_user_input(0, row, 1, &format!("{}", row * 10))
            .unwrap();
    }
    model.new_defined_name("Rel", None, formula).unwrap();
    model.new_defined_name("Abs", None, "Data!$A$5").unwrap();
    model
}

fn formula_of(model: &UserModel, name: &str) -> String {
    model
        .get_defined_name_list()
        .into_iter()
        .find(|(n, _, _)| n == name)
        .map(|(_, _, formula)| formula)
        .unwrap()
}

#[test]
fn a_row_edit_leaves_a_relative_name_and_shifts_an_absolute_one() {
    let mut model = model("Data!A5");
    model.insert_rows(0, 4, 1).unwrap();
    assert_eq!(formula_of(&model, "Rel"), "Data!A5");
    assert_eq!(formula_of(&model, "Abs"), "Data!$A$6");
    model.delete_rows(0, 4, 2).unwrap();
    assert_eq!(formula_of(&model, "Rel"), "Data!A5");
    assert_eq!(formula_of(&model, "Abs"), "Data!$A$4");
}

#[test]
fn a_column_edit_leaves_a_relative_column() {
    let mut model = model("Data!C5:D6");
    model.insert_columns(0, 1, 1).unwrap();
    assert_eq!(formula_of(&model, "Rel"), "Data!C5:D6");
    assert_eq!(formula_of(&model, "Abs"), "Data!$B$5");
}

#[test]
fn a_relative_name_read_from_its_own_sheet_reads_the_literal_address() {
    // t4760-name-relative-same-sheet-insert-row: Data!C1 `=Rel` reads Data!C5 before the edit
    // (the name is anchored at A1); after inserting row 4 it still reads C5.
    let mut model = model("Data!A5");
    model.set_user_input(0, 1, 3, "=Rel").unwrap();
    model.set_user_input(0, 5, 3, "7").unwrap();
    model.insert_rows(0, 4, 1).unwrap();
    model.set_user_input(0, 5, 3, "8").unwrap();
    assert_eq!(model.get_formatted_cell_value(0, 1, 3).unwrap(), "8");
}
