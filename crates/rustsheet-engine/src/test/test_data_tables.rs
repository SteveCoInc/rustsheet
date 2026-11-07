#![allow(clippy::unwrap_used, reason = "tests")]

//! What-if data tables (docs/excel-semantics/recalc.md "Data tables"; probes/data-tables, T-0441).

use crate::test::util::new_empty_model;
use crate::types::{DataTable, DataTableInput};

fn table(
    range: (i32, i32, i32, i32),
    row: Option<(i32, i32)>,
    column: Option<(i32, i32)>,
) -> DataTable {
    let cell = |c: Option<(i32, i32)>| c.map(|(r, c)| DataTableInput::Cell(r, c));
    DataTable {
        range,
        row_input: cell(row),
        column_input: cell(column),
    }
}

/// probes/data-tables: one-variable column and row tables, a two-variable table and a table whose
/// formula reads another formula; readers see the results and the input keeps its value.
#[test]
fn tables_substitute_their_inputs_and_readers_see_the_results() {
    let mut model = new_empty_model();
    for (cell, value) in [
        ("A1", "1"),
        ("B1", "2"),
        ("B3", "=A1*10+B1"),
        ("A4", "1"),
        ("A5", "2"),
        ("A6", "3"),
        ("D7", "1"),
        ("E7", "2"),
        ("F7", "3"),
        ("C8", "=A1*10+B1"),
        ("A10", "=A1*10+B1"),
        ("B10", "100"),
        ("C10", "200"),
        ("D10", "300"),
        ("A11", "1"),
        ("A12", "2"),
        ("B14", "=B3*2"),
        ("A15", "5"),
        ("A16", "6"),
        ("H1", "=SUM(B4:B6)"),
        ("H6", "=SUM(B11:D12)"),
        ("H8", "=B3"),
        ("H9", "=SUM(B15:B16)"),
        ("H10", "=ISFORMULA(B5)"),
        ("H11", "=FORMULATEXT(B4)"),
        ("H12", "=FORMULATEXT(C12)"),
    ] {
        model.set_a1(cell, value);
    }
    model
        .add_data_table(0, table((4, 2, 6, 2), None, Some((1, 1))))
        .unwrap();
    model
        .add_data_table(0, table((8, 4, 8, 6), Some((1, 1)), None))
        .unwrap();
    model
        .add_data_table(0, table((11, 2, 12, 4), Some((1, 2)), Some((1, 1))))
        .unwrap();
    model
        .add_data_table(0, table((15, 2, 16, 2), None, Some((1, 1))))
        .unwrap();
    model.evaluate();

    let texts = |cells: &[&str]| {
        cells
            .iter()
            .map(|c| model.get_text_a1(c))
            .collect::<Vec<_>>()
    };
    assert_eq!(texts(&["B4", "B5", "B6"]), ["12", "22", "32"]);
    assert_eq!(texts(&["D8", "E8", "F8"]), ["12", "22", "32"]);
    assert_eq!(
        texts(&["B11", "C11", "D11", "B12", "C12", "D12"]),
        ["110", "210", "310", "120", "220", "320"]
    );
    assert_eq!(texts(&["B15", "B16"]), ["104", "124"]);
    assert_eq!(
        texts(&["H1", "H6", "H8", "H9"]),
        ["66", "1290", "12", "228"]
    );
    assert_eq!(
        texts(&["H10", "H11", "H12"]),
        ["TRUE", "{=TABLE(,A1)}", "{=TABLE(B1,A1)}"]
    );
    assert_eq!(model.get_text_a1("A1"), "1", "the input keeps its value");
}

/// A result recomputes when the model changes, and text and error values substitute too.
#[test]
fn results_follow_edits_and_carry_any_value_type() {
    let mut model = new_empty_model();
    model.set_a1("A1", "x");
    model.set_a1("C1", "=LEN(A1)&\"/\"&ISERROR(A1)");
    model.set_a1("B2", "abc");
    model.set_a1("B3", "=1/0");
    model
        .add_data_table(0, table((2, 3, 3, 3), None, Some((1, 1))))
        .unwrap();
    model.evaluate();
    assert_eq!(model.get_text_a1("C2"), "3/FALSE");
    assert_eq!(
        model.get_text_a1("C3"),
        "#DIV/0!",
        "an error input propagates"
    );
    model.set_a1("C1", "=A1&\"!\"");
    model.evaluate();
    assert_eq!(model.get_text_a1("C2"), "abc!");
    assert_eq!(model.get_text_a1("C3"), "#DIV/0!");
}

