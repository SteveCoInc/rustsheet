#![allow(clippy::unwrap_used, clippy::panic, reason = "tests")]
//! Every value cell of the oracle-rendered aggregate probes, against the result tree: the
//! source, the definition and the expected cells all come from the probes' case and sidecar
//! files (probes/t4135-pivot-aggregates, t4589-pivot-total-order, t4614-pivot-total-order-2,
//! t4137-pivot-captions), read through the sidecar's own line encoding (`rowItems`/`colItems`).

use serde_json::Value as Json;

use super::*;
use crate::pivot::probe_cases::{address, position, probe_pivot, read, subtotal_named};
use crate::pivot::{
    CacheBuilder, CellArea, DataField, ItemType, PageField, PivotItem, RecordValue,
    MAX_RECORD_VALUES,
};

/// One line of a sidecar's `rowItems`/`colItems`: the cache items of its real fields, its data
/// field and, for a custom subtotal line, its function.
struct Line {
    items: Vec<u32>,
    data: usize,
    function: Option<Aggregate>,
}

/// Decodes `items` (each `x` after the `r` items it repeats from the line before) over `axis`.
fn lines(items: &Json, axis: &[AxisField]) -> Vec<Line> {
    let mut previous: Vec<u32> = Vec::new();
    let mut out = Vec::new();
    for line in items.as_array().unwrap() {
        let repeat = usize::try_from(line["r"].as_u64().unwrap_or(0)).unwrap();
        let mut full: Vec<u32> = previous.iter().copied().take(repeat).collect();
        full.extend(
            line["x"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| u32::try_from(x.as_u64().unwrap()).unwrap()),
        );
        previous.clone_from(&full);
        let kind = line["t"].as_str().unwrap_or("data");
        let items = if kind == "grand" {
            Vec::new()
        } else {
            full.iter()
                .zip(axis)
                .filter(|(_, f)| **f != AxisField::Values)
                .map(|(x, _)| *x)
                .collect()
        };
        out.push(Line {
            items,
            data: usize::try_from(line["i"].as_u64().unwrap_or(0)).unwrap(),
            function: match kind {
                "data" | "grand" | "default" => None,
                custom => Some(subtotal_named(custom)),
            },
        });
    }
    out
}

/// Checks every value cell of the sheet `name`'s pivot of probe `stem`; returns the number of
/// cells checked.
fn check(stem: &str, name: &str) -> usize {
    let (case, oracle) = (read(stem, "case"), read(stem, "oracle"));
    let (cache, table) = probe_pivot(&case, name);
    let mut work = MAX_PIVOT_WORK;
    let result = compute(&table, &cache, &mut work).unwrap();
    let pivot = oracle["pivots"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["sheet"] == name)
        .unwrap();
    let location = pivot["location"].as_str().unwrap();
    let (left, top) = position(location.split(':').next().unwrap());
    let first_row = u32::try_from(pivot["first_data_row"].as_u64().unwrap()).unwrap();
    let first_column = u32::try_from(pivot["first_data_col"].as_u64().unwrap()).unwrap();
    let values_on_rows = table.row_fields.contains(&AxisField::Values);
    let row_lines = lines(&pivot["row_items"], &table.row_fields);
    let column_lines = lines(&pivot["col_items"], &table.column_fields);
    let error_caption = table.options.error_caption.clone();
    let missing_caption = table.options.missing_caption.clone();
    let mut wrong = Vec::new();
    let mut checked = 0;
    for (i, row_line) in (0u32..).zip(&row_lines) {
        for (j, column_line) in (0u32..).zip(&column_lines) {
            let cell = address(left + first_column + j, top + first_row + i);
            let data = if values_on_rows {
                row_line.data
            } else {
                column_line.data
            };
            let function = row_line
                .function
                .or(column_line.function)
                .unwrap_or(table.data_fields[data].function);
            let actual = match (
                result.rows().find(&row_line.items),
                result.columns().find(&column_line.items),
            ) {
                (Some(row), Some(column)) => result.value_as(row, column, data, function),
                _ => None,
            };
            let expected = &pivot["cells"][&cell];
            let text = expected["v"].as_str();
            let ok = match (expected["t"].as_str(), &actual) {
                (None | Some("empty"), None) => true,
                (Some("n"), Some(Ok(n))) => expected["v"].as_f64() == Some(*n),
                (Some("e"), Some(Err(e))) => text == Some(e.to_string().as_str()),
                (Some("s"), Some(Err(_))) => {
                    table.options.show_error && text == error_caption.as_deref()
                }
                (Some("s"), None) => text == missing_caption.as_deref(),
                _ => false,
            };
            if !ok {
                wrong.push(format!(
                    "{name}!{cell}: Excel {expected}, result {actual:?}"
                ));
            }
            checked += 1;
        }
    }
    assert!(wrong.is_empty(), "{stem}:\n{}", wrong.join("\n"));
    checked
}

