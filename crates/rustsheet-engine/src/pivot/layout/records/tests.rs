//! The records behind every value cell of oracle-rendered probe pivots, against the cells Excel
//! saved in the probes' sidecars: the data field's function over the records' source values must
//! give Excel's value, and the grand total's records must be every record the filters keep. The
//! definitions come from the case files through the shared probe harness (`pivot::probe_cases`).

use super::*;
use crate::pivot::layout::lay_out;
use crate::pivot::probe_cases::{position, probe_pivot, read};
use crate::pivot::{compute, Aggregate, AxisField, CacheItem, MAX_PIVOT_WORK};

/// Probe sheets whose pivots hold one or more Sum, Count or Average fields over numbers:
/// one row and one column field (t4137 K2, with an empty intersection), a page field's
/// selection beside a field's hidden page items (K4), two data fields (K6), every geometry
/// (t4138), an inner row field with a hidden item (t4164 SH0, SH1H), Count and Average alone
/// (t4135), two fields per axis (t4634 T1 to T3) and Average totals (A1 to A3).
const SHEETS: &[(&str, &[&str])] = &[
    ("t4137-pivot-captions", &["K2", "K4", "K6"]),
    (
        "t4138-pivot-geometry",
        &["G1", "G2", "G3", "G4", "G5", "G6", "G7", "G8", "G9"],
    ),
    ("t4164-pivot-filter-totals", &["SH0", "SH1H"]),
    ("t4135-pivot-aggregates", &["A_sum", "A_count", "A_average"]),
    (
        "t4634-pivot-total-order-4",
        &["T1", "T2", "T3", "A1", "A2", "A3"],
    ),
];

/// The data field a value cell shows: the line's of the axis that holds Values, else the first.
fn data_field(table: &PivotTable, layout: &PivotLayout, row: u32, column: u32) -> usize {
    let has_values = |fields: &[AxisField]| fields.contains(&AxisField::Values);
    let index =
        |lines: &[AxisLine], at: u32, first: u32| lines[(at - first) as usize].data_field as usize;
    let (top, left) = (layout.location.top, layout.location.left);
    if has_values(&table.row_fields) {
        index(&layout.rows, row, top + layout.geometry.first_data_row)
    } else if has_values(&table.column_fields) {
        index(
            &layout.columns,
            column,
            left + layout.geometry.first_data_column,
        )
    } else {
        0
    }
}

/// The data field's function over `records`, as the source values give it: Count counts every
/// non-blank value, Sum and Average skip text, blanks and booleans (pivots.md "Aggregates over
/// text, blanks, booleans and errors").
fn recomputed(table: &PivotTable, cache: &PivotCache, field: usize, records: &[u32]) -> f64 {
    let data = &table.data_fields[field];
    let source = &cache.fields[data.field as usize];
    let values: Vec<&CacheItem> = (records.iter())
        .map(|&r| &source.items[source.records[r as usize] as usize])
        .collect();
    let numbers: Vec<f64> = (values.iter())
        .filter_map(|v| match v {
            CacheItem::Number(n) => Some(*n),
            _ => None,
        })
        .collect();
    let count = |n: usize| f64::from(u32::try_from(n).unwrap());
    let sum = numbers.iter().fold(0.0, |acc, x| acc + x);
    match data.function {
        Aggregate::Count => count(values.iter().filter(|v| ***v != CacheItem::Missing).count()),
        Aggregate::Sum => sum,
        Aggregate::Average if numbers.is_empty() => 0.0,
        Aggregate::Average => sum / count(numbers.len()),
        other => panic!("aggregate {other:?} is out of scope"),
    }
}

