//! Seeded generators for tests, shared across crates (T-4756): this crate's tests use them, and
//! the xlsx crate's tests reach them through the `test-support` feature, so a generator is
//! written once (directive 3). Nothing here is part of the engine's API.

use crate::pivot::{
    refresh, Aggregate, Axis, AxisField, CacheSource, CellArea, DataField, PivotBudget, PivotCache,
    PivotField, PivotTable, MAX_RECORD_VALUES,
};
use crate::Model;

/// Deterministic xorshift64 generator for seeded tests; the wrapped state must be non-zero.
pub struct Rng(pub u64);

impl Rng {
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    pub fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }

    /// An index below `len` (`len` > 0).
    fn index(&mut self, len: usize) -> usize {
        let len64 = u64::try_from(len).unwrap_or(u64::MAX);
        usize::try_from(self.below(len64)).unwrap_or(0)
    }
}

const REGIONS: [&str; 5] = ["East", "West", "North", "South", "Mid"];
const PRODUCTS: [&str; 6] = ["A", "B", "C", "D", "E", "F"];

/// One generated pivot source row: Region, Product and an integer Sales.
pub type PivotRecord = (String, String, i64);

/// 1 to 30 records over a seed-chosen subset of regions and products; about one text in eight
/// is written in lower case, the variant a refresh folds into the first-seen item. The sales
/// are integers, so every sum is exact in any order (T-4155).
pub fn pivot_records(seed: u64) -> Vec<PivotRecord> {
    let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    let regions = 1 + rng.index(REGIONS.len());
    let products = 1 + rng.index(PRODUCTS.len());
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
            let region = REGIONS[rng.index(regions)];
            let product = PRODUCTS[rng.index(products)];
            let sales = i64::try_from(rng.below(250)).unwrap_or(0) - 50;
            (cased(&mut rng, region), cased(&mut rng, product), sales)
        })
        .collect()
}

/// `model` with `records` at Data!A1:C(n+1) and a Sum of Sales pivot over them anchored at
/// Sheet1!A3 (Region rows, Product columns), refreshed once so it is rendered. `model` is an
/// empty workbook whose only sheet is Sheet1.
pub fn add_sum_pivot(mut model: Model, records: &[PivotRecord]) -> Result<Model, String> {
    model.add_sheet("Data")?;
    let data = 1;
    for (column, header) in (1..).zip(["Region", "Product", "Sales"]) {
        model.set_user_input(data, 1, column, header.to_string())?;
    }
    for (row, (region, product, sales)) in (2..).zip(records) {
        model.set_user_input(data, row, 1, region.clone())?;
        model.set_user_input(data, row, 2, product.clone())?;
        model.set_user_input(data, row, 3, sales.to_string())?;
    }
    model.evaluate();
    let source = PivotCache {
        source: Some(CacheSource {
            sheet_name: "Data".into(),
            range: format!("A1:C{}", records.len() + 1),
        }),
        ..PivotCache::default()
    };
    let cache =
        refresh(&source, &model, MAX_RECORD_VALUES).map_err(|error| format!("{error:?}"))?;
    let fields = [Some(Axis::Row), Some(Axis::Column), None]
        .into_iter()
        .zip(&cache.fields)
        .map(|(axis, field)| PivotField::new(axis, field.items.len()))
        .collect();
    let sheet_id = model.workbook.worksheet(0)?.sheet_id;
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
    refresh_first_pivot(&mut model)?;
    Ok(model)
}

/// Refreshes the workbook's first pivot and recalculates.
pub fn refresh_first_pivot(model: &mut Model) -> Result<(), String> {
    let plan = model
        .plan_pivot_refresh(0, &mut PivotBudget::default())
        .map_err(|error| format!("{error:?}"))?;
    model
        .apply_pivot_refresh(plan)
        .map_err(|error| format!("{error:?}"))?;
    model.evaluate();
    Ok(())
}

/// Every cell of Sheet1!A1:Z60 as formatted text: the rendered pivot and what is around it.
pub fn pivot_sheet_text(model: &Model) -> Result<Vec<String>, String> {
    (1..=60)
        .flat_map(|row| (1..=26).map(move |column| (row, column)))
        .map(|(row, column)| model.get_formatted_cell_value(0, row, column))
        .collect()
}
