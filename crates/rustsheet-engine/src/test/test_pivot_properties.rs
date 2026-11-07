#![allow(clippy::unwrap_used, clippy::panic, reason = "tests")]

//! Metamorphic and property tests over generated pivots (T-4155; plan §6 layer 5 of
//! docs/research/pivot-tables-2026-11-04/plan.md). No oracle: each property relates the engine
//! to itself or to a brute-force aggregation of the generated records, over seeded random
//! sources (`Rng`), so a failure names its seed. The sources are Region and Product texts and
//! integer Sales, so every sum is exact in any order; the pivot is a Sum value field with grand
//! totals, Excel's default report.
//! - A refresh of a refreshed pivot changes nothing (cells, location, geometry, cache).
//! - `GETPIVOTDATA` of each non-empty value cell's tuple returns that cell, the grand totals
//!   included (the field left out of the tuple).
//! - Each value cell is the sum of its (Region, Product) pair's records, and each grand total
//!   the sum of its line's records.
//! - Permuting the source rows changes only order-dependent output: the value of each (Region,
//!   Product) pair, compared case-folded, stays; which casing labels an item (first seen) and
//!   the item order may change.
//!
//! Import, export and import as a fixed point is in the xlsx crate
//! (`export/pivots/tests.rs`, T-4756), which reuses this generator through `test_support`.

use std::collections::BTreeMap;

use crate::test::util::{new_empty_model, Rng};
use crate::test_support::{
    add_sum_pivot, pivot_records, pivot_sheet_text, refresh_first_pivot, PivotRecord,
};
use crate::Model;

/// Seeds per property: enough to vary item counts, empty intersections and case variants,
/// small enough for the CI time budget (each model is a few dozen cells).
const SEEDS: u64 = 48;

/// The label the renderer writes for a grand-total line.
const GRAND_TOTAL: &str = "Grand Total";

type Record = PivotRecord;

/// The seeded records of `seed` (shared with the xlsx crate's round-trip test, T-4756).
fn records(seed: u64) -> Vec<Record> {
    pivot_records(seed)
}

/// A model with `records` and a rendered Sum of Sales pivot over them ([`add_sum_pivot`]).
fn pivot_model(records: &[Record]) -> Model {
    add_sum_pivot(new_empty_model(), records).unwrap()
}

fn refresh_pivot(model: &mut Model) {
    refresh_first_pivot(model).unwrap();
}

fn sheet_text(model: &Model) -> Vec<String> {
    pivot_sheet_text(model).unwrap()
}

/// One value cell of the rendered pivot: its 1-based sheet position, its row and column labels
/// (`Grand Total` for a total line) and its text.
struct ValueCell {
    row: i32,
    column: i32,
    region: String,
    product: String,
    text: String,
}

/// The value cells of the pivot, read off the grid through its location and geometry: the row
/// labels in the location's first column, the column labels on its header row.
fn value_cells(model: &Model) -> Vec<ValueCell> {
    let table = &model.workbook.pivot_tables[0];
    let (location, geometry) = (&table.location, &table.geometry);
    let as_i32 = |n: u32| i32::try_from(n).unwrap();
    let top = as_i32(location.top) + 1;
    let left = as_i32(location.left) + 1;
    let labels_row = top + as_i32(geometry.first_header_row);
    let first_row = top + as_i32(geometry.first_data_row);
    let first_column = left + as_i32(geometry.first_data_column);
    let mut cells = Vec::new();
    for row in first_row..=as_i32(location.bottom) + 1 {
        for column in first_column..=as_i32(location.right) + 1 {
            cells.push(ValueCell {
                row,
                column,
                region: model._get_text_at(0, row, left),
                product: model._get_text_at(0, labels_row, column),
                text: model._get_text_at(0, row, column),
            });
        }
    }
    cells
}

