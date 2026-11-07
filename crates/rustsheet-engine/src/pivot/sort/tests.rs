#![allow(clippy::unwrap_used, clippy::panic, reason = "tests")]
//! The value sorts against the probes' sidecars: each pivot is the case file's definition (the
//! shared probe harness, `pivot::probe_cases`) with its field's `sortType` and its `autoSortScope`
//! typed by hand as the importer types them (the importer's own reading is tested in
//! crates/rustsheet-xlsx/src/import/pivots/table/tests.rs, and the eval harness runs every probe
//! through it), and every expectation is a cell Excel saved.

use serde_json::Value as Json;

use super::*;
use crate::pivot::probe_cases::{
    item_of, probe_model, probe_pivot, probe_pivot_over, read, sheet, sidecar_differences,
};
use crate::pivot::{compute, refresh, AxisField, CacheSource, MAX_PIVOT_WORK, MAX_RECORD_VALUES};

/// A scope on data field `data` and, per `(field, x)`, a reference to item `x` of `field`.
fn scope(data: u32, references: &[(u32, u32)]) -> AutoSortScope {
    AutoSortScope {
        data_field: Some(data),
        references: references
            .iter()
            .map(|&(field, x)| SortReference {
                field,
                items: vec![x],
            })
            .collect(),
    }
}

/// The cache field named `name`.
fn field_named(cache: &PivotCache, name: &str) -> u32 {
    u32::try_from(cache.fields.iter().position(|f| f.name == name).unwrap()).unwrap()
}

/// `cache` with each field's items in the order the case builder writes them for sheet
/// `name`'s pivot: the items its `order` lists first, then the rest as they were (the builder's
/// `_Field.reorder`), the records following their items. The file's cache order is what breaks a
/// value sort's ties.
fn written_order(case: &Json, name: &str, mut cache: PivotCache) -> PivotCache {
    let fields = &sheet(case, name)["pivot_tables"][0]["definition"]["fields"];
    for index in 0..cache.fields.len() {
        let Some(order) = fields[&cache.fields[index].name]["order"].as_array() else {
            continue;
        };
        let index = u32::try_from(index).unwrap();
        let first: Vec<u32> = order.iter().map(|v| item_of(&cache, index, v)).collect();
        let field = &mut cache.fields[index as usize];
        let count = u32::try_from(field.items.len()).unwrap();
        let moved: Vec<u32> = (first.iter().copied())
            .chain((0..count).filter(|i| !first.contains(i)))
            .collect();
        let mut to = vec![0; moved.len()];
        for (new, &old) in (0u32..).zip(&moved) {
            to[old as usize] = new;
        }
        field.items = moved
            .iter()
            .map(|&old| field.items[old as usize].clone())
            .collect();
        for record in &mut field.records {
            *record = to[*record as usize];
        }
    }
    cache
}

/// Sheet `name`'s pivot over `cache`, its field `field` sorted by `scope` as its `sortType` says,
/// against the sidecar.
fn sorted_differences(
    stem: &str,
    case: &Json,
    name: &str,
    cache: PivotCache,
    field: &str,
    scope: AutoSortScope,
) -> Vec<String> {
    let (cache, mut table) = probe_pivot_over(case, name, cache);
    let index = field_named(&cache, field);
    let attrs = &sheet(case, name)["pivot_tables"][0]["definition"]["fields"][field]["attrs"];
    let pivot_field = &mut table.fields[index as usize];
    pivot_field.sort_type = match attrs["sortType"].as_str().unwrap() {
        "ascending" => SortType::Ascending,
        "descending" => SortType::Descending,
        other => panic!("sortType {other}"),
    };
    pivot_field.auto_sort = Some(scope);
    sidecar_differences(stem, name, &cache, &table, None, &[])
}

