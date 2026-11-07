//! Pivot caches and pivot tables in the exported package (SPEC-0032 "Export", T-4176).
//!
//! Each cache the model holds is written as a `pivotCacheDefinition` part, listed in the
//! workbook's `<pivotCaches>`, with its `pivotCacheRecords` part ([`cache`]); each pivot table as
//! a `pivotTableDefinition` part related from its worksheet ([`table`]). They used to be dropped
//! by the regenerating export; the source-preserving tiers (SPEC-0034) still keep a loaded
//! package's own pivot parts byte for byte while the pivots are unchanged, so these parts are
//! what a pivot is saved as once anything about it changed (the M15 ownership matrix's `Replay`
//! row for pivots, docs/research/xlsx-round-trip-2026-11-04.md, now `Modelled`).
//!
//! A cache the export cannot write faithfully is dropped with every pivot on it, never written
//! half: a grouped field (T-4734) or no worksheet source. A pivot whose sheet or cache is gone is
//! dropped too. Each drop is logged to stderr, as the importer logs a skipped part.
//!
//! Parts are numbered from 1 in the order written, and cache `n` has `cacheId` `n`: ids only
//! cross-reference (SPEC-0033 D4), and the import maps a pivot to its cache by them.
// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::print_stderr)]
// LINT-DEBT-END

mod cache;
mod table;
#[cfg(test)]
mod tests;

use std::fmt::Write as _;
use std::io::{Seek, Write};

use rustsheet_engine::pivot::{
    compute, lay_out, OpaqueEntry, PivotLayout, PivotTable, MAX_PIVOT_WORK, MAX_RECORD_VALUES,
};
use rustsheet_engine::types::Workbook;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::error::XlsxError;

pub(super) const MAIN_NS: &str = "http://schemas.openxmlformats.org/spreadsheetml/2006/main";
pub(super) const REL_NS: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const CACHE_DEFINITION_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.pivotCacheDefinition+xml";
const CACHE_RECORDS_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.pivotCacheRecords+xml";
const TABLE_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.pivotTable+xml";
const CACHE_DEFINITION_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/pivotCacheDefinition";
const CACHE_RECORDS_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/pivotCacheRecords";
const TABLE_RELATIONSHIP: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/pivotTable";
const RELATIONSHIPS_NS: &str = "http://schemas.openxmlformats.org/package/2006/relationships";

/// One exported cache: its model index and its definition part.
struct ExportedCache {
    index: usize,
    definition: String,
}

/// One exported pivot table: its worksheet's index, its cache's part number and its part.
struct ExportedTable {
    sheet: usize,
    cache_part: usize,
    xml: String,
}

/// The pivot parts of an export, decided once so the workbook, its relationships, the sheets'
/// relationships and the content types agree.
pub(super) struct PivotPlan {
    caches: Vec<ExportedCache>,
    tables: Vec<ExportedTable>,
}

