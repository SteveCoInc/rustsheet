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
