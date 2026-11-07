#![allow(clippy::unwrap_used)]
//! A column delete that takes a filtered table's last criterion unfilters the table and shows the
//! rows its filter hid; other column edits keep the filter (probes/t4371-filtered-table-*,
//! excel-graph 2026-11-07, T-4371; docs/excel-semantics/numbers.md).

use crate::{Model, UserModel};

/// Table `Sales` on A1:C7 (Cat a,b,a,c,a,b; Val 1-6; Ext 10-60), filtered through the API, rows
/// 3, 5 and 7 hidden, and E1 = `SUBTOTAL(103,Sales[Val])`, E2 = `SUBTOTAL(3,Sales[Val])`.
fn filtered() -> Model {
    let mut m = Model::new_empty("book", "en", "UTC", "en").unwrap();
    for (column, header) in [(1, "Cat"), (2, "Val"), (3, "Ext")] {
        m.set_user_input(0, 1, column, header.to_string()).unwrap();
    }
    for (row, cat) in (2..=7).zip(["a", "b", "a", "c", "a", "b"]) {
        m.set_user_input(0, row, 1, cat.to_string()).unwrap();
        m.set_user_input(0, row, 2, (row - 1).to_string()).unwrap();
        m.set_user_input(0, row, 3, (10 * (row - 1)).to_string())
            .unwrap();
    }
    m.add_table(0, "Sales", (1, 1, 7, 3), true).unwrap();
    for row in [3, 5, 7] {
        m.set_row_hidden(0, row, true).unwrap();
    }
    m.set_user_input(0, 1, 5, "=SUBTOTAL(103,Sales[Val])".to_string())
        .unwrap();
    m.set_user_input(0, 2, 5, "=SUBTOTAL(3,Sales[Val])".to_string())
        .unwrap();
    m.evaluate();
    m
}

fn hidden_rows(m: &Model) -> Vec<i32> {
    (1..=8)
        .filter(|&r| m.is_row_hidden(0, r).unwrap())
        .collect()
}

/// SUBTOTAL 103 and 3 over `Sales[Val]`, wherever the delete moved them.
fn counts(m: &Model, column: i32) -> (String, String) {
    let v = |row| m.get_formatted_cell_value(0, row, column).unwrap();
    (v(1), v(2))
}

#[test]
fn deleting_the_filtered_first_column_unfilters_and_shows_the_rows() {
    let mut m = filtered();
    m.delete_columns(0, 1, 1).unwrap();
    m.evaluate();
    assert!(!m.workbook.tables["Sales"].has_filters);
    assert_eq!(hidden_rows(&m), Vec::<i32>::new());
    assert_eq!(counts(&m, 4), ("6".to_string(), "6".to_string()));
}

#[test]
fn other_column_edits_keep_the_filter_and_the_hidden_rows() {
    for (insert, column) in [(false, 3), (true, 2), (true, 1)] {
        let mut m = filtered();
        if insert {
            m.insert_columns(0, column, 1).unwrap();
        } else {
            m.delete_columns(0, column, 1).unwrap();
        }
        m.evaluate();
        let at = if insert { 6 } else { 4 };
        assert!(m.workbook.tables["Sales"].has_filters, "{insert} {column}");
        assert_eq!(hidden_rows(&m), vec![3, 5, 7], "{insert} {column}");
        assert_eq!(counts(&m, at), ("3".to_string(), "3".to_string()));
    }
}

/// A row hidden outside the table stays hidden: only the table's data rows are the filter's.
#[test]
fn a_hidden_row_outside_the_table_stays_hidden() {
    let mut m = filtered();
    m.set_row_hidden(0, 8, true).unwrap();
    m.delete_columns(0, 1, 1).unwrap();
    assert_eq!(hidden_rows(&m), vec![8]);
}

