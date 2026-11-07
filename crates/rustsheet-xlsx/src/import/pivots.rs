//! Pivot caches and pivot tables (SPEC-0015 "xlsx import", T-2082).
//!
//! Every part is streamed with a pull parser, never built into a tree: a records part can hold
//! millions of values, and [`CacheBuilder`] counts the SPEC-0015 limits before each push, so
//! nothing past them is allocated (CLAUDE.md directive 5). Anything the model cannot hold
//! faithfully (an OLAP or external source, a cache saved without records, a bad index, a limit)
//! skips that cache, with every pivot on it, or that pivot: it is logged and dropped whole, never
//! truncated or guessed, so GETPIVOTDATA on it is `#REF!` and the rest of the workbook loads as
//! usual (docs/excel-semantics/web.md "GETPIVOTDATA against a pivot table"). A grouped field
//! imports as its group items ([`groups`], T-4225). A calculated field (`cacheField@formula`,
//! T-4174) imports as a field with its formula and no records, after the other fields; a cache
//! whose formula Excel for the web refuses, or the engine cannot compute
//! (`check_calculated_fields`), is skipped.

mod filters;
mod groups;
mod sort;
mod table;

use std::cell::Cell;
use std::collections::HashMap;
use std::io::{BufRead, Read, Seek};

use quick_xml::events::{BytesStart, Event};
use rustsheet_engine::expressions::token::get_error_by_english_name;
use rustsheet_engine::pivot::{
    check_calculated_fields, CacheBuilder, CacheField, CacheItem, CacheSource,
    CalculatedFieldError, DateGroupBy, DateGroupError, NumberGroupError, PivotCache,
    PivotCacheError, PivotTable, RecordValue, MAX_CALCULATED_FIELDS, MAX_FIELDS, MAX_ITEMS,
    MAX_RECORD_VALUES,
};
use rustsheet_engine::types::DateSystem;

use self::groups::{apply_groups, FieldGroup, RangeGroup};
pub(crate) use self::table::{data_field_x14_show_as, SUBTOTAL_FUNCTIONS};
use self::table::{flag, read_pivot_table};
use super::iso_date::pivot_cache_date_serial;
use super::package::{
    part_folder_and_rels, part_key, resolve_relative_target, ImportLimits, Package,
};
use super::xml::{find_start_attribute, start_attribute};
use super::{part_relationships, Relationship};
use crate::error::XlsxError;

/// Most pivot fields on one axis of a pivot (SPEC-0015 "Untrusted-input limits").
const MAX_AXIS_FIELDS: usize = 256;
/// Most data fields of one pivot.
const MAX_DATA_FIELDS: usize = 256;
/// Most pivots on one sheet; the ones past it are dropped.
const MAX_PIVOTS_PER_SHEET: usize = 64;
/// Most bytes the pivots' verbatim stores hold over a workbook (SPEC-0032 "Model"), each entry
/// charged its text and its own size (`OpaqueEntry::cost`); never more than the package may
/// inflate. A pivot whose store would pass what is left is dropped (web.md "Pivot caches past
/// the import limits are dropped"). Excel-saved definitions keep a few kilobytes, mostly
/// `rowItems`, about 30 bytes per rendered line.
const MAX_OPAQUE_BYTES: usize = 32 << 20;
/// Most pivots in a workbook; the ones past it are dropped. Every accepted pivot allocates a
/// hidden-items bitset per field sized to its cache, so the per-sheet cap alone leaves the total
/// to the file (T-2394, web.md "Pivot caches past the import limits are dropped").
pub const MAX_PIVOTS: usize = 1_000;
/// Deepest element nesting read in a pivot part: well-formed ones nest about six deep, and the
/// open-element stack must not grow with the file.
const MAX_DEPTH: usize = 32;

/// Why a cache or a pivot was skipped. It is logged, never an import error.
struct Skip(String);

impl From<XlsxError> for Skip {
    fn from(error: XlsxError) -> Self {
        Skip(error.to_string())
    }
}

impl From<DateGroupError> for Skip {
    fn from(error: DateGroupError) -> Self {
        Skip(error.to_string())
    }
}

impl From<NumberGroupError> for Skip {
    fn from(error: NumberGroupError) -> Self {
        Skip(error.to_string())
    }
}

impl From<CalculatedFieldError> for Skip {
    fn from(error: CalculatedFieldError) -> Self {
        Skip(error.to_string())
    }
}

impl From<PivotCacheError> for Skip {
    fn from(error: PivotCacheError) -> Self {
        Skip(error.to_string())
    }
}

