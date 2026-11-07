//! The patch tier of the source-preserving export (SPEC-0034 D3): a workbook whose structure is
//! unchanged since its load is written as its source, with only the changed `<c>` elements
//! rewritten and new shared strings and style records appended. Every other byte, part and
//! relationship is copied.
//!
//! The patch never guesses. Anything it does not handle exactly is a fallback reason, and the
//! caller regenerates instead:
//! - a structural change, or a style table that changed other than by appending records;
//! - out-of-order rows or cells, a cell without `r`, or a prefixed worksheet;
//! - a changed formula, or a cleared cell, in a shared-formula group;
//! - a rich value or dynamic array that needs value or cell metadata (`vm`, `cm`) in a source that
//!   has its own `metadata.xml`, whose layout is not the export's;
//! - shared strings the model interned from inline cells, which the part does not hold;
//! - a part over the import's size limit, not UTF-8, or a zip with clashing names;
//! - a patched sheet that does not index again (rows or cells out of order).
//!
//! A source without `metadata.xml` gets the export's metadata parts when a patched cell needs them
//! (SPEC-0034 D3.4), and a calcChain is dropped with its relationship and override when the set of
//! formula cells changed (E2).
//!
//! What the patch leaves as the source had it is listed in docs/excel-semantics/xlsx-storage.md
//! ("A patched package keeps the source's hints"): row `spans`, a `<dimension>` that only grows,
//! and a `<row>` emptied by clearing its cells.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Cursor;

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use rustsheet_engine::expressions::utils::{number_to_column, parse_reference_a1};
use rustsheet_engine::types::{Cell, FormulaValue, Worksheet};
use rustsheet_engine::Model;

use super::escape::escape_xml;
use super::rich_errors::{has_rich_errors, metadata_parts};
use super::worksheets::{cell_xml, CellWriter};
use crate::error::XlsxError;
use crate::import::rich_errors::{METADATA_PART, RICH_VALUE_PART, RICH_VALUE_STRUCTURE_PART};
use crate::import::{part_paths, ImportLimits, PartNames, CONTENT_TYPES_PART, WORKBOOK_RELS_PART};
use package::{add_parts, read_entry, remove_part, rewrite_zip, NewPart};
use strings::{append_strings, count_si, shared_refs};

mod package;
mod strings;
mod styles;
mod tag;
#[cfg(test)]
mod tests;

/// Deepest element nesting the patcher reads in a worksheet (directive 5).
const MAX_DEPTH: usize = 256;

/// The styles part, as the importer reads it.
const STYLES_PART: &str = "xl/styles.xml";

/// The calculation chain, which lists every formula cell (SPEC-0034 E2).
const CALC_CHAIN_PART: &str = "xl/calcChain.xml";

/// A patched package and the sheets it patched, or why the source cannot be patched.
pub(super) type Patch = Result<(Vec<u8>, Vec<String>), String>;

/// Patches `source` to hold `model`'s cells. `pristine` is `source` as imported (SPEC-0034 D2).
/// Every failure, an odd zip included, is a fallback reason: the caller regenerates.
///
/// # Errors
/// None today; the outer `Result` keeps room for a failure that must not fall back.
pub(super) fn patch(model: &Model, pristine: &Model, source: &[u8]) -> Result<Patch, XlsxError> {
    Ok(try_patch(model, pristine, source))
}