/// A deleted input cell (`del1`) makes every result `#REF!`; a table needs an input, a row above
/// and a column left of its results, and may not overlap another.
#[test]
fn invalid_tables_are_refused_and_a_deleted_input_is_ref() {
    let mut model = new_empty_model();
    model.set_a1("B1", "=1");
    let deleted = DataTable {
        range: (2, 2, 2, 2),
        row_input: None,
        column_input: Some(DataTableInput::Deleted),
    };
    model.add_data_table(0, deleted).unwrap();
    model.evaluate();
    assert_eq!(model.get_text_a1("B2"), "#REF!");
    assert!(model
        .add_data_table(0, table((1, 2, 2, 2), None, Some((1, 1))))
        .is_err());
    assert!(model
        .add_data_table(0, table((3, 3, 3, 3), None, None))
        .is_err());
    assert!(model
        .add_data_table(0, table((2, 2, 3, 3), None, Some((5, 5))))
        .is_err());
    assert!(model
        .add_data_table(0, table((5, 5, 5, 5), None, Some((5, 5))))
        .is_err());
    assert!(model.remove_data_table(0, (2, 2, 2, 2)));
}

/// Directive 5: a table is bounded in cells and per sheet, and the rules hold whoever adds it
/// (the importer runs the same `DataTable::check`; recalc.md, DEVIATION).
#[test]
fn tables_are_bounded() {
    use crate::types::{DataTableError, MAX_DATA_TABLES_PER_SHEET};
    let mut model = new_empty_model();
    let whole_sheet = table((2, 2, 1_048_576, 16_384), None, Some((1, 1)));
    assert_eq!(
        model.add_data_table(0, whole_sheet),
        Err(DataTableError::TooLarge)
    );
    assert_eq!(
        model.add_data_table(0, table((2, 2, 3, 3), None, Some((2, 2)))),
        Err(DataTableError::BadInput)
    );
    assert_eq!(
        model.add_data_table(7, table((2, 2, 2, 2), None, Some((1, 1)))),
        Err(DataTableError::NoSheet)
    );
    for i in 0..MAX_DATA_TABLES_PER_SHEET {
        let row = 2 + i32::try_from(i).unwrap();
        model
            .add_data_table(0, table((row, 2, row, 2), None, Some((1, 1))))
            .unwrap();
    }
    assert_eq!(
        model.add_data_table(0, table((2, 4, 2, 4), None, Some((1, 1)))),
        Err(DataTableError::TooLarge)
    );
    assert_eq!(
        model.add_data_table(0, table((2, 2, 2, 2), None, Some((1, 1)))),
        Err(DataTableError::TooLarge)
    );
}

/// The column table of probes/t0452-data-table-row-column-edits: results B4:B6, formula B3
/// `=D1*10+B1`, column input D1, values 1-3 in A4:A6; H1 sums the results and H2 reads B5.
fn edit_model() -> crate::model::Model {
    let mut model = new_empty_model();
    for (cell, value) in [
        ("D1", "1"),
        ("B1", "2"),
        ("B3", "=D1*10+B1"),
        ("A4", "1"),
        ("A5", "2"),
        ("A6", "3"),
        ("H1", "=SUM(B4:B6)"),
        ("H2", "=FORMULATEXT(B5)"),
        ("H3", "=B5"),
    ] {
        model.set_a1(cell, value);
    }
    model
        .add_data_table(0, table((4, 2, 6, 2), None, Some((1, 4))))
        .unwrap();
    model.evaluate();
    model
}

