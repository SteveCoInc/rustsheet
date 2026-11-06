#![allow(clippy::unwrap_used, clippy::panic, reason = "tests")]
//! The grid writer against probes/t4128-pivot-spike: R, a pivot whose source holds a formula,
//! re-rendered by a refresh after the recalculation, and S, whose layout would meet the constant
//! S!C5, collapsed to `#SPILL!` at its anchor. The definitions come from the case file, the
//! sheets are the case's cells in a real model (its formulas evaluated), and the expectations
//! are the sidecar's `pivots`, never the engine's. The rest pins the engine's own readings
//! (pivots.md "The grid writer's own readings", UNVERIFIED, T-4664).

use serde_json::Value as Json;

use super::*;
use crate::pivot::probe_cases::{
    address, position, probe_model, probe_pivot, probe_pivot_over, read, sheet,
};
use crate::pivot::{CacheField, CacheItem, CacheSource, OpaqueEntry, OpaqueNode, OpaqueOwner};

const SPIKE: &str = "t4128-pivot-spike";

fn sheet_of(model: &Model, name: &str) -> u32 {
    model.get_sheet_index_by_name(name).unwrap()
}

/// The value cell `at` (A1) of sheet `sheet` holds, as the renderer types one; `None` when empty.
fn cell(model: &Model, sheet: u32, at: &str) -> Option<PivotCell> {
    let (column, row) = position(at);
    let worksheet = model.workbook.worksheet(sheet).unwrap();
    let (row, column) = (
        i32::try_from(row + 1).unwrap(),
        i32::try_from(column + 1).unwrap(),
    );
    match worksheet.cell(row, column)? {
        Cell::EmptyCell { .. } => None,
        Cell::NumberCell { v, .. } => Some(PivotCell::Number(*v)),
        Cell::ErrorCell { ei, .. } => Some(PivotCell::Error(ei.clone())),
        Cell::SharedString { si, .. } => Some(PivotCell::Text(
            model.workbook.shared_strings[usize::try_from(*si).unwrap()].clone(),
        )),
        other => panic!("{at}: {other:?}"),
    }
}

/// Every stored, non-empty cell of sheet `sheet`, as sorted A1 addresses.
fn stored(model: &Model, sheet: u32) -> Vec<String> {
    let mut cells: Vec<String> = (model.stored_in(sheet, &area("A1:Z100")).into_iter())
        .map(|(row, column)| address(column, row))
        .collect();
    cells.sort();
    cells
}

fn area(range: &str) -> CellArea {
    let (from, to) = range.split_once(':').unwrap_or((range, range));
    let ((left, top), (right, bottom)) = (position(from), position(to));
    CellArea {
        top,
        left,
        bottom,
        right,
    }
}

/// Checks sheet `name` of `model` against the sidecar's pivot there: every cell of its capture
/// window, the location and the geometry.
fn assert_excels(model: &Model, name: &str) {
    let oracle = read(SPIKE, "oracle");
    let pivots = oracle["pivots"].as_array().unwrap();
    let pivot: &Json = pivots.iter().find(|p| p["sheet"] == name).unwrap();
    let sheet = sheet_of(model, name);
    let mut wrong = Vec::new();
    for (at, expected) in pivot["cells"].as_object().unwrap() {
        let value = &expected["v"];
        let expected = match expected["t"].as_str().unwrap() {
            "empty" => None,
            "s" => Some(PivotCell::Text(value.as_str().unwrap().into())),
            "n" => Some(PivotCell::Number(value.as_f64().unwrap())),
            "e" if value == "#SPILL!" => Some(PivotCell::Error(Error::SPILL)),
            other => panic!("{other} {value}"),
        };
        let mine = cell(model, sheet, at);
        if mine != expected {
            wrong.push(format!("{at}: Excel {expected:?}, engine {mine:?}"));
        }
    }
    assert!(wrong.is_empty(), "{name}:\n{}", wrong.join("\n"));
    let table = &model.workbook.pivot_tables[0];
    assert_eq!(table.location, area(pivot["location"].as_str().unwrap()));
    let geometry = &table.geometry;
    let saved = ["first_header_row", "first_data_row", "first_data_col"]
        .map(|key| u32::try_from(pivot[key].as_u64().unwrap()).unwrap());
    assert_eq!(
        [
            geometry.first_header_row,
            geometry.first_data_row,
            geometry.first_data_column
        ],
        saved
    );
}