impl PivotPlan {
    /// The pivot parts of `workbook`.
    pub(super) fn new(workbook: &Workbook) -> PivotPlan {
        let date_system = workbook.settings.date_system;
        // Per model cache, its part number once written.
        let mut cache_parts = vec![None; workbook.pivot_caches.len()];
        let mut caches = Vec::new();
        // The record values the exported caches may still hold, over all of them.
        let mut values_left = MAX_RECORD_VALUES;
        for (index, pivot_cache) in workbook.pivot_caches.iter().enumerate() {
            let written = cache::check(pivot_cache, date_system, values_left).and_then(|values| {
                cache::definition_xml(pivot_cache, date_system).map(|xml| (values, xml))
            });
            match written {
                Ok((values, definition)) => {
                    values_left = values_left.saturating_sub(values);
                    caches.push(ExportedCache { index, definition });
                    if let Some(part) = cache_parts.get_mut(index) {
                        *part = Some(caches.len());
                    }
                }
                Err(reason) => log_drop("pivot cache", &index.to_string(), &reason.0),
            }
        }
        // One budget for every pivot's layout lines (directive 5): past it, a pivot keeps the
        // lines it was read with.
        let mut work = MAX_PIVOT_WORK;
        let mut tables = Vec::new();
        for (sheet, worksheet) in workbook.worksheets.iter().enumerate() {
            for pivot in (workbook.pivot_tables.iter()).filter(|p| p.sheet_id == worksheet.sheet_id)
            {
                let index = pivot.cache as usize;
                let (Some(Some(part)), Some(pivot_cache)) =
                    (cache_parts.get(index), workbook.pivot_caches.get(index))
                else {
                    log_drop("pivot table", &pivot.name, "its cache is not exported");
                    continue;
                };
                let lines = layout_lines(pivot, pivot_cache, &mut work);
                // At most one cache per model cache, so the id fits.
                let cache_id = u32::try_from(*part).unwrap_or(u32::MAX);
                tables.push(ExportedTable {
                    sheet,
                    cache_part: *part,
                    xml: table::table_xml(pivot, pivot_cache, cache_id, lines.as_ref()),
                });
            }
        }
        let on_no_sheet = (workbook.pivot_tables.iter())
            .filter(|p| !workbook.worksheets.iter().any(|w| w.sheet_id == p.sheet_id));
        for pivot in on_no_sheet {
            log_drop(
                "pivot table",
                &pivot.name,
                "its sheet is not in the workbook",
            );
        }
        PivotPlan { caches, tables }
    }

    /// The content-type overrides of the pivot parts.
    pub(super) fn content_types(&self) -> Vec<String> {
        let mut types = Vec::with_capacity(2 * self.caches.len() + self.tables.len());
        for part in 1..=self.caches.len() {
            types.push(format!(
                r#"<Override PartName="/xl/pivotCache/pivotCacheDefinition{part}.xml" ContentType="{CACHE_DEFINITION_TYPE}"/>"#
            ));
            types.push(format!(
                r#"<Override PartName="/xl/pivotCache/pivotCacheRecords{part}.xml" ContentType="{CACHE_RECORDS_TYPE}"/>"#
            ));
        }
        for part in 1..=self.tables.len() {
            types.push(format!(
                r#"<Override PartName="/xl/pivotTables/pivotTable{part}.xml" ContentType="{TABLE_TYPE}"/>"#
            ));
        }
        types
    }

    /// The workbook's relationships to the caches, `rId{first_id}` on.
    pub(super) fn workbook_relationships(&self, first_id: usize) -> Vec<String> {
        (1..=self.caches.len())
            .map(|part| {
                format!(
                    r#"<Relationship Id="rId{}" Type="{CACHE_DEFINITION_RELATIONSHIP}" Target="pivotCache/pivotCacheDefinition{part}.xml"/>"#,
                    first_id + part - 1
                )
            })
            .collect()
    }

    /// The workbook's `<pivotCaches>`, its relationships `rId{first_id}` on; empty without a
    /// cache.
    pub(super) fn workbook_pivot_caches(&self, first_id: usize) -> String {
        if self.caches.is_empty() {
            return String::new();
        }
        let mut xml = String::from("<pivotCaches>");
        for part in 1..=self.caches.len() {
            let _ = write!(
                xml,
                r#"<pivotCache cacheId="{part}" r:id="rId{}"/>"#,
                first_id + part - 1
            );
        }
        xml.push_str("</pivotCaches>");
        xml
    }

    /// The relationships of the worksheet at `sheet` to its pivot tables, `rId{first_id}` on.
    pub(super) fn sheet_relationships(&self, sheet: usize, first_id: usize) -> Vec<String> {
        (1..)
            .zip(&self.tables)
            .filter(|(_, table)| table.sheet == sheet)
            .zip(first_id..)
            .map(|((part, _), id)| {
                format!(
                    r#"<Relationship Id="rId{id}" Type="{TABLE_RELATIONSHIP}" Target="../pivotTables/pivotTable{part}.xml"/>"#
                )
            })
            .collect()
    }