/// Undo hides the rows again and restores the filter; redo shows them again.
#[test]
fn undo_of_the_delete_hides_the_rows_and_refilters() {
    let mut user = UserModel::from_model(filtered());
    user.delete_columns(0, 1, 1).unwrap();
    assert_eq!(hidden_rows(user.get_model()), Vec::<i32>::new());
    user.undo().unwrap();
    let m = user.get_model();
    assert!(m.workbook.tables["Sales"].has_filters);
    assert_eq!(hidden_rows(m), vec![3, 5, 7]);
    assert_eq!(counts(m, 5), ("3".to_string(), "3".to_string()));
    user.redo().unwrap();
    let m = user.get_model();
    assert!(!m.workbook.tables["Sales"].has_filters);
    assert_eq!(hidden_rows(m), Vec::<i32>::new());
    assert_eq!(counts(m, 4), ("6".to_string(), "6".to_string()));
}

/// [`filtered`] with two criteria: a values filter on Cat keeping "a" and "c" (rows 3 and 7) and
/// one on Ext rejecting 40 (row 5), probes/t4755-two-criteria-filtered-table-* (T-4755).
fn two_criteria() -> Model {
    let mut m = filtered();
    m.filter_table_column("Sales", 0, &[3, 7]).unwrap();
    m.filter_table_column("Sales", 2, &[5]).unwrap();
    m
}

/// A delete of one criterion's column shows only the rows the kept criterion keeps
/// (probes/t4755-two-criteria-filtered-table-delete-one: rows 3 and 7 shown, row 5 still hidden;
/// T-4803), and the undo hides them again.
#[test]
fn deleting_one_of_two_criteria_shows_only_the_rows_it_alone_hid() {
    let mut user = UserModel::from_model(two_criteria());
    user.delete_columns(0, 1, 1).unwrap();
    let m = user.get_model();
    assert!(m.workbook.tables["Sales"].has_filters);
    assert_eq!(hidden_rows(m), vec![5]);
    assert_eq!(counts(m, 4), ("5".to_string(), "5".to_string()));
    user.undo().unwrap();
    assert_eq!(hidden_rows(user.get_model()), vec![3, 5, 7]);
}

/// A row the kept criterion keeps but that was hidden on its own is shown too, as the one-
/// criterion delete shows it (probes/t4755-hand-hidden-filtered-table-delete-filtered-column);
/// a delete of a column with no criterion re-applies nothing.
#[test]
fn the_kept_criterion_decides_every_hidden_data_row() {
    let mut m = two_criteria();
    m.set_row_hidden(0, 4, true).unwrap();
    m.delete_columns(0, 2, 1).unwrap();
    assert_eq!(hidden_rows(&m), vec![3, 4, 5, 7]);
    m.delete_columns(0, 1, 1).unwrap();
    assert_eq!(hidden_rows(&m), vec![5]);
}

/// A kept criterion that nothing here evaluates keeps every hidden row hidden.
#[test]
fn a_kept_criterion_that_cannot_be_told_keeps_the_rows_hidden() {
    let mut m = two_criteria();
    if let Some(table) = m.workbook.tables.get_mut("Sales") {
        table.filter_columns[1].criterion = crate::types::FilterCriterion::Other;
    }
    m.delete_columns(0, 1, 1).unwrap();
    assert_eq!(hidden_rows(&m), vec![3, 5, 7]);
}

/// The values filter keeps the displayed text of the other data rows, and refuses a rejected
/// row whose text a kept row shows (no values filter tells them apart) or a row off the data.
#[test]
fn filter_table_column_derives_the_values_filter() {
    let m = two_criteria();
    let criterion = |i: usize| m.workbook.tables["Sales"].filter_columns[i].clone();
    assert_eq!(criterion(0).col_id, 0);
    assert_eq!(
        criterion(1).criterion,
        crate::types::FilterCriterion::Values {
            values: ["10", "20", "30", "50", "60"].map(String::from).to_vec(),
            blank: false,
            calendar_type: None,
            date_groups: Vec::new(),
        }
    );
    let mut m = filtered();
    assert!(m.filter_table_column("Sales", 0, &[2]).is_err());
    assert!(m.filter_table_column("Sales", 0, &[1]).is_err());
    assert!(m.filter_table_column("Sales", 3, &[5]).is_err());
    assert!(m.filter_table_column("Nope", 0, &[5]).is_err());
}
