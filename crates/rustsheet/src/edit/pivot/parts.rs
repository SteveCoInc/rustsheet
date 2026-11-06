//! [`Workbook::load_pivot_parts`]: a pivot cache and its pivots given as the package parts a
//! case builder writes, read by the xlsx importer and loaded as a file's are (T-4152, SPEC-0032
//! rollout step 7). Harness support for the eval adapter, doc-hidden like the rest of the pivot
//! API until T-4178.

use rustsheet_engine::pivot::MAX_RECORD_VALUES;
use rustsheet_xlsx::import::{read_pivot_parts, PivotParts as XlsxPivotParts, MAX_PIVOTS};

use super::{PivotError, PivotUndo};
use crate::edit::Undo;
use crate::error::{EditError, RefError};
use crate::refs::SheetId;
use crate::workbook::{sheet_index, Workbook};

/// One pivot cache and the pivot tables over it, as the XML of their package parts.
#[derive(Clone, Debug, Default)]
pub struct PivotParts {
    /// The `pivotCacheDefinition` part.
    pub cache_definition: String,
    /// Its `pivotCacheRecords` part.
    pub cache_records: String,
    /// The `cacheId` the tables name the cache by.
    pub cache_id: u32,
    /// Each `pivotTableDefinition` part and the sheet it sits on.
    pub tables: Vec<(SheetId, String)>,
}

impl Workbook {
    /// Adds the pivot cache and tables of each of `parts`, read by the xlsx importer as it reads
    /// a package's, so they are what a file holding those parts loads as. A `refreshOnLoad`
    /// cache is then due as a loaded file's is: the next [`Workbook::recalculate`] refreshes it
    /// and renders its pivots after calculating (SPEC-0032 "Who refreshes, and when"), the cells
    /// Excel renders on open. One edit adds them all. Returns, per entry of `parts`, why the
    /// importer skipped it ([`PivotError::Parts`]), as a load skips it; the others are added.
    /// Test support for the eval adapter, not public API.
    ///
    /// # Errors
    /// [`EditError::Ref`] for an unknown sheet, [`EditError::Pivot`] ([`PivotError::Parts`])
    /// when the workbook would hold more pivots than a load keeps
    /// ([`MAX_PIVOTS`](rustsheet_xlsx::import::MAX_PIVOTS)). Nothing is added then.
    #[doc(hidden)]
    pub fn load_pivot_parts(
        &mut self,
        parts: &[PivotParts],
    ) -> Result<Vec<Option<PivotError>>, EditError> {
        let workbook = &self.model.workbook;
        let tables: usize = parts.iter().map(|p| p.tables.len()).sum();
        let caches = workbook.pivot_caches.len().saturating_add(parts.len());
        if caches > MAX_PIVOTS || workbook.pivot_tables.len().saturating_add(tables) > MAX_PIVOTS {
            let limit = format!("more than {MAX_PIVOTS} pivot caches or tables");
            return Err(PivotError::Parts(limit).into());
        }
        let mut used: usize = (workbook.pivot_caches.iter())
            .map(|c| c.record_count as usize * c.fields.len())
            .sum();
        let mut index =
            u32::try_from(workbook.pivot_caches.len()).map_err(|_| RefError::OutOfBounds)?;
        let mut loaded = Vec::new();
        let mut skipped = Vec::with_capacity(parts.len());
        for part in parts {
            let mut tables = Vec::with_capacity(part.tables.len());
            for (sheet, xml) in &part.tables {
                sheet_index(&self.model, *sheet)?;
                tables.push((sheet.get(), xml.as_bytes()));
            }
            let read = read_pivot_parts(
                XlsxPivotParts {
                    cache_definition: part.cache_definition.as_bytes(),
                    cache_records: part.cache_records.as_bytes(),
                    cache_id: part.cache_id,
                },
                &tables,
                index,
                MAX_RECORD_VALUES.saturating_sub(used),
                self.date_system(),
            );
            match read {
                Ok((cache, tables)) => {
                    used = used.saturating_add(cache.record_count as usize * cache.fields.len());
                    index = index.checked_add(1).ok_or(RefError::OutOfBounds)?;
                    loaded.push((cache, tables));
                    skipped.push(None);
                }
                Err(error) => skipped.push(Some(PivotError::Parts(error.to_string()))),
            }
        }
        if loaded.is_empty() {
            return Ok(skipped);
        }
        let due = loaded.iter().any(|(cache, _)| cache.refresh_on_load);
        self.edit(|tx| {
            let model = &mut tx.model.workbook;
            let undo = PivotUndo::Added {
                caches: model.pivot_caches.len(),
                tables: model.pivot_tables.len(),
            };
            for (cache, tables) in loaded {
                model.pivot_caches.push(cache);
                model.pivot_tables.extend(tables);
            }
            tx.sheet_op = true;
            tx.undo.push(Undo::Pivot(undo));
            Ok(())
        })?;
        self.pivots_due |= due;
        Ok(skipped)
    }
}

#[cfg(test)]
mod tests;
