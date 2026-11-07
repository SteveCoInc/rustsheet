//! The `pivotTableDefinition` part (SPEC-0015 "xlsx import", SPEC-0032 "Model (T-4139)").
//!
//! The attributes the model types are read into [`PivotTable`]; every other attribute and element
//! is kept verbatim in [`PivotTable::opaque`], owned by the typed element it was read in, for
//! export (T-4176). The collections' `count` attributes are the one exception: they follow from
//! the lists. What is kept is charged, entry by entry ([`OpaqueEntry::cost`]), to the workbook's
//! opaque budget (`MAX_OPAQUE_BYTES`), so the store never grows past it (web.md "Pivot caches past the import limits
//! are dropped").

use std::cell::Cell;
use std::collections::HashMap;
use std::io::BufRead;

use quick_xml::events::BytesStart;
use rustsheet_engine::expressions::utils::{is_valid_column_number, is_valid_row};
use rustsheet_engine::pivot::{
    element_name, Aggregate, AutoShow, Axis, AxisField, CellArea, DataField, ItemType, OpaqueEntry,
    OpaqueNode, OpaqueOwner, PageField, PivotCache, PivotField, PivotGeometry, PivotItem,
    PivotOptions, PivotTable, ShowDataAs, SortType, NO_BASE_ITEM,
};

use super::super::package::ImportLimits;
use super::super::worksheets::parse_range;
use super::filters::read_filters;
use super::{
    at, attribute, index_attribute, local_name, shown, skip, walk, walk_keeping, Skip, Tag,
    MAX_AXIS_FIELDS, MAX_DATA_FIELDS,
};

const TABLE: &str = "pivotTableDefinition";

/// The location of a pivot until its `<location>` is read; a part without one is skipped.
const UNREAD_LOCATION: CellArea = CellArea {
    top: 0,
    left: 0,
    bottom: 0,
    right: 0,
};

/// Most `<item>`s a pivot field lists past its cache field's items: one of each subtotal, grand
/// and blank type (`ST_ItemType` has 14 besides `data`), so a field never lists more than its
/// cache holds plus these.
const MAX_EXTRA_ITEMS: usize = 14;

/// The root attributes the model types.
const ROOT_TYPED: [&str; 21] = [
    "name",
    "cacheId",
    "compact",
    "outline",
    "outlineData",
    "compactData",
    "rowGrandTotals",
    "colGrandTotals",
    "dataOnRows",
    "dataPosition",
    "rowHeaderCaption",
    "colHeaderCaption",
    "grandTotalCaption",
    "dataCaption",
    "showMissing",
    "missingCaption",
    "showError",
    "errorCaption",
    "pageWrap",
    "pageOverThenDown",
    "gridDropZones",
];

/// The custom subtotal functions by their `ST_ItemType` names: an `<item t>` names its subtotal
/// line so, and a `<pivotField>` asks for it with `<name>Subtotal="1"`. In the schema's order.
/// `countA` is COUNTA and `count` Count Numbers, as ECMA-376 reads them and Excel computes them;
/// Excel labels both lines `<item> Count` (`ItemType::Function`). Excel: see
/// docs/excel-semantics/pivots.md "Which function the countA and count custom subtotals compute"
/// (probes/t4622-pivot-count-subtotals).
pub(crate) const SUBTOTAL_FUNCTIONS: [(&str, Aggregate); 11] = [
    ("sum", Aggregate::Sum),
    ("countA", Aggregate::Count),
    ("avg", Aggregate::Average),
    ("max", Aggregate::Max),
    ("min", Aggregate::Min),
    ("product", Aggregate::Product),
    ("count", Aggregate::CountNums),
    ("stdDev", Aggregate::StdDev),
    ("stdDevP", Aggregate::StdDevP),
    ("var", Aggregate::Var),
    ("varP", Aggregate::VarP),
];

/// The function of the custom subtotal named `name` (`sum`, `countA`, ...).
fn subtotal_function(name: &str) -> Option<Aggregate> {
    let mut functions = SUBTOTAL_FUNCTIONS.iter();
    functions.find(|(n, _)| *n == name).map(|(_, f)| *f)
}

