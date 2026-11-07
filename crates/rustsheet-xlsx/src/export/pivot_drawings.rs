//! `PivotChart` drawings kept by a save that regenerates the package (T-4950).
//!
//! The regenerating export writes no drawing of its own, so a loaded workbook whose save cannot
//! keep the source (SPEC-0034's `Regenerated` tier: a `refreshOnLoad` pivot rendered on the first
//! recalculation is enough) used to lose its charts. A [`DrawingPlan`] replays from the source
//! each sheet's drawing whose closure is pivot charts only: the drawing, its charts and each
//! chart's `chartStyle` and `chartColorStyle` parts, copied under export-owned names
//! (`xl/drawings/drawingN.xml`, `xl/charts/chartN.xml`, `styleN.xml`, `colorsN.xml`) with their
//! relationships, the sheet's `<drawing r:id>` and the content-type overrides. Each chart's series
//! are written by [`rewrite_pivot_chart`] from the pivot its `c:pivotSource` names, as Excel for
//! the web writes them when it renders the pivot (docs/excel-semantics/pivots.md "Excel for the web
//! rewrites a `PivotChart`'s caches when it renders the pivot"); every other byte is the source's.
//! This is the ownership matrix's `Replay` row for charts and drawings, and its `ReplayIfClean`
//! row for a drawing after a structural edit (docs/research/xlsx-round-trip-2026-11-04.md,
//! decision 0275).
//!
//! A drawing is dropped, with a log line, when anything in it is not a pivot chart the rewrite
//! writes: an anchor holding a picture, shape, group or anything but a chart frame, a
//! relationship of another type (an image, a hyperlink, chart user shapes, an embedded workbook),
//! an `r:` reference its relationships do not hold, a chart not bound to a pivot or bound to one
//! the export does not write, and any [`rewrite_pivot_chart`] error. So is the drawing of a sheet
//! that was renamed, moved or deleted, or whose structure changed (rows, columns, merges,
//! anything but cells: [`sheet_structure_same`]), since its anchors and references may no longer
//! say where it is; how a drawing moves with a structural edit is T-4567's spec, still open. The
//! closure rule is that of the matrix: what is kept never points outside what is kept.
//!
//! The source is read under the import's limits ([`ImportLimits`], one byte budget for every
//! part read), each rewritten chart is capped by the rewrite ([`MAX_CHART_PART_BYTES`]), every
//! kept part together by [`MAX_KEPT_BYTES`], and the pivots' work by one [`MAX_PIVOT_WORK`]
//! (directive 5); past a budget, a drawing is dropped.
//!
//! [`MAX_CHART_PART_BYTES`]: super::MAX_CHART_PART_BYTES

use std::collections::BTreeSet;
use std::io::{Cursor, Read, Seek, Write};

use quick_xml::events::{BytesStart, Event};
use quick_xml::name::ResolveResult;
use quick_xml::{NsReader, XmlVersion};
use rustsheet_engine::expressions::token::Error;
use rustsheet_engine::pivot::{chart_data, compute_on, lay_out, PivotTable, MAX_PIVOT_WORK};
use rustsheet_engine::types::{Cell, Worksheet};
use rustsheet_engine::Model;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use super::patch::{sheet_structure_same, MAX_DEPTH};
use super::pivot_charts::{pivot_source, rewrite_pivot_chart, PivotChartFrame};
use super::pivots::{log_drop, relationships_xml, PivotPlan, REL_NS};
use crate::error::XlsxError;
use crate::import::{
    part_folder_and_rels, part_paths, part_relationships, resolve_relative_target, ImportLimits,
    Package,
};

const DRAWING_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/drawing";
const CHART_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/chart";
const CHART_STYLE_RELATIONSHIP: &str =
    "http://schemas.microsoft.com/office/2011/relationships/chartStyle";
const CHART_COLORS_RELATIONSHIP: &str =
    "http://schemas.microsoft.com/office/2011/relationships/chartColorStyle";
