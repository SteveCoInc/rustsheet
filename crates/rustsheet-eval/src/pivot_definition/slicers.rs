//! A spec sheet's `slicers` and `timelines` (T-4767's `tools/src/rustsheet_tools/pivot_slicers.py`)
//! as the slicer and timeline cache parts the case builder writes for them, so the engine adapter
//! loads them through the xlsx importer with the pivots they drive (T-4188), as it loads a
//! defined pivot.
//!
//! Only the cache parts are mirrored: they hold all the filter state the importer reads (the
//! pivots they drive, the field, the selection). The slicers and timelines parts, the drawing
//! frames, the extension-list entries, the defined names and the pivot cache's
//! `x14:pivotCacheDefinition` carry none. The names are still given out in the builder's order
//! (`_Package.unique`, shapes included), so every cache is named as the builder names it. A
//! table slicer filters a table, not a pivot, so it writes no part here (its names are taken).
//! The golden parts in `tools/tests/slicer_parts_golden/` guard the mirror both ways.

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;

use quick_xml::escape::escape;
use quick_xml::events::Event;
use quick_xml::XmlVersion;
use serde_json::Value as Json;

use super::checks::{managed_attrs, raw_element};
use super::groups::iso_date;
use super::{bounded, MAIN_NS};
use crate::case::{CaseSpec, SheetSpec};

const X14_NS: &str = "http://schemas.microsoft.com/office/spreadsheetml/2009/9/main";
const X15_NS: &str = "http://schemas.microsoft.com/office/spreadsheetml/2010/11/main";
const MC_NS: &str = "http://schemas.openxmlformats.org/markup-compatibility/2006";
const SLICER_KEYS: [&str; 14] = [
    "pivot",
    "table",
    "field",
    "column",
    "anchor",
    "name",
    "caption",
    "selected",
    "item_attrs",
    "tabular",
    "slicer",
    "cache_ext",
    "pivot_cache",
    "table_slicer",
];
const TIMELINE_KEYS: [&str; 11] = [
    "pivot",
    "field",
    "anchor",
    "name",
    "caption",
    "bounds",
    "selection",
    "state",
    "timeline",
    "pivot_filter",
    "timeline_pivot_cache",
];

/// What a slicer or timeline needs of its pivot (`_Pivot`): the `sheetId` of its sheet, its
/// cache's `cacheId`, and that cache's definition part, whose fields it reads.
pub(crate) struct BuiltPivot<'a> {
    pub(crate) tab_id: u32,
    pub(crate) cache_id: u32,
    pub(crate) cache_definition: &'a str,
}

/// One slicer or timeline cache part, and the pivot it drives.
pub(crate) struct SlicerPart {
    pub(crate) pivot: String,
    pub(crate) xml: String,
}

/// The slicer and timeline cache parts of `spec`, in the builder's order (sheet by sheet, each
/// sheet's slicers, then its timelines), over the defined pivots `pivots` by name.
///
/// # Errors
/// An entry the builder would refuse.
pub(crate) fn slicer_parts(
    spec: &CaseSpec,
    pivots: &HashMap<String, BuiltPivot<'_>>,
) -> Result<Vec<SlicerPart>, String> {
    let mut names = Names::default();
    let mut out = Vec::new();
    let mut fields = HashMap::new();
    for sheet in &spec.sheets {
        for entry in bounded(&sheet.slicers, "slicers")? {
            if let Some(part) = slicer(entry, sheet, pivots, &mut fields, &mut names)? {
                out.push(part);
            }
        }
        for entry in bounded(&sheet.timelines, "timelines")? {
            out.push(timeline(entry, sheet, pivots, &mut names)?);
        }
    }
    Ok(out)
}

/// The names given so far, folded (`_Package.used`).
#[derive(Default)]
struct Names(HashSet<String>);

impl Names {
    /// `base`, or `base` with the first free number after `sep` (`_Package.unique`).
    fn unique(&mut self, base: &str, sep: &str) -> String {
        let mut name = base.to_string();
        let mut k = 0_usize;
        while self.0.contains(&name.to_lowercase()) {
            k += 1;
            name = format!("{base}{sep}{k}");
        }
        self.0.insert(name.to_lowercase());
        name
    }
}