fn try_patch(model: &Model, pristine: &Model, source: &[u8]) -> Patch {
    let (live, base) = (&model.workbook, &pristine.workbook);
    if let Some(reason) = structure_differs(model, pristine) {
        return Err(reason);
    }
    if !live.shared_strings.starts_with(&base.shared_strings) {
        return Err("the shared-string table was reordered".into());
    }
    let paths = part_paths(source, &ImportLimits::default())
        .map_err(|_| "the source's parts do not resolve under the default limits")?;
    if paths.worksheets.len() != live.worksheets.len() {
        return Err("the sheets do not map to worksheet parts one to one".into());
    }
    let zip_err = |e: zip::result::ZipError| format!("the source zip: {e}");
    let mut archive = zip::ZipArchive::new(Cursor::new(source)).map_err(zip_err)?;
    let names = PartNames::from_archive(&archive).map_err(|e| e.to_string())?;
    if !names.clashes().is_empty() {
        return Err("the source has part names that clash".into());
    }
    let has_calc_chain = names.entry(CALC_CHAIN_PART).is_some();
    // Without metadata parts of its own, the source can take the export's, whose `vm` and `cm`
    // numbering the export's cell writer uses (SPEC-0034 D3.4).
    let export_layout = [METADATA_PART, RICH_VALUE_PART, RICH_VALUE_STRUCTURE_PART]
        .iter()
        .all(|part| names.entry(part).is_none());
    let mut drop_calc_chain = false;
    let strings_entry = names.entry(paths.shared_strings).map(str::to_string);
    let source_strings = match &strings_entry {
        Some(entry) => Some(read_entry(&mut archive, entry)?),
        None => None,
    };
    // The importer also interns inline and constant `str` cells, after the part's own `<si>`s:
    // indexes past the part then name strings it does not hold.
    let in_part = source_strings.as_deref().map_or(Ok(0), count_si)?;
    let strings_line_up = base.shared_strings.len() == in_part;
    let mut replaced: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let mut sheet_entries = Vec::new();
    let mut patched_sheets = Vec::new();
    for (index, (sheet, before)) in live.worksheets.iter().zip(&base.worksheets).enumerate() {
        let changed = changed_cells(sheet, before);
        if changed.is_empty() {
            continue;
        }
        let at = |c: &(i32, i32), w: &Worksheet| w.sheet_data.cell(c.0, c.1).cloned();
        // A chain naming a cell that lost its formula, or missing a new one, is stale: drop it,
        // as the regenerating export does (SPEC-0034 E2).
        drop_calc_chain |= has_calc_chain
            && changed.iter().any(|c| {
                at(c, sheet).is_some_and(|x| x.has_formula())
                    != at(c, before).is_some_and(|x| x.has_formula())
            });
        if !strings_line_up
            && changed
                .iter()
                .any(|c| matches!(at(c, sheet), Some(Cell::SharedString { .. })))
        {
            return Err("the model's shared strings include inline strings the part lacks".into());
        }
        let Some(entry) = names.entry(&paths.worksheets[index]).map(str::to_string) else {
            return Err(format!("no part for sheet {}", sheet.name));
        };
        let xml = read_entry(&mut archive, &entry)?;
        let writer = cell_writer_for(model, index);
        let patched = splice(
            &xml,
            sheet,
            before,
            &changed,
            &writer.as_ref(),
            export_layout,
        )
        .map_err(|reason| format!("sheet {}: {reason}", sheet.name))?;
        replaced.insert(entry.clone(), patched.into_bytes());
        sheet_entries.push(entry);
        patched_sheets.push(sheet.name.clone());
    }
    let added = &live.shared_strings[base.shared_strings.len()..];
    if !added.is_empty() {
        let (Some(entry), Some(xml), true) = (strings_entry, source_strings, strings_line_up)
        else {
            return Err("new strings need a shared-string part that holds every string".into());
        };
        let refs = (shared_refs(live), shared_refs(base));
        let text = append_strings(&xml, added, live.shared_strings.len(), refs)
            .ok_or("the shared-string part has no closing </sst>")?;
        replaced.insert(entry, text.into_bytes());
    }
    if live.styles != base.styles {
        let added = styles::appended(&base.styles, &live.styles)
            .ok_or("the style table changed other than by appending")?;
        let entry = names
            .entry(STYLES_PART)
            .ok_or("new styles need a styles part the source lacks")?
            .to_string();
        let xml = read_entry(&mut archive, &entry)?;
        let text = styles::patch(&xml, &base.styles, &added)?;
        replaced.insert(entry, text.into_bytes());
    }
    // Only a source with no metadata of its own takes the export's; one that has its own keeps
    // it, and its cells' `vm`/`cm` were checked against it while splicing.
    let mut needs_metadata = false;
    if export_layout {
        for entry in &sheet_entries {
            let xml = replaced.get(entry).map(|b| String::from_utf8_lossy(b));
            needs_metadata |= xml.is_some_and(|xml| cells_carry(&xml, &["vm", "cm"]));
        }
    }
    if needs_metadata {
        // A `vm` the source already holds, with no part to resolve it, loads as a plain error;
        // under the export's metadata it would name a rich error, so such a source falls back.
        for path in &paths.worksheets {
            let Some(entry) = names.entry(path) else {
                continue;
            };
            if cells_carry(&read_entry(&mut archive, entry)?, &["vm"]) {
                return Err("a source cell holds a vm with no metadata to resolve it".into());
            }
        }
    }
    let mut removed = BTreeSet::new();
    let mut added = Vec::new();
    if needs_metadata || drop_calc_chain {
        let rels_entry = names
            .entry(WORKBOOK_RELS_PART)
            .ok_or("the source has no workbook relationships")?
            .to_string();
        let types_entry = names
            .entry(CONTENT_TYPES_PART)
            .ok_or("the source has no content types")?
            .to_string();
        let mut rels = read_entry(&mut archive, &rels_entry)?;
        let mut types = read_entry(&mut archive, &types_entry)?;
        if drop_calc_chain {
            let calc_chain = names.entry(CALC_CHAIN_PART).unwrap_or(CALC_CHAIN_PART);
            removed.insert(calc_chain.to_string());
            (rels, types) = remove_part(&rels, &types, "calcChain", CALC_CHAIN_PART)
                .ok_or("the calcChain relationship or override cannot be removed exactly")?;
        }
        if needs_metadata {
            let new = metadata_parts(has_rich_errors(live));
            if new.iter().any(|(name, ..)| names.entry(name).is_some()) {
                return Err("a metadata part to add is already in the source".into());
            }
            let parts: Vec<NewPart<'_>> = new
                .iter()
                .map(|(name, content_type, relationship, _)| NewPart {
                    name,
                    content_type,
                    relationship,
                })
                .collect();
            (rels, types) = add_parts(&rels, &types, &parts)
                .ok_or("the metadata parts cannot be linked exactly")?;
            added.extend(
                new.into_iter()
                    .map(|(name, _, _, xml)| (name.to_string(), xml.into_bytes())),
            );
        }
        replaced.insert(rels_entry, rels.into_bytes());
        replaced.insert(types_entry, types.into_bytes());
    }
    let package = rewrite_zip(&mut archive, &replaced, &removed, &added)?;
    Ok((package, patched_sheets))
}

