//! [`Workbook::load_pivot_parts`]: a pivot cache and its pivots given as the package parts a
//! case builder writes, read by the xlsx importer and loaded as a file's are (T-4152, SPEC-0032
//! rollout step 7). Harness support for the eval adapter, doc-hidden like the rest of the pivot
//! API until T-4178.

use rustsheet_engine::pivot::MAX_RECORD_VALUES;
use rustsheet_xlsx::import::{read_pivot_parts, PivotParts as XlsxPivotParts};

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
    /// Adds the pivot cache and tables of `parts`, read by the xlsx importer as it reads a
    /// package's, so they are what a file holding those parts loads as. A `refreshOnLoad` cache
    /// is then due as a loaded file's is: the next [`Workbook::recalculate`] refreshes it and
    /// renders its pivots after calculating (SPEC-0032 "Who refreshes, and when"), the cells
    /// Excel renders on open. Test support for the eval adapter, not public API.
    ///
    /// # Errors
    /// [`EditError::Ref`] for an unknown sheet, [`EditError::Pivot`] ([`PivotError::Parts`])
    /// for a part the importer would skip. Nothing is added then.
    #[doc(hidden)]
    pub fn load_pivot_parts(&mut self, parts: &PivotParts) -> Result<(), EditError> {
        let mut tables = Vec::with_capacity(parts.tables.len());
        for (sheet, xml) in &parts.tables {
            sheet_index(&self.model, *sheet)?;
            tables.push((sheet.get(), xml.as_bytes()));
        }
        let pivots = &self.model.workbook;
        let used: usize = (pivots.pivot_caches.iter())
            .map(|c| c.record_count as usize * c.fields.len())
            .sum();
        let index = u32::try_from(pivots.pivot_caches.len()).map_err(|_| RefError::OutOfBounds)?;
        let (cache, tables) = read_pivot_parts(
            XlsxPivotParts {
                cache_definition: parts.cache_definition.as_bytes(),
                cache_records: parts.cache_records.as_bytes(),
                cache_id: parts.cache_id,
            },
            &tables,
            index,
            MAX_RECORD_VALUES.saturating_sub(used),
            self.date_system(),
        )
        .map_err(|error| PivotError::Parts(error.to_string()))?;
        let due = cache.refresh_on_load;
        self.edit(|tx| {
            let model = &mut tx.model.workbook;
            let undo = PivotUndo::Added {
                caches: model.pivot_caches.len(),
                tables: model.pivot_tables.len(),
            };
            model.pivot_caches.push(cache);
            model.pivot_tables.extend(tables);
            tx.sheet_op = true;
            tx.undo.push(Undo::Pivot(undo));
            Ok(())
        })?;
        self.pivots_due |= due;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
