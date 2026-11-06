#![allow(clippy::unwrap_used, clippy::panic, reason = "tests")]
//! The layout lines and location of every oracle-rendered probe pivot in scope, against Excel's
//! saved `rowItems`, `colItems` and `location` in the probes' sidecars: the definitions come from
//! the case files through the shared probe harness (`pivot::probe_cases`), the expectations from
//! the sidecars, never from the engine.

use serde_json::Value as Json;

use super::*;
use crate::pivot::probe_cases::{address, diagonal, probe_pivot, read, subtotal_named};
use crate::pivot::{
    compute, Aggregate, Axis, CacheBuilder, CacheItem, DataField, PageField, PivotItem,
    RecordValue, MAX_PIVOT_WORK, MAX_RECORD_VALUES,
};

/// A sidecar line as `(t, r, i, x)`.
type Expected = (ItemType, u32, u32, Vec<u32>);

fn kind_named(t: &str) -> ItemType {
    match t {
        "data" => ItemType::Data,
        "default" => ItemType::Default,
        "grand" => ItemType::Grand,
        "blank" => ItemType::Blank,
        function => ItemType::Function(subtotal_named(function)),
    }
}

fn expected(items: &Json) -> Vec<Expected> {
    let number = |v: &Json| u32::try_from(v.as_u64().unwrap_or(0)).unwrap();
    items
        .as_array()
        .unwrap()
        .iter()
        .map(|line| {
            (
                kind_named(line["t"].as_str().unwrap_or("data")),
                number(&line["r"]),
                number(&line["i"]),
                line["x"].as_array().unwrap().iter().map(number).collect(),
            )
        })
        .collect()
}

fn actual(lines: &[AxisLine]) -> Vec<Expected> {
    lines
        .iter()
        .map(|l| (l.kind, l.repeat, l.data_field, l.items.clone()))
        .collect()
}

/// The A1 range of `area`.
fn range(area: &CellArea) -> String {
    let (from, to) = (
        address(area.left, area.top),
        address(area.right, area.bottom),
    );
    if from == to {
        from
    } else {
        format!("{from}:{to}")
    }
}

/// Checks the lines of sheet `name`'s pivot of probe `stem` and, with `located`, its location.
fn check(stem: &str, name: &str, located: bool) {
    let (case, oracle) = (read(stem, "case"), read(stem, "oracle"));
    let (cache, table) = probe_pivot(&case, name);
    let layout = layout(&table, &cache);
    let pivot = oracle["pivots"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["sheet"] == name)
        .unwrap();
    assert_eq!(
        actual(&layout.rows),
        expected(&pivot["row_items"]),
        "{stem} {name} rowItems"
    );
    assert_eq!(
        actual(&layout.columns),
        expected(&pivot["col_items"]),
        "{stem} {name} colItems"
    );
    if !located {
        return;
    }
    let count = |key: &str| u32::try_from(pivot[key].as_u64().unwrap()).unwrap();
    let excel = PivotGeometry {
        first_header_row: count("first_header_row"),
        first_data_row: count("first_data_row"),
        first_data_column: count("first_data_col"),
        row_page_count: count("row_page_count"),
        column_page_count: count("col_page_count"),
    };
    assert_eq!(layout.geometry, excel, "{stem} {name} geometry");
    assert_eq!(
        range(&layout.location),
        pivot["location"].as_str().unwrap(),
        "{stem} {name} location"
    );
}

fn check_all(stem: &str, sheets: &[&str]) {
    for sheet in sheets {
        check(stem, sheet, true);
    }
}

#[test]
fn the_capture_probes_lines_and_locations_are_excels() {
    // Compact with a column field (C); tabular with Values on columns (T to T4).
    check_all("t4123-pivot-capture", &["C", "T", "T2", "T3", "T4"]);
    // A #SPILL! pivot keeps its lines; its one-cell location is the grid's (T-4151).
    check("t4128-pivot-spike", "S", false);
}

