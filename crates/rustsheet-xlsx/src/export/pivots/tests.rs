#![allow(clippy::unwrap_used, reason = "tests")]
//! Export, then import, gives the pivots back (SPEC-0032 "Test plan": import, export, import is a
//! fixed point; T-4176).

use std::io::{Cursor, Read};

use rustsheet_engine::expressions::token::Error;
use rustsheet_engine::pivot::{
    Axis, AxisField, CacheBuilder, CacheItem, CacheSource, CellArea, DataField, NumberGroups,
    OpaqueNode, PivotCache, PivotField, PivotTable, RecordValue, MAX_PIVOT_WORK, MAX_RECORD_VALUES,
};
use rustsheet_engine::types::DateSystem;
use rustsheet_engine::Model;

use super::layout_lines;
use crate::export::save_xlsx_to_writer;
use crate::import::{load_from_xlsx, load_from_xlsx_bytes};

fn export(model: &Model) -> Vec<u8> {
    save_xlsx_to_writer(model, Cursor::new(Vec::new()))
        .unwrap()
        .into_inner()
}

fn round_trip(model: &Model) -> Model {
    let workbook = load_from_xlsx_bytes(&export(model), "back", "en", "UTC").unwrap();
    Model::from_workbook(workbook, "en").unwrap()
}

/// The part `name` of the package `bytes`.
fn part(bytes: &[u8], name: &str) -> String {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut text = String::new();
    archive
        .by_name(name)
        .unwrap()
        .read_to_string(&mut text)
        .unwrap();
    text
}

/// `tables` without their kept `rowItems` and `colItems`, which the export writes from the
/// engine's layout when it fits the location.
fn without_lines(tables: &[PivotTable]) -> Vec<PivotTable> {
    let mut tables = tables.to_vec();
    for table in &mut tables {
        table.opaque.retain(|entry| match &entry.node {
            OpaqueNode::Element(xml) => {
                !xml.starts_with("<rowItems") && !xml.starts_with("<colItems")
            }
            OpaqueNode::Attributes(_) => true,
        });
    }
    tables
}

/// The Excel-saved pivot of `tests/example.xlsx` comes back equal, its saved `rowItems` and
/// `colItems` included: the engine's lines for it are Excel's, written as Excel writes them.
#[test]
fn an_excel_saved_pivot_round_trips() {
    let model = load_from_xlsx("tests/example.xlsx", "en", "UTC", "en").unwrap();
    let workbook = &model.workbook;
    assert_eq!(workbook.pivot_tables.len(), 1);
    let pivot = &workbook.pivot_tables[0];
    let cache = &workbook.pivot_caches[pivot.cache as usize];
    // Its source is the Table `Table1`.
    assert_eq!(cache.source_name.as_deref(), Some("Table1"));
    let mut work = MAX_PIVOT_WORK;
    assert!(
        layout_lines(pivot, cache, &mut work).is_some(),
        "the engine lays out the saved location"
    );
    let back = round_trip(&model);
    assert_eq!(back.workbook.pivot_caches, workbook.pivot_caches);
    assert_eq!(back.workbook.pivot_tables, workbook.pivot_tables);
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

/// A grouped cache is dropped with its pivots: the import keeps no base values to write the
/// group back from (T-4734).
#[test]
fn a_grouped_cache_is_dropped_with_its_pivots() {
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
