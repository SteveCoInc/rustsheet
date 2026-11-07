#![allow(clippy::unwrap_used, clippy::panic, reason = "tests")]
//! Every probe's defined pivot, loaded through the importer from the builder's parts, comes
//! back from the xlsx export equal (T-4176; SPEC-0032 "Test plan": import, export, import is a
//! fixed point). The export writes the engine's `rowItems`/`colItems` where its layout fits the
//! location, which the builder's parts leave out, so those are compared apart.

use std::io::Cursor;
use std::path::Path;

use rustsheet_engine::pivot::{CacheField, OpaqueNode, PivotTable, MAX_RECORD_VALUES};
use rustsheet_engine::types::DateSystem;
use rustsheet_engine::Model;
use rustsheet_xlsx::export::save_xlsx_to_writer;
use rustsheet_xlsx::import::{load_from_xlsx_bytes, read_pivot_parts, PivotParts};

use super::parts;
use crate::case::CaseSpec;

fn without_lines(mut table: PivotTable) -> PivotTable {
    table.opaque.retain(|entry| match &entry.node {
        OpaqueNode::Element(xml) => !xml.starts_with("<rowItems") && !xml.starts_with("<colItems"),
        OpaqueNode::Attributes(_) => true,
    });
    table
}

#[test]
fn every_probe_pivot_round_trips_through_the_export() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../eval/suites/probes/cases");
    let mut paths: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.to_string_lossy().ends_with(".case.json"))
        .collect();
    paths.sort();
    let (mut checked, mut grouped, mut not_loaded) = (0, 0, 0);
    for path in paths {
        let text = std::fs::read_to_string(&path).unwrap();
        if !text.contains("\"definition\"") {
            continue;
        }
        let spec: CaseSpec = serde_json::from_str(&text).unwrap();
        for sheet in &spec.sheets {
            for pivot in &sheet.pivot_tables {
                let Some(definition) = &pivot.definition else {
                    continue;
                };
                let what = format!("{} {}", path.display(), pivot.name);
                let built = parts(&spec, sheet, pivot, definition, 1).unwrap();
                let mut model = Model::new_empty("probe", "en", "UTC", "en").unwrap();
                let sheet_id = model.workbook.worksheets[0].sheet_id;
                let table = (sheet_id, built.table.as_bytes());
                let loaded = read_pivot_parts(
                    PivotParts {
                        cache_definition: built.cache_definition.as_bytes(),
                        cache_records: built.cache_records.as_bytes(),
                        cache_id: 1,
                    },
                    &[table],
                    0,
                    MAX_RECORD_VALUES,
                    DateSystem::Excel1900,
                );
                // A calculated field, a `showDataAs` data field: the importer drops the pivot.
                let Ok((cache, tables)) = loaded else {
                    not_loaded += 1;
                    continue;
                };
                // A manual group's added field counts too (T-4171).
                let is_grouped = cache.fields.iter().any(CacheField::is_grouped);
                model.workbook.pivot_caches.push(cache);
                model.workbook.pivot_tables.extend(tables);
                let bytes = save_xlsx_to_writer(&model, Cursor::new(Vec::new()))
                    .unwrap()
                    .into_inner();
                let back = load_from_xlsx_bytes(&bytes, "back", "en", "UTC").unwrap();
                if is_grouped {
                    // Dropped with its pivots until grouped fields export (T-4734).
                    assert!(back.pivot_tables.is_empty(), "{what}");
                    grouped += 1;
                    continue;
                }
                assert_eq!(back.pivot_caches, model.workbook.pivot_caches, "{what}");
                let tables =
                    |t: &[PivotTable]| t.iter().cloned().map(without_lines).collect::<Vec<_>>();
                assert_eq!(
                    tables(&back.pivot_tables),
                    tables(&model.workbook.pivot_tables),
                    "{what}"
                );
                checked += 1;
            }
        }
    }
    eprintln!(
        "{checked} probe pivots round-trip, {grouped} grouped dropped, {not_loaded} not loaded"
    );
    assert!(checked >= 100, "only {checked} probe pivots checked");
}
