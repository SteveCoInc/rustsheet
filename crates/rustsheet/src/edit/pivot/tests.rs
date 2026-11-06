#![allow(clippy::unwrap_used, reason = "tests")]
//! Tests of `Txn::add_pivot_table` (edit/pivot.rs).

use super::*;
use crate::{Input, Workbook, WorkbookOptions};

/// Region/Sales over A1:B4 and a pivot rendered at D1:E4.
fn book() -> (Workbook, PivotTableInput) {
    let mut wb = Workbook::new(&WorkbookOptions::default()).unwrap();
    wb.edit(|tx| {
        for (a1, input) in [
            ("A1", Input::from("Region")),
            ("B1", Input::from("Sales")),
            ("A2", Input::from("East")),
            ("B2", Input::from(10.0)),
            ("A3", Input::from("West")),
            ("B3", Input::from(20.0)),
            ("A4", Input::from("East")),
            ("B4", Input::from(5.0)),
        ] {
            tx.set(a1, input)?;
        }
        Ok(())
    })
    .unwrap();
    let input = PivotTableInput {
        name: "Pv".into(),
        source: wb.range_ref("A1:B4").unwrap(),
        location: wb.range_ref("D1:E4").unwrap(),
        row: "Region".into(),
        column: None,
        value: "Sales".into(),
        function: PivotFunction::Sum,
        caption: "Sum of Sales".into(),
        group: None,
        unused_items: Vec::new(),
        show_all: false,
        column_unused_items: Vec::new(),
        column_show_all: false,
    };
    (wb, input)
}

#[test]
fn the_cache_and_definition_are_built_from_the_source_cells() {
    let (mut wb, input) = book();
    wb.edit(|tx| tx.add_pivot_table(&input)).unwrap();
    let [cache] = &wb.model.workbook.pivot_caches[..] else {
        panic!("one cache");
    };
    assert_eq!(cache.record_count, 3);
    assert_eq!(cache.fields[0].name, "Region");
    assert_eq!(cache.fields[0].items.len(), 2);
    assert_eq!(cache.fields[0].records, vec![0, 1, 0]);
    let source = cache.source.as_ref().unwrap();
    assert_eq!(
        (source.sheet_name.as_str(), source.range.as_str()),
        ("Sheet1", "A1:B4")
    );
    let [pivot] = &wb.model.workbook.pivot_tables[..] else {
        panic!("one pivot");
    };
    assert_eq!(
        pivot.location,
        CellArea {
            top: 0,
            left: 3,
            bottom: 3,
            right: 4
        }
    );
    assert_eq!(pivot.row_fields, vec![AxisField::Field(0)]);
    assert_eq!(pivot.data_fields[0].field, 1);
    assert_eq!(pivot.fields[0].axis, Some(Axis::Row));
}

/// Unused items are appended to the row field's items after the records' own, and the row
/// field takes the input's `showAll` (T-4276; probes/t4248-getpivotdata-recordless-item).
#[test]
fn unused_items_follow_the_records_items_and_show_all_is_the_row_fields() {
    let (mut wb, input) = book();
    let input = PivotTableInput {
        unused_items: vec!["North".into()],
        show_all: true,
        ..input
    };
    wb.edit(|tx| tx.add_pivot_table(&input)).unwrap();
    let [cache] = &wb.model.workbook.pivot_caches[..] else {
        panic!("one cache");
    };
    assert_eq!(
        cache.fields[0].items.last(),
        Some(&CacheItem::Text("North".into()))
    );
    assert!(!cache.fields[0].records.contains(&2));
    let pivot = &wb.model.workbook.pivot_tables[0];
    assert!(pivot.fields[0].show_all);
    assert_eq!(pivot.fields[0].hidden.len(), 1);
    // On a grouped row field their place among the group items is unprobed: refused.
    let grouped = PivotTableInput {
        group: Some(PivotRangeGroup {
            by: None,
            start: 0.0,
            end: 30.0,
            interval: 10.0,
            items: ["<0", "0-10", "10-20", "20-30", ">30"]
                .map(String::from)
                .to_vec(),
        }),
        ..input
    };
    let err = wb.edit(|tx| tx.add_pivot_table(&grouped)).unwrap_err();
    assert!(
        matches!(&err, EditError::Pivot(PivotError::GroupedWithUnusedItems)),
        "{err:?}"
    );
}

