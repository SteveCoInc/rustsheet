#![allow(clippy::unwrap_used, clippy::panic, reason = "tests")]
#![expect(
    clippy::float_cmp,
    reason = "each total is compared bit-exact with the one Excel rendered"
)]
//! Tests of the added group fields a refresh derives again (T-4171), against the sidecars of
//! probes/t4170-pivot-manual-groups and probes/t4170-pivot-date-levels: each old cache is the one
//! the importer builds from the case builder's parts (the saved group items are the sidecar's
//! `cache_fields`), the source the probe's Data sheet, and each expected total a cell Excel
//! rendered.

use serde_json::Value as Json;

use super::*;
use crate::pivot::probe_cases::{probe_model, read, source_cache};
use crate::pivot::{
    group_by_dates, keep_ungrouped, refresh, DateGroupBy, DateGroups, MAX_RECORD_VALUES,
};

/// The saved `<groupItems>` of the sidecar's cache field `name` (every pivot of a probe saves the
/// same cache).
fn group_items(oracle: &Json, name: &str) -> Vec<CacheItem> {
    let field = oracle["pivots"][0]["cache_fields"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == name)
        .unwrap();
    (field["group_items"].as_array().unwrap().iter())
        .map(|item| {
            assert_eq!(item["t"], "s");
            CacheItem::Text(item["v"].as_str().unwrap().to_string())
        })
        .collect()
}

/// The cells of the sidecar's pivot on `sheet`, by address.
fn cells<'a>(oracle: &'a Json, sheet: &str) -> &'a serde_json::Map<String, Json> {
    (oracle["pivots"].as_array().unwrap().iter())
        .find(|p| p["sheet"] == sheet)
        .unwrap()["cells"]
        .as_object()
        .unwrap()
}

/// The value right of the first label cell `label` in column A of `cells`: a compact pivot's row
/// total.
fn row_total(cells: &serde_json::Map<String, Json>, label: &str) -> f64 {
    let row = (1..100)
        .find(|row| {
            cells
                .get(&format!("A{row}"))
                .is_some_and(|c| c["v"] == label)
        })
        .unwrap_or_else(|| panic!("no row {label}"));
    cells[&format!("B{row}")]["v"].as_f64().unwrap()
}

/// Per item of `field`, the sum of `values`' numbers over its records.
fn sums(field: &CacheField, values: &CacheField) -> Vec<f64> {
    let mut sums = vec![0.0; field.items.len()];
    for (&item, &value) in field.records.iter().zip(&values.records) {
        let CacheItem::Number(n) = values.items[value as usize] else {
            panic!("not a number");
        };
        sums[item as usize] += n;
    }
    sums
}

/// The manual-group cache as the importer builds it: Product and S from the source, and
/// Product2 grouping Product with the builder's `<discretePr>` (A, B in Group2; C, D in Group1; E
/// in North; F ungrouped) over the saved group items F, Group2, Group1, North.
fn manual_cache(case: &Json, oracle: &Json) -> PivotCache {
    let mut cache = source_cache(case, "Data!A1:B9");
    let product = &cache.fields[0];
    let labels = group_items(oracle, "Product2");
    let discrete = (product.items.iter())
        .map(|item| {
            let group = match item {
                CacheItem::Text(t) if t == "A" || t == "B" => "Group2",
                CacheItem::Text(t) if t == "C" || t == "D" => "Group1",
                CacheItem::Text(t) if t == "E" => "North",
                _ => "F",
            };
            let label = CacheItem::Text(group.to_string());
            u32::try_from(labels.iter().position(|l| *l == label).unwrap()).unwrap()
        })
        .collect::<Vec<u32>>();
    let mut added = group_by_discrete(product, &discrete, labels).unwrap();
    added.name = "Product2".to_string();
    added.added_group = Some(AddedGroup {
        base: 0,
        discrete: Some(discrete),
    });
    cache.fields.push(added);
    cache
}

/// MG (rows Product2 alone): each group's total is Excel's, after a refresh that reads the source
/// columns only.
#[test]
fn a_refresh_derives_a_manual_group_field_again() {
    let (case, oracle) = (
        read("t4170-pivot-manual-groups", "case"),
        read("t4170-pivot-manual-groups", "oracle"),
    );
    let old = manual_cache(&case, &oracle);
    let fresh = refresh(&old, &probe_model(&case), MAX_RECORD_VALUES).unwrap();
    let names: Vec<&str> = fresh.fields.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["Product", "S", "Product2"]);
    let product2 = &fresh.fields[2];
    assert_eq!(product2.items, group_items(&oracle, "Product2"));
    assert_eq!(product2.added_group, old.fields[2].added_group);
    let mg = cells(&oracle, "MG");
    let sums = sums(product2, &fresh.fields[1]);
    for (item, sum) in product2.items.iter().zip(sums) {
        let CacheItem::Text(label) = item else {
            panic!("a label");
        };
        assert_eq!(sum, row_total(mg, label), "{label}");
    }
}