#[test]
fn the_geometry_probes_lines_and_locations_are_excels() {
    // Compact, outline and tabular rows, two column fields, Values on rows and on columns, no
    // row field, no axis field, and wrapped page fields.
    check_all(
        "t4138-pivot-geometry",
        &["G1", "G2", "G3", "G4", "G5", "G6", "G7", "G8", "G9"],
    );
}

#[test]
fn the_caption_and_row_label_probes_lines_and_locations_are_excels() {
    // K4: page fields only; K5: tabular custom subtotals.
    check_all(
        "t4137-pivot-captions",
        &["K1", "K2", "K3", "K4", "K5", "K6"],
    );
    // R1: an outer row field left in outline form inside a tabular pivot.
    check_all(
        "t4583-pivot-row-labels",
        &["R1", "R2", "R3", "R4", "R5", "R6", "C1", "C2"],
    );
    // L3 and L4 split R1: `compact="0"` alone keeps the outer field's own line and top subtotals
    // but gives the inner field its own column; `outline="0"` alone gives tabular lines and its own
    // column too. Only both on share the column. L1, L7, L8: column fields; L2, L6: page fields;
    // L5: rowHeaderCaption.
    check_all(
        "t4609-pivot-row-labels-2",
        &["L1", "L2", "L3", "L4", "L5", "L6", "L7", "L8"],
    );
}

#[test]
fn the_item_and_aggregate_probes_lines_and_locations_are_excels() {
    check_all("t4134-pivot-items", &["I1", "I2", "I4", "I5"]);
    check_all(
        "t4135-pivot-aggregates",
        &[
            "A_sum",
            "A_count",
            "A_average",
            "A_max",
            "A_min",
            "A_product",
            "A_countNums",
            "A_stdDev",
            "A_stdDevp",
            "A_var",
            "A_varp",
            "EC",
        ],
    );
    check_all("t4589-pivot-total-order", &["S1", "E1", "E2", "E3", "E4"]);
    check_all(
        "t4614-pivot-total-order-2",
        &["S2", "S3", "P2", "C1", "C2", "C3", "C4"],
    );
    check_all(
        "t4616-pivot-total-order-3",
        &[
            "S1", "S2", "S3", "S4", "S5", "S6", "V1", "V2", "V3", "V4", "P1", "P2", "I1", "I2",
            "I3", "I4", "M1", "M2", "M3",
        ],
    );
}

/// Region (a, a, b) by Product (x, y, x) on rows, Sum of V, at A3.
fn two_fields() -> (PivotCache, PivotTable) {
    let mut builder = CacheBuilder::new(None, MAX_RECORD_VALUES);
    for name in ["Region", "Product", "V"] {
        builder.add_field(name.to_string()).unwrap();
    }
    for (region, product, v) in [("a", "x", 1.0), ("a", "y", 2.0), ("b", "x", 3.0)] {
        builder
            .push_record(vec![
                RecordValue::Inline(CacheItem::Text(region.to_string())),
                RecordValue::Inline(CacheItem::Text(product.to_string())),
                RecordValue::Inline(CacheItem::Number(v)),
            ])
            .unwrap();
    }
    let cache = builder.finish();
    let area = CellArea {
        top: 2,
        left: 0,
        bottom: 2,
        right: 0,
    };
    let mut table = PivotTable::new("P".to_string(), 1, area, 0);
    let len = |f: usize| cache.fields[f].items.len();
    table.fields = vec![
        PivotField::new(Some(Axis::Row), len(0)),
        PivotField::new(Some(Axis::Row), len(1)),
        PivotField::new(None, len(2)),
    ];
    table.row_fields = vec![AxisField::Field(0), AxisField::Field(1)];
    table.data_fields = vec![DataField {
        field: 2,
        ..DataField::default()
    }];
    (cache, table)
}

fn layout(table: &PivotTable, cache: &PivotCache) -> PivotLayout {
    let mut work = MAX_PIVOT_WORK;
    let result = compute(table, cache, &mut work).unwrap();
    lay_out(table, cache, &result, &mut work).unwrap()
}