#[test]
fn bad_pivots_are_typed_errors_and_leave_no_pivot() {
    let (mut wb, input) = book();
    for (change, expected) in [
        (
            PivotTableInput {
                row: "Nope".into(),
                ..input.clone()
            },
            PivotError::UnknownField("Nope".into()),
        ),
        (
            PivotTableInput {
                value: "Region".into(),
                ..input.clone()
            },
            PivotError::RepeatedField,
        ),
        (
            PivotTableInput {
                source: wb.range_ref("A1:B1").unwrap(),
                ..input.clone()
            },
            PivotError::NoRecords,
        ),
        (
            PivotTableInput {
                source: wb.range_ref("B2:B4").unwrap(),
                ..input.clone()
            },
            PivotError::Header("B2".into()),
        ),
    ] {
        let err = wb.edit(|tx| tx.add_pivot_table(&change)).unwrap_err();
        assert!(
            matches!(&err, EditError::Pivot(e) if *e == expected),
            "{err:?}"
        );
    }
    assert!(wb.model.workbook.pivot_tables.is_empty());
    assert!(wb.model.workbook.pivot_caches.is_empty());
}

#[test]
fn a_failed_edit_removes_the_pivot() {
    let (mut wb, input) = book();
    let failed: Result<((), _), _> = wb.edit(|tx| {
        tx.add_pivot_table(&input)?;
        tx.add_pivot_table(&PivotTableInput {
            row: "Nope".into(),
            ..input.clone()
        })
    });
    assert!(failed.is_err());
    assert!(wb.model.workbook.pivot_tables.is_empty());
    assert!(wb.model.workbook.pivot_caches.is_empty());
}

/// A `Sum of S` pivot of row field `row` over `source`, grouped by `group`, laid out from
/// `location`.
fn grouped_input(
    source: RangeRef,
    location: RangeRef,
    row: &str,
    group: PivotRangeGroup,
) -> PivotTableInput {
    PivotTableInput {
        name: "Pv".into(),
        source,
        location,
        row: row.into(),
        column: None,
        value: "S".into(),
        function: PivotFunction::Sum,
        caption: "Sum of S".into(),
        group: Some(group),
        unused_items: Vec::new(),
        show_all: false,
        column_unused_items: Vec::new(),
        column_show_all: false,
    }
}

/// Rows 1 to 6 of column `col`.
fn column(wb: &Workbook, col: &str) -> Vec<Value> {
    (1..=6)
        .map(|row| wb.value(format!("{col}{row}").as_str()).unwrap())
        .collect()
}

