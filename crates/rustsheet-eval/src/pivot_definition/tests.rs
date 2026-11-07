#![allow(clippy::unwrap_used, reason = "tests")]
//! Tests of the builder's parts for a defined pivot (`pivot_definition.rs`). The parts of every
//! probe's pivot were compared with `pivot_definition.py`'s, attribute order aside, when this
//! module was written (T-4152).

use std::fmt::Write as _;

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
fn an_item_order_orders_the_items_but_not_the_cache() {
    let parts = parts_of(&case(
        r#"{"rows": ["K"], "data": [{"field": "V"}], "fields": {"K": {"item_order": [true, "east"]}}}"#,
    ));
    assert!(parts
        .cache_definition
        .contains(r#"count="3"><s v="east"/><m/><b v="1"/>"#));
    assert!(
        parts.table.contains(
            r#"<items count="4"><item x="2"/><item x="0"/><item x="1"/><item t="default"/>"#
        ),
        "{}",
        parts.table
    );
}

#[test]
fn an_item_order_refuses_what_the_python_builder_refuses() {
    // T-4164: `_item_order`'s refusals (not a list, not in the field, repeated), and the
    // `MAX_LIST` bound every list of a definition has.
    for (bad, message) in [
        (r#""east""#, "is a list of items"),
        (r#"["north"]"#, "is not in field"),
        (r#"["east", "east"]"#, "repeats an item"),
    ] {
        let definition = format!(
            r#"{{"rows": ["K"], "data": [{{"field": "V"}}], "fields": {{"K": {{"item_order": {bad}}}}}}}"#
        );
        let error = try_parts(&definition).err().unwrap();
        assert!(error.contains(message), "{bad}: {error}");
    }
    let many = serde_json::to_string(&vec!["east"; MAX_LIST + 1]).unwrap();
    let definition = format!(
        r#"{{"rows": ["K"], "data": [{{"field": "V"}}], "fields": {{"K": {{"item_order": {many}}}}}}}"#
    );
    let error = try_parts(&definition).err().unwrap();
    assert!(error.contains("more than"), "{error}");
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

/// `canonical` of `tools/tests/test_pivot_parts_golden.py`: one line per element, depth-indented,
/// its local name and its attributes by local name, sorted, values unescaped, namespace
/// declarations left out; non-blank text on its own `#text` line.
fn canonical(xml: &str) -> String {
    use quick_xml::events::{BytesStart, Event};
    use quick_xml::XmlVersion;
    let local = |name: &str| name.rsplit(':').next().unwrap().to_string();
    let element_line = |element: &BytesStart<'_>, depth: usize| {
        let mut attrs: Vec<(String, String)> = (element.attributes())
            .map(Result::unwrap)
            .filter(|a| !a.key.into_inner().starts_with("xmlns"))
            .map(|a| {
                let value = a.normalized_value(XmlVersion::Implicit1_0).unwrap();
                (local(a.key.into_inner()), value.into_owned())
            })
            .collect();
        attrs.sort();
        let attrs = attrs.iter().fold(String::new(), |mut out, (k, v)| {
            let _ = write!(out, " {k}={v}");
            out
        });
        format!(
            "{}{}{attrs}",
            "  ".repeat(depth),
            local(element.name().into_inner())
        )
    };
    let mut reader = quick_xml::Reader::from_str(xml);
    let (mut lines, mut depth, mut text) = (Vec::new(), 0_usize, String::new());
    let flush = |lines: &mut Vec<String>, text: &mut String, depth: usize| {
        if !text.trim().is_empty() {
            lines.push(format!("{}#text {}", "  ".repeat(depth), text.trim()));
        }
        text.clear();
    };
    loop {
        match reader.read_event().unwrap() {
            Event::Start(element) => {
                flush(&mut lines, &mut text, depth);
                lines.push(element_line(&element, depth));
                depth += 1;
            }
            Event::Empty(element) => {
                flush(&mut lines, &mut text, depth);
                lines.push(element_line(&element, depth));
            }
            Event::End(_) => {
                flush(&mut lines, &mut text, depth);
                depth -= 1;
            }
            Event::Text(t) => text.push_str(&t.xml_content(XmlVersion::Implicit1_0)),
            Event::GeneralRef(r) => text.push_str(&crate::xlsx_snapshot::resolve_entity(&r)),
            Event::Eof => break,
            _ => {}
        }
    }
    lines.join("\n")
}

#[test]
fn the_parts_are_the_python_builders_golden_parts() {
    let golden = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tools/tests/pivot_parts_golden"
    );
    let mut seen = 0;
    for entry in std::fs::read_dir(golden).unwrap() {
        let path = entry.unwrap().path();
        let file = path.file_stem().unwrap().to_str().unwrap().to_string();
        let (stem, sheet_name) = file.split_once("__").unwrap();
        let case = format!(
            "{}/../../eval/suites/probes/cases/{stem}.case.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let spec: CaseSpec = serde_json::from_str(&std::fs::read_to_string(case).unwrap()).unwrap();
        let sheet = spec.sheets.iter().find(|s| s.name == sheet_name).unwrap();
        let pivot = &sheet.pivot_tables[0];
        let built = parts(&spec, sheet, pivot, pivot.definition.as_ref().unwrap(), 1).unwrap();
        let mut text = String::new();
        for (name, xml) in [
            ("cache_definition", &built.cache_definition),
            ("cache_records", &built.cache_records),
            ("table", &built.table),
        ] {
            let _ = writeln!(text, "== {name}\n{}", canonical(xml));
        }
        assert_eq!(text, std::fs::read_to_string(&path).unwrap(), "{file}");
        seen += 1;
    }
    assert!(seen >= 8, "{seen} golden files");
}

#[test]
fn a_group_step_too_small_to_move_is_refused() {
    let spec = case(r#"{"rows": ["K"], "data": [{"field": "V"}]}"#);
    let mut definition: Json = serde_json::from_str(
        r#"{"rows": ["V"], "data": [{"field": "V", "subtotal": "count"}],
            "fields": {"V": {"group": {"start": 1e20, "end": 1e20, "interval": 2}}}}"#,
    )
    .unwrap();
    let sheet = &spec.sheets[1];
    let pivot = &sheet.pivot_tables[0];
    let error = parts(&spec, sheet, pivot, &definition, 1).err().unwrap();
    assert!(error.contains("10,000 bins"), "{error}");
    definition["fields"]["V"]["group"] = serde_json::json!({"start": 0, "end": 1e9, "interval": 1});
    assert!(parts(&spec, sheet, pivot, &definition, 1).is_err());
}

#[test]
fn unbounded_sources_and_lists_are_refused() {
    let mut spec = case(r#"{"rows": ["K"], "data": [{"field": "V"}]}"#);
    spec.sheets[1].pivot_tables[0].source = "D!A1:XFD1048576".into();
    let sheet = &spec.sheets[1];
    let pivot = &sheet.pivot_tables[0];
    let error = parts(&spec, sheet, pivot, pivot.definition.as_ref().unwrap(), 1)
        .err()
        .unwrap();
    assert!(error.contains("more than"), "{error}");
    let spec = case(r#"{"rows": ["K"], "data": [{"field": "V"}]}"#);
    let sheet = &spec.sheets[1];
    let pivot = &sheet.pivot_tables[0];
    let many =
        serde_json::json!({"rows": ["K"], "data": vec![serde_json::json!({"field": "V"}); 300]});
    assert!(parts(&spec, sheet, pivot, &many, 1).is_err());
    let bad = serde_json::json!({"rows": ["K"], "data": [{"field": "V"}], "table": {"a\" b": 1}});
    assert!(parts(&spec, sheet, pivot, &bad, 1).is_err());
    let show = serde_json::json!({"rows": ["K"], "data": [{"field": "V", "showDataAs": "x\"/>"}]});
    assert!(parts(&spec, sheet, pivot, &show, 1).is_err());
}

/// The parts of `definition`, or the builder's refusal.
fn try_parts(definition: &str) -> Result<Parts, String> {
    let spec = case(definition);
    let sheet = &spec.sheets[1];
    let pivot = &sheet.pivot_tables[0];
    parts(&spec, sheet, pivot, pivot.definition.as_ref().unwrap(), 7)
}

#[test]
fn calculated_items_and_page_attributes_refuse_what_the_builder_refuses() {
    // T-4130: `_calculated_items` keys, the managed `formula`, a raw pivotArea, and the
    // managed pageField attributes.
    let ok = try_parts(
        r#"{"rows": ["K"], "data": [{"field": "V"}],
            "calculated_items": [{"field": "K", "name": "Both", "formula": "east+west",
                                  "attrs": {"field": 0}, "pivot_area": "<pivotArea outline=\"0\"/>"}]}"#,
    )
    .unwrap();
    assert!(ok.cache_definition.contains(
        r#"<calculatedItem formula="east+west" field="0"><pivotArea outline="0"/></calculatedItem>"#
    ));
    for bad in [
        r#"{"field": "K", "name": "X", "formula": "1", "colour": "red"}"#,
        r#"{"field": "K", "name": "X", "formula": "1", "attrs": {"formula": "2"}}"#,
        r#"{"field": "K", "name": "X", "formula": "1", "pivot_area": "<references/>"}"#,
        r#"{"field": "K", "name": "X", "formula": "1", "pivot_area": "<pivotArea a=\"&x;\"/>"}"#,
        r#"{"field": "K", "name": "X"}"#,
        r#"{"field": "K", "name": "east", "formula": "1"}"#,
    ] {
        let definition = format!(
            r#"{{"rows": ["K"], "data": [{{"field": "V"}}], "calculated_items": [{bad}]}}"#
        );
        assert!(try_parts(&definition).is_err(), "{bad}");
    }
    assert!(try_parts(
        r#"{"pages": [{"field": "K", "attrs": {"hier": 0}}], "data": [{"field": "V"}]}"#
    )
    .is_err());
    let page = try_parts(
        r#"{"pages": [{"field": "K", "attrs": {"cap": "C"}}], "data": [{"field": "V"}]}"#,
    )
    .unwrap();
    assert!(page
        .table
        .contains(r#"<pageField fld="0" hier="-1" cap="C"/>"#));
}