/// Whether some `<c>` start tag of a worksheet's XML carries one of `keys`. A part that does not
/// parse counts as carrying them, so the caller falls back.
fn cells_carry(xml: &str, keys: &[&str]) -> bool {
    let mut reader = Reader::from_str(xml);
    loop {
        match reader.read_event() {
            Ok(Event::Start(e) | Event::Empty(e)) if e.local_name().as_ref() == "c" => {
                if keys.iter().any(|k| attr(&e, k).is_some()) {
                    return true;
                }
            }
            Ok(Event::Eof) => return false,
            Err(_) => return true,
            _ => {}
        }
    }
}

/// Why the workbook's structure differs between `model` and `pristine`, if it does: everything
/// the patcher does not rewrite must be equal (SPEC-0034 D2.1). Each [`Worksheet`] field is named,
/// with no `..`, so a field added to the model is a compile error here until it is classified.
fn structure_differs(model: &Model, pristine: &Model) -> Option<String> {
    let (a, b) = (&model.workbook, &pristine.workbook);
    let workbook_fields = [
        ("defined names", a.defined_names == b.defined_names),
        // A table that only appended records keeps every `s=` index; `styles::patch` writes it.
        ("styles", styles::appended(&b.styles, &a.styles).is_some()),
        ("settings", a.settings == b.settings),
        ("metadata", a.metadata == b.metadata),
        ("tables", a.tables == b.tables),
        ("views", a.views == b.views),
        ("theme", a.theme == b.theme),
        ("external books", a.external_books == b.external_books),
        ("pivot caches", a.pivot_caches == b.pivot_caches),
        ("pivot tables", a.pivot_tables == b.pivot_tables),
        ("name", a.name == b.name),
        ("sheet count", a.worksheets.len() == b.worksheets.len()),
    ];
    if let Some((what, _)) = workbook_fields.iter().find(|(_, same)| !same) {
        return Some(format!("the workbook's {what} changed"));
    }
    for (x, y) in a.worksheets.iter().zip(&b.worksheets) {
        let Worksheet {
            dimension: _,
            cols,
            rows,
            name,
            sheet_data: _,
            shared_formulas: _,
            sheet_id,
            load_rank,
            state,
            color,
            merged_cells,
            comments,
            frozen_rows,
            frozen_columns,
            views,
            show_grid_lines,
            conditional_formatting,
            links,
            data_tables,
        } = x;
        let same = *cols == y.cols
            && *rows == y.rows
            && *name == y.name
            && *sheet_id == y.sheet_id
            && *load_rank == y.load_rank
            && *state == y.state
            && *color == y.color
            && *merged_cells == y.merged_cells
            && *comments == y.comments
            && *frozen_rows == y.frozen_rows
            && *frozen_columns == y.frozen_columns
            && *views == y.views
            && *show_grid_lines == y.show_grid_lines
            && *conditional_formatting == y.conditional_formatting
            && *links == y.links
            && *data_tables == y.data_tables;
        if !same {
            return Some(format!("sheet {name}'s structure changed"));
        }
    }
    None
}

