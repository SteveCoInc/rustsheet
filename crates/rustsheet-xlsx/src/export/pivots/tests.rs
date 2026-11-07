#![allow(clippy::unwrap_used, reason = "tests")]
//! Export, then import, gives the pivots back (SPEC-0032 "Test plan": import, export, import is a
//! fixed point; T-4176).

use std::io::{Cursor, Read};

use rustsheet_engine::expressions::token::Error;
use rustsheet_engine::pivot::{
    Axis, AxisField, CacheBuilder, CacheField, CacheItem, CacheSource, CellArea, DataField,
    NumberGroups, OpaqueEntry, OpaqueNode, PivotBudget, PivotCache, PivotField, PivotTable,
    RecordValue, MAX_PIVOT_WORK, MAX_RECORDS, MAX_RECORD_VALUES,
};
use rustsheet_engine::types::DateSystem;
use rustsheet_engine::Model;

use super::layout_lines;
use crate::export::save_xlsx_to_writer;
use crate::import::{load_from_xlsx, load_from_xlsx_bytes};

pub(super) fn export(model: &Model) -> Vec<u8> {
    save_xlsx_to_writer(model, Cursor::new(Vec::new()))
        .unwrap()
        .into_inner()
}

pub(super) fn round_trip(model: &Model) -> Model {
    let workbook = load_from_xlsx_bytes(&export(model), "back", "en", "UTC").unwrap();
    Model::from_workbook(workbook, "en").unwrap()
}

/// The part `name` of the package `bytes`.
pub(super) fn part(bytes: &[u8], name: &str) -> String {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut text = String::new();
    archive
        .by_name(name)
        .unwrap()
        .read_to_string(&mut text)
        .unwrap();
    text
}

/// `tables` without their kept `rowItems` and `colItems`, as the engine leaves a pivot it lays out
/// again.
pub(super) fn without_lines(tables: &[PivotTable]) -> Vec<PivotTable> {
    let mut tables = tables.to_vec();
    for table in &mut tables {
        table.opaque.retain(|entry| !entry.is_layout_lines());
    }
    tables
}

