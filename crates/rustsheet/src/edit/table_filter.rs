//! [`Txn::filter_table_column`]: a filtered table's values filter on one column, as a workbook
//! Excel saved with that filter loads it (T-4803).

use super::{Txn, Undo};
use crate::error::{EditError, RefError};
use crate::workbook::engine_rc;

impl Txn<'_> {
    /// Gives the filtered table `table` a values filter on its column `column` (0-based, within
    /// the table) that rejects exactly the data rows `rejected` (0-based sheet rows): it keeps the
    /// text each other data row displays, as Excel's filter menu does when the rejected rows'
    /// values are unchecked. The criterion is written to xlsx and decides which rows a later
    /// column delete that takes another criterion shows (T-4803); it hides no row itself (hide
    /// the rejected rows with [`Txn::set_row_hidden`]).
    ///
    /// # Errors
    /// [`EditError::Table`] when there is no such filtered table or column, a rejected row is not
    /// one of its data rows, or a rejected row displays the text of a kept one (ignoring case).
    pub fn filter_table_column(
        &mut self,
        table: &str,
        column: usize,
        rejected: &[u32],
    ) -> Result<(), EditError> {
        let column = u32::try_from(column).map_err(|_| RefError::OutOfBounds)?;
        let rows = (rejected.iter())
            .map(|&row| engine_rc(row, 0).map(|(row, _)| row))
            .collect::<Option<Vec<i32>>>()
            .ok_or(RefError::OutOfBounds)?;
        let before =
            (self.model.filter_table_column(table, column, &rows)).map_err(EditError::Table)?;
        self.sheet_op = true;
        self.undo.push(Undo::TableBefore(Box::new(before)));
        Ok(())
    }
}