impl From<quick_xml::events::attributes::AttrError> for Skip {
    fn from(error: quick_xml::events::attributes::AttrError) -> Self {
        Skip(error.to_string())
    }
}

fn skip<T>(reason: impl Into<String>) -> Result<T, Skip> {
    Err(Skip(reason.into()))
}

/// A value from the file as a skip reason quotes it: its first 64 characters, so a reason never
/// copies an attacker's long value.
fn shown(value: &str) -> String {
    const MAX_CHARS: usize = 64;
    value.chars().take(MAX_CHARS).collect()
}

/// A worksheet pivots may sit on.
pub(super) struct PivotSheet {
    /// The sheet's stable `sheetId`.
    pub(super) sheet_id: u32,
    /// The worksheet part (`xl/worksheets/sheet1.xml`).
    pub(super) path: String,
}

/// The pivot caches listed in the workbook (`caches`: each `<pivotCache>`'s `cacheId` and
/// `r:id`, resolved through the workbook relationships `rels`) and the pivot tables of `sheets`.
pub(super) fn load_pivots<R: Read + Seek>(
    archive: &mut Package<R>,
    rels: &HashMap<String, Relationship>,
    caches: &[(Option<String>, Option<String>)],
    sheets: &[PivotSheet],
    date_system: DateSystem,
) -> (Vec<PivotCache>, Vec<PivotTable>) {
    let (pivot_caches, cache_ids) =
        load_caches(archive, rels, caches, MAX_RECORD_VALUES, date_system);
    if pivot_caches.is_empty() {
        return (pivot_caches, Vec::new());
    }
    let mut parts = Vec::new();
    for sheet in sheets {
        match pivot_table_parts(archive, &sheet.path) {
            Ok(found) => {
                // One log line per sheet for the pivots dropped, not one per pivot: a file can
                // list millions.
                let mut dropped = 0_usize;
                for (n, part) in found.into_iter().enumerate() {
                    if n < MAX_PIVOTS_PER_SHEET && parts.len() < MAX_PIVOTS {
                        parts.push((sheet.sheet_id, part));
                    } else {
                        dropped += 1;
                    }
                }
                if dropped > 0 {
                    log_skip(
                        "pivot tables of",
                        &sheet.path,
                        &Skip(format!(
                            "{dropped} dropped past the limits ({MAX_PIVOTS_PER_SHEET} on a sheet, {MAX_PIVOTS} in a workbook)"
                        )),
                    );
                }
            }
            Err(reason) => log_skip("pivot tables of", &sheet.path, &reason),
        }
    }
    // Part names match case-insensitively with `\` as `/` (`part_key`), so count on that key.
    let keys: Vec<String> = parts.iter().map(|(_, part)| part_key(part)).collect();
    let reached = occurrences(keys.iter());
    let limits = *archive.limits();
    let mut opaque_left = usize::try_from(limits.max_uncompressed_bytes)
        .unwrap_or(usize::MAX)
        .min(MAX_OPAQUE_BYTES);
    let mut pivot_tables = Vec::new();
    for ((sheet_id, part), key) in parts.iter().zip(&keys) {
        let left = Cell::new(opaque_left);
        let result = if reached.get(key).is_some_and(|&n| n > 1) {
            skip("the part is reached twice")
        } else {
            archive.open(part).map_err(Skip::from).and_then(|reader| {
                read_pivot_table(reader, *sheet_id, &pivot_caches, &cache_ids, &limits, &left)
            })
        };
        match result {
            Ok(pivot) => {
                opaque_left = left.get();
                pivot_tables.push(pivot);
            }
            Err(reason) => log_skip("pivot table", part, &reason),
        }
    }
    (pivot_caches, pivot_tables)
}

/// How many times each value occurs (a file can list millions, so never a pairwise scan).
fn occurrences<'a, T: Eq + std::hash::Hash + 'a>(
    values: impl Iterator<Item = &'a T>,
) -> HashMap<&'a T, usize> {
    let mut counts = HashMap::new();
    for value in values {
        *counts.entry(value).or_insert(0) += 1;
    }
    counts
}

/// Logs a skipped cache or pivot to stderr, as the theme importer does: the importer has no
/// warnings channel. Part names and reasons come from the file, so they are quoted (no raw
/// newlines or escapes) and cut short.
fn log_skip(what: &str, part: &str, reason: &Skip) {
    const MAX_CHARS: usize = 200;
    let part: String = part.chars().take(MAX_CHARS).collect();
    let reason: String = reason.0.chars().take(MAX_CHARS).collect();
    eprintln!("RustSheet: skipped {what} {part:?}: {reason:?}");
}