const DRAWING_TYPE: &str = "application/vnd.openxmlformats-officedocument.drawing+xml";
const CHART_TYPE: &str = "application/vnd.openxmlformats-officedocument.drawingml.chart+xml";
const CHART_STYLE_TYPE: &str = "application/vnd.ms-office.chartstyle+xml";
const CHART_COLORS_TYPE: &str = "application/vnd.ms-office.chartcolorstyle+xml";
/// The `a:graphicData` uri of a chart frame.
const CHART_URI: &str = "http://schemas.openxmlformats.org/drawingml/2006/chart";

/// Most bytes every kept part may hold together (directive 5): the import's package budget
/// ([`ImportLimits::max_uncompressed_bytes`], 256 MiB), which the source parts were read within.
pub(super) const MAX_KEPT_BYTES: usize = 256 << 20;

/// The children an anchor of a drawing that holds charts only may have.
const ANCHORS: [&str; 3] = ["twoCellAnchor", "oneCellAnchor", "absoluteAnchor"];
const ANCHOR_CHILDREN: [&str; 6] = ["from", "to", "pos", "ext", "graphicFrame", "clientData"];

/// One kept part: its name in the saved package, its content type (none for a relationships
/// part, which the package's `rels` default covers), its bytes, and the source part it was copied
/// or rewritten from (none for a relationships part, which is written anew), which the save report
/// leaves out of its drops (T-4562).
struct KeptPart {
    name: String,
    content_type: Option<&'static str>,
    bytes: Vec<u8>,
    source: Option<String>,
}

/// One sheet's kept drawing: the model sheet's index, its part number and its parts, the drawing
/// first.
struct KeptDrawing {
    sheet: usize,
    number: usize,
    parts: Vec<KeptPart>,
}

/// The drawings a regenerated save keeps, decided once so the sheets, their relationships and
/// the content types agree.
#[derive(Default)]
pub(super) struct DrawingPlan {
    drawings: Vec<KeptDrawing>,
}

/// What the kept drawings may still spend, and what they took.
struct Budget<'a> {
    model: &'a Model,
    pristine: &'a Model,
    pivots: &'a PivotPlan,
    work: u64,
    bytes_left: usize,
    /// The chart parts kept so far, by part key: a chart two drawings share is not copied twice.
    charts: BTreeSet<String>,
    next_chart: usize,
}

/// The source package, read within the import's limits.
type Source<'a> = Package<Cursor<&'a [u8]>>;

impl DrawingPlan {
    /// The drawings of `source` that a save of `model` keeps. `pristine` is `source` as imported,
    /// so its sheets are the source's in order; `pivots` the pivots the save writes.
    pub(super) fn new(
        model: &Model,
        pristine: &Model,
        source: &[u8],
        pivots: &PivotPlan,
    ) -> DrawingPlan {
        let mut plan = DrawingPlan::default();
        let limits = ImportLimits::default();
        let sheets = &pristine.workbook.worksheets;
        let paths = match part_paths(source, &limits) {
            Ok(paths) if paths.worksheets.len() == sheets.len() => paths,
            _ => {
                log_drop(
                    "drawings",
                    "",
                    "the source's sheets do not map to its parts",
                );
                return plan;
            }
        };
        let Ok(mut archive) = Package::new(Cursor::new(source), limits) else {
            log_drop("drawings", "", "the source package does not open");
            return plan;
        };
        let mut budget = Budget {
            model,
            pristine,
            pivots,
            work: MAX_PIVOT_WORK,
            bytes_left: MAX_KEPT_BYTES,
            charts: BTreeSet::new(),
            next_chart: 1,
        };
        for (index, (sheet, path)) in sheets.iter().zip(&paths.worksheets).enumerate() {
            let number = plan.drawings.len() + 1;
            let first_chart = budget.next_chart;
            match keep_sheet(&mut budget, &mut archive, index, sheet, path, number) {
                Ok(Some(parts)) => plan.drawings.push(KeptDrawing {
                    sheet: index,
                    number,
                    parts,
                }),
                Ok(None) => {}
                Err(reason) => {
                    // A dropped drawing's charts give their numbers back.
                    budget.next_chart = first_chart;
                    log_drop("drawing of sheet", &sheet.name, &reason);
                }
            }
        }
        plan
    }

