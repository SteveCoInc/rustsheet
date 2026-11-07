//! Timings for GETPIVOTDATA's per-pivot index (SPEC-0015 "Untrusted-input limits", T-2083). A
//! 100,000-record source (Region: 100 items, Product: 10, Sales: the row number) feeds one Sum
//! pivot with Region rows and Product columns:
//! - 1,000 GETPIVOTDATA cells, each a different Region/Product intersection (1,000 queries, one
//!   index build);
//! - 1,000 GETPIVOTDATA cells asking the same grand total (one query, memoized);
//! - one call lifted over 100 Region items, `=SUM(GETPIVOTDATA("Sales",$H$1,"Region",F1:F100))`.
//!
//! Each recalculation rebuilds the index (it is dropped at each evaluation), so the timings
//! include its one pass over the records.
//!
//! The same source, as a cache, also times the engine's result tree (`rustsheet_engine::pivot::
//! compute`, SPEC-0032 stages 1 to 3, T-4142): the filtered records and every accumulator of the
//! Region by Product pivot, its totals included, with a Sum and an Average data field, and with
//! a hidden Product item and a page selection on Region.
//!
//! `cargo bench -p rustsheet --bench pivot [-- FILTER...]`

mod common;

use std::error::Error;
use std::io::Write;

use rustsheet::{Input, PivotFunction, PivotTableInput, Workbook};
use rustsheet_engine::pivot::{
    compute, Aggregate, Axis, AxisField, CacheBuilder, CacheItem, CellArea, DataField, PageField,
    PivotCache, PivotField, PivotTable, RecordValue, MAX_PIVOT_WORK, MAX_RECORD_VALUES,
};

const RECORDS: u32 = 100_000;
const REGIONS: u32 = 100;
const PRODUCTS: u32 = 10;
const RUNS: usize = 11;

/// The source in A1:C100001, the pivot registered at H1:S103 and the region names in F1:F100,
/// then `formulas` in column E.
fn workbook(formulas: &[String]) -> Result<Workbook, Box<dyn Error>> {
    let mut wb = common::filled_workbook(|tx| {
        tx.set("A1", "Region")?;
        tx.set("B1", "Product")?;
        tx.set("C1", "Sales")?;
        for r in 0..RECORDS {
            let row = r + 2;
            tx.set(
                format!("A{row}").as_str(),
                format!("R{}", r % REGIONS).as_str(),
            )?;
            tx.set(
                format!("B{row}").as_str(),
                format!("P{}", (r / REGIONS) % PRODUCTS).as_str(),
            )?;
            tx.set(format!("C{row}").as_str(), f64::from(r))?;
        }
        for r in 0..REGIONS {
            tx.set(format!("F{}", r + 1).as_str(), format!("R{r}").as_str())?;
        }
        Ok(())
    })?;
    let input = PivotTableInput {
        name: "Bench".into(),
        source: wb.range_ref(&format!("A1:C{}", RECORDS + 1))?,
        location: wb.range_ref("H1:S103")?,
        row: "Region".into(),
        column: Some("Product".into()),
        value: "Sales".into(),
        function: PivotFunction::Sum,
        caption: "Sum of Sales".into(),
        group: None,
        unused_items: Vec::new(),
        show_all: false,
        column_unused_items: Vec::new(),
        column_show_all: false,
    };
    wb.edit(|tx| {
        tx.add_pivot_table(&input)?;
        for (i, formula) in formulas.iter().enumerate() {
            tx.set(
                format!("E{}", i + 1).as_str(),
                Input::Formula(formula.clone()),
            )?;
        }
        Ok(())
    })?;
    Ok(wb)
}

fn cases() -> Vec<(&'static str, Vec<String>)> {
    let intersections = (0..1_000)
        .map(|i| {
            format!(
                r#"=GETPIVOTDATA("Sales",$H$1,"Region","R{}","Product","P{}")"#,
                i % REGIONS,
                (i / REGIONS) % PRODUCTS
            )
        })
        .collect();
    let totals = vec![r#"=GETPIVOTDATA("Sales",$H$1)"#.to_string(); 1_000];
    let lifted = vec![r#"=SUM(GETPIVOTDATA("Sales",$H$1,"Region",F1:F100))"#.to_string()];
    vec![
        ("1,000 distinct intersections", intersections),
        ("1,000 equal grand totals", totals),
        ("one call lifted over 100 items", lifted),
    ]
}

/// The bench source as a pivot cache: Region, Product and Sales.
fn cache() -> Result<PivotCache, Box<dyn Error>> {
    let mut builder = CacheBuilder::new(None, MAX_RECORD_VALUES);
    for name in ["Region", "Product", "Sales"] {
        builder.add_field(name.to_string())?;
    }
    for r in 0..RECORDS {
        builder.push_record(vec![
            RecordValue::Inline(CacheItem::Text(format!("R{}", r % REGIONS))),
            RecordValue::Inline(CacheItem::Text(format!("P{}", (r / REGIONS) % PRODUCTS))),
            RecordValue::Inline(CacheItem::Number(f64::from(r))),
        ])?;
    }
    Ok(builder.finish())
}

/// Region rows, Product columns, Sum and Average of Sales; `filtered` hides one product and adds
/// Region as a page field too, selecting nothing (`(All)`), so stage 1 tests every record.
fn table(cache: &PivotCache, filtered: bool) -> PivotTable {
    let area = CellArea {
        top: 0,
        left: 0,
        bottom: 0,
        right: 0,
    };
    let mut table = PivotTable::new("Bench".into(), 1, area, 0);
    let items = |field: usize| cache.fields.get(field).map_or(0, |f| f.items.len());
    table.fields = vec![
        PivotField::new(Some(Axis::Row), items(0)),
        PivotField::new(Some(Axis::Column), items(1)),
        PivotField::new(None, items(2)),
    ];
    table.row_fields = vec![AxisField::Field(0)];
    table.column_fields = vec![AxisField::Field(1), AxisField::Values];
    table.data_fields = [Aggregate::Sum, Aggregate::Average]
        .into_iter()
        .map(|function| DataField {
            field: 2,
            function,
            ..DataField::default()
        })
        .collect();
    if filtered {
        if let Some(product) = table.fields.get_mut(1) {
            product.hide(3);
        }
        table.page_fields = vec![PageField {
            field: 0,
            item: None,
            hierarchy: Some(-1),
        }];
    }
    table
}

fn main() -> Result<(), Box<dyn Error>> {
    let filters: Vec<String> = std::env::args()
        .skip(1)
        .filter(|a| !a.starts_with('-'))
        .collect();
    let mut out = std::io::stdout().lock();
    for (name, formulas) in cases() {
        if !filters.is_empty() && !filters.iter().any(|f| name.contains(f.as_str())) {
            continue;
        }
        let mut wb = workbook(&formulas)?;
        let t = common::timings(RUNS, || wb.recalculate());
        writeln!(out, "{name}: {t}")?;
    }
    let source = cache()?;
    for (name, filtered) in [
        ("result tree over 100,000 records", false),
        ("result tree with filters", true),
    ] {
        if !filters.is_empty() && !filters.iter().any(|f| name.contains(f.as_str())) {
            continue;
        }
        let table = table(&source, filtered);
        let t = common::timings(RUNS, || {
            let mut work = MAX_PIVOT_WORK;
            assert!(compute(&table, &source, &mut work).is_ok(), "{name}");
        });
        writeln!(out, "{name}: {t}")?;
    }
    Ok(())
}
