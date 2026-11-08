#![allow(clippy::unwrap_used, clippy::panic, reason = "tests")]
//! Refreshing pivots over a defined name, a Table and a shared cache, against
//! probes/t4506-pivot-named-sources and t4506-pivot-shared-cache (T-4506): each pivot's cache is
//! the case builder's, read from the case's `source` range, while Excel refreshes it from the
//! name, the Table or the one shared cache. The expectations are the sidecars' `pivots`.

use serde_json::Value as Json;

use super::*;
use crate::pivot::probe_cases::{position, probe_model, probe_pivot_nth, read, sheet};
use crate::pivot::refresh::SourceCells;
use crate::pivot::CacheSource;

const NAMED: &str = "t4506-pivot-named-sources";
const SHARED: &str = "t4506-pivot-shared-cache";

/// Probe `stem`'s cells, defined names and Tables in a model, evaluated.
fn model_of(case: &Json) -> Model {
    let mut model = probe_model(case);
    for (name, formula) in case["names"].as_object().into_iter().flatten() {
        let formula = formula.as_str().unwrap();
        let added = model.define_loaded_name(name, None, formula);
        assert!(added.is_ok(), "{name} {formula}: {added:?}");
    }
    for (index, spec) in (0u32..).zip(case["sheets"].as_array().unwrap()) {
        for table in spec["tables"].as_array().into_iter().flatten() {
            let (from, to) = table["ref"].as_str().unwrap().split_once(':').unwrap();
            let ((left, top), (right, bottom)) = (position(from), position(to));
            let line = |n: u32| i32::try_from(n + 1).unwrap();
            let range = (line(top), line(left), line(bottom), line(right));
            let name = table["name"].as_str().unwrap();
            model.add_table(index, name, range, false).unwrap();
        }
    }
    model.evaluate();
    model
}

/// The cache the case builder saves for the `nth` pivot of sheet `name`, read by the engine from
/// its `source` range, then given the pivot's `source_name` in place of that range.
fn builder_cache(model: &Model, case: &Json, name: &str, nth: usize) -> PivotCache {
    let spec = &sheet(case, name)["pivot_tables"][nth];
    let (sheet_name, range) = spec["source"].as_str().unwrap().split_once('!').unwrap();
    let source = PivotCache {
        source: Some(CacheSource {
            sheet_name: sheet_name.into(),
            range: range.into(),
        }),
        ..PivotCache::default()
    };
    let mut cache = refresh(&source, model, MAX_RECORD_VALUES).unwrap();
    if let Some(name) = spec["definition"]["source_name"].as_str() {
        cache.source = None;
        cache.source_name = Some(name.to_string());
    }
    cache
}

/// Adds the `nth` pivot of sheet `name` over the cache at `cache`, or over a new cache
/// ([`builder_cache`]) when `cache` is `None`.
fn add_pivot(model: &mut Model, case: &Json, name: &str, nth: usize, cache: Option<u32>) {
    let built = builder_cache(model, case, name, nth);
    let (built, mut table) = probe_pivot_nth(case, name, nth, built);
    let index = model.get_sheet_index_by_name(name).unwrap();
    table.sheet_id = model.workbook.worksheet(index).unwrap().sheet_id;
    table.cache = cache.unwrap_or_else(|| {
        model.workbook.pivot_caches.push(built);
        u32::try_from(model.workbook.pivot_caches.len() - 1).unwrap()
    });
    model.workbook.pivot_tables.push(table);
}

