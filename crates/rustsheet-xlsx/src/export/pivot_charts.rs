//! A `PivotChart`'s chart part with its series written from the pivot it plots (T-4187).
//!
//! [`rewrite_pivot_chart`] takes a chart part bound to a pivot (`c:pivotSource`) and replaces its
//! series with one per series of the pivot's [`PivotChartData`], each with the references and
//! caches Excel for the web writes when it renders the pivot. The evidence is the chart parts of
//! the file Excel for the web saved for probes/t4187-pivot-chart-caches, committed as
//! eval/roundtrip/excel-saved/cases/t4187-pivot-chart-caches.xlsx (docs/excel-semantics/pivots.md
//! "Excel for the web rewrites a `PivotChart`'s caches when it renders the pivot"):
//! - `c:tx` reads the series' column from the location's first row down to the row above the
//!   first data row (`'C'!$B$3`, `M!$B$3:$B$4`), cached as the series' name (`Total` with no
//!   label, [`ChartSeries::name`]);
//! - `c:cat` reads the row-label columns, `c:val` the series' column, both from the first data
//!   row to the location's last row, grand total included (`'C'!$A$4:$A$6`, `'C'!$B$4:$B$6`),
//!   while their caches hold the plotted points only;
//! - the value cache's `c:formatCode` is the data field's `numFmtId` code, `General` without one;
//! - a cell no record reaches has no `c:pt`;
//! - one series per column line, whatever the part held (M: one stale series became three);
//! - a pivot collapsed to `#SPILL!` gives every reference as the anchor, the category cache the
//!   categories, and the value cache its count with one point, 0, at index 0.
//!
//! Everything outside the series is kept as the part has it: the `c:pivotSource` name (Excel
//! writes its own file name there), `c:pivotFmts` (Excel adds one per series), the axes'
//! `c:numFmt`. Each series keeps its own formatting children, and a series past the part's
//! last takes the last one's. What the evidence does not show is UNVERIFIED (pivots.md lists
//! it): a multi-level category cache (the shape the corpus charts show is written), a series of
//! several labels (named by its labels joined with a space), the format code with Values on rows,
//! number spelling past integers, error points (left out) and several series of a collapsed
//! pivot.
//!
//! The output is not linear in the part: each series copies its template's formatting children
//! and the category cache, so a large last `c:ser` times many column lines would multiply. The
//! rewritten part is therefore capped at [`MAX_CHART_PART_BYTES`], checked before each piece is
//! appended, and a part past it is refused ([`PivotChartError::TooLarge`]); the part's nesting is
//! capped too (CLAUDE.md directive 5; pivots.md "A rewritten `PivotChart` part is capped", a
//! DEVIATION).
//!
//! [`chart_data`]: rustsheet_engine::pivot::chart_data
//! [`ChartSeries::name`]: rustsheet_engine::pivot::ChartSeries::name

use std::fmt::Write as _;
use std::ops::Range;

use quick_xml::events::Event;
use quick_xml::Reader;
use rustsheet_engine::expressions::utils::{number_to_column, quote_name};
use rustsheet_engine::number_format::get_num_fmt;
use rustsheet_engine::pivot::{PivotChartData, PivotLayout, PivotTable};
use rustsheet_engine::types::NumFmt;

use super::escape::escape_xml_text;
use super::worksheets::saved_number;
use crate::import::reference_of;

use super::patch::MAX_DEPTH;

/// Most bytes a rewritten chart part may hold (directive 5). 64 MiB, half the pivot label budget
/// ([`MAX_PIVOT_TEXT_BYTES`](rustsheet_engine::pivot::MAX_PIVOT_TEXT_BYTES)) and a quarter of
/// the import's package budget: Excel caps a chart at 255 series, and 255 series of about 4,000
/// categories at some 60 cached bytes each fit. Excel's own limit on a `PivotChart`'s part is
/// unknown, so a larger rewrite is a DEVIATION (pivots.md "A rewritten `PivotChart` part is
/// capped").
pub const MAX_CHART_PART_BYTES: usize = 64 << 20;