/// The sum of each case-folded (Region, Product) pair, `grand total` standing for all of a
/// field.
fn expected_sums(records: &[Record]) -> BTreeMap<(String, String), String> {
    let total = GRAND_TOTAL.to_lowercase();
    let mut sums: BTreeMap<(String, String), i64> = BTreeMap::new();
    for (region, product, sales) in records {
        let (region, product) = (region.to_lowercase(), product.to_lowercase());
        for key in [
            (region.clone(), product.clone()),
            (region, total.clone()),
            (total.clone(), product),
            (total.clone(), total.clone()),
        ] {
            *sums.entry(key).or_insert(0) += sales;
        }
    }
    sums.into_iter()
        .map(|(key, sum)| (key, sum.to_string()))
        .collect()
}

/// The rendered values by case-folded (Region, Product) label pair; empty cells left out.
fn rendered_sums(model: &Model) -> BTreeMap<(String, String), String> {
    value_cells(model)
        .into_iter()
        .filter(|cell| !cell.text.is_empty())
        .map(|cell| {
            (
                (cell.region.to_lowercase(), cell.product.to_lowercase()),
                cell.text,
            )
        })
        .collect()
}

#[test]
fn refreshing_a_refreshed_pivot_changes_nothing() {
    for seed in 0..SEEDS {
        let mut model = pivot_model(&records(seed));
        let cells = sheet_text(&model);
        let table = model.workbook.pivot_tables[0].clone();
        let cache = model.workbook.pivot_caches[0].clone();
        refresh_pivot(&mut model);
        assert_eq!(sheet_text(&model), cells, "seed {seed}");
        assert_eq!(model.workbook.pivot_tables[0], table, "seed {seed}");
        assert_eq!(model.workbook.pivot_caches[0], cache, "seed {seed}");
    }
}

#[test]
fn each_value_cell_is_the_sum_of_its_records() {
    for seed in 0..SEEDS {
        let records = records(seed);
        let model = pivot_model(&records);
        assert_eq!(
            rendered_sums(&model),
            expected_sums(&records),
            "seed {seed}"
        );
    }
}

#[test]
fn getpivotdata_of_each_value_cell_returns_the_cell() {
    for seed in 0..SEEDS {
        let mut model = pivot_model(&records(seed));
        let cells: Vec<ValueCell> = value_cells(&model)
            .into_iter()
            .filter(|cell| !cell.text.is_empty())
            .collect();
        // One formula per value cell, in column AB (outside the window `sheet_text` reads).
        for (i, cell) in cells.iter().enumerate() {
            let mut formula = r#"=GETPIVOTDATA("Sales",$A$3"#.to_string();
            for (name, label) in [("Region", &cell.region), ("Product", &cell.product)] {
                if label != GRAND_TOTAL {
                    formula.push_str(&format!(r#","{name}","{label}""#));
                }
            }
            formula.push(')');
            model._set(&format!("AB{}", i + 1), &formula);
        }
        model.evaluate();
        for (i, cell) in cells.iter().enumerate() {
            let row = i32::try_from(i + 1).unwrap();
            assert_eq!(
                model._get_text_at(0, row, 28),
                cell.text,
                "seed {seed}: {} / {} at R{}C{}",
                cell.region,
                cell.product,
                cell.row,
                cell.column
            );
        }
    }
}

#[test]
fn permuting_the_source_rows_keeps_every_value() {
    for seed in 0..SEEDS {
        let records = records(seed);
        let mut permuted = records.clone();
        let mut rng = Rng(seed + 1);
        // Fisher-Yates.
        for i in (1..permuted.len()).rev() {
            let j = usize::try_from(rng.below(i as u64 + 1)).unwrap();
            permuted.swap(i, j);
        }
        let model = pivot_model(&records);
        let other = pivot_model(&permuted);
        assert_eq!(rendered_sums(&model), rendered_sums(&other), "seed {seed}");
        // The items' order and casing may follow the source, but each axis holds the same items.
        let labels = |model: &Model| {
            let mut found: Vec<(String, String)> = value_cells(model)
                .into_iter()
                .map(|cell| (cell.region.to_lowercase(), cell.product.to_lowercase()))
                .collect();
            found.sort();
            found
        };
        assert_eq!(labels(&model), labels(&other), "seed {seed}");
    }
}