fn line(kind: ItemType, repeat: u32, items: &[u32]) -> Expected {
    (kind, repeat, 0, items.to_vec())
}

#[test]
fn grand_totals_off_drop_their_axis_grand_line() {
    let (cache, mut table) = two_fields();
    table.fields[1].axis = Some(Axis::Column);
    table.row_fields = vec![AxisField::Field(0)];
    table.column_fields = vec![AxisField::Field(1)];
    let grand = |lines: &[AxisLine]| lines.iter().any(|l| l.kind == ItemType::Grand);
    let both = layout(&table, &cache);
    assert!(grand(&both.rows) && grand(&both.columns));
    table.options.row_grand_totals = false;
    let rows_off = layout(&table, &cache);
    assert!(!grand(&rows_off.rows) && grand(&rows_off.columns));
    table.options.column_grand_totals = false;
    let off = layout(&table, &cache);
    assert!(!grand(&off.rows) && !grand(&off.columns));
    // The two header rows, a and b; the label column, x and y.
    assert_eq!(range(&off.location), "A3:C6");
}

#[test]
fn outline_subtotals_at_the_bottom_follow_the_children() {
    let (cache, mut table) = two_fields();
    table.fields[0].subtotal_top = false;
    let rows = actual(&layout(&table, &cache).rows);
    assert_eq!(
        rows,
        [
            line(ItemType::Data, 0, &[0]),
            line(ItemType::Data, 1, &[0]),
            line(ItemType::Data, 1, &[1]),
            line(ItemType::Default, 0, &[0]),
            line(ItemType::Data, 0, &[1]),
            line(ItemType::Data, 1, &[0]),
            line(ItemType::Default, 0, &[1]),
            line(ItemType::Grand, 0, &[0]),
        ]
    );
    let values: Vec<bool> = layout(&table, &cache)
        .rows
        .iter()
        .map(|l| l.values)
        .collect();
    assert_eq!(values, [false, true, true, true, false, true, true, true]);
}

#[test]
fn a_field_without_subtotals_keeps_its_own_line_without_values() {
    let (cache, mut table) = two_fields();
    table.fields[0].default_subtotal = false;
    let result = layout(&table, &cache);
    assert_eq!(result.rows.len(), 6);
    assert!(!result.rows[0].values && result.rows[1].values);
    // Tabular: no line of its own and no subtotal line.
    table.fields[0].outline = false;
    table.fields[0].compact = false;
    let tabular = actual(&layout(&table, &cache).rows);
    assert_eq!(
        tabular,
        [
            line(ItemType::Data, 0, &[0, 0]),
            line(ItemType::Data, 1, &[1]),
            line(ItemType::Data, 0, &[1, 0]),
            line(ItemType::Grand, 0, &[0]),
        ]
    );
}

#[test]
fn an_item_is_its_position_in_the_fields_items() {
    let (cache, mut table) = two_fields();
    table.row_fields = vec![AxisField::Field(0)];
    // b listed first, a after a subtotal line; a is cache item 0, b cache item 1.
    table.fields[0].items = vec![
        PivotItem::data(1),
        PivotItem {
            kind: ItemType::Default,
            index: None,
            expanded: true,
            filtered: false,
        },
        PivotItem::data(0),
    ];
    let rows = actual(&layout(&table, &cache).rows);
    assert_eq!(
        rows,
        [
            line(ItemType::Data, 0, &[0]),
            line(ItemType::Data, 0, &[2]),
            line(ItemType::Grand, 0, &[0])
        ]
    );
    // Unlisted items take the positions after the listed ones.
    assert_eq!(positions(Some(&table.fields[0]), 4), [2, 0, 3, 4]);
    assert_eq!(positions(None, 2), [0, 1]);
}

