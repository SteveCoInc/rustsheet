//! The `pivotTableDefinition` part (SPEC-0015 "xlsx import", SPEC-0032 "Model (T-4139)").
//!
//! The attributes the model types are read into [`PivotTable`]; every other attribute and element
//! is kept verbatim in [`PivotTable::opaque`], owned by the typed element it was read in, for
//! export (T-4176). The collections' `count` attributes are the one exception: they follow from
//! the lists. What is kept is charged, entry by entry ([`OpaqueEntry::cost`]), to the workbook's
//! opaque budget, so the store never grows past it (web.md "Pivot caches past the import limits
//! are dropped").

use std::cell::Cell;
use std::collections::HashMap;
use std::io::BufRead;

use quick_xml::events::BytesStart;
use rustsheet_engine::expressions::utils::{is_valid_column_number, is_valid_row};
use rustsheet_engine::pivot::{
    Aggregate, Axis, AxisField, CellArea, DataField, ItemType, OpaqueEntry, OpaqueNode,
    OpaqueOwner, PageField, PivotCache, PivotField, PivotItem, PivotOptions, PivotTable,
    ShowDataAs, SortType, NO_BASE_ITEM,
};

use super::super::package::ImportLimits;
use super::super::worksheets::parse_range;
use super::{
    attribute, index_attribute, local_name, skip, walk_keeping, Skip, Tag, MAX_AXIS_FIELDS,
    MAX_DATA_FIELDS,
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
const ROOT_TYPED: [&str; 20] = [
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
];

/// The custom subtotal attributes of a `<pivotField>`, in the schema's order, and the subtotal
/// line (`item@t`) each asks for.
const CUSTOM_SUBTOTALS: [(&str, ItemType); 11] = [
    ("sumSubtotal", ItemType::Sum),
    ("countASubtotal", ItemType::CountA),
    ("avgSubtotal", ItemType::Average),
    ("maxSubtotal", ItemType::Max),
    ("minSubtotal", ItemType::Min),
    ("productSubtotal", ItemType::Product),
    ("countSubtotal", ItemType::Count),
    ("stdDevSubtotal", ItemType::StdDev),
    ("stdDevPSubtotal", ItemType::StdDevP),
    ("varSubtotal", ItemType::Var),
    ("varPSubtotal", ItemType::VarP),
];

/// The `<pivotField>` attributes the model types, besides [`CUSTOM_SUBTOTALS`].
const FIELD_TYPED: [&str; 9] = [
    "axis",
    "showAll",
    "compact",
    "outline",
    "subtotalTop",
    "defaultSubtotal",
    "sortType",
    "insertBlankRow",
    "numFmtId",
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
                self.located = true;
                |key| key == "ref"
            }
            "pivotField" => {
                self.read_field(cache, element)?;
                |key| FIELD_TYPED.contains(&key) || CUSTOM_SUBTOTALS.iter().any(|(n, _)| *n == key)
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
                // "Show values as" (% of total, running total, ...) changes every value, which
                // GETPIVOTDATA does not compute yet (T-4161, T-4163): such a pivot is skipped.
                if data.show_data_as != ShowDataAs::Normal {
                    return skip("a data field shown as a calculation");
                }
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
            Some(other) => return skip(format!("unknown axis {other:?}")),
        };
        let mut field = PivotField::new(axis, cache_field.items.len());
        // ECMA-376's default is on; Excel writes `showAll="0"` (web.md, T-4276).
        field.show_all = flag(element, "showAll", field.show_all)?;
        field.compact = flag(element, "compact", field.compact)?;
        field.outline = flag(element, "outline", field.outline)?;
        field.subtotal_top = flag(element, "subtotalTop", field.subtotal_top)?;
        field.default_subtotal = flag(element, "defaultSubtotal", field.default_subtotal)?;
        for (name, line) in CUSTOM_SUBTOTALS {
            if flag(element, name, false)? {
                field.custom_subtotals.push(line);
            }
        }
        field.sort_type = match attribute(element, "sortType")?.as_deref() {
            None | Some("manual") => SortType::Manual,
            Some("ascending") => SortType::Ascending,
            Some("descending") => SortType::Descending,
            Some(other) => return skip(format!("unknown sortType {other:?}")),
        };
        field.insert_blank_row = flag(element, "insertBlankRow", field.insert_blank_row)?;
        field.number_format = number(element, "numFmtId")?;
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
    fn keep(&mut self, owner: OpaqueOwner, node: OpaqueNode) -> Result<(), Skip> {
        let entry = OpaqueEntry { owner, node };
        let Some(left) = self.opaque_left.get().checked_sub(entry.cost()) else {
            return skip("unread pivot definition content past the import limit");
        };
        self.opaque_left.set(left);
        self.table.opaque.push(entry);
        Ok(())
    }
}

