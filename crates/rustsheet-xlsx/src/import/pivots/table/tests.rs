#![allow(clippy::unwrap_used, reason = "tests")]

//! The pivot definition model on the spike and corpus shapes (T-4139). The fixtures are the
//! `pivotTableDefinition` parts Excel rendered: probes t4123, t4128, t4137 and t4138 as
//! `rustsheet-spec2xlsx` builds them (Excel for the web opened and rendered each, the sidecars'
//! `pivots`), and the Excel-saved pivot of `tests/example.xlsx` (corpus shape: `mc:Ignorable`,
//! `xr:uid`, `rowItems`, `extLst`).

use std::io::Cursor;

use quick_xml::events::Event;
use rustsheet_engine::pivot::{CacheField, CacheItem};

use super::*;

const CAPTIONS: &str = include_str!("fixtures/t4137-captions.xml");
const PAGES: &str = include_str!("fixtures/t4137-pages.xml");
const SUBTOTALS: &str = include_str!("fixtures/t4137-subtotals.xml");
const ERRORS: &str = include_str!("fixtures/t4137-errors.xml");
const CALC_TOP_SORT: &str = include_str!("fixtures/t4128-calc-top-sort.xml");
const PAGE_WRAP: &str = include_str!("fixtures/t4138-pg8.xml");
const DATA_ON_ROWS: &str = include_str!("fixtures/t4138-pg6.xml");
const TABULAR_PERCENT: &str = include_str!("fixtures/t4123-tabular.xml");
const CORPUS: &str = include_str!("fixtures/example-corpus.xml");

/// A cache whose fields hold `items` text items each.
fn cache(items: &[usize]) -> PivotCache {
    let fields = items
        .iter()
        .enumerate()
        .map(|(f, &n)| CacheField {
            name: format!("F{f}"),
            items: (0..n).map(|i| CacheItem::Text(format!("i{i}"))).collect(),
            ..CacheField::default()
        })
        .collect();
    PivotCache {
        fields,
        ..PivotCache::default()
    }
}