/// Whether `key` is a `<pivotField>` custom subtotal attribute (`sumSubtotal`, ...).
fn is_subtotal_attribute(key: &str) -> bool {
    key.strip_suffix("Subtotal")
        .and_then(subtotal_function)
        .is_some()
}

/// The `<pivotField>` attributes the model types, besides the custom subtotals
/// ([`is_subtotal_attribute`]).
const FIELD_TYPED: [&str; 10] = [
    "axis",
    "showAll",
    "compact",
    "outline",
    "subtotalTop",
    "defaultSubtotal",
    "sortType",
    "insertBlankRow",
    "numFmtId",
    "subtotalCaption",
];

/// The `<location>` attributes the model types.
const LOCATION_TYPED: [&str; 6] = [
    "ref",
    "firstHeaderRow",
    "firstDataRow",
    "firstDataCol",
    "rowPageCount",
    "colPageCount",
];

const DATA_FIELD_TYPED: [&str; 7] = [
    "name",
    "fld",
    "subtotal",
    "showDataAs",
    "baseField",
    "baseItem",
    "numFmtId",
];

/// Whether `name` is an element the model reads at `path` (the enclosing local names); any other
/// element is kept whole.
fn typed(path: &[String], name: &str) -> bool {
    let path: Vec<&str> = path.iter().map(String::as_str).collect();
    match path.as_slice() {
        [] => true,
        [TABLE] => matches!(
            name,
            "location" | "pivotFields" | "rowFields" | "colFields" | "pageFields" | "dataFields"
        ),
        [TABLE, "pivotFields"] => name == "pivotField",
        [TABLE, "pivotFields", "pivotField"] => name == "items",
        [TABLE, "pivotFields", "pivotField", "items"] => name == "item",
        [TABLE, "rowFields" | "colFields"] => name == "field",
        [TABLE, "pageFields"] => name == "pageField",
        [TABLE, "dataFields"] => name == "dataField",
        _ => false,
    }
}

/// A `pivotTableDefinition` part on the sheet `sheet_id`, against the loaded `caches`. What it
/// keeps verbatim is charged to `opaque_left` as it is read; the caller keeps the charge only
/// for a pivot that loads.
pub(super) fn read_pivot_table<B: BufRead>(
    part: B,
    sheet_id: u32,
    caches: &[PivotCache],
    cache_ids: &HashMap<u32, u32>,
    limits: &ImportLimits,
    opaque_left: &Cell<usize>,
) -> Result<PivotTable, Skip> {
    let mut reader = TableReader {
        caches,
        cache_ids,
        limits,
        opaque_left,
        cache: None,
        // The root and `<location>` fill in the rest.
        table: PivotTable::new(String::new(), sheet_id, UNREAD_LOCATION, 0),
        located: false,
    };
    walk_keeping(
        part,
        |path, element| !typed(path, &local_name(element)),
        opaque_left,
        |path, tag| match tag {
            Tag::Open(element) => reader.open(path, element),
            Tag::Kept(xml) => {
                let owner = reader.owner(path)?;
                reader.read_extension(owner, &xml)?;
                reader.keep(owner, OpaqueNode::Element(xml))
            }
            Tag::Close(_) => Ok(()),
        },
    )?;
    let Some(cache) = reader.cache else {
        return skip("not a pivotTableDefinition");
    };
    if !reader.located {
        return skip("no location");
    }
    if cache.fields.len() != reader.table.fields.len() {
        return skip("pivotFields do not match the cache fields");
    }
    Ok(reader.table)
}

/// The state of [`read_pivot_table`].
struct TableReader<'a> {
    caches: &'a [PivotCache],
    cache_ids: &'a HashMap<u32, u32>,
    limits: &'a ImportLimits,
    opaque_left: &'a Cell<usize>,
    /// The pivot's cache, once the root has named it.
    cache: Option<&'a PivotCache>,
    table: PivotTable,
    /// Whether `<location>` was read.
    located: bool,
}

