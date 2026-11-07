//! The table ops of [`Txn`]: add a table and a what-if data table (moved out of edit.rs,
//! T-4828).

use rustsheet_engine::types::{DataTable, DataTableInput, MAX_DATA_TABLE_CELLS};

use super::{Txn, Undo};
use crate::error::{EditError, RefError};
use crate::refs::{CellRef, RangeRef};
use crate::workbook::{engine_rc, sheet_index};

impl Txn<'_> {
    /// Adds a table (an Excel `ListObject`) over `range`. Its first row is the header row, whose
    /// cells (set them first) name the columns. With `filtered` the table carries an active
    /// filter, as an imported table whose autoFilter has a criterion does: on such a sheet every
    /// hidden row counts as filtered, which SUBTOTAL 1-11 skips (docs/excel-semantics/numbers.md,
    /// "SUBTOTAL and AGGREGATE over hidden and filtered rows"). Hide the filtered rows with
    /// [`Txn::set_row_hidden`]; filters are not re-applied. Like a sheet-level op it reparses
    /// and re-evaluates the workbook.
    ///
    /// # Errors
    /// [`EditError::Ref`] for an unknown sheet, [`EditError::Table`] for an invalid or taken
    /// name, a range outside the sheet, without a data row or overlapping another table, or an
    /// empty or repeated header.
    pub fn add_table(
        &mut self,
        range: RangeRef,
        name: &str,
        filtered: bool,
    ) -> Result<(), EditError> {
        self.add_table_with_totals(range, name, filtered, false)
    }

    /// [`Txn::add_table`] for a `range` whose last row is the table's totals row when
    /// `totals_row` is set, so `Table[#Totals]` reads it and `Table[#All]` includes it. The totals
    /// cells keep what they hold (set them first or after); a totals-row function is not recorded.
    ///
    /// # Errors
    /// As [`Txn::add_table`]; a totals row needs a data row between the header and itself.
    pub fn add_table_with_totals(
        &mut self,
        range: RangeRef,
        name: &str,
        filtered: bool,
        totals_row: bool,
    ) -> Result<(), EditError> {
        self.add_table_with_rows(range, name, filtered, (true, totals_row))
    }

    /// [`Txn::add_table_with_totals`] for a table with or without a header row
    /// (`(header_row, totals_row)`). Without one the data starts on the first row of `range` and
    /// the columns are named `Column1`, `Column2`, ...; such a table takes no filter.
    ///
    /// # Errors
    /// As [`Txn::add_table`]; a table without a header row needs one data row and no filter.
    pub fn add_table_with_rows(
        &mut self,
        range: RangeRef,
        name: &str,
        filtered: bool,
        (header_row, totals_row): (bool, bool),
    ) -> Result<(), EditError> {
        let index = sheet_index(self.model, range.sheet)?;
        let (top, left) = engine_rc(range.top, range.left).ok_or(RefError::OutOfBounds)?;
        let (bottom, right) = engine_rc(range.bottom, range.right).ok_or(RefError::OutOfBounds)?;
        self.model
            .add_table_with_rows(
                index,
                name,
                (top, left, bottom, right),
                filtered,
                (header_row, totals_row),
            )
            .map_err(EditError::Table)?;
        self.sheet_op = true;
        self.undo.push(Undo::Table(name.to_string()));
        Ok(())
    }

    /// Adds a what-if data table whose result cells are `range` (docs/excel-semantics/recalc.md,
    /// "Data tables"). With a `column_input` the values in the column left of `range` replace that
    /// cell and each result column has its formula in the row above; with a `row_input` the
    /// values in the row above replace it and each result row has its formula in the column to
    /// the left; with both, the one formula is at the corner above and left of `range`. The
    /// result cells are emptied, and the next evaluation fills them.
    ///
    /// # Errors
    /// [`EditError::Ref`] for an unknown sheet, an input on another sheet or out of bounds;
    /// [`EditError::InvalidInput`] when the range has no row above or column left of it, has no
    /// input, holds an input or overlaps another data table.
    pub fn add_data_table(
        &mut self,
        range: RangeRef,
        row_input: Option<CellRef>,
        column_input: Option<CellRef>,
    ) -> Result<(), EditError> {
        let sheet = sheet_index(self.model, range.sheet)?;
        let (top, left) = engine_rc(range.top, range.left).ok_or(RefError::OutOfBounds)?;
        let (bottom, right) = engine_rc(range.bottom, range.right).ok_or(RefError::OutOfBounds)?;
        let input = |cell: Option<CellRef>| -> Result<Option<DataTableInput>, EditError> {
            let Some(cell) = cell else { return Ok(None) };
            if cell.sheet != range.sheet {
                return Err(RefError::Unsupported.into());
            }
            let (r, c) = engine_rc(cell.row, cell.col).ok_or(RefError::OutOfBounds)?;
            Ok(Some(DataTableInput::Cell(r, c)))
        };
        let table = DataTable {
            range: (top, left, bottom, right),
            row_input: input(row_input)?,
            column_input: input(column_input)?,
        };
        let area = u64::from(range.bottom.saturating_sub(range.top) + 1)
            * u64::from(range.right.saturating_sub(range.left) + 1);
        // Checked before the cells are logged; the engine checks the same bound.
        if area > MAX_DATA_TABLE_CELLS {
            return Err(EditError::InvalidInput {
                cell: CellRef {
                    sheet: range.sheet,
                    row: range.top,
                    col: range.left,
                },
                message: "data table too large to edit".to_string(),
            });
        }
        for row in range.top..=range.bottom {
            for col in range.left..=range.right {
                let cell = CellRef {
                    sheet: range.sheet,
                    row,
                    col,
                };
                let (r, c) = engine_rc(row, col).ok_or(RefError::OutOfBounds)?;
                self.log_cell(sheet, (range.sheet.get(), r, c), cell)?;
            }
        }
        self.model
            .add_data_table(sheet, table)
            .map_err(|error| EditError::InvalidInput {
                cell: CellRef {
                    sheet: range.sheet,
                    row: range.top,
                    col: range.left,
                },
                message: error.to_string(),
            })?;
        // Rollback must re-evaluate: the table filled its results when the edit evaluated.
        self.sheet_op = true;
        self.undo.push(Undo::DataTable {
            sheet_id: range.sheet.get(),
            range: (top, left, bottom, right),
        });
        Ok(())
    }
}
