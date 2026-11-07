//! A pivot table as its `pivotTableDefinition` part (T-4176): the inverse of the importer's
//! `import/pivots/table.rs`.
//!
//! The typed model is written attribute by attribute, each only when it is not the schema's
//! default (the importer reads an absent attribute as that default), and the verbatim rest
//! ([`PivotTable::opaque`]) goes back where it was read: each entry's attributes on its owner,
//! and its elements inside its owner, after the owner's typed children. A child of the root is
//! written where its name falls in `CT_pivotTableDefinition`'s sequence, the invariant the model
//! relies on (`OpaqueNode::Element`). The collections' `count` attributes are written from the
//! lists.
//!
//! The layout lines (`rowItems`, `colItems`) are the saved ones, written back as read, while the
//! pivot still holds them: the engine has not laid it out since it was read, so its cells are the
//! saved ones too (pivots.md "Pivots without refreshOnLoad keep their saved cells"). A pivot the
//! engine laid out (rendered or refreshed, which drops the saved lines:
//! [`rustsheet_engine::pivot::OpaqueEntry::is_layout_lines`]) or one made through the API gets the
//! engine's (`lines`), when its layout gives the location the pivot holds, so they agree with the
//! cells the engine rendered there (docs/excel-semantics/pivots.md "The grid writer's own
//! readings").

use std::collections::HashMap;
use std::fmt::Write as _;

use rustsheet_engine::expressions::utils::number_to_column;
use rustsheet_engine::pivot::{
    element_name, Aggregate, AxisField, AxisLine, CellArea, DataField, ItemType, OpaqueNode,
    OpaqueOwner, PivotCache, PivotField, PivotItem, PivotLayout, PivotOptions, PivotTable,
    ShowDataAs, SortType, NO_BASE_ITEM,
};

use super::super::escape::escape_xml;
use super::super::xml_constants::XML_DECLARATION;
use super::MAIN_NS;
use crate::import::PIVOT_SUBTOTAL_FUNCTIONS;

/// `CT_pivotTableDefinition`'s children in schema order. A kept element of another name goes
/// before `extLst`, the extension point.
const ROOT_CHILDREN: [&str; 17] = [
    "location",
    "pivotFields",
    "rowFields",
    "rowItems",
    "colFields",
    "colItems",
    "pageFields",
    "dataFields",
    "formats",
    "conditionalFormats",
    "chartFormats",
    "pivotHierarchies",
    "pivotTableStyleInfo",
    "filters",
    "rowHierarchiesUsage",
    "colHierarchiesUsage",
    "extLst",
];

/// The rank of a root child named `name` in [`ROOT_CHILDREN`]; an unknown name ranks just
/// before `extLst`.
fn rank(name: &str) -> usize {
    let extension = ROOT_CHILDREN.len() - 1;
    match ROOT_CHILDREN.iter().position(|n| *n == name) {
        Some(at) if at < extension => 2 * at,
        Some(_) => 2 * extension,
        None => 2 * extension - 1,
    }
}

/// The verbatim rest, by owner.
#[derive(Default)]
struct Kept<'a> {
    /// The owner's kept attributes, each starting with a space.
    attributes: String,
    /// The owner's kept child elements, in file order.
    elements: Vec<&'a str>,
}

/// One kept owner's entries, or none.
struct Opaque<'a>(HashMap<OpaqueOwner, Kept<'a>>);

impl<'a> Opaque<'a> {
    fn of(table: &'a PivotTable) -> Self {
        let mut by_owner: HashMap<OpaqueOwner, Kept<'a>> = HashMap::new();
        for entry in &table.opaque {
            let kept = by_owner.entry(entry.owner).or_default();
            match &entry.node {
                OpaqueNode::Attributes(xml) => kept.attributes.push_str(xml),
                OpaqueNode::Element(xml) => kept.elements.push(xml),
            }
        }
        Opaque(by_owner)
    }

    fn attributes(&self, owner: OpaqueOwner) -> &str {
        self.0.get(&owner).map_or("", |k| k.attributes.as_str())
    }

    fn elements(&self, owner: OpaqueOwner) -> &[&'a str] {
        self.0.get(&owner).map_or(&[], |k| k.elements.as_slice())
    }

    fn has(&self, owner: OpaqueOwner) -> bool {
        self.0.contains_key(&owner)
    }
}