/// The cells of `stem`'s sidecar pivot on sheet `name` that `model` does not hold.
fn differences(model: &Model, stem: &str, name: &str, nth: usize) -> Vec<String> {
    let oracle = read(stem, "oracle");
    let pivot = (oracle["pivots"].as_array().unwrap().iter())
        .filter(|p| p["sheet"] == name)
        .nth(nth)
        .unwrap();
    let sheet = model.get_sheet_index_by_name(name).unwrap();
    let mut wrong = Vec::new();
    for (at, expected) in pivot["cells"].as_object().unwrap() {
        let (column, row) = position(at);
        let (row, column) = (
            i32::try_from(row + 1).unwrap(),
            i32::try_from(column + 1).unwrap(),
        );
        let mine = model.get_formatted_cell_value(sheet, row, column).unwrap();
        let excel = match &expected["v"] {
            Json::String(text) => text.clone(),
            Json::Number(n) => n.as_f64().unwrap().to_string(),
            other => panic!("{at}: {other}"),
        };
        if mine != excel {
            wrong.push(format!("{name}!{at}: Excel {excel}, engine {mine}"));
        }
    }
    wrong
}

fn refresh_all(model: &mut Model) {
    for cache in 0..u32::try_from(model.workbook.pivot_caches.len()).unwrap() {
        let plan = model
            .plan_pivot_refresh(cache, &mut PivotBudget::default())
            .unwrap();
        model.apply_pivot_refresh(plan).unwrap();
    }
}

#[test]
fn a_named_or_table_source_is_read_again_at_the_refresh() {
    // N a static name, O an OFFSET name, I an INDEX:INDEX name, T a Table: each cache saved from
    // Data!A1:B4 (three records), refreshed from the five records the name stands for.
    let case = read(NAMED, "case");
    for name in ["N", "O", "I", "T"] {
        let mut model = model_of(&case);
        add_pivot(&mut model, &case, name, 0, None);
        assert_eq!(model.workbook.pivot_caches[0].record_count, 3, "{name}");
        refresh_all(&mut model);
        let cache = &model.workbook.pivot_caches[0];
        assert_eq!(cache.record_count, 5, "{name}");
        // The name stays the cache's source, so the export writes it back.
        assert!(
            cache.source.is_none() && cache.source_name.is_some(),
            "{name}"
        );
        let wrong = differences(&model, NAMED, name, 0);
        assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    }
}

#[test]
fn a_dynamic_name_resolves_as_its_source_stands_at_the_refresh() {
    let case = read(NAMED, "case");
    let mut model = model_of(&case);
    add_pivot(&mut model, &case, "O", 0, None);
    let data = model.get_sheet_index_by_name("Data").unwrap();
    model.set_user_input(data, 7, 1, "Mid".into()).unwrap();
    model.set_user_input(data, 7, 2, "4".into()).unwrap();
    model.evaluate();
    let area = model.pivot_source_area(0).unwrap();
    assert_eq!((area.sheet, area.area.top, area.area.bottom), (data, 0, 6));
    assert_eq!((area.area.left, area.area.right), (0, 1));
}

#[test]
fn a_source_that_is_no_range_or_no_name_does_not_refresh() {
    let case = read(NAMED, "case");
    let mut model = model_of(&case);
    model.define_loaded_name("Scalar", None, "=1+2").unwrap();
    model.define_loaded_name("Loop", None, "=Loop").unwrap();
    model
        .define_loaded_name("Union", None, "=Data!$A$1:$B$2,Data!$A$4:$B$5")
        .unwrap();
    model
        .define_loaded_name("Local", Some(0), "=Data!$A$1:$B$6")
        .unwrap();
    add_pivot(&mut model, &case, "N", 0, None);
    let before = model.workbook.pivot_caches[0].clone();
    // An unknown name, a value, a name that refers to itself, several areas, a sheet-scoped
    // name or a formula in the attribute: no area, and the plan is refused.
    for source in [
        "Nope",
        "Scalar",
        "Loop",
        "Union",
        "Local",
        "Data!Local",
        "OFFSET(Data!$A$1,0,0,6,2)",
        "Data!A1:B6",
    ] {
        model.workbook.pivot_caches[0].source_name = Some(source.to_string());
        assert_eq!(model.pivot_source_area(0), None, "{source}");
        assert_eq!(
            model
                .plan_pivot_refresh(0, &mut PivotBudget::default())
                .err(),
            Some(PivotRefreshError::Cache(PivotCacheError::Source)),
            "{source}"
        );
    }
    model.workbook.pivot_caches[0].source_name = before.source_name.clone();
    assert_eq!(model.workbook.pivot_caches[0], before);
}