#[test]
fn every_t4135_aggregate_cell_is_excels() {
    // Eleven aggregates over numbers, text, blanks, booleans, an error and decimals, with Kind
    // on columns (A_*), and the eleven over the error field under errorCaption (EC).
    let sheets = [
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
    ];
    let checked: usize = sheets
        .iter()
        .map(|s| check("t4135-pivot-aggregates", s))
        .sum();
    assert_eq!(checked, 11 * 3 * 18 + 3 * 11);
}

#[test]
fn every_t4589_total_order_cell_is_excels() {
    for sheet in ["S1", "E1", "E2", "E3", "E4"] {
        check("t4589-pivot-total-order", sheet);
    }
}

#[test]
fn every_t4614_total_order_cell_is_excels() {
    for sheet in ["S2", "S3", "P2", "C1", "C2", "C3", "C4"] {
        check("t4614-pivot-total-order-2", sheet);
    }
}

#[test]
fn every_t4616_total_order_cell_is_excels() {
    // 2-D Sum alone (S1, S4 to S6) and beside Average (S2), Average alone (S3), Var and StdDev
    // in 2-D and over one field (V1 to V4), Product (P1, P2), Count and CountNums subtotals
    // (I1 to I4) and Count beside CountNums (M1 to M3).
    for sheet in [
        "S1", "S2", "S3", "S4", "S5", "S6", "V1", "V2", "V3", "V4", "P1", "P2", "I1", "I2", "I3",
        "I4", "M1", "M2", "M3",
    ] {
        check("t4616-pivot-total-order-3", sheet);
    }
}

#[test]
fn every_t4137_caption_pivot_value_is_excels() {
    // K4: a page selection and hidden items of a page field; K5: custom Sum and Count subtotals;
    // K3: a #DIV/0! under errorCaption; K2: an empty intersection under missingCaption.
    for sheet in ["K1", "K2", "K3", "K4", "K5", "K6"] {
        check("t4137-pivot-captions", sheet);
    }
}

/// One text field `K` over `keys`, and a number field `V` over `values`, Sum of V by K on rows.
fn small(keys: &[&str], values: &[f64]) -> (PivotCache, PivotTable) {
    let mut builder = CacheBuilder::new(None, MAX_RECORD_VALUES);
    builder.add_field("K".to_string()).unwrap();
    builder.add_field("V".to_string()).unwrap();
    for (k, v) in keys.iter().zip(values) {
        builder
            .push_record(vec![
                RecordValue::Inline(CacheItem::Text((*k).to_string())),
                RecordValue::Inline(CacheItem::Number(*v)),
            ])
            .unwrap();
    }
    let cache = builder.finish();
    let area = CellArea {
        top: 0,
        left: 0,
        bottom: 0,
        right: 0,
    };
    let mut table = PivotTable::new("P".to_string(), 1, area, 0);
    let items = cache.fields[0].items.len();
    table.fields = vec![
        PivotField::new(Some(Axis::Row), items),
        PivotField::new(None, cache.fields[1].items.len()),
    ];
    table.row_fields = vec![AxisField::Field(0)];
    table.data_fields = vec![DataField {
        field: 1,
        ..DataField::default()
    }];
    (cache, table)
}

fn grand(table: &PivotTable, cache: &PivotCache) -> Option<Result<f64, Error>> {
    let mut work = MAX_PIVOT_WORK;
    compute(table, cache, &mut work).unwrap().value(0, 0, 0)
}

#[test]
fn a_page_item_that_names_no_data_item_reads_as_all() {
    let (cache, mut table) = small(&["a", "b"], &[1.0, 2.0]);
    table.fields[0].axis = Some(Axis::Page);
    table.row_fields.clear();
    table.fields[0].items = vec![
        PivotItem::data(1),
        PivotItem {
            kind: ItemType::Default,
            index: None,
            expanded: true,
            filtered: false,
        },
    ];
    let page = |item| PageField {
        field: 0,
        item,
        hierarchy: Some(-1),
    };
    // The pivot item 0 is the cache item 1 (`b`).
    table.page_fields = vec![page(Some(0))];
    assert_eq!(grand(&table, &cache), Some(Ok(2.0)));
    for malformed in [Some(1), Some(7), None] {
        table.page_fields = vec![page(malformed)];
        assert_eq!(grand(&table, &cache), Some(Ok(3.0)), "{malformed:?}");
    }
}

#[test]
fn hidden_items_filter_on_an_axis_and_not_off_every_axis() {
    let (cache, mut table) = small(&["a", "b"], &[1.0, 2.0]);
    table.fields[0].hide(0);
    assert_eq!(grand(&table, &cache), Some(Ok(2.0)));
    table.fields[0].axis = None;
    table.row_fields.clear();
    assert_eq!(grand(&table, &cache), Some(Ok(3.0)));
}