/// Checks every cell of the sidecar of sheet `name`'s pivot of probe `stem`; the count of value
/// cells checked.
fn check(stem: &str, name: &str) -> usize {
    let (case, oracle) = (read(stem, "case"), read(stem, "oracle"));
    let (cache, table) = probe_pivot(&case, name);
    let mut work = MAX_PIVOT_WORK;
    let result = compute(&table, &cache, &mut work).unwrap();
    let layout = lay_out(&table, &cache, &result, &mut work).unwrap();
    let pivot = (oracle["pivots"].as_array().unwrap().iter())
        .find(|p| p["sheet"] == name)
        .unwrap();
    let mut checked = 0;
    for (at, cell) in pivot["cells"].as_object().unwrap() {
        let (column, row) = position(at);
        let found = records_behind(&table, &cache, &result, &layout, row, column, &mut work);
        let records = found.unwrap();
        match (cell["t"].as_str().unwrap(), records) {
            ("n", Some(records)) => {
                let excel = cell["v"].as_f64().unwrap();
                let field = data_field(&table, &layout, row, column);
                let mine = recomputed(&table, &cache, field, &records);
                let close = (mine - excel).abs() <= 1e-12 * excel.abs().max(1.0);
                assert!(close, "{stem} {name} {at}: Excel {excel}, records {mine}");
                checked += 1;
            }
            // A number outside the value area: a page or label item; an error is the error's.
            // So is any other cell no record reaches.
            ("e", Some(_)) | (_, None) => {}
            // A label, a caption or an empty cell; a value cell no record reaches is empty, or
            // `missingCaption` (t4137 K2).
            (_, Some(records)) => {
                assert!(records.is_empty(), "{stem} {name} {at}: {records:?}");
            }
        }
    }
    // The grand total, the last row and column lines, covers every record the filters keep.
    let (rows, columns) = (layout.rows.len() as u32, layout.columns.len() as u32);
    let (bottom, right) = (
        layout.location.top + layout.geometry.first_data_row + rows - 1,
        layout.location.left + layout.geometry.first_data_column + columns - 1,
    );
    let total = records_behind(&table, &cache, &result, &layout, bottom, right, &mut work);
    assert_eq!(
        total.unwrap().as_deref(),
        Some(result.records()),
        "{stem} {name}"
    );
    checked
}

#[test]
fn every_value_cells_records_give_excels_value() {
    let mut checked = 0;
    for (stem, sheets) in SHEETS {
        for sheet in *sheets {
            let cells = check(stem, sheet);
            assert!(cells > 0, "{stem} {sheet}: no value cell");
            checked += cells;
        }
    }
    // Every Excel number in the value areas of the 23 pivots.
    assert_eq!(checked, 486);
}

#[test]
fn hidden_items_and_page_selections_drop_their_records() {
    // t4164 SH0: Product C hidden, so East's subtotal (B4, 13) and the grand total (B10, 23)
    // leave out East C 6 and West C 4 (Data rows 4 and 7).
    let case = read("t4164-pivot-filter-totals", "case");
    let (cache, table) = probe_pivot(&case, "SH0");
    let mut work = MAX_PIVOT_WORK;
    let result = compute(&table, &cache, &mut work).unwrap();
    let layout = lay_out(&table, &cache, &result, &mut work).unwrap();
    let behind = |a1: &str, work: &mut u64| {
        let (column, row) = position(a1);
        records_behind(&table, &cache, &result, &layout, row, column, work).unwrap()
    };
    assert_eq!(behind("B4", &mut work), Some(vec![0, 1]));
    assert_eq!(behind("B10", &mut work), Some(vec![0, 1, 3, 4]));
    // Labels and headers are no value cells, nor is a cell past the pivot.
    for a1 in ["A4", "A10", "B3", "A3", "C4", "B11", "Z99"] {
        assert_eq!(behind(a1, &mut work), None, "{a1}");
    }
    // t4137 K4: the page field Product selects A and Units hides two items.
    let case = read("t4137-pivot-captions", "case");
    let (cache, table) = probe_pivot(&case, "K4");
    let result = compute(&table, &cache, &mut work).unwrap();
    assert!(result.records().len() < cache.record_count as usize);
}

#[test]
fn the_records_cost_work_and_a_short_budget_stops_them() {
    let case = read("t4164-pivot-filter-totals", "case");
    let (cache, table) = probe_pivot(&case, "SH0");
    let mut work = MAX_PIVOT_WORK;
    let result = compute(&table, &cache, &mut work).unwrap();
    let layout = lay_out(&table, &cache, &result, &mut work).unwrap();
    let (column, row) = position("B10");
    let mut short = 1;
    let found = records_behind(&table, &cache, &result, &layout, row, column, &mut short);
    assert_eq!(found, Err(PivotComputeError::Limit));
    assert_eq!(short, 0);
}

#[test]
fn a_pivot_with_no_data_field_has_no_value_cell() {
    let case = read("t4164-pivot-filter-totals", "case");
    let (cache, mut table) = probe_pivot(&case, "SH0");
    table.data_fields.clear();
    let mut work = MAX_PIVOT_WORK;
    let result = compute(&table, &cache, &mut work).unwrap();
    let layout = lay_out(&table, &cache, &result, &mut work).unwrap();
    let (column, row) = position("B10");
    let found = records_behind(&table, &cache, &result, &layout, row, column, &mut work);
    assert_eq!(found, Ok(None));
}