/// The Excel-saved pivot of `tests/example.xlsx` comes back equal, its saved `rowItems` and
/// `colItems` included, whether they are written back as read (the engine has not laid it out)
/// or from the engine's layout once it has: the engine's lines for it are Excel's, written as
/// Excel writes them.
#[test]
fn an_excel_saved_pivot_round_trips() {
    let mut model = load_from_xlsx("tests/example.xlsx", "en", "UTC", "en").unwrap();
    let saved = model.workbook.pivot_tables.clone();
    assert_eq!(saved.len(), 1);
    let cache = &model.workbook.pivot_caches[saved[0].cache as usize];
    // Its source is the Table `Table1`.
    assert_eq!(cache.source_name.as_deref(), Some("Table1"));
    let mut work = MAX_PIVOT_WORK;
    assert!(
        layout_lines(&saved[0], cache, None, &mut work).is_none(),
        "a pivot the engine has not laid out keeps its saved lines"
    );
    let back = round_trip(&model);
    assert_eq!(back.workbook.pivot_caches, model.workbook.pivot_caches);
    assert_eq!(back.workbook.pivot_tables, saved);

    // Laid out by the engine, it holds no saved lines and gets the engine's, equal to Excel's.
    let placed = model
        .plan_pivot_render(0, &mut PivotBudget::default())
        .unwrap();
    let pivot = placed.placements.into_iter().next().unwrap().pivot;
    assert!(!pivot.opaque.iter().any(OpaqueEntry::is_layout_lines));
    model.workbook.pivot_tables = vec![pivot];
    let back = round_trip(&model);
    assert_eq!(back.workbook.pivot_tables, saved);
    let bytes = export(&model);
    let table = part(&bytes, "xl/pivotTables/pivotTable1.xml");
    assert!(table.contains(r#"<rowItems count="10"><i><x/></i><i r="1"><x/></i>"#));
    assert!(table.contains(r#"<colItems count="1"><i/></colItems>"#));
    // Excel's schema order: the kept style and extensions after the data fields.
    let data = table.find("<dataFields").unwrap();
    let style = table.find("<pivotTableStyleInfo").unwrap();
    let extensions = table.find("<extLst>").unwrap();
    assert!(table.find("<colItems").unwrap() < data && data < style && style < extensions);
}

/// The case builder's pivots of `tests/pivot_tables.xlsx` (probes/getpivotdata-real-pivot) come
/// back equal, and every cell, GETPIVOTDATA over them included, recalculates the same.
#[test]
fn built_pivots_round_trip() {
    let mut model = load_from_xlsx("tests/pivot_tables.xlsx", "en", "UTC", "en").unwrap();
    model.evaluate();
    let mut back = round_trip(&model);
    assert_eq!(back.workbook.pivot_caches, model.workbook.pivot_caches);
    assert_eq!(
        without_lines(&back.workbook.pivot_tables),
        without_lines(&model.workbook.pivot_tables)
    );
    back.evaluate();
    for (sheet, worksheet) in (0u32..).zip(&model.workbook.worksheets) {
        for (row, column, _) in worksheet.sheet_data.cells() {
            assert_eq!(
                back.get_formatted_cell_value(sheet, row, column),
                model.get_formatted_cell_value(sheet, row, column),
                "{}!R{row}C{column}",
                worksheet.name
            );
        }
    }
}

/// A cache of every item type over Sheet1!A1:C4, with fields `K` (texts, a blank, a boolean),
/// `D` (dates) and `V` (numbers and an error).
fn mixed_cache() -> PivotCache {
    let mut builder = CacheBuilder::new(
        Some(CacheSource {
            sheet_name: "Sheet1".into(),
            range: "A1:C4".into(),
        }),
        MAX_RECORD_VALUES,
    );
    for name in ["K", "D", "V"] {
        builder.add_field(name.into()).unwrap();
    }
    let rows = [
        [
            CacheItem::Text("East & <West>".into()),
            CacheItem::Date(45000.5),
            CacheItem::Number(0.1),
        ],
        [
            CacheItem::Missing,
            CacheItem::Date(45001.0),
            CacheItem::Number(-2.5e-7),
        ],
        [
            CacheItem::Boolean(true),
            CacheItem::Date(45000.5),
            CacheItem::Error(Error::NA),
        ],
    ];
    for row in rows {
        builder
            .push_record(row.into_iter().map(RecordValue::Inline).collect())
            .unwrap();
    }
    let mut cache = builder.finish();
    cache.refresh_on_load = true;
    cache
}

/// A pivot made through the API, with nothing kept verbatim: `K` on rows, `D` on columns, the
/// sum of `V`, at Sheet1!E1.
fn api_pivot(sheet_id: u32) -> PivotTable {
    let mut pivot = PivotTable::new(
        "Pv \"1\"".into(),
        sheet_id,
        CellArea {
            top: 0,
            left: 4,
            bottom: 5,
            right: 7,
        },
        0,
    );
    pivot.fields = vec![
        PivotField::new(Some(Axis::Row), 3),
        PivotField::new(Some(Axis::Column), 2),
        PivotField::new(None, 3),
    ];
    pivot.fields[0].hide(2);
    pivot.row_fields = vec![AxisField::Field(0)];
    pivot.column_fields = vec![AxisField::Field(1)];
    pivot.data_fields = vec![DataField {
        caption: "Sum of V".into(),
        field: 2,
        ..DataField::default()
    }];
    pivot
}

fn model_with(cache: PivotCache) -> Model {
    let mut model = Model::new_empty("pivots", "en", "UTC", "en").unwrap();
    let sheet_id = model.workbook.worksheets[0].sheet_id;
    model.workbook.pivot_caches.push(cache);
    model.workbook.pivot_tables.push(api_pivot(sheet_id));
    model
}

/// Every item type, a refreshOnLoad cache and a pivot with nothing kept verbatim: the cache
/// comes back equal; the pivot gains its `<items>`, the data field's flag and its namespace, and
/// is a fixed point from then on.
#[test]
fn an_api_pivot_round_trips_and_reaches_a_fixed_point() {
    let model = model_with(mixed_cache());
    let bytes = export(&model);
    let definition = part(&bytes, "xl/pivotCache/pivotCacheDefinition1.xml");
    assert!(definition.contains(r#" refreshOnLoad="1""#), "{definition}");
    assert!(
        definition.contains(r#"<worksheetSource ref="A1:C4" sheet="Sheet1"/>"#),
        "{definition}"
    );
    assert!(
        definition.contains(
            r#"<sharedItems containsBlank="1" count="3"><s v="East &amp; &lt;West&gt;"/><m/><b v="1"/></sharedItems>"#
        ),
        "{definition}"
    );
    assert!(definition.contains(r#"<d v="2023-03-15T12:00:00"/><d v="2023-03-16T00:00:00"/>"#));
    assert!(definition.contains(r##"<e v="#N/A"/>"##));
    let records = part(&bytes, "xl/pivotCache/pivotCacheRecords1.xml");
    assert!(records.contains(r#"count="3"><r><x v="0"/><x v="0"/><x v="0"/></r>"#));
    let table = part(&bytes, "xl/pivotTables/pivotTable1.xml");
    assert!(table.contains(r#"name="Pv &quot;1&quot;" cacheId="1" dataCaption="Values""#));
    assert!(
        table.contains(
            r#"<pivotField axis="axisRow"><items count="4"><item x="0"/><item x="1"/><item x="2" h="1"/><item t="default"/></items></pivotField>"#
        ),
        "{table}"
    );
    assert!(table.contains(r#"<pivotField dataField="1"/>"#), "{table}");

    let back = round_trip(&model);
    assert_eq!(back.workbook.pivot_caches, model.workbook.pivot_caches);
    let pivot = &back.workbook.pivot_tables[0];
    assert_eq!(pivot.fields[0].items.len(), 4);
    assert!(pivot.fields[0].is_hidden(2));
    assert_eq!(
        pivot.data_fields,
        model.workbook.pivot_tables[0].data_fields
    );
    let again = round_trip(&back);
    assert_eq!(again.workbook.pivot_tables, back.workbook.pivot_tables);
    assert_eq!(again.workbook.pivot_caches, back.workbook.pivot_caches);
}

/// A calculated field is written as the builder and Excel write it, with its formula as read and
/// no record values, and reads back equal (T-4174); one after it, or one whose formula the import
/// refuses, drops the cache.
#[test]
fn a_calculated_field_round_trips() {
    let mut cache = mixed_cache();
    cache.fields.push(CacheField {
        name: "Dbl \"x\"".into(),
        formula: Some("V*2+'K'+LEN(\"a\")".into()),
        ..CacheField::default()
    });
    let mut model = model_with(cache);
    let pivot = &mut model.workbook.pivot_tables[0];
    pivot.fields.push(PivotField::new(None, 0));
    pivot.data_fields.push(DataField {
        caption: "Sum of Dbl".into(),
        field: 3,
        ..DataField::default()
    });
    let bytes = export(&model);
    let definition = part(&bytes, "xl/pivotCache/pivotCacheDefinition1.xml");
    assert!(
        definition.contains(
            r#"<cacheFields count="4">"#
        ) && definition.contains(
            r#"<cacheField name="Dbl &quot;x&quot;" numFmtId="0" formula="V*2+&apos;K&apos;+LEN(&quot;a&quot;)" databaseField="0"/></cacheFields>"#
        ),
        "{definition}"
    );
    let records = part(&bytes, "xl/pivotCache/pivotCacheRecords1.xml");
    assert!(
        records.contains(r#"<r><x v="0"/><x v="0"/><x v="0"/></r>"#),
        "{records}"
    );
    let back = round_trip(&model);
    assert_eq!(back.workbook.pivot_caches, model.workbook.pivot_caches);
    assert_eq!(
        back.workbook.pivot_tables[0].data_fields,
        model.workbook.pivot_tables[0].data_fields
    );

    let formula = model.workbook.pivot_caches[0].fields[3].formula.take();
    model.workbook.pivot_caches[0].fields[3].formula = Some("NOW()".into());
    assert!(round_trip(&model).workbook.pivot_caches.is_empty());
    model.workbook.pivot_caches[0].fields[3].formula = formula;
    model.workbook.pivot_caches[0].fields.swap(2, 3);
    assert!(round_trip(&model).workbook.pivot_caches.is_empty());
}

/// Classic layout and a field's subtotal caption are written and read back (pivots.md "Classic
/// layout (gridDropZones)", "subtotalCaption labels a field's subtotal lines").
#[test]
fn classic_layout_and_subtotal_captions_round_trip() {
    let mut model = model_with(mixed_cache());
    let pivot = &mut model.workbook.pivot_tables[0];
    pivot.options.grid_drop_zones = true;
    pivot.fields[0].subtotal_caption = Some("Tot <&>".into());
    let table = part(&export(&model), "xl/pivotTables/pivotTable1.xml");
    assert!(table.contains(r#" gridDropZones="1""#), "{table}");
    assert!(
        table.contains(r#" subtotalCaption="Tot &lt;&amp;&gt;""#),
        "{table}"
    );
    let back = round_trip(&model);
    let pivot = &back.workbook.pivot_tables[0];
    assert!(pivot.options.grid_drop_zones);
    assert_eq!(pivot.fields[0].subtotal_caption.as_deref(), Some("Tot <&>"));
    assert_eq!(pivot.fields[1].subtotal_caption, None);
    // Neither is written when off.
    let plain = part(
        &export(&model_with(mixed_cache())),
        "xl/pivotTables/pivotTable1.xml",
    );
    assert!(!plain.contains("gridDropZones") && !plain.contains("subtotalCaption"));
}

/// A range grouping whose field keeps no values of its own (built through the API without
/// `group_by_numbers`) has nothing to write the shared items from: the cache is dropped with its
/// pivots (T-4734).
#[test]
fn a_range_group_without_its_own_values_is_dropped_with_its_pivots() {
    let mut cache = mixed_cache();
    cache.fields[2].number_groups = Some(NumberGroups {
        start: 0.0,
        end: 1.0,
        interval: 1.0,
        whole: false,
        order: Vec::new(),
    });
    let back = round_trip(&model_with(cache));
    assert!(back.workbook.pivot_caches.is_empty());
    assert!(back.workbook.pivot_tables.is_empty());
}

/// Saved layout lines are written back as read while the pivot holds them, even when the
/// engine's would differ: its cells are still the saved ones (the engine has not laid it out).
#[test]
fn saved_lines_are_kept_until_the_engine_lays_the_pivot_out() {
    let mut model = load_from_xlsx("tests/example.xlsx", "en", "UTC", "en").unwrap();
    let stale = r#"<rowItems count="1"><i t="grand"><x/></i></rowItems>"#;
    for entry in &mut model.workbook.pivot_tables[0].opaque {
        if entry.is_layout_lines()
            && matches!(&entry.node, OpaqueNode::Element(x) if x.starts_with("<rowItems"))
        {
            entry.node = OpaqueNode::Element(stale.into());
        }
    }
    let table = part(&export(&model), "xl/pivotTables/pivotTable1.xml");
    assert!(table.contains(stale), "{table}");
}

/// The cache limits hold for a cache built through the API or decoded, whose fields are
/// public: one past them, alone or with the caches before it, is not written (directive 5).
#[test]
fn a_cache_past_the_limits_is_not_written() {
    use super::cache::check;
    let cache = mixed_cache();
    assert_eq!(
        check(&cache, DateSystem::Excel1900, MAX_RECORD_VALUES),
        Ok(9)
    );
    // Nine record values, with eight left by the caches before it.
    assert!(check(&cache, DateSystem::Excel1900, 8).is_err());
    let mut huge = mixed_cache();
    huge.record_count = u32::try_from(MAX_RECORDS).unwrap() + 1;
    assert!(check(&huge, DateSystem::Excel1900, MAX_RECORD_VALUES).is_err());
    // Within the record limit, but past the record values one: checked before any record.
    huge.record_count = u32::try_from(MAX_RECORDS).unwrap();
    huge.fields = vec![huge.fields[0].clone(); 11];
    assert!(check(&huge, DateSystem::Excel1900, MAX_RECORD_VALUES).is_err());
    let back = round_trip(&model_with(huge));
    assert!(back.workbook.pivot_caches.is_empty());
    assert!(back.workbook.pivot_tables.is_empty());
}

/// A number that is not finite, or a record that is not an index into its field's items, has no
/// faithful part: the cache is not written.
#[test]
fn non_finite_numbers_and_bad_records_are_not_written() {
    use super::cache::check;
    for n in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut cache = mixed_cache();
        cache.fields[2].items[0] = CacheItem::Number(n);
        assert!(check(&cache, DateSystem::Excel1900, MAX_RECORD_VALUES).is_err());
    }
    let mut cache = mixed_cache();
    cache.fields[0].records[1] = 3;
    assert!(check(&cache, DateSystem::Excel1900, MAX_RECORD_VALUES).is_err());
    let mut cache = mixed_cache();
    cache.fields[0].records.pop();
    assert!(check(&cache, DateSystem::Excel1900, MAX_RECORD_VALUES).is_err());
}

/// Two text items of a field that differ only in case, which the import's exact interning of
/// inline values can yield, make Excel for the web call the file corrupt (pivots.md
/// "Case-variant text items make Excel for the web call the file corrupt"): the cache is dropped
/// with its pivots. Texts that differ otherwise are written.
#[test]
fn case_variant_texts_drop_the_cache() {
    let mut cache = mixed_cache();
    cache.fields[0].items[0] = CacheItem::Text("East".into());
    cache.fields[0].items.push(CacheItem::Text("EAST".into()));
    let back = round_trip(&model_with(cache.clone()));
    assert!(back.workbook.pivot_caches.is_empty());
    assert!(back.workbook.pivot_tables.is_empty());
    cache.fields[0].items[3] = CacheItem::Text("East ".into());
    let back = round_trip(&model_with(cache));
    assert_eq!(back.workbook.pivot_caches.len(), 1);
}

/// A 1904 workbook's cache dates are days from 1904-01-01.
#[test]
fn dates_follow_the_date_system() {
    let mut model = model_with(mixed_cache());
    model.workbook.settings.date_system = DateSystem::Excel1904;
    let bytes = export(&model);
    let definition = part(&bytes, "xl/pivotCache/pivotCacheDefinition1.xml");
    assert!(
        definition.contains(r#"<d v="2027-03-16T12:00:00"/>"#),
        "{definition}"
    );
    let back = round_trip(&model);
    assert_eq!(back.workbook.pivot_caches, model.workbook.pivot_caches);
}

/// Seeds of the fixed-point property: T-4155's 48, so it sees the same pivots its engine
/// properties do.
const FIXED_POINT_SEEDS: u64 = 48;

/// Import, export and import is a fixed point over generated pivots (T-4756; plan §6 layer 5 of
/// docs/research/pivot-tables-2026-11-04/plan.md): for each seed, a second save and reload gives
/// the same pivot caches, pivot tables and rendered Sheet1 cells as the first reload.
#[test]
fn import_export_import_is_a_fixed_point_over_generated_pivots() {
    use rustsheet_engine::test_support::{add_sum_pivot, pivot_records, pivot_sheet_text};
    for seed in 0..FIXED_POINT_SEEDS {
        let empty = Model::new_empty("model", "en", "UTC", "en").unwrap();
        let model = add_sum_pivot(empty, &pivot_records(seed)).unwrap();
        let once = round_trip(&model);
        let twice = round_trip(&once);
        // Not vacuous: the reload keeps the pivot and its rendered report.
        assert_eq!(once.workbook.pivot_tables.len(), 1, "seed {seed}");
        assert_eq!(
            pivot_sheet_text(&once).unwrap(),
            pivot_sheet_text(&model).unwrap(),
            "seed {seed}: the first reload renders what was saved"
        );
        assert_eq!(
            twice.workbook.pivot_caches, once.workbook.pivot_caches,
            "seed {seed}: caches"
        );
        assert_eq!(
            twice.workbook.pivot_tables, once.workbook.pivot_tables,
            "seed {seed}: tables"
        );
        assert_eq!(
            pivot_sheet_text(&twice).unwrap(),
            pivot_sheet_text(&once).unwrap(),
            "seed {seed}: rendered cells"
        );
    }
}