    /// The content-type overrides of the kept parts.
    pub(super) fn content_types(&self) -> Vec<String> {
        (self.drawings.iter().flat_map(|d| &d.parts))
            .filter_map(|part| {
                let content_type = part.content_type?;
                Some(format!(
                    r#"<Override PartName="/{}" ContentType="{content_type}"/>"#,
                    part.name
                ))
            })
            .collect()
    }

    /// The relationship of the model sheet at `sheet` to its kept drawing, `rId{id}`; `None`
    /// when it keeps none.
    pub(super) fn sheet_relationship(&self, sheet: usize, id: usize) -> Option<String> {
        let drawing = self.drawings.iter().find(|d| d.sheet == sheet)?;
        Some(format!(
            r#"<Relationship Id="rId{id}" Type="{DRAWING_RELATIONSHIP}" Target="../drawings/drawing{}.xml"/>"#,
            drawing.number
        ))
    }

    /// The source parts the kept drawings were copied or rewritten from, by part key.
    pub(super) fn sources(&self) -> BTreeSet<String> {
        (self.drawings.iter().flat_map(|d| &d.parts))
            .filter_map(|p| p.source.as_deref().map(crate::import::part_key))
            .collect()
    }

    /// Writes the kept parts.
    pub(super) fn write<W: Write + Seek>(
        &self,
        zip: &mut ZipWriter<W>,
        options: SimpleFileOptions,
    ) -> Result<(), XlsxError> {
        let mut folders = BTreeSet::new();
        for part in self.drawings.iter().flat_map(|d| &d.parts) {
            if let Some((folder, _)) = part.name.rsplit_once('/') {
                if folders.insert(folder) {
                    zip.add_directory(folder, options)?;
                }
            }
            zip.start_file(part.name.as_str(), options)?;
            zip.write_all(&part.bytes)?;
        }
        Ok(())
    }
}

/// The kept parts of the drawing of the source sheet at `index` (`path` its part), numbered
/// `number`; `None` when the sheet has no drawing.
///
/// # Errors
/// Why the drawing is dropped.
fn keep_sheet(
    budget: &mut Budget<'_>,
    archive: &mut Source<'_>,
    index: usize,
    sheet: &Worksheet,
    path: &str,
    number: usize,
) -> Result<Option<Vec<KeptPart>>, String> {
    let (folder, rels_path) = part_folder_and_rels(path);
    let rels = part_relationships(archive, &rels_path).map_err(|e| e.to_string())?;
    let mut drawings = rels
        .iter()
        .filter(|(_, r)| r.rel_type.ends_with("/drawing"));
    let Some((_, drawing)) = drawings.next() else {
        return Ok(None);
    };
    if drawings.next().is_some() {
        return Err("the sheet relates to more than one drawing".into());
    }
    if drawing.external || drawing.rel_type != DRAWING_RELATIONSHIP {
        return Err("not a drawing part of the package".into());
    }
    let live = (budget.model.workbook.worksheets.get(index))
        .filter(|live| live.sheet_id == sheet.sheet_id)
        .ok_or("the sheet was moved or deleted")?;
    if live.name != sheet.name {
        return Err("the sheet was renamed".into());
    }
    if !sheet_structure_same(live, sheet) {
        return Err("the sheet's structure changed".into());
    }
    let drawing_path =
        resolve_relative_target(folder, &drawing.target).ok_or("the drawing is outside xl")?;
    let parts = keep_drawing(budget, archive, &drawing_path, number)?;
    let size = parts.iter().map(|p| p.bytes.len()).sum::<usize>();
    budget.bytes_left = (budget.bytes_left.checked_sub(size))
        .ok_or("the kept drawings would pass their byte budget")?;
    Ok(Some(parts))
}

