//! The patch tier of the source-preserving export (SPEC-0034 D3): a workbook whose structure is
//! unchanged since its load is written as its source, with only the changed `<c>` elements
//! rewritten and new shared strings appended. Every other byte, part and relationship is copied.
//!
//! The patch never guesses. Anything it does not handle exactly is a fallback reason, and the
//! caller regenerates instead:
//! - a structural change;
//! - a style table that changed (T-4561);
//! - out-of-order rows or cells, or a cell without `r`;
//! - a changed formula in a shared-formula group;
//! - a cell needing `metadata.xml` the source lacks.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{Cursor, Read, Seek, Write};

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;
use rustsheet_engine::types::{Cell, Worksheet};
use rustsheet_engine::Model;

use super::escape::escape_xml;
use super::worksheets::{cell_xml, CellWriter};
use crate::error::XlsxError;
use crate::import::{part_paths, ImportLimits, PartNames};

/// Deepest element nesting the patcher reads in a worksheet (directive 5).
const MAX_DEPTH: usize = 256;

/// A patched package, or why the source cannot be patched.
pub(super) type Patch = Result<(Vec<u8>, Vec<String>), String>;

/// Patches `source` to hold `model`'s cells. `pristine` is `source` as imported (SPEC-0034 D2).
/// The inner `Err` is a fallback reason; the outer one an I/O or zip failure.
pub(super) fn patch(model: &Model, pristine: &Model, source: &[u8]) -> Result<Patch, XlsxError> {
    let (live, base) = (&model.workbook, &pristine.workbook);
    if let Some(reason) = structure_differs(model, pristine) {
        return Ok(Err(reason));
    }
    if !live.shared_strings.starts_with(&base.shared_strings) {
        return Ok(Err("the shared-string table was reordered".into()));
    }
    let Ok(paths) = part_paths(source, &ImportLimits::default()) else {
        return Ok(Err(
            "the source's parts do not resolve under the default limits".into(),
        ));
    };
    if paths.worksheets.len() != live.worksheets.len() {
        return Ok(Err(
            "the sheets do not map to worksheet parts one to one".into()
        ));
    }
    let mut archive = zip::ZipArchive::new(Cursor::new(source))?;
    let names = PartNames::from_archive(&archive)?;
    let has_metadata = names
        .entry(crate::import::rich_errors::METADATA_PART)
        .is_some();
    let mut replaced: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let mut patched_sheets = Vec::new();
    for (index, (sheet, before)) in live.worksheets.iter().zip(&base.worksheets).enumerate() {
        let changed = changed_cells(sheet, before);
        if changed.is_empty() {
            continue;
        }
        let Some(entry) = names.entry(&paths.worksheets[index]).map(str::to_string) else {
            return Ok(Err(format!("no part for sheet {}", sheet.name)));
        };
        let xml = read_entry(&mut archive, &entry)?;
        let writer = cell_writer_for(model, index);
        let patched = match splice(
            &xml,
            sheet,
            before,
            &changed,
            &writer.as_ref(),
            has_metadata,
        ) {
            Ok(text) => text,
            Err(reason) => return Ok(Err(format!("sheet {}: {reason}", sheet.name))),
        };
        replaced.insert(entry, patched.into_bytes());
        patched_sheets.push(sheet.name.clone());
    }
    let added = &live.shared_strings[base.shared_strings.len()..];
    if !added.is_empty() {
        let Some(entry) = names.entry(paths.shared_strings).map(str::to_string) else {
            return Ok(Err(
                "new strings need a shared-string part the source lacks".into(),
            ));
        };
        let xml = read_entry(&mut archive, &entry)?;
        let Some(text) = append_strings(&xml, added, live.shared_strings.len()) else {
            return Ok(Err("the shared-string part has no closing </sst>".into()));
        };
        replaced.insert(entry, text.into_bytes());
    }
    Ok(Ok((rewrite_zip(&mut archive, &replaced)?, patched_sheets)))
}