/// A source value new to the base field becomes a group item of its own after the old ones
/// (UNVERIFIED, T-4802); a label equal to a group's is refused, and a base field gone from the
/// source drops the added field.
#[test]
fn a_new_base_item_is_its_own_group_and_a_gone_base_drops_the_field() {
    let (case, oracle) = (
        read("t4170-pivot-manual-groups", "case"),
        read("t4170-pivot-manual-groups", "oracle"),
    );
    let old = manual_cache(&case, &oracle);
    let mut model = probe_model(&case);
    model.set_user_input(0, 10, 1, "G".to_string()).unwrap();
    model.set_user_input(0, 10, 2, "1000".to_string()).unwrap();
    model.evaluate();
    let mut wider = old.clone();
    wider.source.as_mut().unwrap().range = "A1:B10".to_string();
    let fresh = refresh(&wider, &model, MAX_RECORD_VALUES).unwrap();
    let product2 = &fresh.fields[2];
    assert_eq!(
        product2.items.last(),
        Some(&CacheItem::Text("G".to_string()))
    );
    assert_eq!(product2.records.last(), Some(&4));
    let discrete = product2.added_group.as_ref().unwrap().discrete.as_ref();
    assert_eq!(discrete.unwrap()[6], 4);
    model.set_user_input(0, 11, 1, "north".to_string()).unwrap();
    model.set_user_input(0, 11, 2, "1".to_string()).unwrap();
    model.evaluate();
    wider.source.as_mut().unwrap().range = "A1:B11".to_string();
    assert_eq!(
        refresh(&wider, &model, MAX_RECORD_VALUES),
        Err(PivotCacheError::Group)
    );
    // Only S left in the source: Product2 goes with Product.
    let mut narrow = old.clone();
    narrow.source.as_mut().unwrap().range = "B1:B9".to_string();
    let fresh = refresh(&narrow, &probe_model(&case), MAX_RECORD_VALUES).unwrap();
    let names: Vec<&str> = fresh.fields.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["S"]);
}

/// The date-level cache as the importer builds it: Date grouped in place by months, Quarters and
/// Years added (`<fieldGroup base="0">`), all from 2019-01-01 to 2020-12-31.
fn date_cache(case: &Json, oracle: &Json) -> PivotCache {
    let mut cache = source_cache(case, "Data!A1:B10");
    let date = cache.fields[0].clone();
    let level = |by, name: &str| {
        let labels = group_items(oracle, name);
        let groups = DateGroups::new(by, 43466.0, 44196.0, labels.len()).unwrap();
        group_by_dates(&date, groups, labels).unwrap()
    };
    // Grouped in place, it keeps its own values, as the importer and a refresh keep them.
    cache.fields[0] = keep_ungrouped(date.clone(), level(DateGroupBy::Months, "Date"));
    for (by, name) in [
        (DateGroupBy::Quarters, "Quarters"),
        (DateGroupBy::Years, "Years"),
    ] {
        let mut added = level(by, name);
        added.name = name.to_string();
        added.added_group = Some(AddedGroup {
            base: 0,
            discrete: None,
        });
        cache.fields.push(added);
    }
    cache
}

/// DY (rows Years, Date): each year's total is Excel's, and the months still group in place.
#[test]
fn a_refresh_derives_date_levels_again_before_grouping_the_base_in_place() {
    let (case, oracle) = (
        read("t4170-pivot-date-levels", "case"),
        read("t4170-pivot-date-levels", "oracle"),
    );
    let old = date_cache(&case, &oracle);
    let fresh = refresh(&old, &probe_model(&case), MAX_RECORD_VALUES).unwrap();
    let names: Vec<&str> = fresh.fields.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["Date", "S", "Quarters", "Years"]);
    assert_eq!(fresh.fields, old.fields);
    let dy = cells(&oracle, "DY");
    let years = &fresh.fields[3];
    for (item, sum) in years.items.iter().zip(sums(years, &fresh.fields[1])) {
        let CacheItem::Text(label) = item else {
            panic!("a label");
        };
        assert_eq!(sum, row_total(dy, label), "{label}");
    }
}

/// A blank in the base would make the refresh drop a date grouping (`regroup`); an added level
/// has nothing to fall back to, so the refresh is refused (UNVERIFIED, T-4802).
#[test]
fn a_date_level_over_a_blank_is_refused() {
    let (case, oracle) = (
        read("t4170-pivot-date-levels", "case"),
        read("t4170-pivot-date-levels", "oracle"),
    );
    let mut old = date_cache(&case, &oracle);
    old.source.as_mut().unwrap().range = "A1:B11".to_string();
    assert_eq!(
        refresh(&old, &probe_model(&case), MAX_RECORD_VALUES),
        Err(PivotCacheError::Group)
    );
}

/// The added fields' record values count against the budget before they are derived.
#[test]
fn added_fields_count_against_the_record_value_budget() {
    let (case, oracle) = (
        read("t4170-pivot-date-levels", "case"),
        read("t4170-pivot-date-levels", "oracle"),
    );
    let old = date_cache(&case, &oracle);
    // 9 records of the 2 source fields fit; with Quarters and Years they do not.
    assert_eq!(
        refresh(&old, &probe_model(&case), 9 * 3),
        Err(PivotCacheError::TooManyRecordValues)
    );
    assert!(refresh(&old, &probe_model(&case), 9 * 4).is_ok());
}

/// A manual group that leaves a base item without a group item is refused.
#[test]
fn a_discrete_group_must_cover_its_base() {
    let base = CacheField {
        items: vec![CacheItem::Number(1.0), CacheItem::Number(2.0)],
        records: vec![0, 1, 1],
        ..CacheField::default()
    };
    let labels = vec![CacheItem::Text("G".to_string())];
    assert_eq!(
        group_by_discrete(&base, &[0], labels.clone()),
        Err(PivotCacheError::Discrete)
    );
    assert_eq!(
        group_by_discrete(&base, &[0, 1], labels.clone()),
        Err(PivotCacheError::Discrete)
    );
    assert_eq!(
        group_by_discrete(&base, &[0, 0], labels).unwrap().records,
        [0, 0, 0]
    );
}