/// The drawing at `path` and its charts as kept parts, the drawing numbered `number`.
fn keep_drawing(
    budget: &mut Budget<'_>,
    archive: &mut Source<'_>,
    path: &str,
    number: usize,
) -> Result<Vec<KeptPart>, String> {
    let xml = read(archive, path)?;
    let (folder, rels_path) = part_folder_and_rels(path);
    let rels = part_relationships(archive, &rels_path).map_err(|e| e.to_string())?;
    let ids = relationship_ids(&rels)?;
    check_part(&xml, &ids, true)?;
    let mut charts = Vec::new();
    let mut relationships = Vec::new();
    for (id, rel) in &rels {
        if rel.external || rel.rel_type != CHART_RELATIONSHIP {
            return Err(format!(
                "it relates to a part of type {}",
                short_kind(&rel.rel_type)
            ));
        }
        let chart_path =
            resolve_relative_target(folder, &rel.target).ok_or("a chart is outside xl")?;
        let chart = budget.next_chart;
        budget.next_chart += 1;
        charts.extend(keep_chart(budget, archive, &chart_path, chart)?);
        relationships.push((
            id.as_str(),
            CHART_RELATIONSHIP,
            format!("../charts/chart{chart}.xml"),
        ));
    }
    let name = format!("xl/drawings/drawing{number}.xml");
    let mut parts = vec![KeptPart {
        name: name.clone(),
        content_type: Some(DRAWING_TYPE),
        bytes: xml.into_bytes(),
        source: Some(path.to_string()),
    }];
    parts.push(relationships_part(&name, &relationships));
    parts.extend(charts);
    Ok(parts)
}

/// The chart at `path`, its series rewritten from its pivot, and its style parts, numbered
/// `number`.
fn keep_chart(
    budget: &mut Budget<'_>,
    archive: &mut Source<'_>,
    path: &str,
    number: usize,
) -> Result<Vec<KeptPart>, String> {
    if !budget.charts.insert(crate::import::part_key(path)) {
        return Err("two drawings share a chart".into());
    }
    let xml = read(archive, path)?;
    let (folder, rels_path) = part_folder_and_rels(path);
    let rels = part_relationships(archive, &rels_path).map_err(|e| e.to_string())?;
    let ids = relationship_ids(&rels)?;
    check_part(&xml, &ids, false)?;
    let mut parts = Vec::new();
    let mut relationships = Vec::new();
    for (id, rel) in &rels {
        let (file, content_type) = match rel.rel_type.as_str() {
            CHART_STYLE_RELATIONSHIP if !rel.external => ("style", CHART_STYLE_TYPE),
            CHART_COLORS_RELATIONSHIP if !rel.external => ("colors", CHART_COLORS_TYPE),
            kind => {
                return Err(format!(
                    "a chart relates to a part of type {}",
                    short_kind(kind)
                ))
            }
        };
        if relationships
            .iter()
            .any(|(_, kind, _)| *kind == rel.rel_type)
        {
            return Err(format!("a chart relates to two {file} parts"));
        }
        let style_path =
            resolve_relative_target(folder, &rel.target).ok_or("a chart style is outside xl")?;
        let target = format!("{file}{number}.xml");
        parts.push(KeptPart {
            name: format!("xl/charts/{target}"),
            content_type: Some(content_type),
            bytes: read(archive, &style_path)?.into_bytes(),
            source: Some(style_path),
        });
        relationships.push((id.as_str(), rel.rel_type.as_str(), target));
    }
    let rewritten = rewrite(budget, &xml)?;
    let name = format!("xl/charts/chart{number}.xml");
    let mut kept = vec![KeptPart {
        name: name.clone(),
        content_type: Some(CHART_TYPE),
        bytes: rewritten.into_bytes(),
        source: Some(path.to_string()),
    }];
    if !relationships.is_empty() {
        kept.push(relationships_part(&name, &relationships));
    }
    kept.extend(parts);
    Ok(kept)
}

