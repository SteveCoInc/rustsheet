#![allow(clippy::unwrap_used, reason = "tests")]
//! Tests of the builder's parts for a defined pivot (`pivot_definition.rs`). The parts of every
//! probe's pivot were compared with `pivot_definition.py`'s, attribute order aside, when this
//! module was written (T-4152).

use super::*;
use crate::engine::{Engine, RustSheetEngine};

/// A case with K/V source cells on `D` and one defined pivot on `P` at A3.
fn case(definition: &str) -> CaseSpec {
    serde_json::from_str(&format!(
        r#"{{"format": "rustsheet-case/v1", "sheets": [
            {{"name": "D", "cells": {{"A1": "K", "B1": "V", "A2": "east", "B2": 1,
              "A3": "East", "B3": 2, "A4": "=\"west\"", "B4": 3, "A5": true, "B5": 4}}}},
            {{"name": "P", "cells": {{}}, "pivot_tables": [
              {{"name": "Pv", "source": "D!A1:B5", "location": "A3", "definition": {definition}}}]}}]}}"#
    ))
    .unwrap()
}

fn parts_of(spec: &CaseSpec) -> Parts {
    let sheet = &spec.sheets[1];
    let pivot = &sheet.pivot_tables[0];
    parts(spec, sheet, pivot, pivot.definition.as_ref().unwrap(), 7).unwrap()
}

#[test]
fn number_texts_are_the_builders() {
    assert_eq!(number_text(3.0), "3");
    assert_eq!(number_text(-0.0), "0");
    assert_eq!(number_text(0.3), "0.3");
    assert_eq!(number_text(0.300_000_000_000_000_04), "0.30000000000000004");
}

#[test]
fn the_cache_lists_one_item_per_casing_and_a_formula_as_missing() {
    let parts = parts_of(&case(r#"{"rows": ["K"], "data": [{"field": "V"}]}"#));
    assert!(
        parts.cache_definition.contains(
            r#"<sharedItems containsBlank="1" count="3"><s v="east"/><m/><b v="1"/></sharedItems>"#
        ),
        "{}",
        parts.cache_definition
    );
    assert!(parts.cache_definition.contains(r#" refreshOnLoad="1""#));
    // V is all numbers off the axes: inline values, no shared items.
    assert!(parts
        .cache_records
        .contains(r#"<r><x v="0"/><n v="1"/></r><r><x v="0"/><n v="2"/></r><r><x v="1"/>"#));
}

#[test]
fn an_order_lists_its_items_first() {
    let parts = parts_of(&case(
        r#"{"rows": ["K"], "data": [{"field": "V"}], "fields": {"K": {"order": [true]}}}"#,
    ));
    assert!(parts
        .cache_definition
        .contains(r#"count="3"><b v="1"/><s v="east"/><m/>"#));
}

#[test]
fn the_table_gets_the_builders_defaults_and_names() {
    let parts = parts_of(&case(
        r#"{"rows": ["K"], "data": [{"field": "V", "subtotal": "count"}, {"field": "V"}],
            "table": {"outline": false}, "refresh_on_load": false}"#,
    ));
    let table = &parts.table;
    assert!(
        table.contains(r#"name="Pv" cacheId="7" dataCaption="Values""#),
        "{table}"
    );
    assert!(
        table.contains(r#" outlineData="1" outline="0">"#),
        "{table}"
    );
    assert!(table.contains(r#"<location ref="A3:B4""#));
    assert!(table.contains(r#"<colFields count="1"><field x="-2"/></colFields>"#));
    assert!(table.contains(
        r#"<dataField name="Count of V" fld="1" subtotal="count" baseField="0" baseItem="0"/>"#
    ));
    assert!(!parts.cache_definition.contains("refreshOnLoad"));
}

#[test]
fn the_adapter_scores_the_engines_render() {
    let spec = case(r#"{"rows": ["K"], "data": [{"field": "V"}]}"#);
    let evaluated = RustSheetEngine.evaluate_spec(&spec, None).unwrap();
    let cell = |a1: &str| evaluated.value("P", Addr::parse(a1).unwrap());
    assert_eq!(cell("A3"), Some(Value::Text("Row Labels".into())));
    assert_eq!(cell("B3"), Some(Value::Text("Sum of V".into())));
    // The refresh reads the formula's value: `west` is a new item, after the cached ones.
    assert_eq!(cell("A4"), Some(Value::Text("east".into())));
    assert_eq!(cell("B4"), Some(Value::Number(3.0)));
    assert_eq!(cell("A7"), Some(Value::Text("Grand Total".into())));
    assert_eq!(cell("B7"), Some(Value::Number(10.0)));
}

#[test]
fn an_empty_text_items_label_is_an_empty_cell() {
    // probes/t4627-pivot-field-name-collisions P_Rec: a `=""` record is an item of its own,
    // listed after `(blank)`, whose label cell Excel leaves empty.
    let spec: CaseSpec = serde_json::from_str(
        r#"{"format": "rustsheet-case/v1", "sheets": [
            {"name": "D", "cells": {"A1": "K", "B1": "V", "A2": "a", "B2": 1, "A3": "=\"\"", "B3": 2,
              "B4": 4}},
            {"name": "P", "cells": {}, "pivot_tables": [{"name": "Pv", "source": "D!A1:B4",
              "location": "A3", "definition": {"rows": ["K"], "data": [{"field": "V"}]}}]}]}"#,
    )
    .unwrap();
    let evaluated = RustSheetEngine.evaluate_spec(&spec, None).unwrap();
    let cell = |a1: &str| evaluated.value("P", Addr::parse(a1).unwrap());
    assert_eq!(cell("A5"), Some(Value::Text("(blank)".into())));
    assert_eq!(cell("A6"), Some(Value::Empty));
    assert_eq!(cell("B6"), Some(Value::Number(2.0)));
}