/// Why the workbook's structure differs between `model` and `pristine`, if it does: everything
/// the patcher does not rewrite must be equal (SPEC-0034 D2.1). Each [`Worksheet`] field is named,
/// with no `..`, so a field added to the model is a compile error here until it is classified.
fn structure_differs(model: &Model, pristine: &Model) -> Option<String> {
    let (a, b) = (&model.workbook, &pristine.workbook);
    let workbook_fields = [
        ("defined names", a.defined_names == b.defined_names),
        ("styles", a.styles == b.styles),
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

/// The rows of a sheet's `sheetData`, and where new rows go (before `</sheetData>`).
struct SheetIndex {
    rows: Vec<SourceRow>,
    /// Offset before `</sheetData>`, or the range of an empty `<sheetData/>` to replace.
    tail: Tail,
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

/// `A1` → (row, column), 1-based.
fn parse_ref(r: &str) -> Option<(i32, i32)> {
    let split = r.find(|c: char| c.is_ascii_digit())?;
    let (letters, digits) = r.split_at(split);
    let mut column = 0i32;
    for c in letters.chars() {
        let d = i32::from(
            u8::try_from(c.to_ascii_uppercase())
                .ok()?
                .checked_sub(b'A')?,
        ) + 1;
        column = column.checked_mul(26)?.checked_add(d)?;
    }
    Some((digits.parse().ok()?, column))
}

/// Indexes the rows and cells of a worksheet's XML, refusing what the patcher will not guess
/// at: rows or cells out of order, a cell without `r`, nesting past [`MAX_DEPTH`].
fn index_sheet(xml: &str) -> Result<SheetIndex, String> {
    let mut reader = Reader::from_str(xml);
    let mut depth = 0usize;
    let mut rows: Vec<SourceRow> = Vec::new();
    let mut tail = None;
    let mut cell: Option<SourceCell> = None;
    loop {
        let start = reader.buffer_position() as usize;
        let event = reader.read_event().map_err(|e| e.to_string())?;
        let end = reader.buffer_position() as usize;
        match &event {
            Event::Start(e) | Event::Empty(e) => {
                let empty = matches!(event, Event::Empty(_));
                if !empty {
                    depth += 1;
                    if depth > MAX_DEPTH {
                        return Err("nested too deep".into());
                    }
                }
                match e.local_name().as_ref() {
                    "sheetData" if empty => tail = Some(Tail::Empty(start, end)),
                    "row" => {
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
                    "c" => {
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
                    "f" => {
                        if let Some(c) = cell.as_mut() {
                            c.shared |= attr(e, "t").as_deref() == Some("shared");
                        }
                    }
                    _ => {}
                }
            }
            Event::End(e) => {
                depth = depth.saturating_sub(1);
                match e.local_name().as_ref() {
                    "c" => {
                        if let (Some(mut c), Some(row)) = (cell.take(), rows.last_mut()) {
                            c.end = end;
                            row.cells.push(c);
                        }
                    }
                    "row" => {
                        if let Some(row) = rows.last_mut() {
                            row.insert_at = Some(start);
                            row.end = end;
                        }
                    }
                    "sheetData" => tail = Some(Tail::Before(start)),
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    let tail = tail.ok_or("no sheetData")?;
    Ok(SheetIndex { rows, tail })
}

/// The `t` and `vm` attributes and the `<v>` element of a cell's XML, as the export wrote them.
fn value_parts(xml: &str) -> Option<(Option<String>, Option<String>, String)> {
    let mut reader = Reader::from_str(xml);
    let mut t = None;
    let mut vm = None;
    let mut v = String::new();
    loop {
        let start = reader.buffer_position() as usize;
        match reader.read_event().ok()? {
            Event::Start(e) | Event::Empty(e) if e.local_name().as_ref() == "c" => {
                t = attr(&e, "t");
                vm = attr(&e, "vm");
            }
            Event::Start(e) if e.local_name().as_ref() == "v" => {
                reader.read_to_end(e.name()).ok()?;
                v = xml[start..reader.buffer_position() as usize].to_string();
            }
            Event::Eof => return Some((t, vm, v)),
            _ => {}
        }
    }
}

/// The source cell `old` with its cached value replaced by the one `new` (the export's XML for the
/// live cell) holds, keeping its `<f>` and every other attribute and child byte for byte.
fn patch_value(old: &str, new: &str) -> Option<String> {
    let (t, vm, v) = value_parts(new)?;
    let mut reader = Reader::from_str(old);
    let mut out = String::with_capacity(old.len() + v.len());
    let mut had_v = false;
    let mut close_at = None;
    loop {
        let start = reader.buffer_position() as usize;
        let event = reader.read_event().ok()?;
        let end = reader.buffer_position() as usize;
        match event {
            Event::Start(ref e) | Event::Empty(ref e) if e.local_name().as_ref() == "c" => {
                out.push_str(&rewrite_start_tag(e, t.as_deref(), vm.as_deref()));
                if matches!(event, Event::Empty(_)) {
                    // `<c .../>` had no value: it becomes a full element.
                    out.pop();
                    if out.ends_with('/') {
                        out.pop();
                    }
                    out.push('>');
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

/// `<c ...>` with `t` and `vm` set to the given values (or removed), every other attribute kept in
/// its order and spelling.
fn rewrite_start_tag(e: &BytesStart<'_>, t: Option<&str>, vm: Option<&str>) -> String {
    let mut out = String::from("<c");
    for a in e.attributes().flatten() {
        let key = a.key.as_ref();
        if key == "t" || key == "vm" {
            continue;
        }
        out.push(' ');
        out.push_str(key);
        out.push_str("=\"");
        out.push_str(&a.value);
        out.push('"');
    }
    for (key, value) in [("t", t), ("vm", vm)] {
        if let Some(value) = value {
            out.push_str(&format!(" {key}=\"{}\"", escape_xml(value)));
        }
    }
    out.push('>');
    out
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

/// The sheet's XML with `changed` cells rewritten, inserted or removed.
fn splice(
    xml: &str,
    sheet: &Worksheet,
    before: &Worksheet,
    changed: &BTreeSet<(i32, i32)>,
    writer: &CellWriter<'_>,
    has_metadata: bool,
) -> Result<String, String> {
    let index = index_sheet(xml)?;
    // (offset, length to remove, text to insert), applied from the end.
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    let new_xml = |row: i32, column: i32| -> Result<Option<String>, String> {
        let Some(cell) = sheet.sheet_data.cell(row, column) else {
            return Ok(None);
        };
        let text = cell_xml(writer, row, column, cell).ok_or("a cell the export cannot write")?;
        if !has_metadata && (text.contains(" cm=") || text.contains(" vm=")) {
            return Err("a cell needs metadata.xml, which the source lacks".into());
        }
        Ok(Some(text))
    };
    let mut new_rows: BTreeMap<i32, Vec<String>> = BTreeMap::new();
    for &(row, column) in changed {
        let source_row = index.rows.iter().find(|r| r.r == row);
        let source_cell = source_row.and_then(|r| r.cells.iter().find(|c| c.column == column));
        match (source_cell, new_xml(row, column)?) {
            (Some(c), Some(text)) => {
                let old = &xml[c.start..c.end];
                let (Some(a), Some(b)) = (
                    before.sheet_data.cell(row, column),
                    sheet.sheet_data.cell(row, column),
                ) else {
                    return Err("a changed cell vanished".into());
                };
                let replacement = if value_only(a, b) {
                    patch_value(old, &text).ok_or("a cell value could not be patched")?
                } else if c.shared {
                    return Err("a changed formula in a shared-formula group".into());
                } else {
                    text
                };
                edits.push((c.start, c.end - c.start, replacement));
            }
            (Some(c), None) => {
                if c.shared {
                    return Err("a cleared cell in a shared-formula group".into());
                }
                edits.push((c.start, c.end - c.start, String::new()));
            }
            (None, Some(text)) => match source_row {
                Some(r) => {
                    let at = r
                        .cells
                        .iter()
                        .find(|c| c.column > column)
                        .map(|c| c.start)
                        .or(r.insert_at);
                    match at {
                        Some(at) => edits.push((at, 0, text)),
                        None => {
                            // `<row .../>`: write it out as a full row.
                            let open = xml[r.start..r.end].trim_end_matches("/>").to_string();
                            edits.push((r.start, r.end - r.start, format!("{open}>{text}</row>")));
                        }
                    }
                }
                None => new_rows.entry(row).or_default().push(text),
            },
            (None, None) => {}
        }
    }
    for (row, cells) in new_rows {
        let text = format!(r#"<row r="{row}">{}</row>"#, cells.concat());
        match index.rows.iter().find(|r| r.r > row) {
            Some(next) => edits.push((next.start, 0, text)),
            None => match index.tail {
                Tail::Before(at) => edits.push((at, 0, text)),
                Tail::Empty(start, end) => {
                    edits.push((start, end - start, format!("<sheetData>{text}</sheetData>")))
                }
            },
        }
    }
    // Apply from the end so earlier offsets stay valid; insertions at one offset keep their order.
    edits.sort_by(|a, b| b.0.cmp(&a.0));
    let mut out = xml.to_string();
    let mut last = usize::MAX;
    let mut pending: Vec<(usize, usize, String)> = Vec::new();
    for edit in edits {
        if edit.0 != last {
            flush(&mut out, &mut pending);
            last = edit.0;
        }
        pending.push(edit);
    }
    flush(&mut out, &mut pending);
    Ok(out)
}

/// Applies the edits that share one offset, in column order (they were pushed in that order).
fn flush(out: &mut String, pending: &mut Vec<(usize, usize, String)>) {
    let Some(&(at, _, _)) = pending.first() else {
        return;
    };
    let remove = pending.iter().map(|e| e.1).max().unwrap_or(0);
    let text: String = pending.iter().map(|e| e.2.as_str()).collect();
    out.replace_range(at..at + remove, &text);
    pending.clear();
}

/// The shared-string part with `added` appended and its counts updated.
fn append_strings(xml: &str, added: &[String], unique: usize) -> Option<String> {
    let close = xml.rfind("</sst>")?;
    let mut out = String::with_capacity(xml.len() + added.len() * 16);
    let head = &xml[..close];
    out.push_str(&set_count(head, "uniqueCount", unique));
    for s in added {
        let space = if s.starts_with(char::is_whitespace) || s.ends_with(char::is_whitespace) {
            r#" xml:space="preserve""#
        } else {
            ""
        };
        out.push_str(&format!("<si><t{space}>{}</t></si>", escape_xml(s)));
    }
    out.push_str(&xml[close..]);
    Some(out)
}

/// `head` with the first `name="..."` attribute of `<sst` set to `value`, when present.
fn set_count(head: &str, name: &str, value: usize) -> String {
    let Some(sst) = head.find("<sst") else {
        return head.to_string();
    };
    let needle = format!(" {name}=\"");
    let Some(at) = head[sst..].find(&needle).map(|i| i + sst + needle.len()) else {
        return head.to_string();
    };
    let Some(len) = head[at..].find('"') else {
        return head.to_string();
    };
    format!("{}{value}{}", &head[..at], &head[at + len..])
}

fn read_entry<R: Read + Seek>(
    archive: &mut zip::ZipArchive<R>,
    name: &str,
) -> Result<String, XlsxError> {
    let limit = ImportLimits::default().max_uncompressed_bytes;
    let mut text = String::new();
    archive
        .by_name(name)?
        .take(limit)
        .read_to_string(&mut text)?;
    Ok(text)
}

/// The source package with the `replaced` entries' contents swapped in and every other entry
/// copied raw, so it keeps its exact compressed bytes.
fn rewrite_zip<R: Read + Seek>(
    archive: &mut zip::ZipArchive<R>,
    replaced: &BTreeMap<String, Vec<u8>>,
) -> Result<Vec<u8>, XlsxError> {
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..archive.len() {
        let file = archive.by_index_raw(i)?;
        let name = file.name()?.to_string();
        if let Some(bytes) = replaced.get(&name) {
            drop(file);
            out.start_file(name, zip::write::SimpleFileOptions::default())?;
            out.write_all(bytes)?;
        } else {
            out.raw_copy_file(file)?;
        }
    }
    Ok(out.finish()?.into_inner())
}