/// probes/t0452-data-table-row-column-edits: an insert above or left of the table's region moves
/// the table and its input; one below or right leaves the table and moves only what it passes.
#[test]
fn inserts_around_a_data_table_move_it() {
    let mut model = edit_model();
    model.insert_rows(0, 1, 1).unwrap();
    model.evaluate();
    assert_eq!(
        model.workbook.worksheets[0].data_tables[0],
        table((5, 2, 7, 2), None, Some((2, 4)))
    );
    assert_eq!(model.get_text_a1("H2"), "66");
    assert_eq!(model.get_text_a1("H3"), "{=TABLE(,D2)}");
    let mut model = edit_model();
    model.insert_rows(0, 3, 1).unwrap();
    model.evaluate();
    assert_eq!(
        model.workbook.worksheets[0].data_tables[0],
        table((5, 2, 7, 2), None, Some((1, 4)))
    );
    assert_eq!(model.get_text_a1("H1"), "66");
    assert_eq!(model.get_text_a1("H4"), "22");
    let mut model = edit_model();
    model.insert_columns(0, 1, 1).unwrap();
    model.evaluate();
    assert_eq!(
        model.workbook.worksheets[0].data_tables[0],
        table((4, 3, 6, 3), None, Some((1, 5)))
    );
    assert_eq!(model.get_text_a1("I1"), "66");
    assert_eq!(model.get_text_a1("I2"), "{=TABLE(,E1)}");
    let mut model = edit_model();
    model.insert_columns(0, 3, 1).unwrap();
    model.evaluate();
    assert_eq!(
        model.workbook.worksheets[0].data_tables[0],
        table((4, 2, 6, 2), None, Some((1, 5)))
    );
    assert_eq!(model.get_text_a1("I3"), "22");
}

/// Excel refuses an insert or delete of rows or columns that cuts through a data table's region
/// (its results with the row above and the column left of them) and leaves the sheet as it was.
#[test]
fn edits_through_a_data_table_are_refused() {
    let mut model = edit_model();
    let refused = "Cannot change part of a data table".to_string();
    assert_eq!(model.insert_rows(0, 4, 1), Err(refused.clone()));
    assert_eq!(model.insert_rows(0, 5, 1), Err(refused.clone()));
    assert_eq!(model.delete_rows(0, 3, 1), Err(refused.clone()));
    assert_eq!(model.delete_rows(0, 4, 3), Err(refused.clone()));
    assert_eq!(model.insert_columns(0, 2, 1), Err(refused.clone()));
    assert_eq!(model.delete_columns(0, 1, 1), Err(refused.clone()));
    assert_eq!(model.delete_columns(0, 2, 1), Err(refused.clone()));
    assert_eq!(model.move_rows_action(0, 1, 1, 4), Err(refused.clone()));
    assert_eq!(model.move_columns_action(0, 3, 1, -1), Err(refused.clone()));
    assert_eq!(
        model.set_user_input(0, 5, 2, "9".to_string()),
        Err(refused.clone())
    );
    assert_eq!(model.set_user_input(0, 4, 2, String::new()), Err(refused));
    model.evaluate();
    assert_eq!(
        model.workbook.worksheets[0].data_tables[0],
        table((4, 2, 6, 2), None, Some((1, 4)))
    );
    assert_eq!(model.get_text_a1("H1"), "66");
    assert_eq!(model.get_text_a1("A5"), "2");
}

/// Deleting the input's row or column makes the input `#REF!` and every result `#REF!`; deleting
/// every row or column of the region deletes the table, and its readers are `#REF!`.
#[test]
fn deletes_around_a_data_table_shrink_or_delete_it() {
    let mut model = edit_model();
    model.delete_rows(0, 1, 1).unwrap();
    model.evaluate();
    let deleted = DataTable {
        range: (3, 2, 5, 2),
        row_input: None,
        column_input: Some(DataTableInput::Deleted),
    };
    assert_eq!(model.workbook.worksheets[0].data_tables[0], deleted);
    assert_eq!(model.get_text_a1("B4"), "#REF!");
    assert_eq!(model.get_text_a1("H1"), "{=TABLE(,#REF!)}");
    let mut model = edit_model();
    model.delete_columns(0, 4, 1).unwrap();
    model.evaluate();
    assert_eq!(model.get_text_a1("B5"), "#REF!");
    assert_eq!(model.get_text_a1("G2"), "{=TABLE(,#REF!)}");
    let mut model = edit_model();
    model.delete_rows(0, 2, 6).unwrap();
    model.evaluate();
    assert!(model.workbook.worksheets[0].data_tables.is_empty());
    assert_eq!(model.get_text_a1("H1"), "#REF!");
    let mut model = edit_model();
    model.delete_columns(0, 1, 2).unwrap();
    model.evaluate();
    assert!(model.workbook.worksheets[0].data_tables.is_empty());
    assert_eq!(model.get_text_a1("F1"), "#REF!");
}