/// The caches that load, in workbook order, and each one's index by `cacheId`. A `cacheId`
/// listed twice and a definition part reached twice skip every cache involved. `values_left` is
/// the workbook's record-value budget ([`MAX_RECORD_VALUES`]; smaller in tests).
fn load_caches<R: Read + Seek>(
    archive: &mut Package<R>,
    rels: &HashMap<String, Relationship>,
    caches: &[(Option<String>, Option<String>)],
    mut values_left: usize,
    date_system: DateSystem,
) -> (Vec<PivotCache>, HashMap<u32, u32>) {
    let listed: Vec<(Option<u32>, Option<String>)> = caches
        .iter()
        .map(|(id, rel_id)| {
            let id = id.as_deref().and_then(|id| id.parse::<u32>().ok());
            let part = rel_id
                .as_deref()
                .and_then(|rel_id| rels.get(rel_id))
                .filter(|rel| !rel.external && rel.rel_type.ends_with("/pivotCacheDefinition"))
                .and_then(|rel| resolve_relative_target("xl", &rel.target));
            (id, part)
        })
        .collect();
    let id_counts = occurrences(listed.iter().filter_map(|(id, _)| id.as_ref()));
    let part_keys: Vec<Option<String>> = listed
        .iter()
        .map(|(_, part)| part.as_deref().map(part_key))
        .collect();
    let part_counts = occurrences(part_keys.iter().flatten());
    let mut pivot_caches = Vec::new();
    let mut ids = HashMap::new();
    for ((id, part), key) in listed.iter().zip(&part_keys) {
        let name = part.as_deref().unwrap_or("(no part)");
        let result = match (id, part) {
            (None, _) => skip("no valid cacheId"),
            (_, None) => skip("no internal pivotCacheDefinition relationship"),
            (Some(id), _) if id_counts.get(id).is_some_and(|&n| n > 1) => {
                skip(format!("cacheId {id} is listed twice"))
            }
            (_, Some(_))
                if key
                    .as_ref()
                    .is_some_and(|key| part_counts.get(key).is_some_and(|&n| n > 1)) =>
            {
                skip("the part is reached twice")
            }
            (Some(id), Some(part)) => {
                load_cache(archive, part, values_left, date_system).map(|c| (*id, c))
            }
        };
        match result {
            Ok((id, cache)) => {
                values_left =
                    values_left.saturating_sub(cache.record_count as usize * cache.fields.len());
                // At most one cache per listed `<pivotCache>`, bounded by the byte budget.
                ids.insert(id, pivot_caches.len() as u32);
                pivot_caches.push(cache);
            }
            Err(reason) => log_skip("pivot cache", name, &reason),
        }
    }
    (pivot_caches, ids)
}

/// One cache: its definition part, then the records part the definition's `r:id` points to.
fn load_cache<R: Read + Seek>(
    archive: &mut Package<R>,
    part: &str,
    values_left: usize,
    date_system: DateSystem,
) -> Result<PivotCache, Skip> {
    let limits = *archive.limits();
    let definition = read_definition(archive.open(part)?, values_left, &limits, date_system)?;
    let Some(records_id) = definition.records_id.as_deref() else {
        return skip("no records part (saveData=\"0\")");
    };
    let (folder, rels_path) = part_folder_and_rels(part);
    let records = part_relationships(archive, &rels_path)?
        .into_iter()
        .find(|(id, _)| id == records_id)
        .map(|(_, rel)| rel)
        .filter(|rel| !rel.external && rel.rel_type.ends_with("/pivotCacheRecords"))
        .and_then(|rel| resolve_relative_target(folder, &rel.target));
    let Some(records) = records else {
        return skip("no internal pivotCacheRecords relationship");
    };
    let records = archive.open(&records)?;
    finish_cache(definition, records, values_left, &limits, date_system)
}

/// The cache of `definition`, its records read from the `records` part.
fn finish_cache<B: BufRead>(
    definition: Definition,
    records: B,
    values_left: usize,
    limits: &ImportLimits,
    date_system: DateSystem,
) -> Result<PivotCache, Skip> {
    let Definition {
        mut builder,
        field_count,
        records_id: _,
        groups,
        grouping_fields,
        calculated,
        refresh_on_load,
        source_name,
    } = definition;
    read_records(records, &mut builder, field_count, limits, date_system)?;
    let mut cache = builder.finish();
    cache.refresh_on_load = refresh_on_load;
    cache.source_name = source_name;
    apply_groups(&mut cache, &groups, grouping_fields, values_left)?;
    // Excel for the web refuses a file whose calculated field calls NOW, TODAY, RAND or
    // INDIRECT, holds an array constant or a cell reference, or names no field (pivots.md
    // "Pivot flags Excel for the web refuses", T-4505); the engine refuses a few more shapes.
    cache.fields.extend(calculated);
    check_calculated_fields(&cache.fields)?;
    Ok(cache)
}