/// The format code a value cache gets when its data field has none, as Excel writes it.
const GENERAL: &str = "General";

/// Why a chart part was not rewritten.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum PivotChartError {
    /// The part is not well-formed XML, or nests deeper than the cap.
    #[error("the chart part does not parse: {0}")]
    Malformed(String),
    /// The part has no `c:pivotSource`: it is not bound to a pivot.
    #[error("the chart part is not bound to a pivot")]
    NotPivotChart,
    /// The part's shape is one the rewrite does not write.
    #[error("the chart part is not rewritten: {0}")]
    Unsupported(&'static str),
    /// A reference past the sheet's last column.
    #[error("the pivot's location has no A1 reference")]
    Location,
    /// The rewritten part would pass [`MAX_CHART_PART_BYTES`].
    #[error("the rewritten chart part would pass {MAX_CHART_PART_BYTES} bytes")]
    TooLarge,
}

/// The rewritten part, refusing to grow past [`MAX_CHART_PART_BYTES`]: every piece is checked
/// before it is appended, so the text never holds more.
struct Output {
    text: String,
}

impl Output {
    fn push(&mut self, piece: &str) -> Result<(), PivotChartError> {
        if self.text.len().saturating_add(piece.len()) > MAX_CHART_PART_BYTES {
            return Err(PivotChartError::TooLarge);
        }
        self.text.push_str(piece);
        Ok(())
    }
}

/// The pivot a chart part plots, as the rewrite needs it.
#[derive(Debug, Clone, Copy)]
pub struct PivotChartFrame<'a> {
    /// The name of the pivot's worksheet.
    pub sheet: &'a str,
    /// The pivot.
    pub table: &'a PivotTable,
    /// Its layout ([`lay_out`](rustsheet_engine::pivot::lay_out)): where its lines are.
    pub layout: &'a PivotLayout,
    /// What it plots ([`chart_data`](rustsheet_engine::pivot::chart_data)) over `layout`.
    pub data: &'a PivotChartData,
    /// The workbook's number formats, for the data fields' `numFmtId`.
    pub num_fmts: &'a [NumFmt],
    /// Whether the pivot collapsed to `#SPILL!` at the location's top-left cell.
    pub collapsed: bool,
}

/// A direct child of a source `c:ser`: its local name and its bytes in the part.
struct Child {
    name: String,
    range: Range<usize>,
}

/// One `c:ser` of the part: its element prefix (`c:`, or empty) and its children.
struct SourceSeries {
    prefix: String,
    children: Vec<Child>,
}

/// The series of a chart part bound to a pivot, and where they sit.
struct Scan {
    series: Vec<SourceSeries>,
    /// From the first `c:ser`'s start to the last one's end.
    span: Range<usize>,
}

/// The local name of a qualified element name, and its prefix with the colon.
fn split_name(qualified: &str) -> (String, String) {
    match qualified.split_once(':') {
        Some((prefix, local)) => (format!("{prefix}:"), local.to_string()),
        None => (String::new(), qualified.to_string()),
    }
}

fn position(reader: &Reader<&[u8]>) -> Result<usize, PivotChartError> {
    usize::try_from(reader.buffer_position())
        .map_err(|_| PivotChartError::Malformed("a part past the address space".into()))
}