    /// Writes the cache and pivot table parts and their relationships.
    pub(super) fn write<W: Write + Seek>(
        &self,
        workbook: &Workbook,
        zip: &mut ZipWriter<W>,
        options: SimpleFileOptions,
    ) -> Result<(), XlsxError> {
        if !self.caches.is_empty() {
            zip.add_directory("xl/pivotCache", options)?;
            zip.add_directory("xl/pivotCache/_rels", options)?;
        }
        for (part, exported) in (1..).zip(&self.caches) {
            zip.start_file(
                format!("xl/pivotCache/pivotCacheDefinition{part}.xml"),
                options,
            )?;
            zip.write_all(exported.definition.as_bytes())?;
            zip.start_file(
                format!("xl/pivotCache/_rels/pivotCacheDefinition{part}.xml.rels"),
                options,
            )?;
            zip.write_all(
                relationships(
                    CACHE_RECORDS_RELATIONSHIP,
                    &format!("pivotCacheRecords{part}.xml"),
                )
                .as_bytes(),
            )?;
            if let Some(pivot_cache) = workbook.pivot_caches.get(exported.index) {
                zip.start_file(
                    format!("xl/pivotCache/pivotCacheRecords{part}.xml"),
                    options,
                )?;
                zip.write_all(cache::records_xml(pivot_cache).as_bytes())?;
            }
        }
        if !self.tables.is_empty() {
            zip.add_directory("xl/pivotTables", options)?;
            zip.add_directory("xl/pivotTables/_rels", options)?;
        }
        for (part, exported) in (1..).zip(&self.tables) {
            zip.start_file(format!("xl/pivotTables/pivotTable{part}.xml"), options)?;
            zip.write_all(exported.xml.as_bytes())?;
            zip.start_file(
                format!("xl/pivotTables/_rels/pivotTable{part}.xml.rels"),
                options,
            )?;
            let target = format!(
                "../pivotCache/pivotCacheDefinition{}.xml",
                exported.cache_part
            );
            zip.write_all(relationships(CACHE_DEFINITION_RELATIONSHIP, &target).as_bytes())?;
        }
        Ok(())
    }
}

/// A relationships part with one relationship, `rId1`.
fn relationships(kind: &str, target: &str) -> String {
    format!(
        r#"{}
<Relationships xmlns="{RELATIONSHIPS_NS}"><Relationship Id="rId1" Type="{kind}" Target="{target}"/></Relationships>"#,
        super::xml_constants::XML_DECLARATION
    )
}

/// The engine's layout of `pivot`, for a pivot that holds no saved layout lines (the engine laid
/// it out since it was read, or it was made through the API) and whose layout gives the location
/// and geometry it holds, so its lines describe the cells there; `None` otherwise: a pivot that
/// keeps the lines it was read with, as its cells keep the saved ones, or one past the work
/// budget.
fn layout_lines(
    pivot: &PivotTable,
    pivot_cache: &rustsheet_engine::pivot::PivotCache,
    work: &mut u64,
) -> Option<PivotLayout> {
    if pivot.opaque.iter().any(OpaqueEntry::is_layout_lines) {
        return None;
    }
    let result = compute(pivot, pivot_cache, work).ok()?;
    let layout = lay_out(pivot, pivot_cache, &result, work).ok()?;
    (layout.location == pivot.location && layout.geometry == pivot.geometry).then_some(layout)
}

/// Logs a dropped cache or pivot to stderr, as the importer logs a skipped one; the exporter has
/// no warnings channel. Names come from the file, so they are quoted and cut short.
fn log_drop(what: &str, name: &str, reason: &str) {
    const MAX_CHARS: usize = 200;
    let name: String = name.chars().take(MAX_CHARS).collect();
    eprintln!("RustSheet: export dropped {what} {name:?}: {reason}");
}