fn area(row: i32, column: i32, width: i32, height: i32) -> crate::expressions::types::Area {
    crate::expressions::types::Area {
        sheet: 0,
        row,
        column,
        width,
        height,
    }
}

/// Excel accepts a clear that covers every result of a table (B4:B6 alone, the region A3:B6 or
/// more) and deletes the table, and refuses one that covers only some results (B5:C5), contents
/// or all (probed by upload, recalc.md).
#[test]
fn a_clear_over_every_result_deletes_the_table_and_over_some_is_refused() {
    let refused = Err("Cannot change part of a data table".to_string());
    let mut model = edit_model();
    assert_eq!(model.range_clear_contents(&area(5, 2, 2, 1)), refused);
    assert_eq!(model.range_clear_all(&area(4, 2, 1, 2)), refused);
    model.evaluate();
    assert_eq!(model.get_text_a1("H1"), "66");
    for clear in [area(4, 2, 1, 3), area(3, 1, 2, 4), area(1, 1, 8, 9)] {
        let mut model = edit_model();
        model.range_clear_contents(&clear).unwrap();
        model.evaluate();
        assert!(model.workbook.worksheets[0].data_tables.is_empty());
        assert_eq!(model.get_text_a1("B5"), "");
        let mut model = edit_model();
        model.range_clear_all(&clear).unwrap();
        assert!(model.workbook.worksheets[0].data_tables.is_empty());
    }
}

/// A paste over part of the results, or a cut from them, is refused before it changes anything
/// (our choice for the cut, unprobed); a paste over every result replaces the table.
#[test]
fn pastes_over_part_of_a_data_table_are_refused_unchanged() {
    let mut model = crate::UserModel::from_model(edit_model());
    let refused = Err("Cannot change part of a data table".to_string());
    model.merge_cells(&area(5, 3, 2, 1)).unwrap();
    model.set_selected_cell(5, 1).unwrap();
    assert_eq!(
        model.paste_csv_string(&area(5, 1, 4, 1), "7\t8\t9\t1"),
        refused
    );
    assert_eq!(model.get_model().get_merged_cells(0).unwrap().len(), 1);
    assert_eq!(model.get_formatted_cell_value(0, 5, 1), Ok("2".to_string()));
    assert_eq!(
        model.get_formatted_cell_value(0, 5, 2),
        Ok("22".to_string())
    );
    model.set_selected_cell(4, 2).unwrap();
    assert_eq!(model.paste_csv_string(&area(4, 2, 1, 3), "1\n2\n3"), Ok(()));
    assert!(model.get_model().workbook.worksheets[0]
        .data_tables
        .is_empty());
    assert_eq!(model.get_formatted_cell_value(0, 1, 8), Ok("6".to_string()));
}

/// A fill whose target holds a result cell is refused before it changes anything (our choice,
/// unprobed): each write into the results would be.
#[test]
fn fills_into_a_data_table_are_refused_unchanged() {
    let mut model = crate::UserModel::from_model(edit_model());
    let refused = Err("Cannot change part of a data table".to_string());
    model.merge_cells(&area(5, 3, 2, 1)).unwrap();
    assert_eq!(model.auto_fill_rows(&area(3, 2, 3, 1), 6), refused);
    assert_eq!(model.auto_fill_columns(&area(5, 1, 1, 1), 4), refused);
    assert_eq!(model.get_model().get_merged_cells(0).unwrap().len(), 1);
    assert_eq!(
        model.get_formatted_cell_value(0, 5, 2),
        Ok("22".to_string())
    );
    assert_eq!(model.get_formatted_cell_value(0, 3, 3), Ok(String::new()));
    model.auto_fill_rows(&area(1, 4, 1, 1), 2).unwrap();
}