/// Reads the series of `xml`: all of them must be children of one element (one chart group),
/// written one after another.
fn scan(xml: &str) -> Result<Scan, PivotChartError> {
    let malformed = |e: quick_xml::Error| PivotChartError::Malformed(e.to_string());
    let mut reader = Reader::from_reader(xml.as_bytes());
    // The element path: (local name, start offset).
    let mut path: Vec<(String, usize)> = Vec::new();
    let mut series: Vec<SourceSeries> = Vec::new();
    let mut ranges: Vec<Range<usize>> = Vec::new();
    let mut parent: Option<usize> = None;
    let mut pivot_source = false;
    // The open `c:ser`'s depth, and its open child's start.
    let mut open: Option<(usize, Option<(String, usize)>)> = None;
    loop {
        let start = position(&reader)?;
        let event = reader.read_event().map_err(malformed)?;
        let end = position(&reader)?;
        match event {
            Event::Start(e) => {
                let (prefix, name) = split_name(e.name().as_ref());
                let depth = path.len();
                if depth >= MAX_DEPTH {
                    return Err(PivotChartError::Malformed("nested too deep".into()));
                }
                if depth == 0 && name != "chartSpace" {
                    return Err(PivotChartError::Malformed(
                        "the root is not chartSpace".into(),
                    ));
                }
                pivot_source |= depth == 1 && name == "pivotSource";
                match &mut open {
                    Some((at, child)) if depth == *at + 1 => *child = Some((name.clone(), start)),
                    None if name == "ser" => {
                        let here = path.last().map(|p| p.1);
                        if parent.is_some_and(|p| Some(p) != here) {
                            return Err(PivotChartError::Unsupported(
                                "series in more than one chart group",
                            ));
                        }
                        parent = here;
                        series.push(SourceSeries {
                            prefix,
                            children: Vec::new(),
                        });
                        ranges.push(start..start);
                        open = Some((depth, None));
                    }
                    _ => {}
                }
                path.push((name, start));
            }
            Event::Empty(e) => {
                let (_, name) = split_name(e.name().as_ref());
                if path.is_empty() && name != "chartSpace" {
                    return Err(PivotChartError::Malformed(
                        "the root is not chartSpace".into(),
                    ));
                }
                pivot_source |= path.len() == 1 && name == "pivotSource";
                if let (Some((at, _)), Some(current)) = (&open, series.last_mut()) {
                    if path.len() == *at + 1 {
                        current.children.push(Child {
                            name,
                            range: start..end,
                        });
                    }
                }
            }
            Event::End(_) => {
                path.pop();
                let depth = path.len();
                match &mut open {
                    Some((at, _)) if depth == *at => {
                        if let Some(range) = ranges.last_mut() {
                            range.end = end;
                        }
                        open = None;
                    }
                    Some((at, child)) if depth == *at + 1 => {
                        if let (Some((name, from)), Some(current)) =
                            (child.take(), series.last_mut())
                        {
                            current.children.push(Child {
                                name,
                                range: from..end,
                            });
                        }
                    }
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if !path.is_empty() {
        return Err(PivotChartError::Malformed(
            "an element is not closed".into(),
        ));
    }
    if !pivot_source {
        return Err(PivotChartError::NotPivotChart);
    }
    let (Some(first), Some(last)) = (ranges.first(), ranges.last()) else {
        return Err(PivotChartError::Unsupported("a chart with no series"));
    };
    // Only white space may sit between two series: the replacement writes them together.
    if ranges.windows(2).any(|w| {
        xml.get(w[0].end..w[1].start)
            .is_none_or(|gap| !gap.trim().is_empty())
    }) {
        return Err(PivotChartError::Unsupported("series not written together"));
    }
    Ok(Scan {
        span: first.start..last.end,
        series,
    })
}

/// `Sheet!$A$1`, or `Sheet!$A$1:$B$2` for more than one cell; rows and columns 0-based.
fn area(sheet: &str, rows: (u32, u32), columns: (u32, u32)) -> Result<String, PivotChartError> {
    let cell = |row: u32, column: u32| -> Result<String, PivotChartError> {
        let column =
            i32::try_from(column.saturating_add(1)).map_err(|_| PivotChartError::Location)?;
        let letters = number_to_column(column).ok_or(PivotChartError::Location)?;
        Ok(format!("${letters}${}", u64::from(row) + 1))
    };
    let from = cell(rows.0, columns.0)?;
    if rows == (rows.0, rows.0) && columns == (columns.0, columns.0) {
        return Ok(format!("{sheet}!{from}"));
    }
    Ok(format!("{sheet}!{from}:{}", cell(rows.1, columns.1)?))
}

/// The three references of one series: `c:tx`, `c:cat`, `c:val`.
fn references(frame: &PivotChartFrame<'_>, line: u32) -> Result<[String; 3], PivotChartError> {
    let sheet = quote_name(frame.sheet);
    let location = &frame.layout.location;
    let geometry = &frame.layout.geometry;
    if frame.collapsed {
        let anchor = area(
            &sheet,
            (location.top, location.top),
            (location.left, location.left),
        )?;
        return Ok([anchor.clone(), anchor.clone(), anchor]);
    }
    let first_data_row = location.top.saturating_add(geometry.first_data_row);
    let column = location
        .left
        .saturating_add(geometry.first_data_column)
        .saturating_add(line);
    // The header rows above the values; one row when there are none (UNVERIFIED).
    let header = (
        location.top,
        first_data_row.saturating_sub(1).max(location.top),
    );
    // The row-label columns left of the values; the first column when there are none
    // (UNVERIFIED: no evidence has a pivot with no row field).
    let labels = (
        location.left,
        location
            .left
            .saturating_add(geometry.first_data_column)
            .saturating_sub(1)
            .max(location.left),
    );
    let rows = (first_data_row, location.bottom.max(first_data_row));
    Ok([
        area(&sheet, header, (column, column))?,
        area(&sheet, rows, labels)?,
        area(&sheet, rows, (column, column))?,
    ])
}

/// The value cache's `c:formatCode` for the series on column line `line`: its data field's
/// `numFmtId` code, `General` without one (t4187 F: `numFmtId` 4 is `#,##0.00`). With Values
/// not on columns every series is the first data field's (UNVERIFIED with Values on rows).
fn format_code(frame: &PivotChartFrame<'_>, line: u32) -> String {
    let field = (frame.layout.columns.get(line as usize))
        .map_or(0, |l| l.data_field)
        .min(u32::try_from(frame.table.data_fields.len().saturating_sub(1)).unwrap_or(0));
    let id = (frame.table.data_fields.get(field as usize))
        .and_then(|d| d.number_format)
        .unwrap_or(0);
    let code = get_num_fmt(i32::try_from(id).unwrap_or(0), frame.num_fmts);
    if code.eq_ignore_ascii_case(GENERAL) {
        GENERAL.to_string()
    } else {
        code
    }
}

fn point(out: &mut String, p: &str, idx: u32, value: &str) {
    let _ = write!(
        out,
        r#"<{p}pt idx="{idx}"><{p}v>{}</{p}v></{p}pt>"#,
        escape_xml_text(value)
    );
}

fn tx_xml(p: &str, reference: &str, name: &str) -> String {
    let mut out = format!(
        r#"<{p}tx><{p}strRef><{p}f>{}</{p}f><{p}strCache><{p}ptCount val="1"/>"#,
        escape_xml_text(reference)
    );
    point(&mut out, p, 0, name);
    let _ = write!(out, "</{p}strCache></{p}strRef></{p}tx>");
    out
}

fn cat_xml(p: &str, reference: &str, data: &PivotChartData) -> String {
    let count = data.categories.len();
    let levels = data.category_levels();
    let f = escape_xml_text(reference);
    if levels.len() <= 1 {
        let mut out = format!(
            r#"<{p}cat><{p}strRef><{p}f>{f}</{p}f><{p}strCache><{p}ptCount val="{count}"/>"#
        );
        for (idx, label) in levels.into_iter().flatten() {
            point(&mut out, p, idx, label);
        }
        let _ = write!(out, "</{p}strCache></{p}strRef></{p}cat>");
        return out;
    }
    let mut out = format!(
        r#"<{p}cat><{p}multiLvlStrRef><{p}f>{f}</{p}f><{p}multiLvlStrCache><{p}ptCount val="{count}"/>"#
    );
    for level in levels {
        let _ = write!(out, "<{p}lvl>");
        for (idx, label) in level {
            point(&mut out, p, idx, label);
        }
        let _ = write!(out, "</{p}lvl>");
    }
    let _ = write!(out, "</{p}multiLvlStrCache></{p}multiLvlStrRef></{p}cat>");
    out
}

fn val_xml(p: &str, reference: &str, code: &str, count: usize, points: &[(u32, f64)]) -> String {
    let mut out = format!(
        r#"<{p}val><{p}numRef><{p}f>{}</{p}f><{p}numCache><{p}formatCode>{}</{p}formatCode><{p}ptCount val="{count}"/>"#,
        escape_xml_text(reference),
        escape_xml_text(code)
    );
    for (idx, value) in points {
        point(&mut out, p, *idx, &saved_number(*value).to_string());
    }
    let _ = write!(out, "</{p}numCache></{p}numRef></{p}val>");
    out
}

/// A copied `c16:uniqueId` for series `k` past the part's last: the template's id with its
/// first group the series' number, as Excel numbers them (`{00000002-C46C-…}` after
/// `{00000001-C46C-…}`, t4187 M). Anything else in the extension is copied as it is.
fn renumbered_extension(raw: &str, k: usize) -> String {
    const KEY: &str = "uniqueId val=\"{";
    let Some(at) = raw.find(KEY).map(|i| i + KEY.len()) else {
        return raw.to_string();
    };
    match raw.get(at..at + 8) {
        Some(group) if group.bytes().all(|b| b.is_ascii_hexdigit()) => {
            format!("{}{:08X}{}", &raw[..at], k + 1, &raw[at + 8..])
        }
        _ => raw.to_string(),
    }
}

/// Series `k` written over `template` into `out`: its own children in their order, with
/// `c:idx`, `c:order`, `c:tx`, `c:cat` and `c:val` replaced, and `c:tx` added after `c:order`
/// when the template has none.
fn series_xml(
    out: &mut Output,
    xml: &str,
    template: &SourceSeries,
    k: usize,
    copied: bool,
    parts: [&str; 3],
) -> Result<(), PivotChartError> {
    let p = template.prefix.as_str();
    let has = |name: &str| template.children.iter().any(|c| c.name == name);
    if !has("cat") || !has("val") {
        return Err(PivotChartError::Unsupported(
            "a series without c:cat and c:val",
        ));
    }
    if !has("order") {
        return Err(PivotChartError::Unsupported("a series without c:order"));
    }
    out.push(&format!("<{p}ser>"))?;
    for child in &template.children {
        let raw = xml
            .get(child.range.clone())
            .ok_or_else(|| PivotChartError::Malformed("a child past the part".into()))?;
        match child.name.as_str() {
            "idx" => out.push(&format!(r#"<{p}idx val="{k}"/>"#))?,
            "order" => {
                out.push(&format!(r#"<{p}order val="{k}"/>"#))?;
                if !has("tx") {
                    out.push(parts[0])?;
                }
            }
            "tx" => out.push(parts[0])?,
            "cat" => out.push(parts[1])?,
            "val" => out.push(parts[2])?,
            "extLst" if copied => out.push(&renumbered_extension(raw, k))?,
            _ => out.push(raw)?,
        }
    }
    out.push(&format!("</{p}ser>"))
}

/// The sheet and the name of the pivot a chart part is bound to, as its `c:pivotSource` names it
/// (`[file]Sheet!Name`; Excel writes its own file name in the brackets, t4187); `None` when the
/// part names none or does not parse. A sheet written in quotes, `'My sheet'!Name`, is read with
/// its doubled quotes undone (UNVERIFIED: the evidence's names, `C!P_C`, are bare; T-4991).
pub(super) fn pivot_source(chart_xml: &str) -> Option<(String, String)> {
    let mut reader = Reader::from_str(chart_xml);
    let mut depth = 0usize;
    // Whether the open element is `c:pivotSource` (depth 2) or its `c:name` (depth 3).
    let (mut in_source, mut in_name) = (false, false);
    let mut name = String::new();
    loop {
        match reader.read_event().ok()? {
            Event::Start(e) => {
                depth += 1;
                if depth >= MAX_DEPTH {
                    return None;
                }
                let local = e.local_name();
                in_source |= depth == 2 && local.as_ref() == "pivotSource";
                in_name = in_source && depth == 3 && local.as_ref() == "name";
            }
            Event::End(_) => {
                if in_name {
                    return split_pivot_source(&name);
                }
                in_source &= depth != 2;
                depth = depth.saturating_sub(1);
            }
            Event::Text(t) if in_name => {
                name.push_str(&t.xml_content(quick_xml::XmlVersion::Implicit1_0));
            }
            Event::GeneralRef(r) if in_name => name.push_str(&reference_of(&r).ok()?),
            Event::Eof => return None,
            _ => {}
        }
    }
}

/// `[file]Sheet!Name` as (sheet, name).
fn split_pivot_source(source: &str) -> Option<(String, String)> {
    let rest = match source.strip_prefix('[') {
        Some(bracketed) => bracketed.split_once(']')?.1,
        None => source,
    };
    let Some(quoted) = rest.strip_prefix('\'') else {
        let (sheet, name) = rest.split_once('!')?;
        return Some((sheet.to_string(), name.to_string()));
    };
    let mut sheet = String::new();
    let mut chars = quoted.char_indices().peekable();
    while let Some((at, c)) = chars.next() {
        if c != '\'' {
            sheet.push(c);
        } else if chars.next_if(|&(_, next)| next == '\'').is_some() {
            sheet.push('\'');
        } else {
            let name = quoted.get(at + 1..)?.strip_prefix('!')?;
            return Some((sheet, name.to_string()));
        }
    }
    None
}

/// `chart_xml`, a chart part bound to the pivot of `frame`, with its series written from the
/// pivot (see the module documentation).
///
/// # Errors
/// [`PivotChartError`] when the part does not parse, is not bound to a pivot, or has a shape
/// the rewrite does not write: series in two chart groups, none, or one without `c:order`,
/// `c:cat` and `c:val` (a scatter series' `c:xVal`/`c:yVal`; Excel offers no scatter
/// `PivotChart`); [`PivotChartError::TooLarge`] past [`MAX_CHART_PART_BYTES`].
pub fn rewrite_pivot_chart(
    chart_xml: &str,
    frame: &PivotChartFrame<'_>,
) -> Result<String, PivotChartError> {
    let scan = scan(chart_xml)?;
    let count = frame.data.categories.len();
    let mut out = Output {
        text: String::new(),
    };
    out.push(&chart_xml[..scan.span.start])?;
    // The category cache is the same for every series: made once per element prefix.
    let mut categories: Option<(String, String)> = None;
    for (k, plotted) in frame.data.series.iter().enumerate() {
        let copied = k >= scan.series.len();
        let template = scan
            .series
            .get(k)
            .or(scan.series.last())
            .ok_or(PivotChartError::Unsupported("a chart with no series"))?;
        let p = template.prefix.as_str();
        let [tx, cat, val] = references(frame, plotted.line)?;
        // `Total` with no label, the label with one; UNVERIFIED (T-4938) with several.
        let name = plotted
            .name()
            .map_or_else(|| plotted.labels.join(" "), str::to_string);
        let points: Vec<(u32, f64)> = if frame.collapsed {
            vec![(0, 0.0)]
        } else {
            (0u32..)
                .zip(&plotted.values)
                .filter_map(|(i, v)| match v {
                    Some(Ok(v)) => Some((i, *v)),
                    // UNVERIFIED (T-4953): an error point is left out, as an empty one is.
                    Some(Err(_)) | None => None,
                })
                .collect()
        };
        if categories.as_ref().is_none_or(|(prefix, _)| prefix != p) {
            categories = Some((p.to_string(), cat_xml(p, &cat, frame.data)));
        }
        let cat_part = categories.as_ref().map_or("", |(_, xml)| xml.as_str());
        let tx_part = tx_xml(p, &tx, &name);
        let val_part = val_xml(p, &val, &format_code(frame, plotted.line), count, &points);
        series_xml(
            &mut out,
            chart_xml,
            template,
            k,
            copied,
            [&tx_part, cat_part, &val_part],
        )?;
    }
    out.push(&chart_xml[scan.span.end..])?;
    Ok(out.text)
}

#[cfg(test)]
mod tests;