/// The `pivotTableDefinition` part of `table` over `cache`, naming it `cache_id`. `lines` is the
/// engine's layout when it gives the location `table` holds.
pub(super) fn table_xml(
    table: &PivotTable,
    cache: &PivotCache,
    cache_id: u32,
    lines: Option<&PivotLayout>,
) -> String {
    let opaque = Opaque::of(table);
    let mut xml = format!("{XML_DECLARATION}\n<pivotTableDefinition");
    let root = opaque.attributes(OpaqueOwner::Definition);
    // A pivot made through the API has no namespace declaration of its own to keep.
    if !root.contains(" xmlns=") {
        let _ = write!(xml, r#" xmlns="{MAIN_NS}""#);
    }
    xml.push_str(root);
    let _ = write!(
        xml,
        r#" name="{}" cacheId="{cache_id}""#,
        escape_xml(&table.name)
    );
    write_options(&mut xml, &table.options);
    xml.push('>');

    let mut kept: Vec<(usize, &str)> = (opaque.elements(OpaqueOwner::Definition).iter())
        .map(|element| (rank(element_name(element)), *element))
        .filter(|(_, element)| {
            lines.is_none() || !matches!(element_name(element), "rowItems" | "colItems")
        })
        .collect();
    kept.sort_by_key(|(rank, _)| *rank);
    let mut kept = kept.into_iter().peekable();
    let mut typed = |xml: &mut String, name: &str, part: &str| {
        let at = rank(name);
        while let Some((_, element)) = kept.next_if(|(r, _)| *r < at) {
            xml.push_str(element);
        }
        xml.push_str(part);
    };
    typed(&mut xml, "location", &location_xml(table, &opaque));
    typed(&mut xml, "pivotFields", &fields_xml(table, cache, &opaque));
    let axis = |owner, each: fn(u32) -> OpaqueOwner, name, fields: &[AxisField]| {
        axis_fields_xml(&opaque, name, owner, each, fields)
    };
    let rows = axis(
        OpaqueOwner::RowFields,
        OpaqueOwner::RowField,
        "rowFields",
        &table.row_fields,
    );
    typed(&mut xml, "rowFields", &rows);
    if let Some(layout) = lines {
        typed(&mut xml, "rowItems", &lines_xml("rowItems", &layout.rows));
    }
    let columns = axis(
        OpaqueOwner::ColumnFields,
        OpaqueOwner::ColumnField,
        "colFields",
        &table.column_fields,
    );
    typed(&mut xml, "colFields", &columns);
    if let Some(layout) = lines {
        typed(
            &mut xml,
            "colItems",
            &lines_xml("colItems", &layout.columns),
        );
    }
    typed(&mut xml, "pageFields", &page_fields_xml(table, &opaque));
    typed(&mut xml, "dataFields", &data_fields_xml(table, &opaque));
    for (_, element) in kept {
        xml.push_str(element);
    }
    xml.push_str("</pivotTableDefinition>");
    xml
}

/// The pivot-wide attributes that differ from the schema's defaults. `dataCaption` is required,
/// so a pivot without one gets Excel's `Values`.
fn write_options(xml: &mut String, options: &PivotOptions) {
    let defaults = PivotOptions::default();
    let flags = [
        ("compact", options.compact, defaults.compact),
        ("outline", options.outline, defaults.outline),
        ("outlineData", options.outline_data, defaults.outline_data),
        ("compactData", options.compact_data, defaults.compact_data),
        (
            "rowGrandTotals",
            options.row_grand_totals,
            defaults.row_grand_totals,
        ),
        (
            "colGrandTotals",
            options.column_grand_totals,
            defaults.column_grand_totals,
        ),
        ("dataOnRows", options.data_on_rows, defaults.data_on_rows),
        ("showMissing", options.show_missing, defaults.show_missing),
        ("showError", options.show_error, defaults.show_error),
        (
            "pageOverThenDown",
            options.page_over_then_down,
            defaults.page_over_then_down,
        ),
        (
            "gridDropZones",
            options.grid_drop_zones,
            defaults.grid_drop_zones,
        ),
    ];
    for (name, value, default) in flags {
        if value != default {
            let _ = write!(xml, r#" {name}="{}""#, u8::from(value));
        }
    }
    if let Some(position) = options.data_position {
        let _ = write!(xml, r#" dataPosition="{position}""#);
    }
    if options.page_wrap != defaults.page_wrap {
        let _ = write!(xml, r#" pageWrap="{}""#, options.page_wrap);
    }
    let data_caption = Some(options.data_caption.as_deref().unwrap_or("Values"));
    let captions = [
        ("rowHeaderCaption", options.row_header_caption.as_deref()),
        ("colHeaderCaption", options.column_header_caption.as_deref()),
        ("grandTotalCaption", options.grand_total_caption.as_deref()),
        ("dataCaption", data_caption),
        ("missingCaption", options.missing_caption.as_deref()),
        ("errorCaption", options.error_caption.as_deref()),
    ];
    for (name, caption) in captions {
        if let Some(caption) = caption {
            let _ = write!(xml, r#" {name}="{}""#, escape_xml(caption));
        }
    }
}

/// `<location>`: its `ref`, the three offsets the schema requires and the page counts when set.
fn location_xml(table: &PivotTable, opaque: &Opaque<'_>) -> String {
    let geometry = &table.geometry;
    let mut xml = format!(
        r#"<location ref="{}" firstHeaderRow="{}" firstDataRow="{}" firstDataCol="{}""#,
        area_ref(&table.location),
        geometry.first_header_row,
        geometry.first_data_row,
        geometry.first_data_column
    );
    if geometry.row_page_count != 0 {
        let _ = write!(xml, r#" rowPageCount="{}""#, geometry.row_page_count);
    }
    if geometry.column_page_count != 0 {
        let _ = write!(xml, r#" colPageCount="{}""#, geometry.column_page_count);
    }
    close(&mut xml, opaque, OpaqueOwner::Location, "location", "");
    xml
}

/// A 0-based area in A1 notation; a single cell is one reference, as Excel writes it.
fn area_ref(area: &CellArea) -> String {
    let cell = |row: u32, column: u32| {
        let letters = i32::try_from(column + 1)
            .ok()
            .and_then(number_to_column)
            .unwrap_or_default();
        format!("{letters}{}", u64::from(row) + 1)
    };
    let start = cell(area.top, area.left);
    if (area.top, area.left) == (area.bottom, area.right) {
        return start;
    }
    format!("{start}:{}", cell(area.bottom, area.right))
}

/// Ends an element whose attributes are written: its kept attributes, then `children` and its
/// kept elements, or `/>` when it has neither.
fn close(xml: &mut String, opaque: &Opaque<'_>, owner: OpaqueOwner, name: &str, children: &str) {
    xml.push_str(opaque.attributes(owner));
    let elements = opaque.elements(owner);
    if children.is_empty() && elements.is_empty() {
        xml.push_str("/>");
        return;
    }
    xml.push('>');
    xml.push_str(children);
    for element in elements {
        xml.push_str(element);
    }
    let _ = write!(xml, "</{name}>");
}

/// A typed collection: `<name count>`, its children, its kept rest; nothing when it has no
/// children and keeps nothing.
fn collection(
    opaque: &Opaque<'_>,
    owner: OpaqueOwner,
    name: &str,
    count: usize,
    children: &str,
) -> String {
    if count == 0 && !opaque.has(owner) {
        return String::new();
    }
    let mut xml = format!(r#"<{name} count="{count}""#);
    close(&mut xml, opaque, owner, name, children);
    xml
}

/// `<pivotFields>`.
fn fields_xml(table: &PivotTable, cache: &PivotCache, opaque: &Opaque<'_>) -> String {
    let mut children = String::new();
    for (index, field) in (0u32..).zip(&table.fields) {
        let item_count = cache
            .fields
            .get(index as usize)
            .map_or(0, |f| f.items.len());
        let is_data = table.data_fields.iter().any(|d| d.field == index);
        field_xml(&mut children, field, index, item_count, is_data, opaque);
    }
    collection(
        opaque,
        OpaqueOwner::PivotFields,
        "pivotFields",
        table.fields.len(),
        &children,
    )
}

fn field_xml(
    xml: &mut String,
    field: &PivotField,
    index: u32,
    item_count: usize,
    is_data: bool,
    opaque: &Opaque<'_>,
) {
    let owner = OpaqueOwner::Field(index);
    xml.push_str("<pivotField");
    if let Some(axis) = field.axis {
        let axis = match axis {
            rustsheet_engine::pivot::Axis::Row => "axisRow",
            rustsheet_engine::pivot::Axis::Column => "axisCol",
            rustsheet_engine::pivot::Axis::Page => "axisPage",
        };
        let _ = write!(xml, r#" axis="{axis}""#);
    }
    // Excel marks a data field's pivot field; a pivot made through the API keeps no such flag.
    if is_data && !opaque.attributes(owner).contains(" dataField=") {
        xml.push_str(r#" dataField="1""#);
    }
    let defaults = PivotField::default();
    let flags = [
        ("showAll", field.show_all, defaults.show_all),
        ("compact", field.compact, defaults.compact),
        ("outline", field.outline, defaults.outline),
        ("subtotalTop", field.subtotal_top, defaults.subtotal_top),
        (
            "defaultSubtotal",
            field.default_subtotal,
            defaults.default_subtotal,
        ),
        (
            "insertBlankRow",
            field.insert_blank_row,
            defaults.insert_blank_row,
        ),
    ];
    for (name, value, default) in flags {
        if value != default {
            let _ = write!(xml, r#" {name}="{}""#, u8::from(value));
        }
    }
    for function in &field.custom_subtotals {
        if let Some(name) = subtotal_name(*function) {
            let _ = write!(xml, r#" {name}Subtotal="1""#);
        }
    }
    match field.sort_type {
        SortType::Manual => {}
        SortType::Ascending => xml.push_str(r#" sortType="ascending""#),
        SortType::Descending => xml.push_str(r#" sortType="descending""#),
    }
    if let Some(format) = field.number_format {
        let _ = write!(xml, r#" numFmtId="{format}""#);
    }
    if let Some(caption) = &field.subtotal_caption {
        let _ = write!(xml, r#" subtotalCaption="{}""#, escape_xml(caption));
    }
    let items = items_xml(field, index, item_count, opaque);
    close(xml, opaque, owner, "pivotField", &items);
}

/// `<items>`. An axis field that lists none gets one item per cache item and its subtotal
/// lines, as Excel needs `<items>` on every axis field (pivots.md "Excel for the web renders
/// authored pivot definitions"); the layout shows unlisted items in that order already.
fn items_xml(field: &PivotField, index: u32, item_count: usize, opaque: &Opaque<'_>) -> String {
    let owner = OpaqueOwner::Items(index);
    let mut children = String::new();
    if field.items.is_empty() && field.axis.is_some() && !opaque.has(owner) {
        let mut count = 0;
        for x in (0u32..).take(item_count) {
            item_xml(&mut children, field, &PivotItem::data(x), None, opaque);
            count += 1;
        }
        if field.default_subtotal {
            let subtotals: Vec<ItemType> = if field.custom_subtotals.is_empty() {
                vec![ItemType::Default]
            } else {
                field
                    .custom_subtotals
                    .iter()
                    .map(|f| ItemType::Function(*f))
                    .collect()
            };
            for kind in subtotals {
                let line = PivotItem {
                    kind,
                    index: None,
                    expanded: true,
                    filtered: false,
                };
                item_xml(&mut children, field, &line, None, opaque);
                count += 1;
            }
        }
        return collection(opaque, owner, "items", count, &children);
    }
    for (position, item) in (0u32..).zip(&field.items) {
        let owner = OpaqueOwner::Item {
            field: index,
            item: position,
        };
        item_xml(&mut children, field, item, Some(owner), opaque);
    }
    collection(opaque, owner, "items", field.items.len(), &children)
}

fn item_xml(
    xml: &mut String,
    field: &PivotField,
    item: &PivotItem,
    owner: Option<OpaqueOwner>,
    opaque: &Opaque<'_>,
) {
    xml.push_str("<item");
    if let Some(kind) = item_type_name(item.kind) {
        let _ = write!(xml, r#" t="{kind}""#);
    }
    if let Some(x) = item.index {
        let _ = write!(xml, r#" x="{x}""#);
    }
    if field.item_hidden(item) {
        xml.push_str(r#" h="1""#);
    }
    if !item.expanded {
        xml.push_str(r#" sd="0""#);
    }
    if item.filtered {
        xml.push_str(r#" s="1""#);
    }
    match owner {
        Some(owner) => close(xml, opaque, owner, "item", ""),
        None => xml.push_str("/>"),
    }
}

/// `ST_ItemType`'s name of a line; `None` for `data`, the default.
fn item_type_name(kind: ItemType) -> Option<&'static str> {
    match kind {
        ItemType::Data => None,
        ItemType::Default => Some("default"),
        ItemType::Grand => Some("grand"),
        ItemType::Blank => Some("blank"),
        ItemType::Function(function) => subtotal_name(function),
    }
}

/// The `ST_ItemType` name of a custom subtotal (`countA` for [`Aggregate::Count`]).
fn subtotal_name(function: Aggregate) -> Option<&'static str> {
    let mut functions = PIVOT_SUBTOTAL_FUNCTIONS.iter();
    functions.find(|(_, f)| *f == function).map(|(n, _)| *n)
}

/// `<rowFields>` or `<colFields>`.
fn axis_fields_xml(
    opaque: &Opaque<'_>,
    name: &str,
    owner: OpaqueOwner,
    each: fn(u32) -> OpaqueOwner,
    fields: &[AxisField],
) -> String {
    let mut children = String::new();
    for (position, field) in (0u32..).zip(fields) {
        let x = match field {
            AxisField::Values => -2,
            AxisField::Field(index) => i64::from(*index),
        };
        let _ = write!(children, r#"<field x="{x}""#);
        close(&mut children, opaque, each(position), "field", "");
    }
    collection(opaque, owner, name, fields.len(), &children)
}

/// `<pageFields>`.
fn page_fields_xml(table: &PivotTable, opaque: &Opaque<'_>) -> String {
    let mut children = String::new();
    for (position, page) in (0u32..).zip(&table.page_fields) {
        let _ = write!(children, r#"<pageField fld="{}""#, page.field);
        if let Some(item) = page.item {
            let _ = write!(children, r#" item="{item}""#);
        }
        if let Some(hierarchy) = page.hierarchy {
            let _ = write!(children, r#" hier="{hierarchy}""#);
        }
        close(
            &mut children,
            opaque,
            OpaqueOwner::PageField(position),
            "pageField",
            "",
        );
    }
    collection(
        opaque,
        OpaqueOwner::PageFields,
        "pageFields",
        table.page_fields.len(),
        &children,
    )
}

/// `<dataFields>`.
fn data_fields_xml(table: &PivotTable, opaque: &Opaque<'_>) -> String {
    let defaults = DataField::default();
    let mut children = String::new();
    for (position, data) in (0u32..).zip(&table.data_fields) {
        let _ = write!(
            children,
            r#"<dataField name="{}" fld="{}""#,
            escape_xml(&data.caption),
            data.field
        );
        if data.function != defaults.function {
            let _ = write!(
                children,
                r#" subtotal="{}""#,
                data.function.consolidate_name()
            );
        }
        // An x14 mode is the kept dataField extension's `pivotShowAs`, written back verbatim
        // (`ShowDataAs::is_x14`): `showDataAs` cannot name it.
        if data.show_data_as != ShowDataAs::Normal && !data.show_data_as.is_x14() {
            let _ = write!(children, r#" showDataAs="{}""#, data.show_data_as.name());
        }
        if data.base_field != defaults.base_field {
            let _ = write!(children, r#" baseField="{}""#, data.base_field);
        }
        if data.base_item != NO_BASE_ITEM {
            let _ = write!(children, r#" baseItem="{}""#, data.base_item);
        }
        if let Some(format) = data.number_format {
            let _ = write!(children, r#" numFmtId="{format}""#);
        }
        close(
            &mut children,
            opaque,
            OpaqueOwner::DataField(position),
            "dataField",
            "",
        );
    }
    collection(
        opaque,
        OpaqueOwner::DataFields,
        "dataFields",
        table.data_fields.len(),
        &children,
    )
}

/// `<rowItems>` or `<colItems>` as Excel writes them: `t`, `r` and `i` only when set, and an
/// `x` of 0 as `<x/>` (the Excel-saved pivot of crates/rustsheet-xlsx/tests/example.xlsx).
fn lines_xml(name: &str, lines: &[AxisLine]) -> String {
    let mut xml = format!(r#"<{name} count="{}">"#, lines.len());
    for line in lines {
        xml.push_str("<i");
        if let Some(kind) = item_type_name(line.kind) {
            let _ = write!(xml, r#" t="{kind}""#);
        }
        if line.repeat != 0 {
            let _ = write!(xml, r#" r="{}""#, line.repeat);
        }
        if line.data_field != 0 {
            let _ = write!(xml, r#" i="{}""#, line.data_field);
        }
        if line.items.is_empty() {
            xml.push_str("/>");
            continue;
        }
        xml.push('>');
        for x in &line.items {
            if *x == 0 {
                xml.push_str("<x/>");
            } else {
                let _ = write!(xml, r#"<x v="{x}"/>"#);
            }
        }
        xml.push_str("</i>");
    }
    let _ = write!(xml, "</{name}>");
    xml
}
