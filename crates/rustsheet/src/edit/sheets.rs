//! The sheet-list ops of [`Txn`]: add, rename, move and delete a sheet (moved out of edit.rs,
//! T-4828).

use rustsheet_engine::types::Worksheet;
use rustsheet_engine::Model;

use super::{SheetChange, Txn, Undo};
use crate::error::{EditError, RefError};
use crate::refs::SheetId;
use crate::workbook::sheet_index;

impl Txn<'_> {
    /// Adds a sheet at the end.
    ///
    /// The engine reparses and re-evaluates the workbook on every sheet-level op, so these cost a
    /// recalculation each.
    ///
    /// # Errors
    /// [`EditError::SheetName`] when the name is empty, too long, duplicated or has forbidden
    /// characters.
    pub fn add_sheet(&mut self, name: &str) -> Result<SheetId, EditError> {
        self.model
            .add_sheet(name)
            .map_err(|m| EditError::from_engine_message(m, None))?;
        self.sheet_op = true;
        let id = self
            .model
            .workbook
            .worksheets
            .last()
            .map_or(0, Worksheet::get_sheet_id);
        self.undo.push(Undo::AddedSheet(id));
        self.sheets.push(SheetChange::Added(SheetId::new(id)));
        Ok(SheetId::new(id))
    }

    /// Renames a sheet, rewriting formulas that refer to it.
    ///
    /// # Errors
    /// [`EditError::Ref`] for an unknown sheet, [`EditError::SheetName`] for a bad or duplicate
    /// name.
    pub fn rename_sheet(&mut self, id: SheetId, new_name: &str) -> Result<(), EditError> {
        let index = sheet_index(self.model, id)?;
        let old_name = sheet_name(self.model, index);
        self.model
            .rename_sheet_by_index(index, new_name)
            .map_err(|m| EditError::from_engine_message(m, None))?;
        self.sheet_op = true;
        self.undo.push(Undo::Renamed {
            sheet_id: id.get(),
            old_name: old_name.clone(),
        });
        self.sheets.push(SheetChange::Renamed {
            id,
            from: old_name,
            to: new_name.to_string(),
        });
        Ok(())
    }

    /// Moves a sheet to the 0-based `position` in the tab order. References keep naming the sheet,
    /// so a 3-D span (`Sheet1:Sheet3!A1`) covers the sheets between its two ends in the new order.
    ///
    /// # Errors
    /// [`EditError::Ref`] for an unknown sheet or a position past the last sheet.
    pub fn move_sheet(&mut self, id: SheetId, position: u32) -> Result<(), EditError> {
        let index = sheet_index(self.model, id)?;
        if usize::try_from(position).map_or(true, |p| p >= self.model.workbook.worksheets.len()) {
            return Err(RefError::OutOfBounds.into());
        }
        self.model
            .move_sheet(index, position)
            .map_err(|m| EditError::from_engine_message(m, None))?;
        self.sheet_op = true;
        self.undo.push(Undo::Moved {
            sheet_id: id.get(),
            index,
        });
        self.sheets.push(SheetChange::Moved {
            id,
            from: index,
            to: position,
        });
        Ok(())
    }

    /// Deletes a sheet. References to it become `#REF!`.
    ///
    /// # Errors
    /// [`EditError::Ref`] for an unknown sheet, [`EditError::LastSheet`] for the only sheet.
    pub fn delete_sheet(&mut self, id: SheetId) -> Result<(), EditError> {
        let index = sheet_index(self.model, id)?;
        let name = sheet_name(self.model, index);
        if let Some(before) = &mut self.before {
            if let Ok(ws) = self.model.workbook.worksheet(index) {
                for (row, col, _) in ws.sheet_data.cells() {
                    before.touch((id.get(), row, col), ws, self.model);
                }
            }
        }
        let sheet = self
            .model
            .take_sheet(index)
            .map_err(|m| EditError::from_engine_message(m, None))?;
        self.sheet_op = true;
        self.undo.push(Undo::DeletedSheet {
            index,
            sheet: Box::new(sheet),
        });
        self.sheets.push(SheetChange::Deleted { id, name });
        Ok(())
    }
}

fn sheet_name(model: &Model, index: u32) -> String {
    model
        .workbook
        .worksheet(index)
        .map(Worksheet::get_name)
        .unwrap_or_default()
}
