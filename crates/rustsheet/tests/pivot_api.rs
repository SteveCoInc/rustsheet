//! The public pivot table API (SPEC-0032 "API", T-4178): `Txn::add_pivot_table`,
//! `Txn::set_pivot_table`, `Txn::refresh_pivot_table` and `Workbook::pivot_table`, over a
//! Region/Product/Sales source on Sheet1!A1:C5 and a pivot anchored at E1.

use std::error::Error;

use rustsheet::{
    CellRef, ChangeSet, EditError, Input, PivotError, PivotFunction, PivotTableInput, RangeRef,
    Value, Workbook, WorkbookOptions,
};

type Res = Result<(), Box<dyn Error>>;

/// The source: four records, East twice.
fn book() -> Result<Workbook, Box<dyn Error>> {
    let mut wb = Workbook::new(&WorkbookOptions::default())?;
    wb.edit(|tx| {
        for (a1, input) in [
            ("A1", Input::from("Region")),
            ("B1", Input::from("Product")),
            ("C1", Input::from("Sales")),
            ("A2", Input::from("East")),
            ("B2", Input::from("A")),
            ("C2", Input::from(10.0)),
            ("A3", Input::from("West")),
            ("B3", Input::from("B")),
            ("C3", Input::from(20.0)),
            ("A4", Input::from("East")),
            ("B4", Input::from("B")),
            ("C4", Input::from(5.0)),
            ("A5", Input::from("North")),
            ("B5", Input::from("A")),
            ("C5", Input::from(7.0)),
        ] {
            tx.set(a1, input)?;
        }
        Ok(())
    })?;
    Ok(wb)
}

/// `Sum of Sales` by `row`, named `Pv`, anchored at E1.
fn input(wb: &Workbook, row: &str) -> Result<PivotTableInput, Box<dyn Error>> {
    Ok(PivotTableInput::new(
        "Pv",
        wb.range_ref("A1:C5")?,
        wb.range_ref("E1")?,
        row,
        "Sales",
        PivotFunction::Sum,
    ))
}

fn text(s: &str) -> Value {
    Value::Text(s.into())
}

/// The values of `a1`, row by row.
fn values(wb: &Workbook, a1: &str) -> Result<Vec<Vec<Value>>, Box<dyn Error>> {
    let array = wb.values(a1)?;
    Ok((0..array.rows())
        .map(|r| {
            (0..array.cols())
                .map(|c| array.get(r, c).cloned().unwrap_or(Value::Empty))
                .collect()
        })
        .collect())
}

/// The cells whose input `changes` lists, in A1, with their input after.
fn inputs(changes: &ChangeSet) -> Vec<(String, Option<String>)> {
    (changes.inputs.iter())
        .map(|c| (c.cell.to_string(), c.after.clone()))
        .collect()
}

fn by_region() -> Vec<Vec<Value>> {
    vec![
        vec![text("Row Labels"), text("Sum of Sales")],
        vec![text("East"), Value::Number(15.0)],
        vec![text("West"), Value::Number(20.0)],
        vec![text("North"), Value::Number(7.0)],
        vec![text("Grand Total"), Value::Number(42.0)],
    ]
}