#[test]
fn page_fields_fill_down_unless_over_then_down() {
    let (cache, mut table) = two_fields();
    table.page_fields = (0..5)
        .map(|_| PageField {
            field: 2,
            item: None,
            hierarchy: Some(-1),
        })
        .collect();
    let pages = |table: &PivotTable| {
        let g = layout(table, &cache).geometry;
        (g.row_page_count, g.column_page_count)
    };
    assert_eq!(pages(&table), (5, 1));
    table.options.page_wrap = 2;
    assert_eq!(pages(&table), (2, 3));
    table.options.page_over_then_down = true;
    assert_eq!(pages(&table), (3, 2));
    table.options.page_wrap = 0;
    assert_eq!(pages(&table), (1, 5));
}

#[test]
fn values_inside_a_field_give_one_subtotal_line_per_data_field() {
    let (cache, mut table) = two_fields();
    table.data_fields.push(DataField {
        field: 2,
        function: Aggregate::Count,
        ..DataField::default()
    });
    table.row_fields = vec![AxisField::Field(0), AxisField::Values, AxisField::Field(1)];
    // Excel writes `outlineData="1"`: each data field then has a line of its own.
    table.options.outline_data = true;
    let rows = layout(&table, &cache).rows;
    let a: Vec<(ItemType, u32, u32, Vec<u32>)> = actual(&rows).into_iter().take(8).collect();
    assert_eq!(
        a,
        [
            (ItemType::Data, 0, 0, vec![0]),
            (ItemType::Data, 1, 0, vec![0]),
            (ItemType::Data, 2, 0, vec![0]),
            (ItemType::Data, 2, 0, vec![1]),
            (ItemType::Data, 1, 1, vec![1]),
            (ItemType::Data, 2, 1, vec![0]),
            (ItemType::Data, 2, 1, vec![1]),
            (ItemType::Default, 0, 0, vec![0]),
        ]
    );
    assert_eq!(rows[8].kind, ItemType::Default);
    assert_eq!(rows[8].data_field, 1);
}

#[test]
fn past_the_work_or_the_sheet_the_layout_fails() {
    let (cache, mut table) = two_fields();
    let mut work = MAX_PIVOT_WORK;
    let result = compute(&table, &cache, &mut work).unwrap();
    let before = work;
    lay_out(&table, &cache, &result, &mut work).unwrap();
    let mut short = before - work - 1;
    assert_eq!(
        lay_out(&table, &cache, &result, &mut short),
        Err(PivotComputeError::Limit)
    );
    assert_eq!(short, 0);
    // Six lines below one header row: seven rows from the anchor.
    let last_row = u32::try_from(LAST_ROW).unwrap() - 1;
    table.location.top = last_row - 5;
    let mut work = MAX_PIVOT_WORK;
    assert_eq!(
        lay_out(&table, &cache, &result, &mut work),
        Err(PivotComputeError::Limit)
    );
    table.location.top = last_row - 6;
    assert_eq!(layout(&table, &cache).location.bottom, last_row);
}

#[test]
fn a_layout_past_the_cell_cap_fails() {
    let geometry = PivotGeometry {
        first_header_row: 1,
        first_data_row: 1,
        first_data_column: 1,
        ..PivotGeometry::default()
    };
    let (_, table) = two_fields();
    // 2,000 by 2,000 cells is the cap.
    assert!(location(&table, &geometry, [1_999, 1_999]).is_ok());
    assert_eq!(
        location(&table, &geometry, [2_000, 1_999]),
        Err(PivotComputeError::Limit)
    );
}

#[test]
fn the_column_walk_stops_at_the_cap_the_rows_leave() {
    // Item i on rows and on columns: 2,001 lines each way plus the grand totals, past 2,000 by
    // 2,000 cells. The columns stop a line past what the rows' height leaves.
    let (cache, table) = diagonal(2_001, true);
    let mut work = MAX_PIVOT_WORK;
    let result = compute(&table, &cache, &mut work).unwrap();
    assert_eq!(
        lay_out(&table, &cache, &result, &mut work),
        Err(PivotComputeError::Limit)
    );
}