/// A pivot grouped by 0.1 from 0 to 1 is laid out as Excel shows it (only the groups holding
/// a record), and GETPIVOTDATA reads a group by its label or its lower bound
/// (probes/t4228-pivot-range-groups).
#[test]
fn a_grouped_pivot_is_laid_out_and_answers_by_label_and_lower_bound() {
    let mut wb = Workbook::new(&WorkbookOptions::default()).unwrap();
    let items: Vec<String> = ["<0", "0-0.1", "0.1-0.2", "0.2-0.3", "0.3-0.4", "0.4-0.5"]
        .into_iter()
        .chain(["0.5-0.6", "0.6-0.7", "0.7-0.8", "0.8-0.9", "0.9-1", ">1"])
        .map(String::from)
        .collect();
    let (source, location) = (wb.range_ref("A1:B4").unwrap(), wb.range_ref("D1").unwrap());
    wb.edit(|tx| {
        for (a1, input) in [
            ("A1", Input::from("N")),
            ("B1", Input::from("S")),
            ("A2", Input::from(0.3)),
            ("B2", Input::from(1.0)),
            ("A3", Input::from(0.999_999_999_999_999_9)),
            ("B3", Input::from(2.0)),
            ("A4", Input::from(1.0)),
            ("B4", Input::from(4.0)),
            ("F1", Input::from("=GETPIVOTDATA(\"S\",D1,\"N\",0.3)")),
            (
                "F2",
                Input::from("=GETPIVOTDATA(\"S\",D1,\"N\",\"0.1-0.2\")"),
            ),
            ("F3", Input::from("=GETPIVOTDATA(\"S\",D1,\"N\",1)")),
            ("F4", Input::from("=GETPIVOTDATA(\"S\",D1,\"N\",0.35)")),
        ] {
            tx.set(a1, input)?;
        }
        tx.add_pivot_table(&grouped_input(
            source,
            location,
            "N",
            PivotRangeGroup {
                by: None,
                start: 0.0,
                end: 1.0,
                interval: 0.1,
                items,
            },
        ))
    })
    .unwrap();
    let text = |s: &str| Value::Text(s.into());
    assert_eq!(
        column(&wb, "D"),
        [
            text("Row Labels"),
            text("0.3-0.4"),
            text("0.9-1"),
            text(">1"),
            text("Grand Total"),
            Value::Empty
        ]
    );
    assert_eq!(
        column(&wb, "E")[..5],
        [
            text("Sum of S"),
            1.0.into(),
            4.0.into(),
            2.0.into(),
            7.0.into()
        ]
    );
    let ref_error = |cell: &str| matches!(wb.value(cell).unwrap(), Value::Error(_));
    assert_eq!(wb.value("F1").unwrap(), 1.0.into());
    assert!(ref_error("F2"), "a group without a record is #REF!");
    assert_eq!(wb.value("F3").unwrap(), 2.0.into());
    assert!(ref_error("F4"), "0.35 is no group's lower bound");
    let [pivot] = &wb.model.workbook.pivot_tables[..] else {
        panic!("one pivot");
    };
    assert_eq!(
        pivot.location,
        CellArea {
            top: 0,
            left: 3,
            bottom: 4,
            right: 4
        }
    );
}

/// With the row field's `showAll` on, a grouped pivot shows every group, the empty ones with a
/// blank value, and GETPIVOTDATA on an empty group is 0 (probes/t4311-pivot-showall-column-and-
/// groups H!A3, Calc E1:E7 and I1:J8).
#[test]
fn show_all_lays_out_and_answers_the_empty_groups() {
    let mut wb = Workbook::new(&WorkbookOptions::default()).unwrap();
    let items: Vec<String> = ["<0", "0-0.1", "0.1-0.2", "0.2-0.3", "0.3-0.4", ">0.4"]
        .into_iter()
        .map(String::from)
        .collect();
    let (source, location) = (wb.range_ref("A1:B5").unwrap(), wb.range_ref("D1").unwrap());
    wb.edit(|tx| {
        for (a1, input) in [
            ("A1", Input::from("N")),
            ("B1", Input::from("S")),
            ("A2", Input::from(0.05)),
            ("B2", Input::from(1.0)),
            ("A3", Input::from(0.25)),
            ("B3", Input::from(2.0)),
            ("A4", Input::from(0.27)),
            ("B4", Input::from(4.0)),
            ("A5", Input::from(0.35)),
            ("B5", Input::from(8.0)),
            ("G1", Input::from("=GETPIVOTDATA(\"S\",D1,\"N\",\"<0\")")),
            (
                "G2",
                Input::from("=GETPIVOTDATA(\"S\",D1,\"N\",\"0.1-0.2\")"),
            ),
            (
                "G3",
                Input::from("=GETPIVOTDATA(\"S\",D1,\"N\",\"0.2-0.3\")"),
            ),
        ] {
            tx.set(a1, input)?;
        }
        let group = PivotRangeGroup {
            by: None,
            start: 0.0,
            end: 0.4,
            interval: 0.1,
            items,
        };
        tx.add_pivot_table(&PivotTableInput {
            show_all: true,
            ..grouped_input(source, location, "N", group)
        })
    })
    .unwrap();
    let text = |s: &str| Value::Text(s.into());
    let labels: Vec<Value> = (1..=8)
        .map(|row| wb.value(format!("D{row}").as_str()).unwrap())
        .collect();
    assert_eq!(
        labels,
        [
            "Row Labels",
            "<0",
            "0-0.1",
            "0.1-0.2",
            "0.2-0.3",
            "0.3-0.4",
            ">0.4",
            "Grand Total"
        ]
        .map(text)
    );
    let values: Vec<Value> = (2..=8)
        .map(|row| wb.value(format!("E{row}").as_str()).unwrap())
        .collect();
    assert_eq!(
        values,
        [
            Value::Empty,
            1.0.into(),
            Value::Empty,
            6.0.into(),
            8.0.into(),
            Value::Empty,
            15.0.into()
        ]
    );
    assert_eq!(wb.value("G1").unwrap(), 0.0.into());
    assert_eq!(wb.value("G2").unwrap(), 0.0.into());
    assert_eq!(wb.value("G3").unwrap(), 6.0.into());
}