/// `xml` read against `cache` (whatever its `cacheId`) with `budget` opaque bytes; the pivot or
/// the reason it was skipped, and the budget left.
fn read_with(xml: &str, cache: PivotCache, budget: usize) -> (Result<PivotTable, String>, usize) {
    let id = xml
        .split("cacheId=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .and_then(|id| id.parse::<u32>().ok())
        .unwrap();
    let ids = HashMap::from([(id, 0)]);
    let left = Cell::new(budget);
    let result = read_pivot_table(
        Cursor::new(xml.as_bytes()),
        3,
        &[cache],
        &ids,
        &ImportLimits::default(),
        &left,
    );
    (result.map_err(|skip| skip.0), left.get())
}

fn read(xml: &str, items: &[usize]) -> PivotTable {
    read_with(xml, cache(items), usize::MAX).0.unwrap()
}

fn text(entry: &OpaqueEntry) -> &str {
    match &entry.node {
        OpaqueNode::Attributes(xml) | OpaqueNode::Element(xml) => xml,
    }
}

/// The XML the store keeps for `owner`, entries joined in order.
fn kept(pivot: &PivotTable, owner: OpaqueOwner) -> String {
    let entries = pivot.opaque.iter().filter(|e| e.owner == owner);
    entries.map(text).collect()
}

fn data(x: u32) -> PivotItem {
    PivotItem::data(x)
}

fn line(kind: ItemType) -> PivotItem {
    PivotItem {
        kind,
        index: None,
        expanded: true,
        filtered: false,
    }
}

const STYLE: &str = r#"<pivotTableStyleInfo name="PivotStyleLight16" showRowHeaders="1" showColHeaders="1" showRowStripes="0" showColStripes="0" showLastColumn="1"/>"#;

#[test]
fn captions_and_the_values_field_are_typed() {
    // probes/t4137-pivot-captions K1.
    let pivot = read(CAPTIONS, &[2, 3, 6, 6, 6]);
    assert_eq!(pivot.name, "Captions");
    assert_eq!(pivot.sheet_id, 3);
    let options = &pivot.options;
    assert_eq!(options.row_header_caption.as_deref(), Some("Regions"));
    assert_eq!(options.column_header_caption.as_deref(), Some("Products"));
    assert_eq!(options.grand_total_caption.as_deref(), Some("All of it"));
    assert_eq!(options.data_caption.as_deref(), Some("Measures"));
    assert!(options.outline && options.outline_data);
    // Left out by the file: the schema's defaults.
    assert!(options.compact && options.compact_data);
    assert!(options.row_grand_totals && options.column_grand_totals);
    assert!(!options.data_on_rows && options.data_position.is_none());
    assert_eq!(pivot.row_fields, [AxisField::Field(0)]);
    assert_eq!(pivot.column_fields, [AxisField::Values]);
    let captions: Vec<&str> = pivot
        .data_fields
        .iter()
        .map(|d| d.caption.as_str())
        .collect();
    assert_eq!(captions, ["Sum of Sales", "Sum of Units"]);
    assert_eq!(
        pivot.data_fields[1],
        DataField {
            caption: "Sum of Units".into(),
            field: 3,
            base_field: 0,
            base_item: 0,
            ..DataField::default()
        }
    );
    let region = &pivot.fields[0];
    assert_eq!(region.items, [data(0), data(1), line(ItemType::Default)]);
    assert!(!region.show_all && region.compact && region.outline && region.default_subtotal);
    assert_eq!(region.sort_type, SortType::Manual);
    // What the model does not read stays, verbatim, with the element it was read in.
    let root = kept(&pivot, OpaqueOwner::Definition);
    assert_eq!(root.matches(STYLE).count(), 1);
    assert!(root.starts_with(
        r#" xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" updatedVersion="6""#
    ));
    // The whole `<location>` is typed.
    assert_eq!(
        pivot.geometry,
        PivotGeometry {
            first_header_row: 1,
            first_data_row: 1,
            first_data_column: 1,
            row_page_count: 0,
            column_page_count: 0,
        }
    );
    assert_eq!(kept(&pivot, OpaqueOwner::Location), "");
    assert_eq!(kept(&pivot, OpaqueOwner::Field(2)), r#" dataField="1""#);
    // `count` follows from the lists.
    assert_eq!(kept(&pivot, OpaqueOwner::DataFields), "");
    assert_eq!(kept(&pivot, OpaqueOwner::Items(0)), "");
}

#[test]
fn page_fields_hold_their_selection_and_hidden_items() {
    // probes/t4137-pivot-captions K4: Region (All), Product A, Units with items 0 and 1 hidden.
    let pivot = read(PAGES, &[2, 3, 6, 6, 6]);
    let page = |field, item| PageField {
        field,
        item,
        hierarchy: Some(-1),
    };
    assert_eq!(
        pivot.page_fields,
        [page(0, None), page(1, Some(0)), page(3, None)]
    );
    let units = &pivot.fields[3];
    assert_eq!(units.axis, Some(Axis::Page));
    assert!(units.is_hidden(0) && units.is_hidden(1) && !units.is_hidden(2));
    assert!(units.item_hidden(&units.items[0]) && !units.item_hidden(&units.items[2]));
    assert_eq!(units.items.len(), 7);
    assert_eq!(
        kept(&pivot, OpaqueOwner::Field(3)),
        r#" multipleItemSelectionAllowed="1""#
    );
    assert_eq!(
        pivot.geometry,
        PivotGeometry {
            first_header_row: 1,
            first_data_row: 1,
            first_data_column: 1,
            row_page_count: 3,
            column_page_count: 1,
        }
    );
    assert_eq!(kept(&pivot, OpaqueOwner::Location), "");
    assert!(pivot.row_fields.is_empty() && pivot.column_fields.is_empty());
}

#[test]
fn tabular_form_and_custom_subtotals_are_typed_per_field() {
    // probes/t4137-pivot-captions K5: tabular, Region with Sum and Count subtotals.
    let pivot = read(SUBTOTALS, &[2, 3, 6, 6, 6]);
    let options = &pivot.options;
    assert!(!options.compact && !options.compact_data);
    assert!(!options.outline && !options.outline_data);
    let region = &pivot.fields[0];
    assert!(!region.compact && !region.outline && !region.default_subtotal);
    // `countSubtotal` reads as Count Numbers (probes/t4622-pivot-count-subtotals B2!C7).
    assert_eq!(
        region.custom_subtotals,
        [Aggregate::Sum, Aggregate::CountNums]
    );
    assert_eq!(
        region.items,
        [
            data(0),
            data(1),
            line(ItemType::Function(Aggregate::Sum)),
            line(ItemType::Function(Aggregate::CountNums))
        ]
    );
    let product = &pivot.fields[1];
    assert!(product.default_subtotal && product.custom_subtotals.is_empty());
    assert!(product.subtotal_top && !product.insert_blank_row);
    // A field the file leaves alone keeps the schema's form flags.
    assert!(pivot.fields[3].compact && pivot.fields[3].outline);
}

#[test]
fn count_a_subtotal_is_counta_and_count_is_count_numbers() {
    // probes/t4622-pivot-count-subtotals B2: over 10, abc, a blank, 2 and 5, the countA line is 4
    // (C6) and the count line 3 (C7).
    let xml = SUBTOTALS
        .replacen(r#"sumSubtotal="1""#, r#"countASubtotal="1""#, 1)
        .replacen(r#"<item t="sum"/>"#, r#"<item t="countA"/>"#, 1);
    let pivot = read(&xml, &[2, 3, 6, 6, 6]);
    let region = &pivot.fields[0];
    assert_eq!(
        region.custom_subtotals,
        [Aggregate::Count, Aggregate::CountNums]
    );
    assert_eq!(
        region.items[2..],
        [
            line(ItemType::Function(Aggregate::Count)),
            line(ItemType::Function(Aggregate::CountNums))
        ]
    );
}

#[test]
fn error_and_missing_captions_are_typed() {
    // probes/t4137-pivot-captions K3.
    let pivot = read(ERRORS, &[2, 3, 6, 6, 6]);
    assert!(pivot.options.show_error);
    assert_eq!(pivot.options.error_caption.as_deref(), Some("ERR"));
    assert!(pivot.options.show_missing && pivot.options.missing_caption.is_none());
    let missing = CAPTIONS.replace(
        r#"dataCaption="Measures""#,
        r#"dataCaption="Measures" missingCaption="n/a" showMissing="0""#,
    );
    let pivot = read(&missing, &[2, 3, 6, 6, 6]);
    assert!(!pivot.options.show_missing);
    assert_eq!(pivot.options.missing_caption.as_deref(), Some("n/a"));
}

#[test]
fn page_layout_and_values_on_rows_are_typed() {
    // probes/t4138-pivot-geometry PG8 and PG6.
    let pivot = read(PAGE_WRAP, &[2, 3, 2, 6, 6]);
    assert_eq!(pivot.options.page_wrap, 2);
    assert!(pivot.options.page_over_then_down);
    assert_eq!(pivot.page_fields.len(), 3);
    let pivot = read(DATA_ON_ROWS, &[2, 6, 6, 6, 6]);
    assert!(pivot.options.data_on_rows);
    assert_eq!(pivot.row_fields, [AxisField::Field(0), AxisField::Values]);
    let at = DATA_ON_ROWS.replace(r#"dataOnRows="1""#, r#"dataOnRows="1" dataPosition="0""#);
    assert_eq!(read(&at, &[2, 6, 6, 6, 6]).options.data_position, Some(0));
}

#[test]
fn filters_and_unread_field_attributes_are_kept_verbatim() {
    // probes/t4128-pivot-spike CalcTopSort: a descending sort and a Top-N filter (T-4165's).
    let pivot = read(CALC_TOP_SORT, &[2, 3, 6, 6]);
    assert_eq!(pivot.fields[1].sort_type, SortType::Descending);
    assert_eq!(
        kept(&pivot, OpaqueOwner::Field(3)),
        r#" dataField="1" dragToRow="0" dragToCol="0" dragToPage="0""#
    );
    let filters = r#"<filters count="1"><filter fld="1" type="count" evalOrder="-1" id="1" iMeasureFld="0"><autoFilter ref="A1"><filterColumn colId="0"><top10 val="2" filterVal="2"/></filterColumn></autoFilter></filter></filters>"#;
    let elements: Vec<&OpaqueNode> = pivot
        .opaque
        .iter()
        .filter(|e| e.owner == OpaqueOwner::Definition)
        .map(|e| &e.node)
        .filter(|n| matches!(n, OpaqueNode::Element(_)))
        .collect();
    assert_eq!(
        elements,
        [
            &OpaqueNode::Element(STYLE.to_string()),
            &OpaqueNode::Element(filters.to_string())
        ]
    );
}

#[test]
fn an_excel_saved_definition_keeps_its_extensions_and_rendered_lines() {
    let pivot = read(CORPUS, &[3, 3, 3, 2]);
    assert_eq!(pivot.name, "PivotTable1");
    assert_eq!(pivot.row_fields.len(), 3);
    assert_eq!(
        pivot.fields[3].items,
        [data(0), data(1), line(ItemType::Default)]
    );
    assert_eq!(pivot.options.data_caption.as_deref(), Some("Values"));
    assert!(pivot.options.outline && pivot.options.outline_data);
    let root = kept(&pivot, OpaqueOwner::Definition);
    for piece in [
        r#" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" mc:Ignorable="xr""#,
        r#" xr:uid="{D44FF134-98CE-4CDD-8BF9-3EB1A09B9FAE}""#,
        r#" multipleFieldFilters="0""#,
        r#"<rowItems count="10"><i><x/></i><i r="1"><x/></i>"#,
        r#"<colItems count="1"><i/></colItems>"#,
        r#"<extLst><ext uri="{962EF5D1-5CA2-4c93-8EF4-DBF5C05439D2}" xmlns:x14="http://schemas.microsoft.com/office/spreadsheetml/2009/9/main"><x14:pivotTableDefinition hideValuesRow="1" xmlns:xm="http://schemas.microsoft.com/office/excel/2006/main"/></ext>"#,
    ] {
        assert!(root.contains(piece), "{piece}");
    }
}

/// Whether the model types the attribute `key` of an element named `element` (a data `item`
/// also types `h`).
fn typed_attribute(element: &str, key: &str) -> bool {
    match element {
        TABLE => ROOT_TYPED.contains(&key),
        "location" => LOCATION_TYPED.contains(&key),
        "pivotField" => FIELD_TYPED.contains(&key) || is_subtotal_attribute(key),
        "item" => matches!(key, "t" | "x" | "sd" | "s" | "h"),
        "field" => key == "x",
        "pageField" => matches!(key, "fld" | "item" | "hier"),
        "dataField" => DATA_FIELD_TYPED.contains(&key),
        _ => key == "count",
    }
}

#[test]
fn every_attribute_is_typed_or_kept() {
    let all = [
        (CAPTIONS, &[2, 3, 6, 6, 6][..]),
        (PAGES, &[2, 3, 6, 6, 6]),
        (SUBTOTALS, &[2, 3, 6, 6, 6]),
        (ERRORS, &[2, 3, 6, 6, 6]),
        (CALC_TOP_SORT, &[2, 3, 6, 6]),
        (PAGE_WRAP, &[2, 3, 2, 6, 6]),
        (DATA_ON_ROWS, &[2, 6, 6, 6, 6]),
        (CORPUS, &[3, 3, 3, 2]),
    ];
    for (xml, items) in all {
        let pivot = read(xml, items);
        let store: String = pivot.opaque.iter().map(text).collect();
        let mut reader = quick_xml::Reader::from_str(xml);
        loop {
            let element = match reader.read_event().unwrap() {
                Event::Start(e) | Event::Empty(e) => e,
                Event::Eof => break,
                _ => continue,
            };
            let name = element.local_name().as_ref().to_string();
            for attr in element.attributes() {
                let attr = attr.unwrap();
                let written = format!(" {}=\"{}\"", attr.key.0, attr.value);
                assert!(
                    typed_attribute(&name, attr.key.0) || store.contains(&written),
                    "{} lost {written}",
                    pivot.name
                );
            }
        }
    }
}

#[test]
fn a_data_field_shown_as_a_calculation_is_typed_and_its_pivot_loads() {
    // probes/t4123-pivot-capture Tabular: `Pct` is % of grand total in format 10.
    let element = BytesStart::from_content(
        r#"dataField name="Pct" fld="2" showDataAs="percentOfTotal" baseField="0" baseItem="0" numFmtId="10""#,
        9,
    );
    let data = read_data_field(&element, &ImportLimits::default())
        .map_err(|skip| skip.0)
        .unwrap();
    assert_eq!(data.show_data_as, ShowDataAs::PercentOfTotal);
    assert_eq!(data.number_format, Some(10));
    assert_eq!((data.base_field, data.base_item), (0, 0));
    let plain = BytesStart::from_content(r#"dataField fld="2" subtotal="count""#, 9);
    let plain = read_data_field(&plain, &ImportLimits::default())
        .map_err(|skip| skip.0)
        .unwrap();
    assert_eq!(plain.function, Aggregate::Count);
    assert_eq!((plain.base_field, plain.base_item), (-1, NO_BASE_ITEM));
    assert_eq!(plain.number_format, None);
    // The engine shows the values as the mode says (T-4163), so the pivot loads.
    let pivot = read(TABULAR_PERCENT, &[2, 3, 6]);
    assert_eq!(
        pivot.data_fields[1].show_data_as,
        ShowDataAs::PercentOfTotal
    );
    assert_eq!(pivot.data_fields[1].number_format, Some(10));
    assert_eq!(pivot.data_fields[2].function, Aggregate::Count);
    // An x14 mode is only the extension's to name.
    let x14 = TABULAR_PERCENT.replace("percentOfTotal", "rankAscending");
    let (result, _) = read_with(&x14, cache(&[2, 3, 6]), usize::MAX);
    assert_eq!(result.unwrap_err(), r#"unknown showDataAs "rankAscending""#);
}

#[test]
fn an_x14_show_values_as_mode_is_read_from_the_kept_extension() {
    // As the case builder and Excel write it (pivots.md "Show-values-as modes as Excel for the
    // web renders them"): `pivotShowAs` in the x14 dataField extension, no `showDataAs`.
    let ext = r#"<extLst><ext uri="{E15A36E0-9728-4e99-A89B-3F7291B0FE68}" xmlns:x14="http://schemas.microsoft.com/office/spreadsheetml/2009/9/main"><x14:dataField pivotShowAs="percentOfParent"/></ext></extLst>"#;
    let xml = TABULAR_PERCENT.replace(
        r#"<dataField name="Pct" fld="2" showDataAs="percentOfTotal" baseField="0" baseItem="0" numFmtId="10"/>"#,
        &format!(r#"<dataField name="Pct" fld="2" baseField="0" baseItem="0">{ext}</dataField>"#),
    );
    let pivot = read(&xml, &[2, 3, 6]);
    assert_eq!(
        pivot.data_fields[1].show_data_as,
        ShowDataAs::PercentOfParent
    );
    assert_eq!(pivot.data_fields[0].show_data_as, ShowDataAs::Normal);
    // The extension is still kept verbatim, for export.
    assert_eq!(kept(&pivot, OpaqueOwner::DataField(1)), ext);
    let unknown = xml.replace("percentOfParent", "percentOfNothing");
    let (result, _) = read_with(&unknown, cache(&[2, 3, 6]), usize::MAX);
    assert_eq!(
        result.unwrap_err(),
        r#"unknown showDataAs "percentOfNothing""#
    );
}

#[test]
fn repeat_item_labels_are_read_from_the_kept_field_extension() {
    // As Excel and the case builder write it (pivots.md "Repeat item labels",
    // probes/t4698-pivot-repeat-labels): `fillDownLabels` in the x14 pivotField extension.
    let ext = r#"<extLst><ext uri="{2946ED86-A175-432a-8AC1-64E0C546D7DE}" xmlns:x14="http://schemas.microsoft.com/office/spreadsheetml/2009/9/main"><x14:pivotField fillDownLabels="1"/></ext></extLst>"#;
    let xml = TABULAR_PERCENT.replacen(
        "</items></pivotField>",
        &format!("</items>{ext}</pivotField>"),
        1,
    );
    let pivot = read(&xml, &[2, 3, 6]);
    assert!(pivot.fields[0].fill_down_labels);
    assert!(!pivot.fields[1].fill_down_labels);
    // The extension is still kept verbatim, for export.
    assert_eq!(kept(&pivot, OpaqueOwner::Field(0)), ext);
    // Off and malformed read as the default; the definition-wide default fills no field
    // (t4698 TW, TWE).
    for value in ["0", "yes"] {
        let off = xml.replace(
            r#"fillDownLabels="1""#,
            &format!(r#"fillDownLabels="{value}""#),
        );
        assert!(!read(&off, &[2, 3, 6]).fields[0].fill_down_labels);
    }
    let table = r#"<extLst><ext uri="{962EF5D1-5CA2-4c93-8EF4-DBF5C05439D2}" xmlns:x14="http://schemas.microsoft.com/office/spreadsheetml/2009/9/main"><x14:pivotTableDefinition fillDownLabelsDefault="1"/></ext></extLst>"#;
    let wide = TABULAR_PERCENT.replace(
        "</pivotTableDefinition>",
        &format!("{table}</pivotTableDefinition>"),
    );
    let pivot = read(&wide, &[2, 3, 6]);
    assert!(pivot.fields.iter().all(|f| !f.fill_down_labels));
    assert!(kept(&pivot, OpaqueOwner::Definition).contains(table));
}

#[test]
fn the_opaque_budget_is_charged_and_never_passed() {
    let (pivot, left) = read_with(CORPUS, cache(&[3, 3, 3, 2]), 1 << 20);
    let pivot = pivot.unwrap();
    let cost: usize = pivot.opaque.iter().map(OpaqueEntry::cost).sum();
    assert_eq!(left, (1 << 20) - cost);
    // One byte short: the pivot is skipped rather than kept in part.
    let (short, _) = read_with(CORPUS, cache(&[3, 3, 3, 2]), cost - 1);
    assert_eq!(
        short.unwrap_err(),
        "unread pivot definition content past the import limit"
    );
    let (exact, left) = read_with(CORPUS, cache(&[3, 3, 3, 2]), cost);
    assert_eq!(exact.unwrap().opaque, pivot.opaque);
    assert_eq!(left, 0);
}

#[test]
fn malformed_typed_attributes_skip_the_pivot_or_read_as_the_default() {
    let skipped = |from: &str, to: &str| {
        let xml = SUBTOTALS.replacen(from, to, 1);
        read_with(&xml, cache(&[2, 3, 6, 6, 6]), usize::MAX)
            .0
            .unwrap_err()
    };
    assert!(skipped(r#"<item t="sum"/>"#, r#"<item t="total"/>"#).contains("item type"));
    assert!(skipped(
        r#"<pivotField dataField="1""#,
        r#"<pivotField sortType="up""#
    )
    .contains("sortType"));
    // A value from the file is quoted at most 64 characters long.
    let long = format!(r#"<item t="{}"/>"#, "z".repeat(1000));
    assert_eq!(
        skipped(r#"<item t="sum"/>"#, &long),
        format!("unknown item type {:?}", "z".repeat(64))
    );
    // A number nothing computes from yet reads as absent: the pivot still loads.
    let lenient = SUBTOTALS
        .replacen(r#"compactData="0""#, r#"compactData="0" pageWrap="-1""#, 1)
        .replacen(r#"baseItem="0""#, r#"baseItem="x" numFmtId="ten""#, 1);
    let pivot = read(&lenient, &[2, 3, 6, 6, 6]);
    assert_eq!(pivot.options.page_wrap, 0);
    assert_eq!(pivot.data_fields[0].base_item, NO_BASE_ITEM);
    assert_eq!(pivot.data_fields[0].number_format, None);
    // A boolean that is not one reads as the schema's default, as `showAll` always has.
    let lenient = SUBTOTALS.replacen(r#"compact="0""#, r#"compact="maybe""#, 1);
    assert!(read(&lenient, &[2, 3, 6, 6, 6]).options.compact);
    // A field lists at most its cache items plus one line of each other kind.
    let many = format!(
        r#"{}<item t="default"/>"#,
        r#"<item t="blank"/>"#.repeat(13)
    );
    let crowded = SUBTOTALS.replacen(r#"<item t="default"/>"#, &many, 1);
    assert!(read_with(&crowded, cache(&[2, 3, 6, 6, 6]), usize::MAX)
        .0
        .is_ok());
    let more = format!(r#"<item t="blank"/>{many}"#);
    let crowded = SUBTOTALS.replacen(r#"<item t="default"/>"#, &more, 1);
    assert_eq!(
        read_with(&crowded, cache(&[2, 3, 6, 6, 6]), usize::MAX)
            .0
            .unwrap_err(),
        "more items than the cache field holds"
    );
}
