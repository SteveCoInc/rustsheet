#![allow(clippy::unwrap_used, reason = "tests")]
//! Tests of the discrete and date-level groups (`groups.rs`, T-4170): the parts and the
//! refusals of `tools/tests/test_pivot_groups.py`, which the builder and this mirror share.

use super::super::{parts, Parts};
use super::*;
use crate::case::CaseSpec;

/// A case with Product, S and Date source cells on `D` and one defined pivot on `P` at A3.
fn try_parts(definition: &Json) -> Result<Parts, String> {
    let spec: CaseSpec = serde_json::from_str(
        r#"{"format": "rustsheet-case/v1", "sheets": [
            {"name": "D", "cells": {"A1": "Product", "B1": "S", "C1": "Date",
              "A2": "A", "B2": 1, "C2": 43480.0, "A3": "B", "B3": 2, "C3": 43600.5,
              "A4": "C", "B4": 4, "C4": 43900.0, "A5": "a", "B5": 8, "C5": 44000.0}},
            {"name": "P", "cells": {}, "pivot_tables": [
              {"name": "Pv", "source": "D!A1:C5", "location": "A3", "definition": {}}]}]}"#,
    )
    .unwrap();
    let sheet = &spec.sheets[1];
    parts(&spec, sheet, &sheet.pivot_tables[0], definition, 1)
}

fn groups() -> Json {
    serde_json::json!({"name": "Product2", "groups": [{"name": "Group1", "items": ["B", "C"]}]})
}

fn dates() -> Json {
    serde_json::json!({"levels": ["months", "quarters", "years"], "start": 43466, "end": 44196})
}