/// `xml`, a chart part, with its series written from the pivot its `c:pivotSource` names, as
/// the save writes that pivot.
fn rewrite(budget: &mut Budget<'_>, xml: &str) -> Result<String, String> {
    let (sheet_name, pivot_name) = pivot_source(xml).ok_or("a chart is not bound to a pivot")?;
    let workbook = &budget.model.workbook;
    // The name is the source's: its sheet is found among the source's sheets, then followed to
    // the live one, which must not have been renamed (its references would name the old sheet).
    // The sheet matches ignoring case, as sheet names do, and the pivot exactly (both
    // UNVERIFIED for a chart's `c:pivotSource`, T-4991).
    let source_sheet = (budget.pristine.workbook.worksheets.iter())
        .find(|w| w.name.to_uppercase() == sheet_name.to_uppercase())
        .ok_or("a chart's pivot names no sheet of the source")?;
    let sheet = (workbook.worksheets.iter())
        .find(|w| w.sheet_id == source_sheet.sheet_id && w.name == source_sheet.name)
        .ok_or("a chart's pivot's sheet was renamed or deleted")?;
    let (index, table) = (workbook.pivot_tables.iter().enumerate())
        .find(|(_, p)| p.sheet_id == sheet.sheet_id && p.name == pivot_name)
        .ok_or("a chart's pivot is gone")?;
    if !budget.pivots.exports_pivot(index) {
        return Err("a chart's pivot is not written".into());
    }
    let cache = (workbook.pivot_caches.get(table.cache as usize)).ok_or("a pivot has no cache")?;
    let work = &mut budget.work;
    let error = |e: rustsheet_engine::pivot::PivotComputeError| e.to_string();
    let result = compute_on(table, cache, budget.model.pivot_today(), work).map_err(error)?;
    let layout = lay_out(table, cache, &result, work).map_err(error)?;
    let num_fmts = &workbook.styles.num_fmts;
    let data = chart_data(table, cache, &result, &layout, num_fmts, work).map_err(error)?;
    let collapsed = collapsed(sheet, table);
    if !collapsed && (layout.location != table.location || layout.geometry != table.geometry) {
        return Err("a chart's pivot's cells are not the engine's layout of it".into());
    }
    let frame = PivotChartFrame {
        sheet: &sheet.name,
        table,
        layout: &layout,
        data: &data,
        num_fmts,
        collapsed,
    };
    rewrite_pivot_chart(xml, &frame).map_err(|e| e.to_string())
}

/// Whether `table` collapsed to `#SPILL!` on `sheet`: the grid saves a collapsed pivot's location
/// as its anchor and writes the error there (`PivotPlacement::spilled`, pivots.md "A pivot that
/// would meet a foreign cell collapses to #SPILL!").
fn collapsed(sheet: &Worksheet, table: &PivotTable) -> bool {
    let location = &table.location;
    let one_based = |n: u32| n.checked_add(1).and_then(|n| i32::try_from(n).ok());
    let (Some(row), Some(column)) = (one_based(location.top), one_based(location.left)) else {
        return false;
    };
    location.top == location.bottom
        && location.left == location.right
        && matches!(
            sheet.cell(row, column),
            Some(Cell::ErrorCell {
                ei: Error::SPILL,
                ..
            })
        )
}

/// The part `name` of the source, as text.
fn read(archive: &mut Source<'_>, name: &str) -> Result<String, String> {
    let mut bytes = Vec::new();
    (archive.open(name).map_err(|e| format!("{name}: {e}"))?)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("{name}: {e}"))?;
    String::from_utf8(bytes).map_err(|_| format!("{name} is not UTF-8"))
}