/// The cells of `sheet` that differ from `before`, as (row, column).
fn changed_cells(sheet: &Worksheet, before: &Worksheet) -> BTreeSet<(i32, i32)> {
    let mut out = BTreeSet::new();
    for (row, column, cell) in sheet.sheet_data.cells() {
        if before.sheet_data.cell(row, column) != Some(cell) {
            out.insert((row, column));
        }
    }
    for (row, column, _) in before.sheet_data.cells() {
        if sheet.sheet_data.cell(row, column).is_none() {
            out.insert((row, column));
        }
    }
    out
}

/// Owns what a [`CellWriter`] borrows for one sheet.
struct SheetWriter<'a> {
    worksheet: &'a Worksheet,
    parsed: &'a [rustsheet_engine::ParsedFormula],
    legacy: Vec<bool>,
    tables: std::collections::HashMap<(i32, i32), String>,
}

impl SheetWriter<'_> {
    fn as_ref(&self) -> CellWriter<'_> {
        CellWriter {
            worksheet: self.worksheet,
            parsed_formulas: self.parsed,
            legacy_reading: &self.legacy,
            table_formulas: &self.tables,
        }
    }
}

fn cell_writer_for(model: &Model, index: usize) -> SheetWriter<'_> {
    let worksheet = &model.workbook.worksheets[index];
    let parsed = model
        .parsed_formulas
        .get(index)
        .map_or(&[][..], Vec::as_slice);
    SheetWriter {
        worksheet,
        parsed,
        legacy: super::worksheets::legacy_readings(parsed),
        tables: super::data_tables::anchor_formulas(&worksheet.data_tables),
    }
}

/// One `<c>` of the source sheet: its address, byte range, and whether its formula is in a
/// shared-formula group.
struct SourceCell {
    column: i32,
    start: usize,
    end: usize,
    shared: bool,
}

/// One `<row>` of the source sheet: its number, where new cells go (`insert_at`, before
/// `</row>`, or `None` for an empty `<row/>`), its byte range and its cells.
struct SourceRow {
    r: i32,
    start: usize,
    end: usize,
    insert_at: Option<usize>,
    cells: Vec<SourceCell>,
}

/// The rows of a sheet's `sheetData`, where new rows go, and its `<dimension>`.
struct SheetIndex {
    rows: Vec<SourceRow>,
    /// Offset before `</sheetData>`, or the range of an empty `<sheetData/>` to replace.
    tail: Tail,
    /// The `<dimension>` element's byte range and `ref`.
    dimension: Option<(usize, usize, String)>,
}

enum Tail {
    Before(usize),
    Empty(usize, usize),
}

fn attr(e: &BytesStart<'_>, key: &str) -> Option<String> {
    e.attributes().flatten().find_map(|a| {
        (a.key.as_ref() == key)
            .then(|| a.normalized_value(quick_xml::XmlVersion::Implicit1_0).ok())?
            .map(std::borrow::Cow::into_owned)
    })
}

/// `A1` → (row, column), 1-based, through the engine's A1 parser.
fn parse_ref(r: &str) -> Option<(i32, i32)> {
    parse_reference_a1(r).map(|p| (p.row, p.column))
}

