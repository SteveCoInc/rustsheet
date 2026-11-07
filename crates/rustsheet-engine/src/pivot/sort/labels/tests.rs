#![allow(clippy::unwrap_used, clippy::panic, reason = "tests")]
//! The label sorts against the probes' sidecars: each pivot is the case file's definition (the
//! shared probe harness, `pivot::probe_cases`, which reads the definition's `customListSort`) with
//! its field's `sortType` typed by hand as the importer types it (the importer's own reading is
//! tested in crates/rustsheet-xlsx/src/import/pivots/table/tests.rs, and the eval harness runs
//! every probe through it), and every expectation is a cell Excel saved.

use serde_json::Value as Json;

use super::*;
use crate::expressions::token::Error;
use crate::pivot::probe_cases::{probe_pivot, read, sheet, sidecar_differences};
use crate::pivot::{
    compute, Axis, AxisField, CacheField, CacheItem, CellArea, DataField, FilterCriteria,
    PivotCache, PivotFilter, PivotFilterKind, MAX_PIVOT_WORK,
};

/// The `sortType` the case's spec writes on `field` of sheet `name`'s pivot.
fn sort_type(case: &Json, name: &str, field: &str) -> SortType {
    let attrs = &sheet(case, name)["pivot_tables"][0]["definition"]["fields"][field]["attrs"];
    match attrs["sortType"].as_str() {
        Some("ascending") => SortType::Ascending,
        Some("descending") => SortType::Descending,
        None | Some("manual") => SortType::Manual,
        Some(other) => panic!("sortType {other}"),
    }
}

/// Sheet `name`'s pivot of probe `stem`, field `field` sorted as its `sortType` says, with
/// `filters`, against the sidecar.
fn differences(stem: &str, name: &str, field: &str, filters: Vec<PivotFilter>) -> Vec<String> {
    let case = read(stem, "case");
    let (cache, mut table) = probe_pivot(&case, name);
    let index = cache.fields.iter().position(|f| f.name == field).unwrap();
    table.fields[index].sort_type = sort_type(&case, name, field);
    table.filters = filters;
    sidecar_differences(stem, name, &cache, &table, None, &[])
}

fn check(stem: &str, sheets: &[(&str, &str)]) {
    let wrong: Vec<String> = sheets
        .iter()
        .flat_map(|(name, field)| differences(stem, name, field, Vec::new()))
        .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn ascending_and_descending_sort_the_labels() {
    // The cache is Bee, Ant, Cat; SM (`manual`, items Cat, Ant, Bee) keeps its items order;
    // NA's `nonAutoSortDefault` changes nothing.
    check(
        "t4130-pivot-sort",
        &[
            ("SA", "Product"),
            ("SD", "Product"),
            ("NA", "Product"),
            ("SM", "Product"),
        ],
    );
}

#[test]
fn a_label_sort_replaces_the_items_order() {
    // Items written Cat, Bee, Ant: ascending is Ant, Bee, Cat (IOA); descending (IOD) and
    // `manual` (IOM) show Cat, Bee, Ant.
    check(
        "t4164-pivot-item-order",
        &[("IOA", "Product"), ("IOD", "Product"), ("IOM", "Product")],
    );
}

#[test]
fn custom_list_sort_orders_day_names_by_the_list_unless_off() {
    // Wed, Mon, Fri: Mon, Wed, Fri by the day list (CL1); Fri, Mon, Wed with
    // `customListSort="0"` (CL0).
    check("t4130-pivot-sort", &[("CL1", "Day"), ("CL0", "Day")]);
}

#[test]
fn an_inner_label_sort_orders_each_parent_under_a_top_n() {
    // probes/t4128-pivot-spike K: Region then Product descending, the count top 2 on Product
    // (`iMeasureFld="0"`) the importer types from the kept `<filters>`, and a calculated field.
    let top = PivotFilter {
        field: 1,
        kind: PivotFilterKind::Count,
        measure: 0,
        eval_order: -1,
        id: 1,
        criteria: FilterCriteria::Top10 {
            top: true,
            value: "2".to_string(),
        },
        whole_day: false,
    };
    let wrong = differences("t4128-pivot-spike", "K", "Product", vec![top]);
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The row items of a one-field pivot over `items` (one record each), field 0 sorted by `sort`,
/// with `lists` as `customListSort`, under the grand total, as cache item indices.
fn sorted(items: Vec<CacheItem>, sort: SortType, lists: bool) -> Vec<u32> {
    let count = u32::try_from(items.len()).unwrap();
    let cache = PivotCache {
        fields: vec![
            CacheField {
                name: "Item".into(),
                items,
                records: (0..count).collect(),
                ..Default::default()
            },
            CacheField {
                name: "Value".into(),
                items: vec![CacheItem::Number(1.0)],
                records: vec![0; count as usize],
                ..Default::default()
            },
        ],
        ..PivotCache::default()
    };
    let area = CellArea {
        top: 0,
        left: 0,
        bottom: 0,
        right: 0,
    };
    let mut table = PivotTable::new("L".to_string(), 0, area, 0);
    table.fields = vec![
        PivotField::new(Some(Axis::Row), count as usize),
        PivotField::new(None, 1),
    ];
    table.row_fields = vec![AxisField::Field(0)];
    table.data_fields = vec![DataField {
        field: 1,
        ..DataField::default()
    }];
    table.fields[0].sort_type = sort;
    table.options.custom_list_sort = lists;
    let mut work = MAX_PIVOT_WORK;
    let result = compute(&table, &cache, &mut work).unwrap();
    let rows = result.rows();
    (rows.children(0))
        .map(|n| rows.nodes()[n as usize].item.unwrap())
        .collect()
}

fn text(t: &str) -> CacheItem {
    CacheItem::Text(t.to_string())
}

#[test]
fn mixed_items_sort_numbers_texts_booleans_errors_then_the_blank() {
    // UNVERIFIED (T-4799): the engine's own reading of mixed item types.
    let items = vec![
        CacheItem::Missing,
        text("b"),
        CacheItem::Error(Error::NA),
        CacheItem::Boolean(true),
        CacheItem::Number(10.0),
        text("A"),
        CacheItem::Boolean(false),
        CacheItem::Date(2.0),
    ];
    assert_eq!(
        sorted(items.clone(), SortType::Ascending, true),
        [7, 4, 5, 1, 6, 3, 2, 0]
    );
    assert_eq!(
        sorted(items, SortType::Descending, true),
        [2, 3, 6, 1, 5, 4, 7, 0]
    );
}

#[test]
fn listed_names_sort_by_their_list_ahead_of_other_texts() {
    // UNVERIFIED (T-4799): months, case and the place of the other texts.
    let items = vec![
        text("Zed"),
        text("MAR"),
        text("Jan"),
        text("Apple"),
        text("Feb"),
    ];
    assert_eq!(
        sorted(items.clone(), SortType::Ascending, true),
        [2, 4, 1, 3, 0]
    );
    assert_eq!(
        sorted(items.clone(), SortType::Descending, true),
        [0, 3, 1, 4, 2]
    );
    assert_eq!(sorted(items, SortType::Ascending, false), [3, 4, 2, 1, 0]);
}

#[test]
fn a_label_sort_is_charged_and_stops_at_the_limit() {
    let case = read("t4130-pivot-sort", "case");
    let (cache, mut table) = probe_pivot(&case, "SA");
    let spent = |table: &PivotTable| {
        let mut work = MAX_PIVOT_WORK;
        compute(table, &cache, &mut work).unwrap();
        MAX_PIVOT_WORK - work
    };
    let plain = spent(&table);
    table.fields[1].sort_type = SortType::Ascending;
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