/// The ids of a part's relationships, each once.
fn relationship_ids(
    rels: &[(String, crate::import::Relationship)],
) -> Result<BTreeSet<String>, String> {
    let mut ids = BTreeSet::new();
    for (id, _) in rels {
        if !ids.insert(id.clone()) {
            return Err(format!("a relationship id repeats: {id}"));
        }
    }
    Ok(ids)
}

/// The last segment of a relationship type, for a log line.
fn short_kind(kind: &str) -> &str {
    kind.rsplit('/').next().unwrap_or(kind)
}

/// The relationships part of the kept part `name`: (id, type, target) each, the ids the source's
/// so the copied part's references hold.
fn relationships_part(name: &str, relationships: &[(&str, &str, String)]) -> KeptPart {
    let (_, rels_name) = part_folder_and_rels(name);
    let relationships: Vec<(&str, &str, &str)> = (relationships.iter())
        .map(|(id, kind, target)| (*id, *kind, target.as_str()))
        .collect();
    KeptPart {
        name: rels_name,
        content_type: None,
        bytes: relationships_xml(&relationships).into_bytes(),
        source: None,
    }
}

/// Checks that `xml` nests at most [`MAX_DEPTH`] deep and that every attribute in the
/// relationships namespace (`r:id`, `r:embed`, ...) names one of `ids`; with `drawing`, also that
/// the part is a drawing whose anchors each hold a chart frame and nothing else.
///
/// # Errors
/// Why the part is not kept.
fn check_part(xml: &str, ids: &BTreeSet<String>, drawing: bool) -> Result<(), String> {
    let mut reader = NsReader::from_str(xml);
    // The open elements' local names.
    let mut path: Vec<String> = Vec::new();
    loop {
        let event = reader.read_event().map_err(|e| e.to_string())?;
        let (element, empty) = match &event {
            Event::Start(e) => (e, false),
            Event::Empty(e) => (e, true),
            Event::End(_) => {
                path.pop();
                continue;
            }
            Event::Eof => break,
            // No package part declares a document type; one could declare entities.
            Event::DocType(_) => return Err("a part declares a document type".into()),
            _ => continue,
        };
        if path.len() >= MAX_DEPTH {
            return Err("a part nests too deep".into());
        }
        if drawing {
            drawing_slot(&path, element)?;
        }
        for attribute in element.attributes() {
            let attribute = attribute.map_err(|e| e.to_string())?;
            let (namespace, _) = reader.resolver().resolve_attribute(attribute.key);
            if matches!(namespace, ResolveResult::Bound(ns) if ns.as_ref() == REL_NS) {
                let id = (attribute.normalized_value(XmlVersion::Implicit1_0))
                    .map_err(|e| e.to_string())?;
                if !ids.contains(id.as_ref()) {
                    return Err(format!("a reference {id:?} has no relationship"));
                }
            }
        }
        if !empty {
            path.push(element.local_name().as_ref().to_string());
        }
    }
    if path.is_empty() {
        Ok(())
    } else {
        Err("a part has an unclosed element".into())
    }
}

/// Whether `element`, under the open elements `path`, may be in a drawing that holds charts
/// only: the root `wsDr`, its anchors, an anchor's position, frame and client data, and a
/// `graphicData` only of a chart.
fn drawing_slot(path: &[String], element: &BytesStart<'_>) -> Result<(), String> {
    let local = element.local_name();
    let local = local.as_ref();
    let allowed = match path.len() {
        0 => local == "wsDr",
        1 => ANCHORS.contains(&local),
        2 => ANCHOR_CHILDREN.contains(&local),
        _ => true,
    };
    if !allowed {
        return Err(format!("it holds a {local} beside its charts"));
    }
    if local == "graphicData" {
        let uri = (element
            .try_get_attribute("uri")
            .map_err(|e| e.to_string())?)
        .map(|a| a.value.into_owned());
        if uri.as_deref() != Some(CHART_URI) {
            return Err("it holds a graphic frame that is not a chart".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