/// The case's cells in a model holding sheet `name`'s pivot, its cache read by the engine's
/// refresh from the cells as they are before any recalculation.
fn spike(name: &str) -> Model {
    let case = read(SPIKE, "case");
    let mut model = probe_model(&case);
    let spec = &sheet(&case, name)["pivot_tables"][0];
    let (sheet_name, range) = spec["source"].as_str().unwrap().split_once('!').unwrap();
    let empty = PivotCache {
        source: Some(CacheSource {
            sheet_name: sheet_name.into(),
            range: range.into(),
        }),
        ..PivotCache::default()
    };
    let stale = refresh(&empty, &model, MAX_RECORD_VALUES).unwrap();
    let (cache, mut table) = probe_pivot_over(&case, name, stale);
    let index = sheet_of(&model, name);
    table.sheet_id = model.workbook.worksheet(index).unwrap().sheet_id;
    model.workbook.pivot_caches.push(cache);
    model.workbook.pivot_tables.push(table);
    model
}

fn refreshed(model: &mut Model) -> PivotSwap {
    let plan = model.plan_pivot_refresh(0).unwrap();
    model.apply_pivot_refresh(plan).unwrap()
}

#[test]
fn the_spike_r_pivot_re_renders_from_the_recalculated_source() {
    let mut model = spike("R");
    // DataF!C2 is `=1000`, which the file's cache records as missing (`<m/>`): only a refresh
    // reads it (Excel's R!B5 is 1004).
    let sales = &mut model.workbook.pivot_caches[0].fields[2];
    sales.items[0] = CacheItem::Missing;
    model.evaluate();
    refreshed(&mut model);
    assert_excels(&model, "R");
}

#[test]
fn the_spike_s_pivot_collapses_to_spill_at_its_anchor() {
    let mut model = spike("S");
    model.evaluate();
    let plan = model.plan_pivot_refresh(0).unwrap();
    assert!(plan.placements[0].spilled);
    model.apply_pivot_refresh(plan).unwrap();
    assert_excels(&model, "S");
    // The foreign constant stays, and the pivot wrote nothing else.
    let s = sheet_of(&model, "S");
    assert_eq!(stored(&model, s), ["A3", "C5"]);
    // Laid out again from its one-cell location it still meets C5; without C5 it renders
    // from the same anchor.
    refreshed(&mut model);
    assert_excels(&model, "S");
    (model.workbook.worksheet_mut(s).unwrap())
        .cell_clear_contents(5, 3)
        .unwrap();
    refreshed(&mut model);
    assert_eq!(model.workbook.pivot_tables[0].location, area("A3:E7"));
    assert_eq!(cell(&model, s, "C5"), Some(PivotCell::Number(5.0)));
}

#[test]
fn a_refresh_clears_what_the_pivot_owned_and_grows_only_into_empty_cells() {
    let mut model = spike("R");
    model.evaluate();
    refreshed(&mut model);
    let (r, data) = (sheet_of(&model, "R"), sheet_of(&model, "DataF"));
    // A constant left in the pivot's own area (D6, a blank intersection) is cleared; one away
    // from it is kept.
    model.update_cell_with_text(r, 6, 4, "stale").unwrap();
    model.update_cell_with_text(r, 20, 1, "away").unwrap();
    // A source edit changes nothing until the next refresh.
    model.update_cell_with_number(data, 3, 3, 25.0).unwrap();
    model.evaluate();
    assert_eq!(cell(&model, r, "B6"), Some(PivotCell::Number(20.0)));
    refreshed(&mut model);
    assert_eq!(cell(&model, r, "D6"), None);
    assert_eq!(cell(&model, r, "B6"), Some(PivotCell::Number(25.0)));
    assert_eq!(cell(&model, r, "A20"), Some(PivotCell::Text("away".into())));
    // A new product grows the pivot by a column, into empty cells.
    model.update_cell_with_text(data, 3, 2, "D").unwrap();
    model.evaluate();
    refreshed(&mut model);
    assert_eq!(model.workbook.pivot_tables[0].location, area("A3:F7"));
    assert_eq!(cell(&model, r, "E4"), Some(PivotCell::Text("D".into())));
    // A constant where it would grow next collapses it, and its old area is emptied.
    model.update_cell_with_text(data, 4, 2, "E").unwrap();
    model.update_cell_with_text(r, 4, 7, "x").unwrap();
    model.evaluate();
    refreshed(&mut model);
    assert_eq!(model.workbook.pivot_tables[0].location, area("A3"));
    assert_eq!(cell(&model, r, "A3"), Some(PivotCell::Error(Error::SPILL)));
    assert_eq!(stored(&model, r), ["A20", "A3", "G4"]);
}

