//! The active sheet of a [`Workbook`], where API-added names bind their unqualified references.

use super::{sheet_metas, Workbook};
use crate::error::RefError;
use crate::refs::SheetId;

impl Workbook {
    /// The workbook's active sheet: the one an xlsx file marks selected, else the first. A name
    /// added with [`Txn::define_name`](crate::Txn::define_name) or
    /// [`Txn::define_sheet_name`](crate::Txn::define_sheet_name) binds an unqualified reference in
    /// its formula to this sheet, as Excel's `names/add` does (docs/excel-semantics/syntax.md "An
    /// unqualified reference in an API-defined name binds to the active sheet").
    #[must_use]
    pub fn active_sheet(&self) -> SheetId {
        let index = self.model.active_sheet() as usize;
        let sheets = sheet_metas(&self.model);
        sheets
            .get(index)
            .or_else(|| sheets.first())
            .map_or(SheetId::new(0), |s| s.id)
    }

    /// Makes `sheet` the active sheet (see [`Workbook::active_sheet`]); xlsx export writes it as the
    /// workbook view's active tab.
    ///
    /// # Errors
    /// [`RefError::UnknownSheet`] for an unknown sheet.
    pub fn set_active_sheet(&mut self, sheet: SheetId) -> Result<(), RefError> {
        let index = self.sheet_index(sheet)?;
        self.model
            .set_active_sheet(index)
            .map_err(|_| RefError::UnknownSheet(sheet.get().to_string()))
    }
}
