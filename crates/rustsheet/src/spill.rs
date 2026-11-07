//! Whether a cell belongs to a spill or array range it does not anchor.

use rustsheet_engine::types::Cell;

use crate::error::RefError;
use crate::refs::CellRef;
use crate::workbook::{engine_rc, sheet_index, Workbook};

impl Workbook {
    /// Whether `cell` is a member of a spill range other than its anchor (the engine stores it as a
    /// spill cell): it shows a value the anchor's formula computed, not an input of its own, and
    /// writing to it blocks the spill (`#SPILL!`).
    ///
    /// # Errors
    /// [`RefError::UnknownSheet`] when the cell's sheet does not exist, [`RefError::OutOfBounds`]
    /// for a cell past the grid.
    pub fn in_spill(&self, cell: CellRef) -> Result<bool, RefError> {
        let index = sheet_index(&self.model, cell.sheet)?;
        let ws = self
            .model
            .workbook
            .worksheet(index)
            .map_err(|_| RefError::UnknownSheet(cell.sheet.get().to_string()))?;
        let (row, col) = engine_rc(cell.row, cell.col).ok_or(RefError::OutOfBounds)?;
        Ok(matches!(ws.cell(row, col), Some(Cell::SpillCell { .. })))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Workbook, WorkbookOptions};

    #[test]
    fn a_spill_member_is_in_a_spill_and_its_anchor_is_not() -> Result<(), Box<dyn std::error::Error>>
    {
        let mut wb = Workbook::new(&WorkbookOptions::default())?;
        wb.edit(|t| {
            t.set("Sheet1!A1", "=SEQUENCE(2)")?;
            t.set("Sheet1!B1", 7.0)
        })?;
        wb.recalculate();
        let at = |a1: &str| wb.cell_ref(a1);
        assert!(!wb.in_spill(at("Sheet1!A1")?)?);
        assert!(wb.in_spill(at("Sheet1!A2")?)?);
        assert!(!wb.in_spill(at("Sheet1!B1")?)?);
        Ok(())
    }
}