#[test]
fn a_swap_restores_the_cache_and_pivot() {
    let mut model = spike("R");
    model.evaluate();
    let (cache, table) = (
        model.workbook.pivot_caches[0].clone(),
        model.workbook.pivot_tables[0].clone(),
    );
    let data = sheet_of(&model, "DataF");
    model.update_cell_with_number(data, 3, 3, 25.0).unwrap();
    model.evaluate();
    let swap = refreshed(&mut model);
    assert_ne!(model.workbook.pivot_caches[0], cache);
    model.restore_pivot_swap(swap);
    assert_eq!(model.workbook.pivot_caches[0], cache);
    assert_eq!(model.workbook.pivot_tables[0], table);
}

#[test]
fn another_pivots_area_is_foreign_even_where_it_is_blank() {
    let mut model = spike("R");
    model.evaluate();
    refreshed(&mut model);
    // A second pivot over the same cache, anchored on R's blank D6.
    let mut second = model.workbook.pivot_tables[0].clone();
    second.name = "Second".into();
    second.location = area("D6");
    second.geometry = PivotGeometry::default();
    model.workbook.pivot_tables.push(second);
    let plan = model.plan_pivot_render(1).unwrap();
    assert!(plan.placements[0].spilled);
    // A refresh of the cache lays out both, each meeting the other's area.
    let plan = model.plan_pivot_refresh(0).unwrap();
    assert_eq!(plan.placements.len(), 2);
    assert!(plan.placements.iter().all(|p| p.spilled));
}

#[test]
fn a_pivot_moved_by_a_row_insert_refreshes_where_it_now_is() {
    let mut model = spike("R");
    model.evaluate();
    refreshed(&mut model);
    let r = sheet_of(&model, "R");
    // T-2086: the pivot moves with its cells.
    model.insert_rows(r, 1, 2).unwrap();
    assert_eq!(model.workbook.pivot_tables[0].location, area("A5:E9"));
    let plan = model.plan_pivot_refresh(0).unwrap();
    assert!(!plan.placements[0].spilled);
    model.apply_pivot_refresh(plan).unwrap();
    assert_eq!(model.workbook.pivot_tables[0].location, area("A5:E9"));
    assert_eq!(cell(&model, r, "B7"), Some(PivotCell::Number(1004.0)));
    assert_eq!(cell(&model, r, "A3"), None);
    // An insert through it is still refused.
    assert!(model.insert_rows(r, 7, 1).is_err());
}

#[test]
fn the_page_area_is_owned_with_the_location() {
    let geometry = PivotGeometry {
        row_page_count: 2,
        column_page_count: 2,
        ..PivotGeometry::default()
    };
    assert_eq!(
        owned_areas(area("B10:C12"), &geometry),
        [Some(area("B10:C12")), Some(area("B7:F8"))]
    );
    assert_eq!(owned_areas(area("A2"), &geometry), [Some(area("A2")), None]);
}

fn cache_of(names: &[&str]) -> PivotCache {
    PivotCache {
        fields: (names.iter())
            .map(|name| CacheField {
                name: (*name).into(),
                ..CacheField::default()
            })
            .collect(),
        ..PivotCache::default()
    }
}

#[test]
fn field_indices_follow_their_names() {
    let (_, mut table) = probe_pivot(&read(SPIKE, "case"), "S");
    let old = cache_of(&["Region", "Product", "Sales"]);
    // A new source column before the others, and the names refreshed in another case.
    let new = cache_of(&["Day", "SALES", "region", "Product"]);
    let remapped = remap(&table, &old, &new).unwrap();
    assert_eq!(remapped.row_fields, [AxisField::Field(2)]);
    assert_eq!(remapped.column_fields, [AxisField::Field(3)]);
    assert_eq!(remapped.data_fields[0].field, 1);
    assert_eq!(remapped.fields.len(), 4);
    assert_eq!(remapped.fields[2].axis, table.fields[0].axis);
    assert_eq!(remapped.fields[0].axis, None);
    // A field the pivot shows that left the source refuses the refresh.
    let gone = cache_of(&["Region", "Sales"]);
    assert_eq!(
        remap(&table, &old, &gone),
        Err(PivotRefreshError::FieldGone("Product".into()))
    );
    // Unread parts may hold field indices: no renumbering under them.
    table.opaque.push(OpaqueEntry {
        owner: OpaqueOwner::Definition,
        node: OpaqueNode::Element("<filters/>".into()),
    });
    assert_eq!(
        remap(&table, &old, &new),
        Err(PivotRefreshError::FieldsMoved)
    );
    // Appending a field renumbers nothing.
    let appended = cache_of(&["Region", "Product", "Sales", "Day"]);
    assert_eq!(remap(&table, &old, &appended).unwrap().fields.len(), 4);
}