/// `entry` as an object with only `keys`.
fn object<'j>(
    entry: &'j Json,
    keys: &[&str],
    place: &str,
) -> Result<&'j serde_json::Map<String, Json>, String> {
    let entry = entry
        .as_object()
        .ok_or_else(|| format!("{place} is an object"))?;
    if let Some(key) = entry.keys().find(|k| !keys.contains(&k.as_str())) {
        return Err(format!("{place} may not have key {key:?}"));
    }
    Ok(entry)
}

/// A text of `entry` (Python's `str()` of a JSON string).
fn text(entry: &serde_json::Map<String, Json>, key: &str) -> Option<String> {
    entry.get(key).map(|v| match v {
        Json::String(s) => s.clone(),
        other => other.to_string(),
    })
}

/// The shared item texts of each cache field of `definition` that lists them (`_pivots`: an
/// `<s|n|b|e|d|m v/>` item, its `v` or empty).
fn shared_items(definition: &str) -> Result<HashMap<String, Vec<String>>, String> {
    let mut reader = quick_xml::Reader::from_str(definition);
    let mut out = HashMap::new();
    let mut field: Option<String> = None;
    let mut items: Option<Vec<String>> = None;
    loop {
        let event = reader.read_event().map_err(|e| e.to_string())?;
        match &event {
            Event::Start(e) | Event::Empty(e) => {
                let attribute = |key: &str| -> Result<Option<String>, String> {
                    let found = e.try_get_attribute(key).map_err(|err| err.to_string())?;
                    found
                        .map(|a| {
                            a.normalized_value(XmlVersion::Implicit1_0)
                                .map(std::borrow::Cow::into_owned)
                        })
                        .transpose()
                        .map_err(|err| err.to_string())
                };
                match e.local_name().as_ref() {
                    "cacheField" => {
                        field = attribute("name")?;
                        if matches!(event, Event::Empty(_)) {
                            if let Some(name) = field.take() {
                                out.entry(name).or_insert_with(Vec::new);
                            }
                        }
                    }
                    "sharedItems" if matches!(event, Event::Start(_)) => items = Some(Vec::new()),
                    "s" | "n" | "b" | "e" | "d" | "m" if matches!(event, Event::Empty(_)) => {
                        let only_v = e
                            .attributes()
                            .all(|a| a.is_ok_and(|a| a.key.as_ref() == "v"));
                        if let (Some(list), true) = (items.as_mut(), only_v) {
                            list.push(attribute("v")?.unwrap_or_default());
                        }
                    }
                    _ => {}
                }
            }
            Event::End(e) if e.local_name().as_ref() == "cacheField" => {
                if let Some(name) = field.take() {
                    out.insert(name, items.take().unwrap_or_default());
                }
                items = None;
            }
            Event::Eof => return Ok(out),
            _ => {}
        }
    }
}