/// One pivot cache and its pivot tables as package parts, without the package: what a case
/// builder writes for them (`tools/src/rustsheet_tools/pivot_definition.py`). Harness support,
/// not public API.
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct PivotParts<'a> {
    /// The `pivotCacheDefinition` part; its records part is `cache_records`, whatever its `r:id`.
    pub cache_definition: &'a [u8],
    /// The `pivotCacheRecords` part.
    pub cache_records: &'a [u8],
    /// The `cacheId` the tables name the cache by.
    pub cache_id: u32,
}

/// Reads `parts` as the importer reads a package's: the cache, then each `pivotTableDefinition`
/// of `tables` (the `sheetId` it sits on, its part) against it, numbered as cache `cache_index`
/// of the workbook. `values_left` is the workbook's remaining record-value budget
/// ([`MAX_RECORD_VALUES`]). Harness support for spec cases (T-4152): their pivots reach the
/// engine through the importer, the one reader of a pivot definition. The parts are held in
/// memory already, so the package's inflation cap is applied to their total size instead: parts
/// past [`ImportLimits::max_uncompressed_bytes`] are refused unread.
///
/// # Errors
/// [`XlsxError::Limit`] for parts past the size cap, [`XlsxError::Workbook`] with the reason
/// the importer would skip the cache or a pivot for.
#[doc(hidden)]
pub fn read_pivot_parts(
    parts: PivotParts<'_>,
    tables: &[(u32, &[u8])],
    cache_index: u32,
    values_left: usize,
    date_system: DateSystem,
) -> Result<(PivotCache, Vec<PivotTable>), XlsxError> {
    let limits = ImportLimits::default();
    let size = (tables.iter().map(|(_, part)| part.len()))
        .chain([parts.cache_definition.len(), parts.cache_records.len()])
        .fold(0_u64, |total, len| total.saturating_add(len as u64));
    if size > limits.max_uncompressed_bytes {
        return Err(XlsxError::Limit {
            limit: crate::error::ImportLimit::UncompressedBytes,
            max: limits.max_uncompressed_bytes,
        });
    }
    let values_left = values_left.min(MAX_RECORD_VALUES);
    let cache = read_definition(parts.cache_definition, values_left, &limits, date_system)
        .and_then(|definition| {
            if definition.records_id.is_none() {
                return skip("no records part (saveData=\"0\")");
            }
            finish_cache(
                definition,
                parts.cache_records,
                values_left,
                &limits,
                date_system,
            )
        })
        .map_err(|Skip(reason)| XlsxError::Workbook(reason))?;
    let caches = [cache];
    let ids = HashMap::from([(parts.cache_id, 0)]);
    let left = Cell::new(
        usize::try_from(limits.max_uncompressed_bytes)
            .unwrap_or(usize::MAX)
            .min(MAX_OPAQUE_BYTES),
    );
    let mut pivots = Vec::with_capacity(tables.len());
    for &(sheet_id, part) in tables.iter().take(MAX_PIVOTS_PER_SHEET) {
        let mut pivot = read_pivot_table(part, sheet_id, &caches, &ids, &limits, &left)
            .map_err(|Skip(reason)| XlsxError::Workbook(reason))?;
        pivot.cache = cache_index;
        pivots.push(pivot);
    }
    let [cache] = caches;
    Ok((cache, pivots))
}

/// The pivot table parts of the worksheet part `sheet`, in relationship order.
fn pivot_table_parts<R: Read + Seek>(
    archive: &mut Package<R>,
    sheet: &str,
) -> Result<Vec<String>, Skip> {
    let (folder, rels_path) = part_folder_and_rels(sheet);
    let mut parts = Vec::new();
    for (_, rel) in part_relationships(archive, &rels_path)? {
        if !rel.rel_type.ends_with("/pivotTable") {
            continue;
        }
        match resolve_relative_target(folder, &rel.target).filter(|_| !rel.external) {
            Some(part) => parts.push(part),
            None => log_skip(
                "pivot table",
                &rel.target,
                &Skip("not inside the package".into()),
            ),
        }
    }
    Ok(parts)
}

/// An element start, the end of the element named, or an element read whole, as [`walk`] and
/// [`walk_keeping`] report them.
enum Tag<'a> {
    Open(&'a BytesStart<'a>),
    Close(&'a str),
    /// An element [`walk_keeping`]'s `keep` picked: its XML as written, everything inside it
    /// included.
    Kept(String),
}

