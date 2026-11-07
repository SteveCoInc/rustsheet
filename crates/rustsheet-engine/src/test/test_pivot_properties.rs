// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::format_push_string)]
// LINT-DEBT-END
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
//! Import, export and import as a fixed point is not here: the export is T-4176's, in the xlsx
//! crate, which this crate's tests cannot reach.

use std::collections::BTreeMap;

use crate::pivot::{
    refresh, Aggregate, Axis, AxisField, CacheSource, CellArea, DataField, PivotBudget, PivotCache,
    PivotField, PivotTable, MAX_RECORD_VALUES,
};
use crate::test::util::{new_empty_model, Rng};
use crate::Model;

/// Seeds per property: enough to vary item counts, empty intersections and case variants,
/// small enough for the CI time budget (each model is a few dozen cells).
const SEEDS: u64 = 48;

const REGIONS: [&str; 5] = ["East", "West", "North", "South", "Mid"];
const PRODUCTS: [&str; 6] = ["A", "B", "C", "D", "E", "F"];

/// The label the renderer writes for a grand-total line.
const GRAND_TOTAL: &str = "Grand Total";

type Record = (String, String, i64);

/// 1 to 30 records over a seed-chosen subset of regions and products; about one text in eight
/// is written in lower case, the variant a refresh folds into the first-seen item.
fn records(seed: u64) -> Vec<Record> {
    let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    let regions = 1 + rng.below(REGIONS.len() as u64);
    let products = 1 + rng.below(PRODUCTS.len() as u64);
    let count = 1 + rng.below(30);
    let cased = |rng: &mut Rng, text: &str| {
        if rng.below(8) == 0 {
            text.to_lowercase()
        } else {
            text.to_string()
        }
    };
    (0..count)
        .map(|_| {
            let region = REGIONS[usize::try_from(rng.below(regions)).unwrap()];
            let product = PRODUCTS[usize::try_from(rng.below(products)).unwrap()];
            let sales = i64::try_from(rng.below(250)).unwrap() - 50;
            (cased(&mut rng, region), cased(&mut rng, product), sales)
        })
        .collect()
}

/// A model with `records` at Data!A1:C(n+1) and a Sum of Sales pivot over them anchored at
/// Sheet1!A3 (Region rows, Product columns), refreshed once so it is rendered.
fn pivot_model(records: &[Record]) -> Model {
    let mut model = new_empty_model();
    model.add_sheet("Data").unwrap();
    for (column, header) in ["A", "B", "C"].iter().zip(["Region", "Product", "Sales"]) {
        model.set_a1(&format!("Data!{column}1"), header);
    }
    for (i, (region, product, sales)) in records.iter().enumerate() {
        let row = i + 2;
        model.set_a1(&format!("Data!A{row}"), region);
        model.set_a1(&format!("Data!B{row}"), product);
        model.set_a1(&format!("Data!C{row}"), &sales.to_string());
    }
    model.evaluate();
    let source = PivotCache {
        source: Some(CacheSource {
            sheet_name: "Data".into(),
            range: format!("A1:C{}", records.len() + 1),
        }),
        ..PivotCache::default()
    };
    let cache = refresh(&source, &model, MAX_RECORD_VALUES).unwrap();
    let fields = [Some(Axis::Row), Some(Axis::Column), None]
        .into_iter()
        .zip(&cache.fields)
        .map(|(axis, field)| PivotField::new(axis, field.items.len()))
        .collect();
    let sheet_id = model.workbook.worksheet(0).unwrap().sheet_id;
    let anchor = CellArea {
        top: 2,
        left: 0,
        bottom: 2,
        right: 0,
    };
    let table = PivotTable {
        fields,
        row_fields: vec![AxisField::Field(0)],
        column_fields: vec![AxisField::Field(1)],
        data_fields: vec![DataField {
            caption: "Sum of Sales".to_string(),
            field: 2,
            function: Aggregate::Sum,
            ..DataField::default()
        }],
        ..PivotTable::new("P".to_string(), sheet_id, anchor, 0)
    };
    model.workbook.pivot_caches.push(cache);
    model.workbook.pivot_tables.push(table);
    refresh_pivot(&mut model);
    model
}

fn refresh_pivot(model: &mut Model) {
    let plan = model
        .plan_pivot_refresh(0, &mut PivotBudget::default())
        .unwrap();
    model.apply_pivot_refresh(plan).unwrap();
    model.evaluate();
}

/// Every cell of Sheet1!A1:Z60 as text: the rendered pivot and what is around it.
fn sheet_text(model: &Model) -> Vec<String> {
    (1..=60)
        .flat_map(|row| (1..=26).map(move |column| (row, column)))
        .map(|(row, column)| model.get_text_by_index(0, row, column))
        .collect()
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
                region: model.get_text_by_index(0, row, left),
                product: model.get_text_by_index(0, labels_row, column),
                text: model.get_text_by_index(0, row, column),
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
            model.set_a1(&format!("AB{}", i + 1), &formula);
        }
        model.evaluate();
        for (i, cell) in cells.iter().enumerate() {
            let row = i32::try_from(i + 1).unwrap();
            assert_eq!(
                model.get_text_by_index(0, row, 28),
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