/// A column field's unused items are appended to its cache items and it takes
/// `column_show_all` (probes/t4311-pivot-showall-column-and-groups P!A13).
#[test]
fn column_unused_items_and_show_all_are_the_column_fields() {
    let (mut wb, input) = book();
    wb.edit(|tx| {
        for (a1, product) in [("C1", "Product"), ("C2", "A"), ("C3", "B"), ("C4", "B")] {
            tx.set(a1, Input::from(product))?;
        }
        Ok(())
    })
    .unwrap();
    let input = PivotTableInput {
        source: wb.range_ref("A1:C4").unwrap(),
        location: wb.range_ref("E1:H5").unwrap(),
        column: Some("Product".into()),
        column_unused_items: vec!["C".into()],
        column_show_all: true,
        ..input
    };
    wb.edit(|tx| tx.add_pivot_table(&input)).unwrap();
    let cache = &wb.model.workbook.pivot_caches[0];
    assert_eq!(
        cache.fields[2].items.last(),
        Some(&CacheItem::Text("C".into()))
    );
    let pivot = &wb.model.workbook.pivot_tables[0];
    assert!(!pivot.fields[0].show_all);
    assert!(pivot.fields[2].show_all);
}

/// A numeric grouping over a blank, or over date cells among numbers, is dropped as Excel's
/// refresh drops it: each value is its own item in record order and the blank is `(blank)`
/// (probes/t4287-pivot-group-whole-end pivot S, t4287-pivot-group-dates-as-numbers pivot P).
#[test]
fn a_numeric_grouping_over_a_blank_or_mixed_dates_is_dropped() {
    for date_cell in [None, Some("A3")] {
        let mut wb = Workbook::new(&WorkbookOptions::default()).unwrap();
        let (source, location) = (wb.range_ref("A1:B5").unwrap(), wb.range_ref("D1").unwrap());
        wb.edit(|tx| {
            for (a1, input) in [
                ("A1", Input::from("N")),
                ("B1", Input::from("S")),
                ("A2", Input::from(-1.0)),
                ("B2", Input::from(1.0)),
                ("A3", Input::from(4.0)),
                ("B3", Input::from(2.0)),
                ("B4", Input::from(4.0)),
                ("A5", Input::from(7.0)),
                ("B5", Input::from(8.0)),
            ] {
                tx.set(a1, input)?;
            }
            if let Some(a1) = date_cell {
                tx.set("A4", Input::from(10.0))?;
                tx.set_number_format(a1, "yyyy-mm-dd")?;
            }
            tx.add_pivot_table(&grouped_input(
                source,
                location,
                "N",
                PivotRangeGroup {
                    by: None,
                    start: 0.0,
                    end: 9.0,
                    interval: 3.0,
                    items: ["<x", "x", ">x"].map(String::from).to_vec(),
                },
            ))
        })
        .unwrap();
        let text = |s: &str| Value::Text(s.into());
        let third = if date_cell.is_some() {
            10.0.into()
        } else {
            text("(blank)")
        };
        assert_eq!(
            column(&wb, "D")[..6],
            [
                text("Row Labels"),
                (-1.0).into(),
                4.0.into(),
                third,
                7.0.into(),
                text("Grand Total"),
            ],
            "{date_cell:?}"
        );
    }
}