/// Streams the elements of a part to `visit`, with the local names of the enclosing elements,
/// outermost first (an empty element is an `Open` then a `Close`).
fn walk<B: BufRead>(
    part: B,
    visit: impl FnMut(&[String], Tag<'_>) -> Result<(), Skip>,
) -> Result<(), Skip> {
    walk_keeping(part, |_, _| false, &Cell::new(0), visit)
}

/// [`walk`], except that an element `keep` picks is not opened: it is read to its end and
/// reported once as [`Tag::Kept`]. A kept element longer than `max_kept` bytes skips the part, so
/// nothing past it is held (the caller charges what it keeps against that budget).
fn walk_keeping<B: BufRead>(
    part: B,
    keep: impl Fn(&[String], &BytesStart) -> bool,
    max_kept: &Cell<usize>,
    mut visit: impl FnMut(&[String], Tag<'_>) -> Result<(), Skip>,
) -> Result<(), Skip> {
    let mut reader = quick_xml::Reader::from_reader(part);
    let mut buffer = Vec::new();
    let mut path: Vec<String> = Vec::new();
    // The element being kept: its XML so far, and how many of its elements are open.
    let mut kept: Option<(String, usize)> = None;
    loop {
        let event = reader
            .read_event_into(&mut buffer)
            .map_err(XlsxError::from)?;
        if let Some((xml, open)) = kept.as_mut() {
            match &event {
                Event::Start(_) => {
                    *open += 1;
                    if path.len() + *open >= MAX_DEPTH {
                        return skip("elements nest too deep");
                    }
                }
                Event::End(_) => *open -= 1,
                Event::Eof => return skip("an unclosed element"),
                _ => {}
            }
            write_verbatim(xml, &event, max_kept.get())?;
            if *open == 0 {
                let xml = kept.take().map(|(xml, _)| xml).unwrap_or_default();
                visit(&path, Tag::Kept(xml))?;
            }
            buffer.clear();
            continue;
        }
        match event {
            Event::Start(element) if keep(&path, &element) => {
                if path.len() + 1 >= MAX_DEPTH {
                    return skip("elements nest too deep");
                }
                let mut xml = String::new();
                write_verbatim(&mut xml, &Event::Start(element), max_kept.get())?;
                kept = Some((xml, 1));
            }
            Event::Empty(element) if keep(&path, &element) => {
                let mut xml = String::new();
                write_verbatim(&mut xml, &Event::Empty(element), max_kept.get())?;
                visit(&path, Tag::Kept(xml))?;
            }
            Event::Start(element) => {
                visit(&path, Tag::Open(&element))?;
                if path.len() >= MAX_DEPTH {
                    return skip("elements nest too deep");
                }
                path.push(element.local_name().as_ref().to_string());
            }
            Event::Empty(element) => {
                visit(&path, Tag::Open(&element))?;
                visit(&path, Tag::Close(element.local_name().as_ref()))?;
            }
            Event::End(element) => {
                path.pop();
                visit(&path, Tag::Close(element.local_name().as_ref()))?;
            }
            Event::Eof => return Ok(()),
            _ => {}
        }
        buffer.clear();
    }
}

/// Appends `event` to `xml` as the part wrote it (quick-xml keeps names, attributes and text
/// still escaped), refusing to pass `max` bytes. A declaration or a doctype cannot occur inside
/// an element and is dropped.
fn write_verbatim(xml: &mut String, event: &Event, max: usize) -> Result<(), Skip> {
    let (open, body, close): (&str, &str, &str) = match event {
        Event::Start(e) => ("<", e, ">"),
        Event::Empty(e) => ("<", e, "/>"),
        Event::End(e) => ("</", e, ">"),
        Event::Text(t) => ("", t, ""),
        Event::CData(t) => ("<![CDATA[", t, "]]>"),
        Event::Comment(t) => ("<!--", t, "-->"),
        Event::PI(t) => ("<?", t, "?>"),
        Event::GeneralRef(t) => ("&", t, ";"),
        Event::Decl(_) | Event::DocType(_) | Event::Eof => return Ok(()),
    };
    let len = open.len() + body.len() + close.len();
    if xml.len().saturating_add(len) > max {
        return skip("unread pivot definition content past the import limit");
    }
    xml.push_str(open);
    xml.push_str(body);
    xml.push_str(close);
    Ok(())
}

/// Whether the enclosing elements are exactly `names`.
fn at(path: &[String], names: &[&str]) -> bool {
    path.len() == names.len() && path.iter().zip(names).all(|(p, n)| p == n)
}

fn local_name(element: &BytesStart) -> String {
    element.local_name().as_ref().to_string()
}

/// The attribute `name`.
fn attribute(element: &BytesStart, name: &str) -> Result<Option<String>, Skip> {
    Ok(start_attribute(element, name)?)
}

/// The relationship id: an `id` attribute with a prefix (`r:id`).
fn relationship_id(element: &BytesStart) -> Result<Option<String>, Skip> {
    Ok(find_start_attribute(element, |name| {
        name.prefix().is_some() && name.local_name().as_ref() == "id"
    })?)
}

fn index_attribute(element: &BytesStart, name: &str) -> Result<u32, Skip> {
    match attribute(element, name)?.map(|v| v.parse::<u32>()) {
        Some(Ok(index)) => Ok(index),
        _ => skip(format!("no valid {name} index")),
    }
}

/// The cache item an `<s>`, `<n>`, `<b>`, `<e>`, `<d>` or `<m>` element holds; `None` for any
/// other element.
fn cache_item(
    element: &BytesStart,
    limits: &ImportLimits,
    date_system: DateSystem,
) -> Result<Option<CacheItem>, Skip> {
    let tag = local_name(element);
    if tag == "m" {
        return Ok(Some(CacheItem::Missing));
    }
    if !matches!(tag.as_str(), "s" | "n" | "b" | "e" | "d") {
        return Ok(None);
    }
    let Some(v) = attribute(element, "v")? else {
        return skip("a cache value without v");
    };
    let item = match tag.as_str() {
        "s" => {
            limits.check_text(&v)?;
            CacheItem::Text(v)
        }
        "n" => match v.trim().parse::<f64>() {
            Ok(n) if n.is_finite() => CacheItem::Number(n),
            _ => return skip(format!("bad number {:?}", shown(&v))),
        },
        "b" => match v.as_str() {
            "1" | "true" => CacheItem::Boolean(true),
            "0" | "false" => CacheItem::Boolean(false),
            _ => return skip(format!("bad boolean {:?}", shown(&v))),
        },
        "e" => match get_error_by_english_name(&v) {
            Some(error) => CacheItem::Error(error),
            None => return skip(format!("unknown error {:?}", shown(&v))),
        },
        // `<d>` is an ISO 8601 text; the cell serial it stands for depends on the workbook's date
        // system (1904 serials are 1462 days lower), and below serial 61 Excel writes it on the
        // 1899-12-30 epoch (T-3182).
        _ => match pivot_cache_date_serial(&v, date_system) {
            Some(serial) => CacheItem::Date(serial),
            None => return skip(format!("bad date {:?}", shown(&v))),
        },
    };
    Ok(Some(item))
}

const DEFINITION: &str = "pivotCacheDefinition";
const FIELD: [&str; 3] = [DEFINITION, "cacheFields", "cacheField"];
const GROUP: [&str; 4] = [DEFINITION, "cacheFields", "cacheField", "fieldGroup"];

/// What a `pivotCacheDefinition` part holds.
struct Definition {
    /// A builder holding the database fields and their shared items.
    builder: CacheBuilder,
    /// How many database fields: the values of each record.
    field_count: u32,
    /// The `r:id` of the records part.
    records_id: Option<String>,
    /// The `<fieldGroup>`s.
    groups: Vec<FieldGroup>,
    /// The names of the grouping fields (`databaseField="0"`), which follow the database fields.
    grouping_fields: Vec<String>,
    /// The calculated fields (`formula`, `databaseField="0"`), which follow every other field.
    calculated: Vec<CacheField>,
    /// `refreshOnLoad` (default off).
    refresh_on_load: bool,
    /// `worksheetSource@name`, kept for export (`PivotCache::source_name`).
    source_name: Option<String>,
}

/// A `pivotCacheDefinition` part.
fn read_definition<B: BufRead>(
    part: B,
    values_left: usize,
    limits: &ImportLimits,
    date_system: DateSystem,
) -> Result<Definition, Skip> {
    let mut records_id = None;
    let mut refresh_on_load = false;
    let mut source_name = None;
    let mut source: Option<Option<CacheSource>> = None;
    let mut builder: Option<CacheBuilder> = None;
    let mut field_count = 0;
    let mut groups: Vec<FieldGroup> = Vec::new();
    let mut grouping_fields: Vec<String> = Vec::new();
    let mut calculated: Vec<CacheField> = Vec::new();
    // Group items and discrete indices over all groups, counted against MAX_ITEMS as they are read.
    let mut group_values = 0_usize;
    walk(part, |path, tag| {
        let Tag::Open(element) = tag else {
            return Ok(());
        };
        let name = local_name(element);
        if path.is_empty() {
            if name != DEFINITION {
                return skip("not a pivotCacheDefinition");
            }
            records_id = relationship_id(element)?;
            refresh_on_load = flag(element, "refreshOnLoad", false)?;
        } else if at(path, &[DEFINITION]) {
            match name.as_str() {
                "cacheSource" => {
                    if attribute(element, "type")?.as_deref() != Some("worksheet") {
                        return skip("not a worksheet source");
                    }
                    source = Some(None);
                }
                "cacheHierarchies" => return skip("an OLAP cache"),
                "cacheFields" => {
                    let Some(source) = source.take() else {
                        return skip("no worksheet cacheSource before cacheFields");
                    };
                    builder = Some(CacheBuilder::new(source, values_left));
                }
                _ => {}
            }
        } else if at(path, &[DEFINITION, "cacheSource"]) && name == "worksheetSource" {
            // A named source (`name`: a defined name or a table) is kept as no source, its name
            // for export.
            if let Some(name) = attribute(element, "name")? {
                limits.check_text(&name)?;
                source_name = Some(name);
            }
            if let (Some(sheet_name), Some(range)) =
                (attribute(element, "sheet")?, attribute(element, "ref")?)
            {
                if source == Some(None) {
                    source = Some(Some(CacheSource { sheet_name, range }));
                }
            }
        } else if at(path, &[DEFINITION, "cacheFields"]) && name == "cacheField" {
            let Some(builder) = builder.as_mut() else {
                return skip("a cacheField outside cacheFields");
            };
            let formula = attribute(element, "formula")?;
            let Some(field_name) = attribute(element, "name")? else {
                return skip("a cacheField without a name");
            };
            limits.check_text(&field_name)?;
            if field_count as usize + grouping_fields.len() + calculated.len() >= MAX_FIELDS {
                return Err(PivotCacheError::TooManyFields.into());
            }
            let database = attribute(element, "databaseField")?.as_deref() != Some("0");
            // A calculated field has no record values; Excel and the case builder write it
            // `databaseField="0"` after the other fields (`pivot_definition.py`). One that is a
            // database field, a field after one, and one with a child other than an empty
            // `<sharedItems/>` are unprobed (UNVERIFIED, T-4897), and skipped.
            if let Some(formula) = formula {
                limits.check_text(&formula)?;
                if database {
                    return skip("a calculated field that is a database field");
                }
                if calculated.len() >= MAX_CALCULATED_FIELDS {
                    return Err(CalculatedFieldError::TooMany.into());
                }
                calculated.push(CacheField {
                    name: field_name,
                    formula: Some(formula),
                    ..CacheField::default()
                });
                return Ok(());
            }
            if !calculated.is_empty() {
                return skip("a cache field after a calculated field");
            }
            // A grouping field (`Quarters`, `Years`) has no record values; Excel writes them
            // after the database fields, and the records part holds only those.
            if !database {
                grouping_fields.push(field_name);
            } else if grouping_fields.is_empty() {
                builder.add_field(field_name)?;
                field_count += 1;
            } else {
                return skip("a database field after a grouping field");
            }
        } else if !calculated.is_empty() && path.len() >= FIELD.len() {
            // Inside a calculated field: every later field is refused above.
            if !(at(path, &FIELD) && name == "sharedItems") {
                return skip("a calculated field with child elements");
            }
        } else if at(path, &FIELD) && name == "fieldGroup" {
            // The field just opened: a grouping field, or a database field grouped in place.
            let field = field_count + grouping_fields.len() as u32 - 1;
            let base = match attribute(element, "base")? {
                Some(base) => match base.parse::<u32>() {
                    Ok(base) => base,
                    Err(_) => return skip("no valid base index"),
                },
                None => field,
            };
            if groups.last().is_some_and(|g| g.field == field) {
                return skip("a field grouped twice");
            }
            groups.push(FieldGroup {
                field,
                base,
                par: attribute(element, "par")?.and_then(|par| par.parse::<u32>().ok()),
                ..FieldGroup::default()
            });
        } else if at(path, &GROUP) {
            let Some(group) = groups.last_mut() else {
                return Ok(());
            };
            match name.as_str() {
                "rangePr" => group.range = Some(read_range_pr(element, date_system)?),
                "discretePr" => group.discrete = Some(Vec::new()),
                _ => {}
            }
        } else if at(path, &[&GROUP[..], &["discretePr"]].concat()) && name == "x" {
            let index = index_attribute(element, "v")?;
            group_values += 1;
            if group_values > MAX_ITEMS {
                return Err(PivotCacheError::TooManyItems.into());
            }
            if let Some(discrete) = groups.last_mut().and_then(|g| g.discrete.as_mut()) {
                discrete.push(index);
            }
        } else if at(path, &[&GROUP[..], &["groupItems"]].concat()) {
            if let Some(item) = cache_item(element, limits, date_system)? {
                group_values += 1;
                if group_values > MAX_ITEMS {
                    return Err(PivotCacheError::TooManyItems.into());
                }
                if let Some(group) = groups.last_mut() {
                    group.items.push(item);
                }
            }
        } else if at(
            path,
            &[DEFINITION, "cacheFields", "cacheField", "sharedItems"],
        ) && grouping_fields.is_empty()
        {
            if let (Some(builder), Some(item)) =
                (builder.as_mut(), cache_item(element, limits, date_system)?)
            {
                builder.add_shared_item(field_count - 1, item)?;
            }
        }
        Ok(())
    })?;
    match builder {
        Some(builder) => Ok(Definition {
            builder,
            field_count,
            records_id,
            groups,
            grouping_fields,
            calculated,
            refresh_on_load,
            source_name,
        }),
        None => skip("no cacheFields"),
    }
}

/// A `<rangePr>`: `startNum`, `endNum` and `groupInterval` for numbers, `startDate` and `endDate`
/// for dates (read as cache `<d>` items are).
fn read_range_pr(element: &BytesStart, date_system: DateSystem) -> Result<RangeGroup, Skip> {
    let by = match attribute(element, "groupBy")?.as_deref() {
        None | Some("range") => None,
        Some(text) => match DateGroupBy::parse(text) {
            Some(by) => Some(by),
            None => return skip("an unknown groupBy"),
        },
    };
    let number = |name: &str| -> Result<Option<f64>, Skip> {
        match attribute(element, name)? {
            None => Ok(None),
            Some(v) => match v.trim().parse::<f64>() {
                Ok(n) if n.is_finite() => Ok(Some(n)),
                _ => skip(format!("bad {name} {:?}", shown(&v))),
            },
        }
    };
    let interval = number("groupInterval")?.unwrap_or(1.0);
    let (start, end) = if by.is_none() {
        if interval <= 0.0 {
            return skip("a groupInterval that is not positive");
        }
        (number("startNum")?, number("endNum")?)
    } else {
        // Unprobed: a 1904 cache's dates, and day groups of more than one day (date ranges).
        if date_system != DateSystem::Excel1900 {
            return skip("a date group in a 1904 workbook");
        }
        if by == Some(DateGroupBy::Days) && interval != 1.0 {
            return skip("a day group of more than one day");
        }
        let date = |name: &str| -> Result<Option<f64>, Skip> {
            match attribute(element, name)? {
                None => Ok(None),
                Some(v) => match pivot_cache_date_serial(&v, date_system) {
                    Some(serial) => Ok(Some(serial)),
                    None => skip(format!("bad {name} {:?}", shown(&v))),
                },
            }
        };
        (date("startDate")?, date("endDate")?)
    };
    match (start, end) {
        (Some(start), Some(end)) if start <= end => Ok(RangeGroup {
            by,
            start,
            end,
            interval,
        }),
        _ => skip("a rangePr without a valid start and end"),
    }
}

/// A `pivotCacheRecords` part, appended record by record to `builder`.
fn read_records<B: BufRead>(
    part: B,
    builder: &mut CacheBuilder,
    field_count: u32,
    limits: &ImportLimits,
    date_system: DateSystem,
) -> Result<(), Skip> {
    let mut record: Vec<RecordValue> = Vec::new();
    walk(part, |path, tag| {
        match tag {
            Tag::Open(element) if path.is_empty() => {
                if local_name(element) != "pivotCacheRecords" {
                    return skip("not a pivotCacheRecords");
                }
            }
            Tag::Open(element) if at(path, &["pivotCacheRecords", "r"]) => {
                // Checked before the push, so a record never grows past its fields.
                if record.len() >= field_count as usize {
                    return Err(PivotCacheError::BadRecord.into());
                }
                let value = if local_name(element) == "x" {
                    RecordValue::Shared(index_attribute(element, "v")?)
                } else if let Some(item) = cache_item(element, limits, date_system)? {
                    RecordValue::Inline(item)
                } else {
                    return skip("an unknown record value");
                };
                record.push(value);
            }
            Tag::Close("r") if at(path, &["pivotCacheRecords"]) => {
                builder.push_record(std::mem::take(&mut record))?;
            }
            _ => {}
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests;