/// Each `(sheet, field, scope)` of probe `stem` over the cache the builder wrote, against its
/// sidecar.
fn check(stem: &str, sheets: &[(&str, &str, AutoSortScope)]) {
    let case = read(stem, "case");
    let wrong: Vec<String> = sheets
        .iter()
        .flat_map(|(name, field, scope)| {
            let (cache, _) = probe_pivot(&case, name);
            let cache = written_order(&case, name, cache);
            sorted_differences(stem, &case, name, cache, field, scope.clone())
        })
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn a_value_sort_orders_by_the_data_field_within_each_outer_item() {
    // Region 0, Product 1, Year 2, Sales 3; data fields Sum (0) and, on SV2, Max (1).
    check(
        "t4130-pivot-sort",
        &[
            ("SV", "Product", scope(0, &[])),
            ("SVA", "Product", scope(0, &[])),
            // By the 2024 column (Year's item 0).
            ("SVC", "Product", scope(0, &[(2, 0)])),
            ("SVR", "Product", scope(0, &[])),
            // The area's other attributes and the reference's subtotal flags change nothing.
            ("SXA", "Product", scope(0, &[])),
            ("SXR", "Product", scope(0, &[])),
            // By Max of Sales, the second data field.
            ("SV2", "Product", scope(1, &[])),
        ],
    );
}

#[test]
fn ties_follow_the_written_cache_descending_and_its_reverse_ascending() {
    // Product 0, Year 1, Sales 2.
    check(
        "t4164-pivot-value-sort",
        &[
            ("STd", "Product", scope(0, &[])),
            ("STa", "Product", scope(0, &[])),
            // The `<items>` order does not break the tie.
            ("STo", "Product", scope(0, &[])),
            // The cache the file wrote does, not the source order.
            ("STc", "Product", scope(0, &[])),
            ("STca", "Product", scope(0, &[])),
        ],
    );
}

#[test]
fn a_column_scope_sorts_by_that_column_with_a_blank_as_zero() {
    check(
        "t4164-pivot-value-sort",
        &[
            ("SCd", "Product", scope(0, &[(1, 0)])),
            ("SCa", "Product", scope(0, &[(1, 0)])),
            ("SC2", "Product", scope(0, &[(1, 1)])),
        ],
    );
}

#[test]
fn an_error_sorts_last_both_ways() {
    // Over the engine's refresh of the case, which evaluates B3's =NA() as Excel's refresh on
    // open does (the builder caches the formula as missing).
    let stem = "t4164-pivot-value-sort";
    let case = read(stem, "case");
    let mut model = probe_model(&case);
    model.evaluate();
    let empty = PivotCache {
        source: Some(CacheSource {
            sheet_name: "Err".into(),
            range: "A1:B6".into(),
        }),
        ..PivotCache::default()
    };
    let cache = refresh(&empty, &model, MAX_RECORD_VALUES).unwrap();
    let wrong: Vec<String> = ["SEd", "SEa"]
        .iter()
        .flat_map(|name| {
            sorted_differences(stem, &case, name, cache.clone(), "Item", scope(0, &[]))
        })
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn a_value_sort_replaces_the_items_order() {
    check(
        "t4164-pivot-item-order",
        &[("IOV", "Product", scope(0, &[]))],
    );
}

/// The row items `table` over `cache` shows under the grand total, in order.
fn row_items(table: &PivotTable, cache: &PivotCache) -> Vec<Option<u32>> {
    let mut work = MAX_PIVOT_WORK;
    let result = compute(table, cache, &mut work).unwrap();
    let rows = result.rows();
    (rows.children(0))
        .map(|n| rows.nodes()[n as usize].item)
        .collect()
}

#[test]
fn a_scope_that_names_nothing_sorts_nothing() {
    // SV's pivot (cache Bee, Ant, Cat; sums 14, 28, 18) with a scope past its data fields, with
    // no data field, with a column reference with no item, or under `manual`: the cache order,
    // as unsorted.
    let case = read("t4130-pivot-sort", "case");
    let (cache, mut table) = probe_pivot(&case, "SV");
    let before = row_items(&table, &cache);
    assert_eq!(before, [Some(0), Some(1), Some(2)]);
    table.fields[1].sort_type = SortType::Descending;
    let empty_reference = AutoSortScope {
        data_field: Some(0),
        references: vec![SortReference {
            field: 2,
            items: Vec::new(),
        }],
    };
    table.column_fields = vec![AxisField::Field(2)];
    table.fields[2].axis = Some(super::super::Axis::Column);
    for bad in [scope(1, &[]), AutoSortScope::default(), empty_reference] {
        table.fields[1].auto_sort = Some(bad.clone());
        assert_eq!(row_items(&table, &cache), before, "{bad:?}");
    }
    // Region is on no axis and field 99 is past the cache: a reference to either is ignored,
    // and the sort applies (Ant 28, Cat 18, Bee 14).
    for ignored in [0, 99] {
        table.fields[1].auto_sort = Some(scope(0, &[(ignored, 0)]));
        assert_eq!(
            row_items(&table, &cache),
            [Some(1), Some(2), Some(0)],
            "{ignored}"
        );
    }
    table.fields[1].sort_type = SortType::Manual;
    assert_eq!(row_items(&table, &cache), before);
}

#[test]
fn a_value_sort_is_charged_and_stops_at_the_limit() {
    let case = read("t4130-pivot-sort", "case");
    let (cache, mut table) = probe_pivot(&case, "SVR");
    let spent = |table: &PivotTable| {
        let mut work = MAX_PIVOT_WORK;
        compute(table, &cache, &mut work).unwrap();
        MAX_PIVOT_WORK - work
    };
    let plain = spent(&table);
    table.fields[1].sort_type = SortType::Descending;
    table.fields[1].auto_sort = Some(scope(0, &[]));
    let with_sort = spent(&table);
    assert!(with_sort > plain, "{with_sort} <= {plain}");
    for budget in 0..with_sort {
        let mut work = budget;
        assert_eq!(
            compute(&table, &cache, &mut work),
            Err(PivotComputeError::Limit),
            "{budget}"
        );
        assert_eq!(work, 0);
    }
}