#[test]
fn a_discrete_group_adds_a_field_as_excel_saves_it() {
    let built = try_parts(&serde_json::json!({
        "rows": ["Product2", "Product"], "data": [{"field": "S"}],
        "fields": {"Product": {"discrete": groups()}}}))
    .unwrap();
    let cache = &built.cache_definition;
    assert!(cache.contains(r#"<cacheField name="Product" numFmtId="0"><sharedItems count="3"><s v="A"/><s v="B"/><s v="C"/></sharedItems><fieldGroup par="3"/></cacheField>"#), "{cache}");
    assert!(cache.contains(r#"<cacheField name="Product2" numFmtId="0" databaseField="0"><fieldGroup base="0"><discretePr count="3"><x v="0"/><x v="1"/><x v="1"/></discretePr><groupItems count="2"><s v="A"/><s v="Group1"/></groupItems></fieldGroup></cacheField></cacheFields>"#), "{cache}");
    assert_eq!(built.cache_records.matches("<x ").count(), 4);
    assert!(built
        .table
        .contains(r#"<rowFields count="2"><field x="3"/><field x="0"/></rowFields>"#));
}

#[test]
fn the_added_field_lists_its_items_in_first_source_order_even_off_the_axes() {
    let built = try_parts(&serde_json::json!({
        "rows": ["Product"], "data": [{"field": "S"}],
        "fields": {"Product": {"discrete": {"name": "G", "groups": [
            {"name": "Last", "items": ["C"]}, {"name": "First", "items": ["A"]}]}}}}))
    .unwrap();
    assert!(built.table.contains(
        r#"<pivotField showAll="0"><items count="4"><item x="2"/><item x="0"/><item x="1"/>"#
    ));
}

#[test]
fn date_levels_group_in_place_and_add_a_field_per_level() {
    let built = try_parts(&serde_json::json!({
        "rows": ["Years", "Quarters", "Date"], "data": [{"field": "S"}],
        "fields": {"Date": {"date_group": dates()}, "Years": {"subtotals": []}}}))
    .unwrap();
    let cache = &built.cache_definition;
    assert!(cache.contains(r#"<cacheField name="Date" numFmtId="0"><sharedItems containsSemiMixedTypes="0" containsNonDate="0" containsDate="1" containsString="0" minDate="2019-01-15T00:00:00" maxDate="2020-06-18T00:00:00"/><fieldGroup par="4" base="2"><rangePr autoStart="0" autoEnd="0" groupBy="months" startDate="2019-01-01T00:00:00" endDate="2020-12-31T00:00:00"/><groupItems count="14"><s v="&lt;1/1/2019"/><s v="Jan"/>"#), "{cache}");
    assert!(cache.contains(r#"<groupItems count="4"><s v="&lt;1/1/2019"/><s v="2019"/><s v="2020"/><s v="&gt;12/31/2020"/></groupItems></fieldGroup></cacheField></cacheFields>"#));
    assert!(built
        .cache_records
        .contains(r#"<d v="2019-05-15T12:00:00"/>"#));
    assert!(built.table.contains(r#"<pivotField axis="axisRow" defaultSubtotal="0" showAll="0"><items count="4"><item x="0"/>"#));
}

#[test]
fn date_labels_and_dates_are_the_builders() {
    let hours = date_labels(2, 43466.0, 43467.0).unwrap();
    assert_eq!(
        [&hours[0], &hours[1], &hours[12], &hours[13], &hours[25]],
        ["<1/1/2019", "12 AM", "11 AM", "12 PM", ">1/2/2019"]
    );
    let days = date_labels(3, 61.0, 43831.0).unwrap();
    assert_eq!(
        (days.len(), days[60].as_str(), days[0].as_str()),
        (368, "29-Feb", "<3/1/1900")
    );
    assert_eq!(
        date_labels(6, 42736.5, 44196.0).unwrap(),
        ["<1/1/2017", "2017", "2018", "2019", "2020", ">12/31/2020"]
    );
    // `iso_date` on the OLE epoch, below and past the 1900 calendar's leap day.
    for (serial, text) in [
        (0.0, "1899-12-30T00:00:00"),
        (1.0, "1899-12-31T00:00:00"),
        (60.0, "1900-02-28T00:00:00"),
        (61.0, "1900-03-01T00:00:00"),
        (43466.75, "2019-01-01T18:00:00"),
        (2_958_465.999_99, "9999-12-31T23:59:59"),
    ] {
        assert_eq!(iso_date(serial).unwrap(), text);
    }
    assert!(iso_date(2_958_466.0).is_err() && iso_date(-1.0).is_err());
}

#[test]
fn the_builders_refusals_are_refused() {
    let g = groups();
    let d = dates();
    let many_groups = vec![serde_json::json!({"name": "G", "items": ["A"]}); 257];
    let refused = [
        serde_json::json!({"Product": {"discrete": {"name": "P2"}}}),
        serde_json::json!({"Product": {"discrete": {"name": "P2", "groups": [], "colour": 1}}}),
        serde_json::json!({"Product": {"discrete": {"name": "", "groups": []}}}),
        serde_json::json!({"Product": {"discrete": {"name": "P2", "groups": "A"}}}),
        serde_json::json!({"Product": {"discrete": {"name": "P2", "groups": []}}}),
        serde_json::json!({"Product": {"discrete": {"name": "P2", "groups": many_groups}}}),
        serde_json::json!({"Product": {"discrete": {"name": "P2", "groups": [{"name": "G"}]}}}),
        serde_json::json!({"Product": {"discrete": {"name": "P2", "groups": [["A"]]}}}),
        serde_json::json!({"Product": {"discrete": {"name": "P2", "groups": [{"name": 3, "items": ["A"]}]}}}),
        serde_json::json!({"Product": {"discrete": {"name": "P2", "groups": [{"name": "G", "items": "A"}]}}}),
        serde_json::json!({"Product": {"discrete": {"name": "P2", "groups": [{"name": "G", "items": vec!["A"; 257]}]}}}),
        serde_json::json!({"Product": {"discrete": {"name": "P2", "groups": [{"name": "G", "items": ["Z"]}]}}}),
        serde_json::json!({"Product": {"discrete": {"name": "P2", "groups": [{"name": "G", "items": ["A", "a"]}]}}}),
        serde_json::json!({"Product": {"discrete": {"name": "P2", "groups": [{"name": "c", "items": ["A"]}]}}}),
        serde_json::json!({"Product": {"discrete": {"name": "s", "groups": [{"name": "G", "items": ["A"]}]}}}),
        serde_json::json!({"Product": {"discrete": g, "group": {"start": 0}}}),
        serde_json::json!({"Product": {"discrete": g, "colour": 1}}),
        serde_json::json!({"Product": {"discrete": g}, "Product2": {"date_group": dates()}}),
        serde_json::json!({"Product": {"discrete": g}, "Product3": {}}),
        serde_json::json!({"Date": {"date_group": {"levels": ["months"]}}}),
        serde_json::json!({"Date": {"date_group": {"levels": "months", "start": 43466, "end": 44196}}}),
        serde_json::json!({"Date": {"date_group": {"levels": ["weeks"], "start": 43466, "end": 44196}}}),
        serde_json::json!({"Date": {"date_group": {"levels": [], "start": 43466, "end": 44196}}}),
        serde_json::json!({"Date": {"date_group": {"levels": ["years", "months"], "start": 43466, "end": 44196}}}),
        serde_json::json!({"Date": {"date_group": {"levels": ["months", "months"], "start": 43466, "end": 44196}}}),
        serde_json::json!({"Date": {"date_group": {"levels": ["months"], "start": "2019", "end": 44196}}}),
        serde_json::json!({"Date": {"date_group": {"levels": ["months"], "start": 43466, "end": true}}}),
        serde_json::json!({"Date": {"date_group": {"levels": ["months"], "start": 60, "end": 44196}}}),
        serde_json::json!({"Date": {"date_group": {"levels": ["months"], "start": 43466, "end": 43466}}}),
        serde_json::json!({"Date": {"date_group": {"levels": ["months"], "start": 43466, "end": 2_958_466}}}),
        serde_json::json!({"Product": {"date_group": d}}),
    ];
    for fields in refused {
        let definition =
            serde_json::json!({"rows": ["Product"], "data": [{"field": "S"}], "fields": fields});
        assert!(try_parts(&definition).is_err(), "{fields}");
    }
    let discrete = serde_json::json!({"Product": {"discrete": groups()}});
    for definition in [
        serde_json::json!({"data": [{"field": "Product2"}], "fields": discrete}),
        serde_json::json!({"data": [{"field": "S"}], "fields": discrete,
                           "calculated": [{"name": "T", "formula": "S*2"}]}),
        serde_json::json!({"rows": ["Product"], "data": [{"field": "S"}], "fields": discrete,
                           "calculated_items": [{"field": "Product", "name": "AB", "formula": "A+B"}]}),
        serde_json::json!({"pages": [{"field": "Date"}], "data": [{"field": "S"}],
                           "fields": {"Date": {"date_group": dates()}}}),
    ] {
        assert!(try_parts(&definition).is_err(), "{definition}");
    }
}

#[test]
fn an_added_fields_item_order_lists_its_items_first() {
    let with = |order: Json| {
        try_parts(&serde_json::json!({
            "rows": ["Product2", "Product"], "data": [{"field": "S"}],
            "fields": {"Product": {"discrete": groups()}, "Product2": {"item_order": order}}}))
    };
    let built = with(serde_json::json!(["Group1"])).unwrap();
    assert!(built
        .table
        .contains(r#"<items count="3"><item x="1"/><item x="0"/><item t="default"/></items>"#));
    for bad in [
        serde_json::json!("Group1"),
        serde_json::json!(vec!["Group1"; 257]),
        serde_json::json!(["B"]),
        serde_json::json!(["A", "a"]),
    ] {
        assert!(with(bad.clone()).is_err(), "{bad}");
    }
}

#[test]
fn texts_fold_as_the_python_builder_folds_them() {
    // `fold` (Python's `str.lower`) and `to_lowercase` agree: `MASS` is `mass`, `Maß` is not
    // (test_pivot_groups.py pins the same).
    let spec: CaseSpec = serde_json::from_str(
        r#"{"format": "rustsheet-case/v1", "sheets": [
            {"name": "D", "cells": {"A1": "K", "B1": "V", "A2": "Maß", "B2": 1, "A3": "mass", "B3": 2,
              "A4": "MASS", "B4": 4}},
            {"name": "P", "cells": {}, "pivot_tables": [
              {"name": "Pv", "source": "D!A1:B4", "location": "A3", "definition": {}}]}]}"#,
    )
    .unwrap();
    let sheet = &spec.sheets[1];
    let definition = serde_json::json!({"rows": ["K"], "data": [{"field": "V"}]});
    let built = parts(&spec, sheet, &sheet.pivot_tables[0], &definition, 1).unwrap();
    assert!(
        built
            .cache_definition
            .contains(r#"<sharedItems count="2"><s v="Maß"/><s v="mass"/></sharedItems>"#),
        "{}",
        built.cache_definition
    );
}
