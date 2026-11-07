//! Data tables through the facade: writes into their results are refused, and rollback removes an
//! added one (T-0452, docs/excel-semantics/recalc.md).

use super::{book, cell, dump};
use crate::edit::{ChangeSet, EditError, Input};
use crate::value::Value;
use crate::Workbook;

/// A column data table on B2:B4 (formula B1 `=A1*10`, column input A1, values 1-3 in A2:A4).
fn data_table_book() -> Workbook {
    let mut wb = book();
    let range = wb.range_ref("B2:B4").unwrap();
    let input = cell(&wb, "A1");
    wb.edit(|tx| {
        tx.set("A1", 1.0)?;
        tx.set("B1", Input::Formula("=A1*10".into()))?;
        tx.set("A2", 1.0)?;
        tx.set("A3", 2.0)?;
        tx.set("A4", 3.0)?;
        tx.add_data_table(range, None, Some(input))
    })
    .unwrap();
    wb
}

/// Excel refuses every write into a data table's results, a clear included (T-0452,
/// docs/excel-semantics/recalc.md); the cells beside them stay writable.
#[test]
fn writes_into_data_table_results_are_refused() {
    let mut wb = data_table_book();
    assert_eq!(wb.value("B3").unwrap(), Value::Number(20.0));
    let before = dump(&wb);
    for input in [
        Input::Value(Value::Number(99.0)),
        Input::Clear,
        Input::Formula("=1".into()),
        Input::Parse("7".into()),
    ] {
        for at in ["B2", "B3", "B4"] {
            let err = wb.set(at, input.clone()).unwrap_err();
            assert!(
                matches!(&err, EditError::InvalidInput { message, .. }
                    if message.contains("part of a data table")),
                "{at}: {err:?}"
            );
            assert_eq!(dump(&wb), before, "{at}");
        }
    }
    wb.set("A3", 7.0).unwrap();
    assert_eq!(wb.value("B3").unwrap(), Value::Number(70.0));
    wb.set("C3", 5.0).unwrap();
}

/// Rolling back an edit that added a data table removes it: its result cells are writable again.
#[test]
fn rollback_removes_an_added_data_table() {
    let mut wb = book();
    wb.set("A2", 1.0).unwrap();
    let before = dump(&wb);
    let range = wb.range_ref("B2:B4").unwrap();
    let input = cell(&wb, "A1");
    let result: Result<((), ChangeSet), _> = wb.edit(|tx| {
        tx.set("B1", Input::Formula("=A1*10".into()))?;
        tx.add_data_table(range, None, Some(input))?;
        Err(EditError::Engine("abort".into()))
    });
    assert!(result.is_err());
    assert_eq!(dump(&wb), before);
    wb.set("B3", 5.0).unwrap();
}