/// A `<dataField>` but its `fld`, which the caller checks against the cache.
fn read_data_field(element: &BytesStart, limits: &ImportLimits) -> Result<DataField, Skip> {
    let caption = attribute(element, "name")?.unwrap_or_default();
    limits.check_text(&caption)?;
    let show_data_as = match attribute(element, "showDataAs")?.as_deref() {
        None | Some("normal") => ShowDataAs::Normal,
        Some("difference") => ShowDataAs::Difference,
        Some("percent") => ShowDataAs::Percent,
        Some("percentDiff") => ShowDataAs::PercentDiff,
        Some("runTotal") => ShowDataAs::RunTotal,
        Some("percentOfRow") => ShowDataAs::PercentOfRow,
        Some("percentOfCol") => ShowDataAs::PercentOfColumn,
        Some("percentOfTotal") => ShowDataAs::PercentOfTotal,
        Some("index") => ShowDataAs::Index,
        Some(other) => return skip(format!("unknown showDataAs {other:?}")),
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

/// An `xsd:boolean` attribute: `default` when absent or not a boolean, as Excel's own writer never
/// leaves one malformed (and `showAll` has always read so, T-4276).
fn flag(element: &BytesStart, name: &str, default: bool) -> Result<bool, Skip> {
    Ok(match attribute(element, name)?.as_deref() {
        Some("1" | "true") => true,
        Some("0" | "false") => false,
        _ => default,
    })
}

/// A numeric attribute; `None` when absent, and a value that does not parse skips the pivot.
fn number<T: std::str::FromStr>(element: &BytesStart, name: &str) -> Result<Option<T>, Skip> {
    match attribute(element, name)? {
        None => Ok(None),
        Some(text) => match text.parse::<T>() {
            Ok(value) => Ok(Some(value)),
            Err(_) => skip(format!("bad {name} {text:?}")),
        },
    }
}

/// An `item@t`; absent is `data`.
fn item_type(t: Option<&str>) -> Result<ItemType, Skip> {
    Ok(match t.unwrap_or("data") {
        "data" => ItemType::Data,
        "default" => ItemType::Default,
        "sum" => ItemType::Sum,
        "countA" => ItemType::CountA,
        "avg" => ItemType::Average,
        "max" => ItemType::Max,
        "min" => ItemType::Min,
        "product" => ItemType::Product,
        "count" => ItemType::Count,
        "stdDev" => ItemType::StdDev,
        "stdDevP" => ItemType::StdDevP,
        "var" => ItemType::Var,
        "varP" => ItemType::VarP,
        "grand" => ItemType::Grand,
        "blank" => ItemType::Blank,
        other => return skip(format!("unknown item type {other:?}")),
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

/// A `dataField@subtotal`; absent is `sum`.
fn aggregate(subtotal: Option<&str>) -> Result<Aggregate, Skip> {
    Ok(match subtotal.unwrap_or("sum") {
        "sum" => Aggregate::Sum,
        "count" => Aggregate::Count,
        "average" => Aggregate::Average,
        "max" => Aggregate::Max,
        "min" => Aggregate::Min,
        "product" => Aggregate::Product,
        "countNums" => Aggregate::CountNums,
        "stdDev" => Aggregate::StdDev,
        "stdDevp" => Aggregate::StdDevP,
        "var" => Aggregate::Var,
        "varp" => Aggregate::VarP,
        other => return skip(format!("unknown subtotal {other:?}")),
    })
}

#[cfg(test)]
mod tests;