impl TableReader<'_> {
    /// A typed element: its typed attributes are read, the rest kept.
    fn open(&mut self, path: &[String], element: &BytesStart) -> Result<(), Skip> {
        let name = local_name(element);
        if path.is_empty() {
            if name != TABLE {
                return skip("not a pivotTableDefinition");
            }
            self.read_root(element)?;
            return self.keep_attributes(OpaqueOwner::Definition, element, |key| {
                ROOT_TYPED.contains(&key)
            });
        }
        let Some(cache) = self.cache else {
            return skip("no pivotTableDefinition root");
        };
        let field_count = cache.fields.len();
        let field_index = |element: &BytesStart, attr: &str| -> Result<u32, Skip> {
            let index = index_attribute(element, attr)?;
            if index as usize >= field_count {
                return skip(format!("field index {index} is past the cache's fields"));
            }
            Ok(index)
        };
        // Whether an attribute is typed, so not kept. The collections' `count` follows from
        // their lists.
        let typed: fn(&str) -> bool = match name.as_str() {
            "location" => {
                self.table.location = location(attribute(element, "ref")?.as_deref())?;
                self.table.geometry = geometry(element)?;
                self.located = true;
                |key| LOCATION_TYPED.contains(&key)
            }
            "pivotField" => {
                self.read_field(cache, element)?;
                |key| FIELD_TYPED.contains(&key) || is_subtotal_attribute(key)
            }
            "item" => {
                if self.read_item(cache, element)? {
                    |key| matches!(key, "t" | "x" | "sd" | "s" | "h")
                } else {
                    // `h` on a line that names no cache item has nowhere typed to go.
                    |key| matches!(key, "t" | "x" | "sd" | "s")
                }
            }
            "field" => {
                let axis_field = match attribute(element, "x")?.as_deref() {
                    Some("-2") => AxisField::Values,
                    _ => AxisField::Field(field_index(element, "x")?),
                };
                let axis_fields = if path.get(1).is_some_and(|p| p == "rowFields") {
                    &mut self.table.row_fields
                } else {
                    &mut self.table.column_fields
                };
                if axis_fields.len() >= MAX_AXIS_FIELDS {
                    return skip("too many fields on an axis");
                }
                axis_fields.push(axis_field);
                |key| key == "x"
            }
            "pageField" => {
                if self.table.page_fields.len() >= MAX_AXIS_FIELDS {
                    return skip("too many page fields");
                }
                let page = PageField {
                    field: field_index(element, "fld")?,
                    // Not checked against the field's items: nothing reads it yet (T-4142).
                    item: number(element, "item")?,
                    hierarchy: number(element, "hier")?,
                };
                self.table.page_fields.push(page);
                |key| matches!(key, "fld" | "item" | "hier")
            }
            "dataField" => {
                if self.table.data_fields.len() >= MAX_DATA_FIELDS {
                    return skip("too many data fields");
                }
                let mut data = read_data_field(element, self.limits)?;
                data.field = field_index(element, "fld")?;
                self.table.data_fields.push(data);
                |key| DATA_FIELD_TYPED.contains(&key)
            }
            // `pivotFields`, `items`, `rowFields`, `colFields`, `pageFields`, `dataFields`.
            _ => |key| key == "count",
        };
        // Read first, so the element is the last of its kind `owner` counts.
        let mut inner = path.to_vec();
        inner.push(name);
        let owner = self.owner(&inner)?;
        self.keep_attributes(owner, element, typed)
    }

    /// An element kept whole inside the typed element `owner`: the x14 extension attributes the
    /// model types are read from it, and it stays kept verbatim for export.
    /// - In `<dataField>`, `pivotShowAs` is the field's mode (an Excel 2010 mode is written there
    ///   alone, with no `showDataAs`: pivots.md "Show-values-as modes as Excel for the web renders
    ///   them"). Beside a `showDataAs`, the x14 mode wins (UNVERIFIED, T-4750).
    /// - In `<pivotField>`, `fillDownLabels` is the field's Repeat Item Labels (pivots.md "Repeat
    ///   item labels", probes/t4698-pivot-repeat-labels). The definition's own
    ///   `fillDownLabelsDefault` is not read: Excel fills no existing field down for it (t4129 X14,
    ///   t4698 TW, TWE, OW).
    /// - The definition's `<filters>` are its typed filters (`filters::read_filters`, T-4165).
    fn read_extension(&mut self, owner: OpaqueOwner, xml: &str) -> Result<(), Skip> {
        match owner {
            OpaqueOwner::DataField(index) => {
                let mut mode = None;
                each_x14_element(xml, "dataField", |element| {
                    if let Some(name) = attribute(element, "pivotShowAs")? {
                        mode = Some(show_data_as(&name)?);
                    }
                    Ok(())
                })?;
                let data = self.table.data_fields.get_mut(index as usize);
                if let (Some(mode), Some(data)) = (mode, data) {
                    data.show_data_as = mode;
                }
            }
            OpaqueOwner::Field(index) => {
                let mut fill = None;
                each_x14_element(xml, "pivotField", |element| {
                    fill = Some(flag(element, "fillDownLabels", false)?);
                    Ok(())
                })?;
                if let (Some(fill), Some(field)) = (fill, self.table.fields.get_mut(index as usize))
                {
                    field.fill_down_labels = fill;
                }
            }
            OpaqueOwner::Definition if element_name(xml) == "filters" => {
                if let Some(cache) = self.cache {
                    self.table.filters = read_filters(xml, cache, self.limits)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// The root: the pivot's name, its cache and the pivot-wide options.
    fn read_root(&mut self, element: &BytesStart) -> Result<(), Skip> {
        let text = |name: &str| -> Result<Option<String>, Skip> {
            let value = attribute(element, name)?;
            if let Some(value) = &value {
                self.limits.check_text(value)?;
            }
            Ok(value)
        };
        self.table.name = text("name")?.unwrap_or_default();
        let id = index_attribute(element, "cacheId")?;
        let Some(&index) = self.cache_ids.get(&id) else {
            return skip(format!("cacheId {id} has no loaded cache"));
        };
        self.table.cache = index;
        self.cache = self.caches.get(index as usize);
        let defaults = PivotOptions::default();
        self.table.options = PivotOptions {
            compact: flag(element, "compact", defaults.compact)?,
            outline: flag(element, "outline", defaults.outline)?,
            outline_data: flag(element, "outlineData", defaults.outline_data)?,
            compact_data: flag(element, "compactData", defaults.compact_data)?,
            row_grand_totals: flag(element, "rowGrandTotals", defaults.row_grand_totals)?,
            column_grand_totals: flag(element, "colGrandTotals", defaults.column_grand_totals)?,
            data_on_rows: flag(element, "dataOnRows", defaults.data_on_rows)?,
            data_position: number(element, "dataPosition")?,
            row_header_caption: text("rowHeaderCaption")?,
            column_header_caption: text("colHeaderCaption")?,
            grand_total_caption: text("grandTotalCaption")?,
            data_caption: text("dataCaption")?,
            show_missing: flag(element, "showMissing", defaults.show_missing)?,
            missing_caption: text("missingCaption")?,
            show_error: flag(element, "showError", defaults.show_error)?,
            error_caption: text("errorCaption")?,
            page_wrap: number(element, "pageWrap")?.unwrap_or(defaults.page_wrap),
            page_over_then_down: flag(element, "pageOverThenDown", defaults.page_over_then_down)?,
            grid_drop_zones: flag(element, "gridDropZones", defaults.grid_drop_zones)?,
        };
        Ok(())
    }

    /// A `<pivotField>`, the next of the cache's fields.
    fn read_field(&mut self, cache: &PivotCache, element: &BytesStart) -> Result<(), Skip> {
        let Some(cache_field) = cache.fields.get(self.table.fields.len()) else {
            return skip("more pivotFields than cache fields");
        };
        let axis = match attribute(element, "axis")?.as_deref() {
            Some("axisRow") => Some(Axis::Row),
            Some("axisCol") => Some(Axis::Column),
            Some("axisPage") => Some(Axis::Page),
            Some("axisValues") | None => None,
            Some(other) => return skip(format!("unknown axis {:?}", shown(other))),
        };
        let mut field = PivotField::new(axis, cache_field.items.len());
        // ECMA-376's default is on; Excel writes `showAll="0"` (web.md, T-4276).
        field.show_all = flag(element, "showAll", field.show_all)?;
        field.compact = flag(element, "compact", field.compact)?;
        field.outline = flag(element, "outline", field.outline)?;
        field.subtotal_top = flag(element, "subtotalTop", field.subtotal_top)?;
        field.default_subtotal = flag(element, "defaultSubtotal", field.default_subtotal)?;
        for (name, function) in SUBTOTAL_FUNCTIONS {
            if flag(element, &format!("{name}Subtotal"), false)? {
                field.custom_subtotals.push(function);
            }
        }
        field.sort_type = match attribute(element, "sortType")?.as_deref() {
            None | Some("manual") => SortType::Manual,
            Some("ascending") => SortType::Ascending,
            Some("descending") => SortType::Descending,
            Some(other) => return skip(format!("unknown sortType {:?}", shown(other))),
        };
        field.insert_blank_row = flag(element, "insertBlankRow", field.insert_blank_row)?;
        // Read, and kept verbatim for export (they are not in FIELD_TYPED), as `fillDownLabels`
        // is (T-4165).
        if flag(element, "autoShow", false)? {
            field.auto_show = Some(AutoShow {
                count: number(element, "itemPageCount")?.unwrap_or(10),
                top: flag(element, "topAutoShow", true)?,
                measure: number::<i64>(element, "rankBy")?
                    .and_then(|m| u32::try_from(m).ok())
                    .unwrap_or(0),
            });
        }
        field.number_format = number(element, "numFmtId")?;
        field.subtotal_caption = attribute(element, "subtotalCaption")?;
        if let Some(caption) = &field.subtotal_caption {
            self.limits.check_text(caption)?;
        }
        self.table.fields.push(field);
        Ok(())
    }

    /// An `<item>` of the last `<pivotField>`'s `<items>`; whether it names a cache item.
    fn read_item(&mut self, cache: &PivotCache, element: &BytesStart) -> Result<bool, Skip> {
        let current = self.table.fields.len().saturating_sub(1);
        let (Some(field), Some(cache_field)) =
            (self.table.fields.last_mut(), cache.fields.get(current))
        else {
            return skip("an item outside a pivotField");
        };
        if field.items.len() >= cache_field.items.len().saturating_add(MAX_EXTRA_ITEMS) {
            return skip("more items than the cache field holds");
        }
        let kind = item_type(attribute(element, "t")?.as_deref())?;
        let index = if kind == ItemType::Data {
            let item = index_attribute(element, "x")?;
            if item as usize >= cache_field.items.len() {
                return skip(format!("item index {item} is past the field's items"));
            }
            Some(item)
        } else {
            // Subtotal, grand and blank lines name no cache item.
            number(element, "x")?
        };
        let item = PivotItem {
            kind,
            index,
            expanded: flag(element, "sd", true)?,
            filtered: flag(element, "s", false)?,
        };
        // `h` of a cache item lives in the field's hidden bitset, which GETPIVOTDATA reads.
        let names_cache_item = kind == ItemType::Data;
        if let (true, Some(x)) = (names_cache_item, index) {
            if flag(element, "h", false)? {
                field.hide(x);
            }
        }
        field.items.push(item);
        Ok(names_cache_item)
    }

    /// The typed element that encloses what is at `path`: the innermost one, read last.
    fn owner(&self, path: &[String]) -> Result<OpaqueOwner, Skip> {
        let last = |len: usize| u32::try_from(len.saturating_sub(1)).unwrap_or(u32::MAX);
        let fields = &self.table.fields;
        let path: Vec<&str> = path.iter().map(String::as_str).collect();
        Ok(match path.as_slice() {
            [TABLE] => OpaqueOwner::Definition,
            [TABLE, "location"] => OpaqueOwner::Location,
            [TABLE, "pivotFields"] => OpaqueOwner::PivotFields,
            [TABLE, "pivotFields", "pivotField"] => OpaqueOwner::Field(last(fields.len())),
            [TABLE, "pivotFields", "pivotField", "items"] => OpaqueOwner::Items(last(fields.len())),
            [TABLE, "pivotFields", "pivotField", "items", "item"] => OpaqueOwner::Item {
                field: last(fields.len()),
                item: last(fields.last().map_or(0, |f| f.items.len())),
            },
            [TABLE, "rowFields"] => OpaqueOwner::RowFields,
            [TABLE, "rowFields", "field"] => {
                OpaqueOwner::RowField(last(self.table.row_fields.len()))
            }
            [TABLE, "colFields"] => OpaqueOwner::ColumnFields,
            [TABLE, "colFields", "field"] => {
                OpaqueOwner::ColumnField(last(self.table.column_fields.len()))
            }
            [TABLE, "pageFields"] => OpaqueOwner::PageFields,
            [TABLE, "pageFields", "pageField"] => {
                OpaqueOwner::PageField(last(self.table.page_fields.len()))
            }
            [TABLE, "dataFields"] => OpaqueOwner::DataFields,
            [TABLE, "dataFields", "dataField"] => {
                OpaqueOwner::DataField(last(self.table.data_fields.len()))
            }
            _ => return skip("unread content outside the pivot definition"),
        })
    }

    /// Keeps the attributes of `element` that `typed` does not name, verbatim, as one entry.
    fn keep_attributes(
        &mut self,
        owner: OpaqueOwner,
        element: &BytesStart,
        typed: fn(&str) -> bool,
    ) -> Result<(), Skip> {
        let mut xml = String::new();
        for attr in element.attributes() {
            let attr = attr?;
            let (key, value) = (attr.key.0, &*attr.value);
            if typed(key) {
                continue;
            }
            // The value is still escaped; only a `"` inside it (from a `'`-quoted value) needs
            // the other quote.
            let quote = if value.contains('"') { '\'' } else { '"' };
            if xml.len() + key.len() + value.len() + 4 > self.opaque_left.get() {
                return skip("unread pivot definition content past the import limit");
            }
            xml.push(' ');
            xml.push_str(key);
            xml.push('=');
            xml.push(quote);
            xml.push_str(value);
            xml.push(quote);
        }
        if xml.is_empty() {
            return Ok(());
        }
        self.keep(owner, OpaqueNode::Attributes(xml))
    }

    /// Adds an entry to the opaque store, charging its cost to the budget.
    fn keep(&mut self, owner: OpaqueOwner, mut node: OpaqueNode) -> Result<(), Skip> {
        // Held at its length, so the cost charged is the memory kept.
        match &mut node {
            OpaqueNode::Attributes(xml) | OpaqueNode::Element(xml) => xml.shrink_to_fit(),
        }
        let entry = OpaqueEntry { owner, node };
        let Some(left) = self.opaque_left.get().checked_sub(entry.cost()) else {
            return skip("unread pivot definition content past the import limit");
        };
        self.opaque_left.set(left);
        self.table.opaque.push(entry);
        Ok(())
    }
}

/// Calls `read` on each extension element named `name` (`<extLst><ext><x14:name>`) of `xml`, an
/// element kept whole (already bounded by the opaque budget).
fn each_x14_element(
    xml: &str,
    name: &str,
    mut read: impl FnMut(&BytesStart) -> Result<(), Skip>,
) -> Result<(), Skip> {
    walk(xml.as_bytes(), |path, tag| {
        if let Tag::Open(element) = tag {
            if at(path, &["extLst", "ext"]) && local_name(element) == name {
                read(element)?;
            }
        }
        Ok(())
    })
}

/// A `<dataField>` but its `fld`, which the caller checks against the cache.
fn read_data_field(element: &BytesStart, limits: &ImportLimits) -> Result<DataField, Skip> {
    let caption = attribute(element, "name")?.unwrap_or_default();
    limits.check_text(&caption)?;
    let show_data_as = match attribute(element, "showDataAs")? {
        None => ShowDataAs::Normal,
        Some(name) => match show_data_as(&name)? {
            mode if mode.is_x14() => return skip(format!("unknown showDataAs {:?}", shown(&name))),
            mode => mode,
        },
    };
    Ok(DataField {
        caption,
        field: 0,
        function: aggregate(attribute(element, "subtotal")?.as_deref())?,
        show_data_as,
        base_field: number(element, "baseField")?.unwrap_or(-1),
        base_item: number(element, "baseItem")?.unwrap_or(NO_BASE_ITEM),
        number_format: number(element, "numFmtId")?,
    })
}

/// The mode a `showDataAs` or x14 `pivotShowAs` names; an unknown one skips the pivot.
fn show_data_as(name: &str) -> Result<ShowDataAs, Skip> {
    ShowDataAs::from_name(name)
        .map_or_else(|| skip(format!("unknown showDataAs {:?}", shown(name))), Ok)
}

/// An `xsd:boolean` attribute: `default` when absent or not a boolean, as Excel's own writer never
/// leaves one malformed (and `showAll` has always read so, T-4276).
pub(super) fn flag(element: &BytesStart, name: &str, default: bool) -> Result<bool, Skip> {
    Ok(match attribute(element, name)?.as_deref() {
        Some("1" | "true") => true,
        Some("0" | "false") => false,
        _ => default,
    })
}

/// A numeric attribute nothing computes from yet; `None` when absent or not a number, as
/// [`flag`] reads a malformed boolean, so a pivot GETPIVOTDATA answers from is never dropped for
/// it.
fn number<T: std::str::FromStr>(element: &BytesStart, name: &str) -> Result<Option<T>, Skip> {
    Ok(attribute(element, name)?.and_then(|text| text.parse::<T>().ok()))
}

/// An `item@t`; absent is `data`.
fn item_type(t: Option<&str>) -> Result<ItemType, Skip> {
    Ok(match t.unwrap_or("data") {
        "data" => ItemType::Data,
        "default" => ItemType::Default,
        "grand" => ItemType::Grand,
        "blank" => ItemType::Blank,
        other => match subtotal_function(other) {
            Some(function) => ItemType::Function(function),
            None => return skip(format!("unknown item type {:?}", shown(other))),
        },
    })
}

/// A `<location ref>` as a 0-based area, inside the sheet.
fn location(reference: Option<&str>) -> Result<CellArea, Skip> {
    let Some((top, left, bottom, right)) = reference.and_then(|r| parse_range(r).ok()) else {
        return skip("no valid location");
    };
    let valid = is_valid_row(top)
        && is_valid_row(bottom)
        && is_valid_column_number(left)
        && is_valid_column_number(right)
        && top <= bottom
        && left <= right;
    let to_index = |n: i32| u32::try_from(n - 1).ok();
    match (
        valid,
        to_index(top),
        to_index(left),
        to_index(bottom),
        to_index(right),
    ) {
        (true, Some(top), Some(left), Some(bottom), Some(right)) => Ok(CellArea {
            top,
            left,
            bottom,
            right,
        }),
        _ => skip("a location outside the sheet"),
    }
}

/// The offsets and page counts of a `<location>`; a missing or malformed one reads as 0, as
/// [`number`] reads every count, so a pivot is never dropped for it (the layout stage recomputes
/// them, `pivot::layout::lines`).
fn geometry(element: &BytesStart) -> Result<PivotGeometry, Skip> {
    let count = |name: &str| -> Result<u32, Skip> { Ok(number(element, name)?.unwrap_or(0)) };
    Ok(PivotGeometry {
        first_header_row: count("firstHeaderRow")?,
        first_data_row: count("firstDataRow")?,
        first_data_column: count("firstDataCol")?,
        row_page_count: count("rowPageCount")?,
        column_page_count: count("colPageCount")?,
    })
}

/// A `dataField@subtotal`; absent is `sum`.
fn aggregate(subtotal: Option<&str>) -> Result<Aggregate, Skip> {
    let name = subtotal.unwrap_or("sum");
    Aggregate::from_consolidate_name(name)
        .map_or_else(|| skip(format!("unknown subtotal {:?}", shown(name))), Ok)
}

#[cfg(test)]
mod tests;