/// A date grouping over a blank is dropped as Excel's refresh drops it: each second-rounded
/// date is its own item in record order, the blank is `(blank)`, and GETPIVOTDATA takes a
/// date serial or `"(blank)"` but no group label (probes/t4236-pivot-date-groups pivot B).
#[test]
fn a_date_grouping_over_a_blank_is_dropped() {
    let mut wb = Workbook::new(&WorkbookOptions::default()).unwrap();
    let (source, location) = (wb.range_ref("A1:B5").unwrap(), wb.range_ref("D1").unwrap());
    wb.edit(|tx| {
        for (a1, input) in [
            ("A1", Input::from("D")),
            ("B1", Input::from("S")),
            ("B2", Input::from(1.0)),
            ("A3", Input::from(43590.0)),
            ("B3", Input::from(2.0)),
            ("A4", Input::from(43_465.999_994_212_965)),
            ("B4", Input::from(4.0)),
            ("A5", Input::from(43590.0)),
            ("B5", Input::from(8.0)),
            ("F1", Input::from("=GETPIVOTDATA(\"S\",D1,\"D\",43590)")),
            (
                "F2",
                Input::from("=GETPIVOTDATA(\"S\",D1,\"D\",\"(blank)\")"),
            ),
            ("F3", Input::from("=GETPIVOTDATA(\"S\",D1,\"D\",43466)")),
            ("F4", Input::from("=GETPIVOTDATA(\"S\",D1,\"D\",\"May\")")),
        ] {
            tx.set(a1, input)?;
        }
        let months = ["<1/1/2019", "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul"]
            .into_iter()
            .chain(["Aug", "Sep", "Oct", "Nov", "Dec", ">12/31/2019"]);
        tx.add_pivot_table(&grouped_input(
            source,
            location,
            "D",
            PivotRangeGroup {
                by: Some(PivotDateGroupBy::Months),
                start: 43466.0,
                end: 43830.0,
                interval: 1.0,
                items: months.map(String::from).collect(),
            },
        ))
    })
    .unwrap();
    let text = |s: &str| Value::Text(s.into());
    assert_eq!(
        column(&wb, "D"),
        [
            text("Row Labels"),
            text("(blank)"),
            43590.0.into(),
            43466.0.into(),
            text("Grand Total"),
            Value::Empty
        ]
    );
    assert_eq!(
        column(&wb, "E")[..5],
        [
            text("Sum of S"),
            1.0.into(),
            10.0.into(),
            4.0.into(),
            15.0.into()
        ]
    );
    assert_eq!(wb.value("F1").unwrap(), 10.0.into());
    assert_eq!(wb.value("F2").unwrap(), 1.0.into());
    assert_eq!(wb.value("F3").unwrap(), 4.0.into());
    assert!(matches!(wb.value("F4").unwrap(), Value::Error(_)));
}

/// The cache is read as the engine's refresh reads a source: a repeated header is numbered
/// (probes/t4585-pivot-field-names; `rustsheet_engine::pivot::read_source`).
#[test]
fn a_repeated_header_is_numbered() {
    let (mut wb, input) = book();
    wb.edit(|tx| {
        tx.set("C1", Input::from("region"))?;
        tx.set("C2", Input::from("x"))
    })
    .unwrap();
    let input = PivotTableInput {
        source: wb.range_ref("A1:C4").unwrap(),
        row: "region2".into(),
        ..input
    };
    wb.edit(|tx| tx.add_pivot_table(&input)).unwrap();
    let cache = &wb.model.workbook.pivot_caches[0];
    let names: Vec<&str> = cache.fields.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["Region", "Sales", "region2"]);
    assert_eq!(
        cache.fields[2].items,
        [CacheItem::Text("x".into()), CacheItem::Missing]
    );
    assert_eq!(
        wb.model.workbook.pivot_tables[0].row_fields,
        [AxisField::Field(2)]
    );
}
