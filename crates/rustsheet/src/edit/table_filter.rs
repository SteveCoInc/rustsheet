//! [`Txn::filter_table_column`]: a filtered table's values filter on one column, as a workbook
//! Excel saved with that filter loads it (T-4803).

use super::{Txn, Undo};
use crate::error::{EditError, RefError};
use crate::workbook::engine_rc;
use rustsheet_engine::types::{CustomFilter, FilterCriterion};

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

    /// Gives the filtered table `table` a values filter on its column `column` that lists exactly
    /// `values`, the text the kept rows display (a number-formatted column lists "10.00", not the
    /// General "10"; T-4939). The criterion is written to xlsx and re-applied by a later column
    /// delete; it hides no row itself.
    ///
    /// # Errors
    /// [`EditError::Table`] when there is no such filtered table or column.
    pub fn filter_table_column_values(
        &mut self,
        table: &str,
        column: usize,
        values: &[String],
    ) -> Result<(), EditError> {
        let mut values = values.to_vec();
        values.sort_unstable();
        values.dedup();
        let blank = values.iter().any(String::is_empty);
        values.retain(|v| !v.is_empty());
        self.set_criterion(
            table,
            column,
            FilterCriterion::Values {
                values,
                blank,
                calendar_type: None,
                date_groups: Vec::new(),
            },
        )
    }

    /// Gives the filtered table `table` a custom filter on its column `column`: one or two
    /// `(operator, operand)` conditions (`None`: the file's default `equal`), all of which must
    /// hold when `and`, else either (T-4973). It hides no row itself.
    ///
    /// # Errors
    /// [`EditError::Table`] when there is no such filtered table or column, or `conditions` is
    /// empty or has more than two.
    pub fn filter_table_column_custom(
        &mut self,
        table: &str,
        column: usize,
        and: bool,
        conditions: &[(Option<String>, String)],
    ) -> Result<(), EditError> {
        if conditions.is_empty() || conditions.len() > 2 {
            return Err(EditError::Table(
                "a custom filter takes one or two conditions".to_string(),
            ));
        }
        let conditions = (conditions.iter())
            .map(|(operator, val)| CustomFilter {
                operator: operator.clone(),
                val: val.clone(),
            })
            .collect();
        self.set_criterion(table, column, FilterCriterion::Custom { and, conditions })
    }

    fn set_criterion(
        &mut self,
        table: &str,
        column: usize,
        criterion: FilterCriterion,
    ) -> Result<(), EditError> {
        let column = u32::try_from(column).map_err(|_| RefError::OutOfBounds)?;
        let before = (self
            .model
            .set_table_filter_criterion(table, column, criterion))
        .map_err(EditError::Table)?;
        self.sheet_op = true;
        self.undo.push(Undo::TableBefore(Box::new(before)));
        Ok(())
    }
}