fn root() -> String {
    format!(r#"xmlns:mc="{MC_NS}" mc:Ignorable="x" xmlns:x="{MAIN_NS}""#)
}

/// One slicer's cache part (`_slicer`); `None` for a table slicer.
fn slicer(
    entry: &Json,
    sheet: &SheetSpec,
    pivots: &HashMap<String, BuiltPivot<'_>>,
    fields: &mut HashMap<u32, HashMap<String, Vec<String>>>,
    names: &mut Names,
) -> Result<Option<SlicerPart>, String> {
    let place = format!("sheet {}: slicer", sheet.name);
    let e = object(entry, &SLICER_KEYS, &place)?;
    let table = e.contains_key("table");
    if table == e.contains_key("pivot") || !e.contains_key("anchor") {
        return Err(format!("{place} names a pivot or a table, and an anchor"));
    }
    let source = text(e, if table { "column" } else { "field" }).unwrap_or_default();
    let name = names.unique(
        &text(e, "name").unwrap_or_else(|| format!("Slicer_{source}")),
        "",
    );
    let caption = text(e, "caption").unwrap_or_else(|| source.clone());
    let part = if table {
        None
    } else {
        let pivot_name = text(e, "pivot").unwrap_or_default();
        let ext = match e.get("cache_ext") {
            Some(raw) => raw_element(raw.as_str().ok_or("cache_ext is raw XML")?, "extLst")?,
            None => String::new(),
        };
        let pivot = pivots
            .get(&pivot_name)
            .ok_or_else(|| format!("{place}: pivot {pivot_name:?} is not defined"))?;
        let cache_fields = match fields.entry(pivot.cache_id) {
            std::collections::hash_map::Entry::Occupied(found) => found.into_mut(),
            std::collections::hash_map::Entry::Vacant(slot) => {
                slot.insert(shared_items(pivot.cache_definition)?)
            }
        };
        let items = (cache_fields.get(&source))
            .filter(|items| !items.is_empty())
            .ok_or_else(|| {
                format!("{place}: pivot {pivot_name:?} has no listed field {source:?}")
            })?;
        let tabular = tabular(e, &pivot_name, pivot, items, &place)?;
        Some(SlicerPart {
            pivot: pivot_name,
            xml: format!(
                r#"<slicerCacheDefinition xmlns="{X14_NS}" {} name="{}" sourceName="{}">{tabular}{ext}</slicerCacheDefinition>"#,
                root(),
                escape(&name),
                escape(&source)
            ),
        })
    };
    names.unique(&caption, " ");
    Ok(part)
}

/// A pivot slicer's `<pivotTables>` and `<data>` (`_tabular`).
fn tabular(
    e: &serde_json::Map<String, Json>,
    pivot_name: &str,
    pivot: &BuiltPivot<'_>,
    items: &[String],
    place: &str,
) -> Result<String, String> {
    let text_of = |v: &Json| match v {
        Json::String(s) => s.clone(),
        other => other.to_string(),
    };
    let chosen: Vec<String> = match e.get("selected") {
        None => items.to_vec(),
        Some(Json::Array(list)) => bounded(list, "selected")?.iter().map(text_of).collect(),
        Some(_) => return Err(format!("{place}: selected is a list of items")),
    };
    let empty = serde_json::Map::new();
    let item_attrs = match e.get("item_attrs") {
        None | Some(Json::Null) => &empty,
        Some(Json::Object(attrs)) => attrs,
        Some(_) => return Err(format!("{place}: item_attrs is an object")),
    };
    if let Some(missing) = (chosen.iter())
        .chain(item_attrs.keys())
        .find(|v| !items.contains(v))
    {
        return Err(format!("{place}: item {missing:?} is not in the field"));
    }
    let mut listed = String::new();
    for (k, v) in items.iter().enumerate() {
        let selected = if chosen.contains(v) { r#" s="1""# } else { "" };
        let attrs = managed_attrs(item_attrs.get(v), &["x", "s"])?;
        let _ = write!(listed, r#"<i x="{k}"{selected}{attrs}/>"#);
    }
    let attrs = managed_attrs(e.get("tabular"), &["pivotCacheId"])?;
    Ok(format!(
        r#"<pivotTables><pivotTable tabId="{}" name="{}"/></pivotTables><data><tabular pivotCacheId="{}"{attrs}><items count="{}">{listed}</items></tabular></data>"#,
        pivot.tab_id,
        escape(pivot_name),
        pivot.cache_id,
        items.len()
    ))
}

/// A date serial of a timeline entry (`_date`).
fn date(raw: &Json, place: &str) -> Result<String, String> {
    let serial = raw
        .as_f64()
        .filter(|_| raw.is_number())
        .ok_or_else(|| format!("{place} is a date serial"))?;
    iso_date(serial)
}

/// One timeline's cache part (`_timeline`).
fn timeline(
    entry: &Json,
    sheet: &SheetSpec,
    pivots: &HashMap<String, BuiltPivot<'_>>,
    names: &mut Names,
) -> Result<SlicerPart, String> {
    let place = format!("sheet {}: timeline", sheet.name);
    let e = object(entry, &TIMELINE_KEYS, &place)?;
    let pivot_name = text(e, "pivot").unwrap_or_default();
    let source = text(e, "field").unwrap_or_default();
    let pivot = pivots
        .get(&pivot_name)
        .filter(|p| shared_items(p.cache_definition).is_ok_and(|f| f.contains_key(&source)))
        .filter(|_| e.contains_key("anchor"))
        .ok_or_else(|| {
            format!("{place}: names a pivot of the workbook, one of its fields and an anchor")
        })?;
    let pair = |key: &str| match e.get(key) {
        None | Some(Json::Null) => Ok(None),
        Some(Json::Array(list)) if list.len() == 2 => Ok(Some((&list[0], &list[1]))),
        Some(_) => Err(format!(
            "{place}: bounds (and selection) are [start, end] date serials"
        )),
    };
    let bounds = pair("bounds")?
        .ok_or_else(|| format!("{place}: bounds (and selection) are [start, end] date serials"))?;
    let selection = pair("selection")?;
    let name = names.unique(
        &text(e, "name").unwrap_or_else(|| format!("NativeTimeline_{source}")),
        "",
    );
    let caption = text(e, "caption").unwrap_or_else(|| source.clone());
    let chosen = match selection {
        Some((start, end)) => format!(
            r#"<selection startDate="{}" endDate="{}"/>"#,
            date(start, &place)?,
            date(end, &place)?
        ),
        None => String::new(),
    };
    let mut state = managed_attrs(e.get("state"), &["pivotCacheId"])?;
    let typed =
        (e.get("state").and_then(Json::as_object)).is_some_and(|s| s.contains_key("filterType"));
    if !typed {
        let kind = if selection.is_some() {
            "dateBetween"
        } else {
            "unknown"
        };
        let _ = write!(state, r#" filterType="{kind}""#);
    }
    let pivot_filter = match e.get("pivot_filter") {
        Some(raw) => raw_element(
            raw.as_str().ok_or("pivot_filter is raw XML")?,
            "timelinePivotFilter",
        )?,
        None => String::new(),
    };
    let xml = format!(
        r#"<timelineCacheDefinition xmlns="{X15_NS}" {} name="{}" sourceName="{}"><pivotTables><pivotTable tabId="{}" name="{}"/></pivotTables><state minimalRefreshVersion="6" lastRefreshVersion="6" pivotCacheId="{}"{state}>{chosen}<bounds startDate="{}" endDate="{}"/></state>{pivot_filter}</timelineCacheDefinition>"#,
        root(),
        escape(&name),
        escape(&source),
        pivot.tab_id,
        escape(&pivot_name),
        pivot.cache_id,
        date(bounds.0, &place)?,
        date(bounds.1, &place)?
    );
    names.unique(&caption, " ");
    Ok(SlicerPart {
        pivot: pivot_name,
        xml,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests")]
    use super::*;
    use crate::pivot_definition::parts;
    use crate::pivot_definition::tests::canonical;

    /// The slicer and timeline cache parts of probe `stem`, its defined pivots numbered and
    /// placed as the case builder numbers them (cacheId in order, `sheetId` the sheet's place).
    fn built(stem: &str) -> Result<Vec<SlicerPart>, String> {
        let case = format!(
            "{}/../../eval/suites/probes/cases/{stem}.case.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let spec: CaseSpec = serde_json::from_str(&std::fs::read_to_string(case).unwrap()).unwrap();
        built_from(&spec)
    }

    fn built_from(spec: &CaseSpec) -> Result<Vec<SlicerPart>, String> {
        let mut defined = Vec::new();
        for (place, sheet) in (1_u32..).zip(&spec.sheets) {
            for pivot in &sheet.pivot_tables {
                let Some(definition) = &pivot.definition else {
                    continue;
                };
                let cache_id = u32::try_from(defined.len() + 1).unwrap();
                let part = parts(spec, sheet, pivot, definition, cache_id).unwrap();
                defined.push((pivot.name.clone(), place, cache_id, part.cache_definition));
            }
        }
        let pivots = (defined.iter())
            .map(|(name, tab_id, cache_id, xml)| {
                let pivot = BuiltPivot {
                    tab_id: *tab_id,
                    cache_id: *cache_id,
                    cache_definition: xml,
                };
                (name.clone(), pivot)
            })
            .collect();
        slicer_parts(spec, &pivots)
    }

    #[test]
    fn the_parts_are_the_python_builders_golden_parts() {
        for stem in ["t4767-pivot-slicers", "t4767-pivot-timelines"] {
            let mut text = String::new();
            for part in built(stem).unwrap() {
                let _ = writeln!(text, "== cache\n{}", canonical(&part.xml));
            }
            let golden = format!(
                "{}/../../tools/tests/slicer_parts_golden/{stem}.txt",
                env!("CARGO_MANIFEST_DIR")
            );
            assert_eq!(text, std::fs::read_to_string(golden).unwrap(), "{stem}");
        }
    }

    /// One sheet holding a defined pivot P over Data!A1:B3 (Region East, West) and `entries`.
    fn spec_with(key: &str, entries: &str) -> CaseSpec {
        serde_json::from_str(&format!(
            r#"{{"format": "rustsheet-case/v1", "sheets": [{{"name": "Data", "cells": {{"A1": "Region", "B1": "Sales", "A2": "East",
                "B2": 1, "A3": "West", "B3": 2}}}},
              {{"name": "P", "cells": {{}}, "pivot_tables": [{{"name": "P", "source": "Data!A1:B3",
                "location": "A3", "definition": {{"rows": ["Region"], "data": [{{"field": "Sales"}}]}}}}],
                "{key}": {entries}}}], "expected": {{}}}}"#
        ))
        .unwrap()
    }

    #[test]
    fn the_builders_refusals_are_refused() {
        for (key, entries) in [
            ("slicers", r#"[{"pivot": "P", "field": "Region"}]"#),
            (
                "slicers",
                r#"[{"pivot": "P", "table": "T", "anchor": "E3"}]"#,
            ),
            (
                "slicers",
                r#"[{"pivot": "Q", "field": "Region", "anchor": "E3"}]"#,
            ),
            (
                "slicers",
                r#"[{"pivot": "P", "field": "Sales", "anchor": "E3"}]"#,
            ),
            (
                "slicers",
                r#"[{"pivot": "P", "field": "Region", "anchor": "E3", "selected": ["North"]}]"#,
            ),
            (
                "slicers",
                r#"[{"pivot": "P", "field": "Region", "anchor": "E3", "item_attrs": {"East": {"s": 1}}}]"#,
            ),
            (
                "slicers",
                r#"[{"pivot": "P", "field": "Region", "anchor": "E3", "tabular": {"pivotCacheId": 9}}]"#,
            ),
            (
                "slicers",
                r#"[{"pivot": "P", "field": "Region", "anchor": "E3", "cache_ext": "<x/>"}]"#,
            ),
            (
                "slicers",
                r#"[{"pivot": "P", "field": "Region", "anchor": "E3", "colour": 1}]"#,
            ),
            (
                "timelines",
                r#"[{"pivot": "P", "field": "Region", "anchor": "E3"}]"#,
            ),
            (
                "timelines",
                r#"[{"pivot": "P", "field": "Nope", "anchor": "E3", "bounds": [1, 2]}]"#,
            ),
            (
                "timelines",
                r#"[{"pivot": "P", "field": "Region", "anchor": "E3", "bounds": [1, true]}]"#,
            ),
            (
                "timelines",
                r#"[{"pivot": "P", "field": "Region", "anchor": "E3", "bounds": [1, 2], "selection": [1]}]"#,
            ),
            (
                "timelines",
                r#"[{"pivot": "P", "field": "Region", "anchor": "E3", "bounds": [1, 2], "pivot_filter": "<x/>"}]"#,
            ),
        ] {
            assert!(
                built_from(&spec_with(key, entries)).is_err(),
                "{key} {entries}"
            );
        }
        let fine = spec_with(
            "slicers",
            r#"[{"pivot": "P", "field": "Region", "anchor": "E3"}]"#,
        );
        assert_eq!(built_from(&fine).unwrap().len(), 1);
        let many = format!("[{}]", vec![r#"{"pivot": "P"}"#; 300].join(","));
        let error = built_from(&spec_with("slicers", &many)).err().unwrap();
        assert!(error.contains("more than"), "{error}");
    }
}