/// Indexes the rows and cells of a worksheet's XML, refusing what the patcher will not guess
/// at: rows or cells out of order, a cell without `r`, a prefixed `sheetData`, `row` or `c`,
/// nesting past [`MAX_DEPTH`]. Only a `row` directly under `sheetData`, and a `c` directly under
/// such a row, count: the same names inside an extension are someone else's.
fn index_sheet(xml: &str) -> Result<SheetIndex, String> {
    let mut reader = Reader::from_str(xml);
    let mut depth = 0usize;
    let mut sheet_data: Option<usize> = None;
    let mut rows: Vec<SourceRow> = Vec::new();
    let mut tail = None;
    let mut dimension = None;
    let mut cell: Option<SourceCell> = None;
    loop {
        let start = to_offset(reader.buffer_position())?;
        let event = reader.read_event().map_err(|e| e.to_string())?;
        let end = to_offset(reader.buffer_position())?;
        match &event {
            Event::Start(e) | Event::Empty(e) => {
                let empty = matches!(event, Event::Empty(_));
                // The element's own level: 1 for the root.
                let level = depth + 1;
                if !empty {
                    depth = level;
                    if depth > MAX_DEPTH {
                        return Err("nested too deep".into());
                    }
                }
                let local = e.local_name();
                let local = local.as_ref();
                let ours = matches!(local, "sheetData" | "row" | "c" | "dimension");
                if ours && e.name().as_ref() != local {
                    return Err("a prefixed worksheet".into());
                }
                match (local, sheet_data) {
                    ("dimension", None) if level == 2 => {
                        let r = attr(e, "ref").ok_or("a dimension without ref")?;
                        dimension = Some((start, end, r));
                    }
                    ("sheetData", None) if level == 2 => {
                        if empty {
                            tail = Some(Tail::Empty(start, end));
                        } else {
                            sheet_data = Some(level);
                        }
                    }
                    ("row", Some(sd)) if level == sd + 1 => {
                        let r: i32 = attr(e, "r")
                            .and_then(|v| v.parse().ok())
                            .ok_or("a row without r")?;
                        if rows.last().is_some_and(|last| last.r >= r) {
                            return Err("rows out of order".into());
                        }
                        rows.push(SourceRow {
                            r,
                            start,
                            end,
                            insert_at: None,
                            cells: Vec::new(),
                        });
                    }
                    ("c", Some(sd)) if level == sd + 2 => {
                        let (row, column) = attr(e, "r")
                            .as_deref()
                            .and_then(parse_ref)
                            .ok_or("a cell without r")?;
                        let current = rows.last_mut().ok_or("a cell outside a row")?;
                        if row != current.r
                            || current.cells.last().is_some_and(|c| c.column >= column)
                        {
                            return Err("cells out of order".into());
                        }
                        let c = SourceCell {
                            column,
                            start,
                            end,
                            shared: false,
                        };
                        if empty {
                            current.cells.push(c);
                        } else {
                            cell = Some(c);
                        }
                    }
                    ("f", Some(sd)) if level == sd + 3 => {
                        if let Some(c) = cell.as_mut() {
                            c.shared |= attr(e, "t").as_deref() == Some("shared");
                        }
                    }
                    _ => {}
                }
            }
            Event::End(e) => {
                let level = depth;
                depth = depth.saturating_sub(1);
                match (e.local_name().as_ref(), sheet_data) {
                    ("c", Some(sd)) if level == sd + 2 => {
                        if let (Some(mut c), Some(row)) = (cell.take(), rows.last_mut()) {
                            c.end = end;
                            row.cells.push(c);
                        }
                    }
                    ("row", Some(sd)) if level == sd + 1 => {
                        if let Some(row) = rows.last_mut() {
                            row.insert_at = Some(start);
                            row.end = end;
                        }
                    }
                    ("sheetData", Some(sd)) if level == sd => {
                        tail = Some(Tail::Before(start));
                        sheet_data = None;
                    }
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    let tail = tail.ok_or("no sheetData")?;
    Ok(SheetIndex {
        rows,
        tail,
        dimension,
    })
}

fn to_offset(position: u64) -> Result<usize, String> {
    usize::try_from(position).map_err(|_| "a part too large to index".to_string())
}

/// The `t` and `vm` attributes and the `<v>` element of a cell's XML, as the export wrote them.
fn value_parts(xml: &str) -> Option<(Option<String>, Option<String>, String)> {
    let mut reader = Reader::from_str(xml);
    let mut t = None;
    let mut vm = None;
    let mut v = String::new();
    loop {
        let start = usize::try_from(reader.buffer_position()).ok()?;
        match reader.read_event().ok()? {
            Event::Start(e) | Event::Empty(e) if e.local_name().as_ref() == "c" => {
                t = attr(&e, "t");
                vm = attr(&e, "vm");
            }
            Event::Start(e) if e.local_name().as_ref() == "v" => {
                reader.read_to_end(e.name()).ok()?;
                let end = usize::try_from(reader.buffer_position()).ok()?;
                v = xml[start..end].to_string();
            }
            Event::Eof => return Some((t, vm, v)),
            _ => {}
        }
    }
}

/// The source cell `old` with its cached value replaced by the one `new` (the export's XML for the
/// live cell) holds, keeping its `<f>` and every other attribute and child byte for byte. With
/// `export_vm`, the cell takes the export's `vm` (a package on the export's metadata layout);
/// without it, it keeps its own, which the caller allows only for the same rich error
/// ([`rich_value`]).
fn patch_value(old: &str, new: &str, export_vm: bool) -> Option<String> {
    let (t, vm, v) = value_parts(new)?;
    let vm = if export_vm {
        Vm::Set(vm.as_deref())
    } else {
        Vm::Keep
    };
    let mut reader = Reader::from_str(old);
    let mut out = String::with_capacity(old.len() + v.len());
    let mut had_v = false;
    let mut close_at = None;
    loop {
        let start = usize::try_from(reader.buffer_position()).ok()?;
        let event = reader.read_event().ok()?;
        let end = usize::try_from(reader.buffer_position()).ok()?;
        match event {
            Event::Start(ref e) | Event::Empty(ref e) if e.local_name().as_ref() == "c" => {
                out.push_str(&rewrite_start_tag(e, t.as_deref(), vm)?);
                out.push('>');
                if matches!(event, Event::Empty(_)) {
                    // `<c .../>` had no value: it becomes a full element.
                    out.push_str(&v);
                    out.push_str("</c>");
                    return Some(out);
                }
            }
            Event::Start(ref e) if e.local_name().as_ref() == "v" => {
                reader.read_to_end(e.name()).ok()?;
                out.push_str(&v);
                had_v = true;
            }
            Event::Empty(ref e) if e.local_name().as_ref() == "v" => {
                out.push_str(&v);
                had_v = true;
            }
            Event::End(ref e) if e.local_name().as_ref() == "c" => {
                close_at = Some(out.len());
                out.push_str(&old[start..end]);
            }
            Event::Eof => break,
            _ => out.push_str(&old[start..end]),
        }
    }
    if !had_v {
        out.insert_str(close_at?, &v);
    }
    Some(out)
}

/// `<c ...` (without its closing `>`) with `t` set to the given value in its place (or removed,
/// or added last), every other attribute kept in its order and spelling. With `vm` given, `vm` is
/// set the same way; without it, the source's `vm` is kept. `None` for a malformed attribute.
fn rewrite_start_tag(e: &BytesStart<'_>, t: Option<&str>, vm: Vm<'_>) -> Option<String> {
    match vm {
        Vm::Keep => rewrite_tag(e, "c", &[("t", t)]),
        Vm::Set(vm) => rewrite_tag(e, "c", &[("t", t), ("vm", vm)]),
    }
}

/// The start tag `e` as `<name ...` (without its closing `>`), each attribute `changes` names set
/// to its value (escaped here) in its place, or removed for `None`, or added last when absent;
/// every other attribute kept in its order and spelling. A kept value is re-quoted with `"`, so
/// one that held a `"` inside `'...'` gets it as `&quot;`. `None` for a malformed attribute.
pub(crate) fn rewrite_tag(
    e: &BytesStart<'_>,
    name: &str,
    changes: &[(&str, Option<&str>)],
) -> Option<String> {
    let mut out = format!("<{name}");
    // `Some(new)` is a change still to place (`new` its value, or `None` to remove it).
    let mut pending: Vec<(&str, Option<Option<String>>)> = (changes.iter())
        .map(|(key, value)| (*key, Some(value.map(|v| escape_xml(v).into_owned()))))
        .collect();
    for a in e.attributes() {
        let a = a.ok()?;
        let key = a.key.as_ref();
        let value = match pending.iter_mut().find(|(k, _)| *k == key) {
            Some((_, slot)) => match slot.take().flatten() {
                Some(value) => value,
                None => continue,
            },
            None => a.value.replace('"', "&quot;"),
        };
        out.push_str(&format!(" {key}=\"{value}\""));
    }
    for (key, value) in pending {
        if let Some(Some(value)) = value {
            out.push_str(&format!(" {key}=\"{value}\""));
        }
    }
    Some(out)
}

/// What a value patch does with a cell's `vm`.
#[derive(Clone, Copy)]
enum Vm<'a> {
    /// The source's `vm`, unchanged.
    Keep,
    /// The export's: this value, or none.
    Set(Option<&'a str>),
}

/// Whether a value patch may keep the source cell's `vm`. The source's `vm` indexes its own
/// `metadata.xml` and the export's indexes the export's layout, so the numbers cannot be compared
/// (docs/excel-semantics/arrays.md). Only the same error kind keeps its source `vm`; any other rich
/// value falls back: the source's metadata.xml has no block for it.
fn rich_value(old: &str, new: &str, before: &Cell, after: &Cell) -> Result<(), String> {
    let (had, needs) = (old.contains(" vm="), new.contains(" vm="));
    if !had && !needs {
        return Ok(());
    }
    let error = |c: &Cell| match c {
        Cell::CellFormula {
            v: FormulaValue::Error { ei, .. },
            ..
        }
        | Cell::ArrayFormula {
            v: FormulaValue::Error { ei, .. },
            ..
        } => Some(ei.clone()),
        _ => None,
    };
    match (had, error(before), error(after)) {
        (true, Some(a), Some(b)) if a == b => Ok(()),
        _ => Err("a rich value changed, which the source's metadata.xml has no block for".into()),
    }
}

/// Whether only the cached value of a formula cell changed: same variant, formula and style.
fn value_only(before: &Cell, after: &Cell) -> bool {
    match (before, after) {
        (Cell::CellFormula { f, s, .. }, Cell::CellFormula { f: f2, s: s2, .. }) => {
            f == f2 && s == s2
        }
        (
            Cell::ArrayFormula { f, s, r, kind, .. },
            Cell::ArrayFormula {
                f: f2,
                s: s2,
                r: r2,
                kind: k2,
                ..
            },
        ) => f == f2 && s == s2 && r == r2 && kind == k2,
        _ => false,
    }
}

/// One splice: remove `remove` bytes at `at` and write `text`. Edits at one offset are written
/// in `key` (row, column) order, so new rows and cells land in sheet order.
struct Edit {
    at: usize,
    remove: usize,
    key: (i32, i32),
    text: String,
}

/// The sheet's XML with `changed` cells rewritten, inserted or removed, its `<dimension>` grown
/// to cover new cells.
fn splice(
    xml: &str,
    sheet: &Worksheet,
    before: &Worksheet,
    changed: &BTreeSet<(i32, i32)>,
    writer: &CellWriter<'_>,
    export_layout: bool,
) -> Result<String, String> {
    let index = index_sheet(xml)?;
    let mut edits: Vec<Edit> = Vec::new();
    let mut inserts: BTreeMap<i32, Vec<(i32, String)>> = BTreeMap::new();
    for &(row, column) in changed {
        let source_row = find_row(&index.rows, row);
        let source_cell = source_row.and_then(|r| {
            r.cells
                .binary_search_by_key(&column, |c| c.column)
                .ok()
                .map(|i| &r.cells[i])
        });
        let new = new_cell_xml(sheet, writer, row, column)?;
        match (source_cell, new) {
            (Some(c), Some(text)) => {
                let (Some(a), Some(b)) = (
                    before.sheet_data.cell(row, column),
                    sheet.sheet_data.cell(row, column),
                ) else {
                    return Err("a changed cell vanished".into());
                };
                let old = &xml[c.start..c.end];
                let text = if value_only(a, b) {
                    if !export_layout {
                        rich_value(old, &text, a, b)?;
                    }
                    patch_value(old, &text, export_layout)
                        .ok_or("a cell value could not be patched")?
                } else if c.shared {
                    return Err("a changed formula in a shared-formula group".into());
                } else {
                    whole_cell(text, export_layout)?
                };
                edits.push(Edit {
                    at: c.start,
                    remove: c.end - c.start,
                    key: (row, column),
                    text,
                });
            }
            (Some(c), None) => {
                if c.shared {
                    return Err("a cleared cell in a shared-formula group".into());
                }
                edits.push(Edit {
                    at: c.start,
                    remove: c.end - c.start,
                    key: (row, column),
                    text: String::new(),
                });
            }
            (None, Some(text)) => inserts
                .entry(row)
                .or_default()
                .push((column, whole_cell(text, export_layout)?)),
            (None, None) => {}
        }
    }
    insert_cells(xml, &index, inserts, &mut edits);
    if let Some(edit) = grown_dimension(&index, sheet) {
        edits.push(edit);
    }
    let out = apply(xml, edits);
    index_sheet(&out).map_err(|e| format!("the patched sheet does not index again: {e}"))?;
    Ok(out)
}

fn find_row(rows: &[SourceRow], row: i32) -> Option<&SourceRow> {
    rows.binary_search_by_key(&row, |r| r.r)
        .ok()
        .map(|i| &rows[i])
}

/// The export's XML for the live cell, if there is one.
fn new_cell_xml(
    sheet: &Worksheet,
    writer: &CellWriter<'_>,
    row: i32,
    column: i32,
) -> Result<Option<String>, String> {
    let Some(cell) = sheet.sheet_data.cell(row, column) else {
        return Ok(None);
    };
    cell_xml(writer, row, column, cell)
        .map(Some)
        .ok_or_else(|| "a cell the export cannot write".to_string())
}

/// A cell written whole by the export. Its `vm`/`cm` index the export's `metadata.xml` layout,
/// so they are only right in a package that takes the export's metadata parts.
fn whole_cell(text: String, export_layout: bool) -> Result<String, String> {
    if !export_layout && (text.contains(" cm=") || text.contains(" vm=")) {
        return Err("a cell needs value or cell metadata in the source's own layout".into());
    }
    Ok(text)
}

/// The edits that add new cells, row by row: into an existing row before the next cell or its
/// `</row>`, an empty `<row/>` written out once with all of them, new rows before the next
/// source row or at the end of `sheetData`, and an empty `<sheetData/>` written out once.
fn insert_cells(
    xml: &str,
    index: &SheetIndex,
    inserts: BTreeMap<i32, Vec<(i32, String)>>,
    edits: &mut Vec<Edit>,
) {
    let mut tail_rows = String::new();
    for (row, cells) in inserts {
        let at_row = (row, 0);
        match find_row(&index.rows, row) {
            Some(r) => match r.insert_at {
                Some(close) => {
                    for (column, text) in cells {
                        let next = r.cells.partition_point(|c| c.column < column);
                        let at = r.cells.get(next).map_or(close, |c| c.start);
                        edits.push(Edit {
                            at,
                            remove: 0,
                            key: (row, column),
                            text,
                        });
                    }
                }
                None => {
                    // `<row .../>`: written out once as a full row.
                    let open = xml[r.start..r.end].trim_end_matches("/>");
                    let body: String = cells.into_iter().map(|(_, t)| t).collect();
                    edits.push(Edit {
                        at: r.start,
                        remove: r.end - r.start,
                        key: at_row,
                        text: format!("{open}>{body}</row>"),
                    });
                }
            },
            None => {
                let body: String = cells.into_iter().map(|(_, t)| t).collect();
                let text = format!(r#"<row r="{row}">{body}</row>"#);
                let next = index.rows.partition_point(|r| r.r < row);
                match (index.rows.get(next), &index.tail) {
                    (Some(next), _) => edits.push(Edit {
                        at: next.start,
                        remove: 0,
                        key: at_row,
                        text,
                    }),
                    (None, Tail::Before(at)) => edits.push(Edit {
                        at: *at,
                        remove: 0,
                        key: at_row,
                        text,
                    }),
                    (None, Tail::Empty(..)) => tail_rows.push_str(&text),
                }
            }
        }
    }
    if let (Tail::Empty(start, end), false) = (&index.tail, tail_rows.is_empty()) {
        edits.push(Edit {
            at: *start,
            remove: end - start,
            key: (0, 0),
            text: format!("<sheetData>{tail_rows}</sheetData>"),
        });
    }
}

/// The `<dimension>` grown to cover the live sheet's cells, when they reach past it. It is
/// never shrunk: a range larger than the cells is what Excel itself keeps for formatted rows.
fn grown_dimension(index: &SheetIndex, sheet: &Worksheet) -> Option<Edit> {
    let (start, end, old) = index.dimension.as_ref()?;
    let (first, last) = old.split_once(':').unwrap_or((old, old));
    let (r1, c1) = parse_ref(first)?;
    let (r2, c2) = parse_ref(last)?;
    let (mut top, mut left, mut bottom, mut right) = (r1, c1, r2, c2);
    for (row, column, _) in sheet.sheet_data.cells() {
        top = top.min(row);
        bottom = bottom.max(row);
        left = left.min(column);
        right = right.max(column);
    }
    if (top, left, bottom, right) == (r1, c1, r2, c2) {
        return None;
    }
    let corner = |row: i32, column: i32| Some(format!("{}{row}", number_to_column(column)?));
    let range = format!("{}:{}", corner(top, left)?, corner(bottom, right)?);
    Some(Edit {
        at: *start,
        remove: end - start,
        key: (0, 0),
        text: format!(r#"<dimension ref="{range}"/>"#),
    })
}

/// `xml` with `edits` applied from the end, so earlier offsets stay valid. Edits at one offset
/// are joined in key order and remove the longest range among them (at most one removes).
fn apply(xml: &str, mut edits: Vec<Edit>) -> String {
    edits.sort_by(|a, b| b.at.cmp(&a.at).then(a.key.cmp(&b.key)));
    let mut out = xml.to_string();
    let mut i = 0;
    while i < edits.len() {
        let at = edits[i].at;
        let same = edits[i..].iter().take_while(|e| e.at == at).count();
        let group = &edits[i..i + same];
        let remove = group.iter().map(|e| e.remove).max().unwrap_or(0);
        let text: String = group.iter().map(|e| e.text.as_str()).collect();
        out.replace_range(at..at + remove, &text);
        i += same;
    }
    out
}
