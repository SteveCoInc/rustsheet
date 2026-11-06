//! Where a workbook's pivot tables are, for the eval harness (T-4124): it compares them with the
//! pivots Excel saved. Test support like `Txn::add_pivot_table`, not public API.

use crate::workbook::Workbook;

/// A pivot table's sheet, name and `location` (0-based, inclusive).
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PivotPlace {
    /// The name of the sheet that holds the pivot.
    pub sheet: String,
    /// The pivot's name.
    pub name: String,
    /// First row of its location.
    pub top: u32,
    /// First column.
    pub left: u32,
    /// Last row.
    pub bottom: u32,
    /// Last column.
    pub right: u32,
}

impl Workbook {
    /// Every pivot table of the workbook with its sheet and location, in model order; a pivot
    /// whose sheet is gone is left out.
    #[doc(hidden)]
    #[must_use]
    pub fn pivot_places(&self) -> Vec<PivotPlace> {
        let workbook = &self.model.workbook;
        workbook
            .pivot_tables
            .iter()
            .filter_map(|p| {
                let sheet = workbook
                    .worksheets
                    .iter()
                    .find(|w| w.sheet_id == p.sheet_id)?;
                Some(PivotPlace {
                    sheet: sheet.name.clone(),
                    name: p.name.clone(),
                    top: p.location.top,
                    left: p.location.left,
                    bottom: p.location.bottom,
                    right: p.location.right,
                })
            })
            .collect()
    }
}