#[test]
fn the_listed_item_order_is_the_display_and_accumulation_order() {
    // t4614 S2: D1's records, Z first seen, Sum beside Average. Listed A first, the total
    // accumulates A then Z (29.599999999999998); in first-seen order Z then A, Excel's 29.6
    // (t4614 S2!B7).
    let keys = ["Z", "A", "Z", "A", "Z", "A"];
    let (cache, mut table) = small(&keys, &[6.5, 0.1, 2.3, 8.9, 2.4, 9.4]);
    table.data_fields.push(DataField {
        field: 1,
        function: Aggregate::Average,
        ..DataField::default()
    });
    assert_eq!(grand(&table, &cache), Some(Ok(29.6)));
    table.fields[0].items = vec![PivotItem::data(1), PivotItem::data(0)];
    let mut work = MAX_PIVOT_WORK;
    let result = compute(&table, &cache, &mut work).unwrap();
    let items: Vec<Option<u32>> = result.rows().nodes().iter().map(|n| n.item).collect();
    assert_eq!(items, [None, Some(1), Some(0)]);
    assert_eq!(result.value(0, 0, 0), Some(Ok(29.599_999_999_999_998)));
}

#[test]
fn an_empty_result_has_a_root_and_no_value() {
    let (cache, mut table) = small(&["a"], &[1.0]);
    table.fields[0].hide(0);
    let mut work = MAX_PIVOT_WORK;
    let result = compute(&table, &cache, &mut work).unwrap();
    assert!(result.records().is_empty());
    assert_eq!(result.rows().nodes().len(), 1);
    assert_eq!(result.value(0, 0, 0), None);
}

#[test]
fn past_the_work_it_is_given_the_compute_fails_and_spends_it_all() {
    let (cache, table) = small(&["a", "b", "c"], &[1.0, 2.0, 3.0]);
    let mut work = 5;
    assert_eq!(
        compute(&table, &cache, &mut work),
        Err(PivotComputeError::Limit)
    );
    assert_eq!(work, 0);
}

#[test]
fn a_field_past_the_cache_is_malformed() {
    let (cache, mut table) = small(&["a"], &[1.0]);
    table.data_fields[0].field = 9;
    let mut work = MAX_PIVOT_WORK;
    assert_eq!(
        compute(&table, &cache, &mut work),
        Err(PivotComputeError::Malformed)
    );
}

/// A diagonal pivot: record `i` is row item `i` and column item `i`, Sum of its index.
fn diagonal(n: u32, data: bool) -> (PivotCache, PivotTable) {
    let mut builder = CacheBuilder::new(None, MAX_RECORD_VALUES);
    for name in ["R", "C", "V"] {
        builder.add_field(name.to_string()).unwrap();
    }
    for i in 0..n {
        builder
            .push_record(vec![
                RecordValue::Inline(CacheItem::Number(f64::from(i))),
                RecordValue::Inline(CacheItem::Number(f64::from(i))),
                RecordValue::Inline(CacheItem::Number(f64::from(i))),
            ])
            .unwrap();
    }
    let cache = builder.finish();
    let area = CellArea {
        top: 0,
        left: 0,
        bottom: 0,
        right: 0,
    };
    let mut table = PivotTable::new("D".to_string(), 1, area, 0);
    let len = |f: usize| cache.fields[f].items.len();
    table.fields = vec![
        PivotField::new(Some(Axis::Row), len(0)),
        PivotField::new(Some(Axis::Column), len(1)),
        PivotField::new(None, len(2)),
    ];
    table.row_fields = vec![AxisField::Field(0)];
    table.column_fields = vec![AxisField::Field(1)];
    if data {
        table.data_fields = vec![DataField {
            field: 2,
            ..DataField::default()
        }];
    }
    (cache, table)
}

fn spent(table: &PivotTable, cache: &PivotCache) -> u64 {
    let mut work = MAX_PIVOT_WORK;
    compute(table, cache, &mut work).unwrap();
    MAX_PIVOT_WORK - work
}

#[test]
fn a_diagonal_sum_only_pivot_costs_linear_work() {
    // Each row and column total has one child cell, but the root of each axis has n children:
    // summing children by walking every axis child per total would be quadratic (2.5e9 lookups
    // here, minutes; the child index makes it a fraction of a second).
    let (small_cache, small_table) = diagonal(2_000, true);
    let (large_cache, large_table) = diagonal(50_000, true);
    let (small, large) = (
        spent(&small_table, &small_cache),
        spent(&large_table, &large_cache),
    );
    // 25 times the records: n log n sorts allow a little over 25 times the work, n² 625.
    assert!(large < small * 50, "{small} then {large}: not linear");
    let mut work = MAX_PIVOT_WORK;
    let result = compute(&large_table, &large_cache, &mut work).unwrap();
    // 0 + 1 + ... + 49,999.
    assert_eq!(result.value(0, 0, 0), Some(Ok(1_249_975_000.0)));
}

#[test]
fn a_pivot_with_no_data_field_still_pays_for_its_cells() {
    let (cache, table) = diagonal(1_000, false);
    // Four cells per record (item, both totals, grand-grand), each charged.
    assert!(spent(&table, &cache) >= 4_000);
    let mut work = 3_000;
    assert_eq!(
        compute(&table, &cache, &mut work),
        Err(PivotComputeError::Limit)
    );
}