#[test]
fn a_new_pivot_is_laid_out_from_its_anchor_and_its_cells_are_the_change_set() -> Res {
    let mut wb = book()?;
    let pivot = input(&wb, "Region")?;
    let ((), changes) = wb.edit(|tx| tx.add_pivot_table(&pivot))?;
    assert_eq!(values(&wb, "E1:F5")?, by_region());
    let written = inputs(&changes);
    assert_eq!(written.len(), 10, "{written:?}");
    assert!(written.contains(&("E1".into(), Some("Row Labels".into()))));
    assert!(written.contains(&("F5".into(), Some("42".into()))));
    assert!(changes.inputs.iter().all(|c| c.before.is_none()));
    // The view reads the pivot back.
    let sheet = wb.sheet_id("Sheet1")?;
    let view = wb.pivot_table(sheet, "Pv").ok_or("no view")?;
    assert_eq!(view.name(), "Pv");
    assert_eq!(view.sheet(), sheet);
    assert_eq!(view.location(), wb.range_ref("E1:F5")?);
    assert_eq!(view.source(), Some(wb.range_ref("A1:C5")?));
    assert_eq!(view.row_fields(), ["Region"]);
    assert!(view.column_fields().is_empty());
    assert_eq!(view.data_fields(), ["Sum of Sales"]);
    let cells = wb.pivot_table_cells(sheet, "Pv")?;
    assert_eq!(cells.len(), 10);
    for (cell, value) in &cells {
        assert_eq!(&wb.value(*cell)?, value, "{cell}");
    }
    assert_eq!(
        cells.first(),
        Some(&(wb.cell_ref("E1")?, text("Row Labels")))
    );
    // A formula reads the pivot in the same edit's recalculation.
    wb.set("H1", r#"=GETPIVOTDATA("Sales",E1,"Region","East")"#)?;
    assert_eq!(wb.value("H1")?, Value::Number(15.0));
    Ok(())
}

#[test]
fn set_replaces_the_fields_in_place_and_clears_the_cells_it_no_longer_writes() -> Res {
    let mut wb = book()?;
    let sheet = wb.sheet_id("Sheet1")?;
    let pivot = input(&wb, "Region")?;
    wb.edit(|tx| tx.add_pivot_table(&pivot))?;
    // By Product: one row fewer, a Count.
    let by_product = PivotTableInput {
        function: PivotFunction::Count,
        caption: "Count of Sales".into(),
        ..input(&wb, "Product")?
    };
    let ((), changes) = wb.edit(|tx| tx.set_pivot_table(sheet, "Pv", &by_product))?;
    assert_eq!(
        values(&wb, "E1:F5")?,
        vec![
            vec![text("Row Labels"), text("Count of Sales")],
            vec![text("A"), Value::Number(2.0)],
            vec![text("B"), Value::Number(2.0)],
            vec![text("Grand Total"), Value::Number(4.0)],
            vec![Value::Empty, Value::Empty],
        ]
    );
    let changed = inputs(&changes);
    assert!(changed.contains(&("E5".into(), None)), "{changed:?}");
    assert!(changed.contains(&("F5".into(), None)), "{changed:?}");
    assert!(
        changed.contains(&("E2".into(), Some("A".into()))),
        "{changed:?}"
    );
    let view = wb.pivot_table(sheet, "Pv").ok_or("no view")?;
    assert_eq!(view.location(), wb.range_ref("E1:F4")?);
    assert_eq!(view.row_fields(), ["Product"]);
    assert_eq!(view.data_fields(), ["Count of Sales"]);
    // Region by Product: the pivot grows to the right, and can be renamed.
    let crossed = PivotTableInput {
        name: "Crossed".into(),
        column: Some("Product".into()),
        ..input(&wb, "Region")?
    };
    wb.edit(|tx| tx.set_pivot_table(sheet, "Pv", &crossed))?;
    assert!(wb.pivot_table(sheet, "Pv").is_none());
    let view = wb.pivot_table(sheet, "Crossed").ok_or("no view")?;
    assert_eq!(view.column_fields(), ["Product"]);
    assert_eq!(view.location(), wb.range_ref("E1:H6")?);
    assert_eq!(wb.value("H6")?, Value::Number(42.0));
    // One pivot, over one cache: the old cache was replaced, not kept beside it.
    assert_eq!(wb.pivot_places().len(), 1);
    Ok(())
}

#[test]
fn a_failed_set_leaves_the_pivot_and_its_cells_as_they_were() -> Res {
    let mut wb = book()?;
    let sheet = wb.sheet_id("Sheet1")?;
    let pivot = input(&wb, "Region")?;
    wb.edit(|tx| tx.add_pivot_table(&pivot))?;
    let before = wb.pivot_table(sheet, "Pv");
    let by_product = input(&wb, "Product")?;
    // A set that succeeds, then an op that fails: the edit rolls back as a whole.
    let failed = wb.edit(|tx| {
        tx.set_pivot_table(sheet, "Pv", &by_product)?;
        tx.set("A1", Input::Formula("bad".into()))
    });
    assert!(failed.is_err());
    assert_eq!(values(&wb, "E1:F5")?, by_region());
    assert_eq!(wb.pivot_table(sheet, "Pv"), before);
    // A set with a field the source lacks fails by itself.
    let unknown = input(&wb, "Nope")?;
    let err = wb
        .edit(|tx| tx.set_pivot_table(sheet, "Pv", &unknown))
        .err();
    assert!(
        matches!(&err, Some(EditError::Pivot(PivotError::UnknownField(f))) if f == "Nope"),
        "{err:?}"
    );
    assert_eq!(wb.pivot_table(sheet, "Pv"), before);
    Ok(())
}

#[test]
fn set_and_refresh_name_an_unknown_pivot_and_set_does_not_move_one() -> Res {
    let mut wb = book()?;
    let sheet = wb.sheet_id("Sheet1")?;
    let pivot = input(&wb, "Region")?;
    wb.edit(|tx| tx.add_pivot_table(&pivot))?;
    let unknown = |e: Option<EditError>| matches!(&e, Some(EditError::Pivot(PivotError::UnknownPivot(n))) if n == "Nope");
    assert!(unknown(
        wb.edit(|tx| tx.set_pivot_table(sheet, "Nope", &pivot))
            .err()
    ));
    assert!(unknown(
        wb.edit(|tx| tx.refresh_pivot_table(sheet, "Nope")).err()
    ));
    let moved = PivotTableInput {
        location: wb.range_ref("J1")?,
        ..pivot.clone()
    };
    let err = wb.edit(|tx| tx.set_pivot_table(sheet, "Pv", &moved)).err();
    assert!(
        matches!(err, Some(EditError::Pivot(PivotError::Moved))),
        "{err:?}"
    );
    // A second pivot of the same name on the sheet is refused.
    let twin = PivotTableInput {
        location: wb.range_ref("J1")?,
        ..pivot.clone()
    };
    let err = wb.edit(|tx| tx.add_pivot_table(&twin)).err();
    assert!(
        matches!(&err, Some(EditError::Pivot(PivotError::DuplicateName(n))) if n == "Pv"),
        "{err:?}"
    );
    assert_eq!(wb.value("J1")?, Value::Empty);
    // An unknown name has no view, nor a known one on another sheet.
    assert!(wb.pivot_table(sheet, "Nope").is_none());
    let other = wb.edit(|tx| tx.add_sheet("Other"))?.0;
    assert!(wb.pivot_table(other, "Pv").is_none());
    Ok(())
}

#[test]
fn a_source_edit_shows_only_after_a_refresh() -> Res {
    let mut wb = book()?;
    let sheet = wb.sheet_id("Sheet1")?;
    let pivot = input(&wb, "Region")?;
    wb.edit(|tx| {
        tx.add_pivot_table(&pivot)?;
        tx.set("H1", r#"=GETPIVOTDATA("Sales",E1,"Region","East")"#)
    })?;
    wb.set("C2", 100.0)?;
    assert_eq!(wb.value("F2")?, Value::Number(15.0));
    assert_eq!(wb.value("H1")?, Value::Number(15.0));
    let ((), changes) = wb.edit(|tx| tx.refresh_pivot_table(sheet, "Pv"))?;
    assert_eq!(wb.value("F2")?, Value::Number(105.0));
    assert_eq!(wb.value("F5")?, Value::Number(132.0));
    assert_eq!(wb.value("H1")?, Value::Number(105.0));
    let written = inputs(&changes);
    assert!(
        written.contains(&("F2".into(), Some("105".into()))),
        "{written:?}"
    );
    let h1 = CellRef::new(sheet, 0, 7)?;
    assert!(changes.values.iter().any(|v| v.cell == h1));
    // A new record and a new item: the refresh grows the pivot.
    let wider = PivotTableInput {
        source: wb.range_ref("A1:C6")?,
        ..pivot
    };
    wb.edit(|tx| {
        tx.set("A6", "South")?;
        tx.set("B6", "A")?;
        tx.set("C6", 1.0)?;
        tx.set_pivot_table(sheet, "Pv", &wider)
    })?;
    assert_eq!(wb.value("E6")?, text("Grand Total"));
    assert_eq!(wb.value("F6")?, Value::Number(133.0));
    Ok(())
}

#[test]
fn a_pivot_past_a_limit_is_not_added_and_writes_nothing() -> Res {
    // 2,001 row items by 2,001 column items lay out past `MAX_PIVOT_CELLS` (4,000,000 cells).
    const RECORDS: u32 = 2_001;
    let mut wb = Workbook::new(&WorkbookOptions::default())?;
    let sheet = wb.sheet_id("Sheet1")?;
    wb.set_track_changes(false);
    wb.edit(|tx| {
        tx.set("A1", "Row")?;
        tx.set("B1", "Column")?;
        tx.set("C1", "Value")?;
        for r in 1..=RECORDS {
            tx.set(CellRef::new(sheet, r, 0)?, format!("R{r}").as_str())?;
            tx.set(CellRef::new(sheet, r, 1)?, format!("C{r}").as_str())?;
            tx.set(CellRef::new(sheet, r, 2)?, f64::from(r))?;
        }
        Ok(())
    })?;
    wb.set_track_changes(true);
    let stored = wb.stored_cells(sheet)?;
    let big = PivotTableInput {
        column: Some("Column".into()),
        ..PivotTableInput::new(
            "Big",
            RangeRef::new(sheet, 0, 0, RECORDS, 2)?,
            wb.range_ref("E1")?,
            "Row",
            "Value",
            PivotFunction::Sum,
        )
    };
    let err = wb.edit(|tx| tx.add_pivot_table(&big)).err();
    assert!(
        matches!(err, Some(EditError::Pivot(PivotError::Limit))),
        "{err:?}"
    );
    assert!(wb.pivot_table(sheet, "Big").is_none());
    assert!(wb.pivot_places().is_empty());
    assert_eq!(wb.stored_cells(sheet)?, stored);
    Ok(())
}

/// crates/rustsheet-xlsx/tests/pivot_tables.xlsx: `PivotSum`, Region by Product at P!A3:E7, and
/// `PivotAvg` below it, each over its own cache of Data!A1:C7.
#[test]
fn a_loaded_pivot_is_set_and_saved() -> Res {
    let bytes = include_bytes!("../../rustsheet-xlsx/tests/pivot_tables.xlsx");
    let mut wb = Workbook::from_xlsx_bytes(bytes, &rustsheet::LoadOptions::default())?;
    let p = wb.sheet_id("P")?;
    let view = wb.pivot_table(p, "PivotSum").ok_or("no view")?;
    assert_eq!(view.location(), wb.range_ref("P!A3:E7")?);
    assert_eq!(view.source(), Some(wb.range_ref("Data!A1:C7")?));
    assert_eq!(
        (view.row_fields(), view.column_fields()),
        (&["Region".to_string()][..], &["Product".to_string()][..])
    );
    let by_region = PivotTableInput::new(
        "PivotSum",
        wb.range_ref("Data!A1:C7")?,
        wb.range_ref("P!A3")?,
        "Region",
        "Sales",
        PivotFunction::Sum,
    );
    wb.edit(|tx| tx.set_pivot_table(p, "PivotSum", &by_region))?;
    let expected = vec![
        vec![text("Row Labels"), text("Sum of Sales"), Value::Empty],
        vec![text("East"), Value::Number(22.0), Value::Empty],
        vec![text("West"), Value::Number(27.0), Value::Empty],
        vec![text("Grand Total"), Value::Number(49.0), Value::Empty],
        vec![Value::Empty, Value::Empty, Value::Empty],
    ];
    assert_eq!(values(&wb, "P!A3:C7")?, expected);
    // The other pivot keeps its cache and cells.
    assert_eq!(wb.value("P!B21")?, Value::Number(5.5));
    let saved = wb.to_xlsx_bytes(rustsheet::SaveOptions::default())?;
    let back = Workbook::from_xlsx_bytes(&saved, &rustsheet::LoadOptions::default())?;
    assert_eq!(values(&back, "P!A3:C7")?, expected);
    let view = back.pivot_table(p, "PivotSum").ok_or("no view")?;
    assert_eq!(view.location(), back.range_ref("P!A3:B6")?);
    assert!(view.column_fields().is_empty());
    Ok(())
}

#[test]
fn a_pivot_meeting_a_foreign_cell_collapses_and_its_cells_say_so() -> Res {
    let mut wb = book()?;
    let sheet = wb.sheet_id("Sheet1")?;
    let pivot = input(&wb, "Region")?;
    wb.edit(|tx| {
        tx.add_pivot_table(&pivot)?;
        tx.set("G3", "in the way")
    })?;
    // Region by Product would cover E1:H6, G3 included.
    let crossed = PivotTableInput {
        column: Some("Product".into()),
        ..pivot
    };
    wb.edit(|tx| tx.set_pivot_table(sheet, "Pv", &crossed))?;
    let spill = Value::Error(rustsheet::ErrorKind::Spill.into());
    assert_eq!(wb.value("E1")?, spill);
    assert_eq!(wb.value("F2")?, Value::Empty);
    assert_eq!(wb.value("G3")?, text("in the way"));
    let view = wb.pivot_table(sheet, "Pv").ok_or("no view")?;
    assert_eq!(view.location(), wb.range_ref("E1")?);
    assert_eq!(
        wb.pivot_table_cells(sheet, "Pv")?,
        vec![(wb.cell_ref("E1")?, spill)]
    );
    let err = wb.pivot_table_cells(sheet, "Nope").err();
    assert!(
        matches!(&err, Some(PivotError::UnknownPivot(n)) if n == "Nope"),
        "{err:?}"
    );
    Ok(())
}
