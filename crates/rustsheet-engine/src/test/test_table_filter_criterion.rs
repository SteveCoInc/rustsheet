#![allow(clippy::unwrap_used)]
//! Which table filters make a sheet's hidden rows filtered for SUBTOTAL (T-4003): only a
//! filterColumn with a criterion and a `colId` inside the table (excel-graph, 2026-11-04;
//! docs/excel-semantics/xlsx-storage.md, "Table parts in the export").

use crate::types::{FilterColumn, FilterCriterion};
use crate::Model;

/// `Sales` (A1:C7: `Cat` a,b,a,c,a,b, `Val` 1-6, `X` 1-6) filtered with `filters`, rows 3 and 5
/// hidden; returns `SUBTOTAL(9,..)` and `SUBTOTAL(109,..)` of the `Val` column.
fn sums(filters: Vec<FilterColumn>, edit: impl Fn(&mut Model)) -> (String, String) {
    let mut m = Model::new_empty("book", "en", "UTC", "en").unwrap();
    for (column, header) in ["Cat", "Val", "X"].iter().enumerate() {
        m.set_user_input(
            0,
            1,
            i32::try_from(column).unwrap() + 1,
            (*header).to_string(),
        )
        .unwrap();
    }
    for (i, cat) in ["a", "b", "a", "c", "a", "b"].iter().enumerate() {
        let row = i32::try_from(i).unwrap() + 2;
        m.set_user_input(0, row, 1, (*cat).to_string()).unwrap();
        m.set_user_input(0, row, 2, (row - 1).to_string()).unwrap();
        m.set_user_input(0, row, 3, (row - 1).to_string()).unwrap();
    }
    m.add_table(0, "Sales", (1, 1, 7, 3), true).unwrap();
    m.workbook.tables.get_mut("Sales").unwrap().filter_columns = filters;
    m.set_row_hidden(0, 3, true).unwrap();
    m.set_row_hidden(0, 5, true).unwrap();
    edit(&mut m);
    m.set_user_input(0, 1, 5, "=SUBTOTAL(9,B2:B7)".to_string())
        .unwrap();
    m.set_user_input(0, 1, 6, "=SUBTOTAL(109,B2:B7)".to_string())
        .unwrap();
    m.evaluate();
    (
        m.get_formatted_cell_value(0, 1, 5).unwrap(),
        m.get_formatted_cell_value(0, 1, 6).unwrap(),
    )
}

fn values(col_id: u32) -> FilterColumn {
    FilterColumn {
        col_id,
        criterion: FilterCriterion::Values {
            values: vec!["a".to_string(), "c".to_string()],
            blank: false,
        },
    }
}

fn pair(a: &str, b: &str) -> (String, String) {
    (a.to_string(), b.to_string())
}

#[test]
fn a_criterion_inside_the_table_filters_the_hidden_rows() {
    // A values filter on `Cat`: SUBTOTAL 9 = SUBTOTAL 109.
    assert_eq!(sums(vec![values(0)], |_| {}), pair("15", "15"));
    // A filter of another kind (top 10 read as `Other`) is a criterion too.
    let other = FilterColumn {
        col_id: 1,
        criterion: FilterCriterion::Other,
    };
    assert_eq!(sums(vec![other], |_| {}), pair("15", "15"));
    // Filtered with no kept column (the API's `filtered`): filtered, as before.
    assert_eq!(sums(Vec::new(), |_| {}), pair("15", "15"));
}

#[test]
fn a_criterion_less_or_out_of_range_filter_column_does_not() {
    // `<filterColumn colId="0"/>`: rows hidden by hand, SUBTOTAL 9 reads them (Excel 21, 15).
    let unset = FilterColumn {
        col_id: 0,
        criterion: FilterCriterion::Unset,
    };
    assert_eq!(sums(vec![unset], |_| {}), pair("21", "15"));
    // A values filter on `colId` 5 of a 3-column table.
    assert_eq!(sums(vec![values(5)], |_| {}), pair("21", "15"));
    // Either beside a criterion inside the table is filtered.
    assert_eq!(sums(vec![values(5), values(0)], |_| {}), pair("15", "15"));
}

#[test]
fn deleting_the_last_filtered_column_leaves_the_rows_hidden_by_hand() {
    // The criterion on `X` goes with its column; the export's fallback `colId="0"` has none.
    let deleted = sums(vec![values(2)], |m| m.delete_columns(0, 3, 1).unwrap());
    assert_eq!(deleted, pair("21", "15"));
    let filters = |m: &mut Model| {
        m.delete_columns(0, 3, 1).unwrap();
        assert_eq!(
            m.workbook.tables["Sales"].filter_columns,
            [FilterColumn {
                col_id: 0,
                criterion: FilterCriterion::Unset
            }]
        );
    };
    sums(vec![values(2)], filters);
    // A criterion left on another column keeps them filtered.
    let kept = sums(vec![values(0), values(2)], |m| {
        m.delete_columns(0, 3, 1).unwrap();
    });
    assert_eq!(kept, pair("15", "15"));
}