#[test]
fn a_whole_sheet_offset_is_refused_by_the_record_limits() {
    let case = read(NAMED, "case");
    let mut model = model_of(&case);
    model
        .define_loaded_name("Huge", None, "=OFFSET(Data!$A$1,0,0,1048576,16384)")
        .unwrap();
    add_pivot(&mut model, &case, "N", 0, None);
    model.workbook.pivot_caches[0].source_name = Some("Huge".to_string());
    let area = model.pivot_source_area(0).unwrap().area;
    assert_eq!((area.bottom, area.right), (1_048_575, 16_383));
    assert!(model
        .plan_pivot_refresh(0, &mut PivotBudget::default())
        .is_err());
}

#[test]
fn a_table_source_leaves_out_its_totals_row_and_matches_any_case() {
    let case = read(NAMED, "case");
    let mut model = model_of(&case);
    add_pivot(&mut model, &case, "T", 0, None);
    model.workbook.pivot_caches[0].source_name = Some("salest".to_string());
    let table = model.workbook.tables.get_mut("SalesT").unwrap();
    table.totals_row_count = 1;
    let area = model.pivot_source_area(0).unwrap().area;
    assert_eq!((area.top, area.bottom), (0, 4));
    let table = model.workbook.tables.get_mut("SalesT").unwrap();
    table.header_row_count = 0;
    assert_eq!(model.pivot_source_area(0), None);
}

#[test]
fn a_range_source_resolves_as_before() {
    let case = read(NAMED, "case");
    let mut model = model_of(&case);
    add_pivot(&mut model, &case, "N", 0, None);
    let cache = &mut model.workbook.pivot_caches[0];
    let range = CacheSource {
        sheet_name: "Data".into(),
        range: "A1:B4".into(),
    };
    cache.source = Some(range.clone());
    // The range wins over a name beside it (T-5034).
    assert_eq!(model.pivot_source_area(0), model.resolve(&range));
}

#[test]
fn every_pivot_over_a_shared_cache_is_refreshed() {
    // PS1 owns the cache; PS2 (same sheet) and PS3 (another sheet) share it.
    let case = read(SHARED, "case");
    let mut model = model_of(&case);
    add_pivot(&mut model, &case, "S1", 0, None);
    add_pivot(&mut model, &case, "S1", 1, Some(0));
    add_pivot(&mut model, &case, "S2", 0, Some(0));
    assert_eq!(model.workbook.pivot_caches.len(), 1);
    refresh_all(&mut model);
    let mut wrong = differences(&model, SHARED, "S1", 0);
    wrong.extend(differences(&model, SHARED, "S1", 1));
    wrong.extend(differences(&model, SHARED, "S2", 0));
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn a_relative_name_body_is_read_from_the_first_sheets_a1() {
    // The engine's reading (UNVERIFIED, T-5034): a relative body stands for the range it would
    // in the first sheet's A1, so `=Data!A1:B6` is Data!A1:B6 and `=Data!B2:C3` is Data!B2:C3.
    let case = read(NAMED, "case");
    let mut model = model_of(&case);
    model
        .define_loaded_name("Rel", None, "=Data!A1:B6")
        .unwrap();
    model
        .define_loaded_name("RelMoved", None, "=Data!B2:C3")
        .unwrap();
    add_pivot(&mut model, &case, "N", 0, None);
    let data = model.get_sheet_index_by_name("Data").unwrap();
    let at = |model: &mut Model, name: &str| {
        model.workbook.pivot_caches[0].source_name = Some(name.to_string());
        let area = model.pivot_source_area(0).unwrap();
        assert_eq!(area.sheet, data, "{name}");
        let a = area.area;
        (a.top, a.left, a.bottom, a.right)
    };
    assert_eq!(at(&mut model, "Rel"), (0, 0, 5, 1));
    assert_eq!(at(&mut model, "RelMoved"), (1, 1, 2, 2));
}
