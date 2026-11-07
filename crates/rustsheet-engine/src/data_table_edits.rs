//! What-if data tables under row and column edits and cell writes (docs/excel-semantics/recalc.md,
//! "Data tables follow row and column edits around them and refuse edits that cut through them",
//! T-0452).
//!
//! Excel treats a data table's region, its result cells with the row above and the column left of
//! them, as one block: an insert or delete of rows or columns that cuts through it, and any write
//! into a result cell, is refused (excel-graph 2026-11-07). An edit outside the region moves the
//! table and its input cells; deleting every row (column) of the region deletes the table, and
//! deleting an input cell's row (column) makes that input `#REF!`.

use crate::{
    constants::{LAST_COLUMN, LAST_ROW},
    model::Model,
    types::{DataTable, DataTableInput, MergedCell},
};

/// What a refused edit says, after Excel's "Cannot change part of a data table."
pub(crate) const DATA_TABLE_PART: &str = "Cannot change part of a data table";

impl DataTable {
    /// The first and last row (`rows`) or column of the table's region: its results and the
    /// header row above (column left of) them.
    fn region_lines(&self, rows: bool) -> (i32, i32) {
        let (top, left, bottom, right) = self.range;
        if rows {
            (top - 1, bottom)
        } else {
            (left - 1, right)
        }
    }

    /// Whether inserting lines at `at` (`count > 0`), or deleting `-count` lines from `at`, cuts
    /// through the region: an insert strictly inside it, or a delete that takes some of its lines
    /// but not all of them.
    fn split_by(&self, rows: bool, at: i32, count: i32) -> bool {
        let (first, last) = self.region_lines(rows);
        if count > 0 {
            first < at && at <= last
        } else {
            let end = at.saturating_sub(count).saturating_sub(1);
            let meets = at <= last && first <= end;
            meets && !(at <= first && last <= end)
        }
    }

    /// The table after the edit [`DataTable::split_by`] accepts, or `None` when the edit deletes
    /// its whole region. A line an insert pushes past the last row (column) is deleted with it:
    /// unprobed, our choice (recalc.md).
    fn displaced(&self, rows: bool, at: i32, count: i32) -> Option<DataTable> {
        let limit = if rows { LAST_ROW } else { LAST_COLUMN };
        let line = |n: i32| -> Option<i32> {
            if count > 0 {
                if n < at {
                    Some(n)
                } else {
                    n.checked_add(count).filter(|&m| m <= limit)
                }
            } else if n < at {
                Some(n)
            } else if n < at.saturating_sub(count) {
                None
            } else {
                Some(n.saturating_add(count))
            }
        };
        let (top, left, bottom, right) = self.range;
        let range = if rows {
            (line(top)?, left, line(bottom)?, right)
        } else {
            (top, line(left)?, bottom, line(right)?)
        };
        let input = |input: Option<DataTableInput>| {
            input.map(|i| match i {
                DataTableInput::Cell(r, c) => {
                    let moved = if rows {
                        line(r).map(|r| (r, c))
                    } else {
                        line(c).map(|c| (r, c))
                    };
                    moved.map_or(DataTableInput::Deleted, |(r, c)| DataTableInput::Cell(r, c))
                }
                DataTableInput::Deleted => DataTableInput::Deleted,
            })
        };
        Some(DataTable {
            range,
            row_input: input(self.row_input),
            column_input: input(self.column_input),
        })
    }
}

impl Model {
    /// Refuses inserting `count` rows (`rows`) or columns at `at` on `sheet`, or deleting `-count`
    /// of them from `at`, when that cuts through a data table's region, before it changes anything.
    pub(crate) fn refuse_data_table_split(
        &self,
        sheet: u32,
        rows: bool,
        at: i32,
        count: i32,
    ) -> Result<(), String> {
        let ws = self.workbook.worksheet(sheet)?;
        if ws.data_tables.iter().any(|t| t.split_by(rows, at, count)) {
            return Err(DATA_TABLE_PART.to_string());
        }
        Ok(())
    }

    /// Refuses a move by `delta` of the rows (`rows`) or columns `start..=end` on `sheet` when a
    /// line it shifts, moved or passed over, is in a data table's region. Unprobed in Excel: a move
    /// is refused whole rather than risk moving part of a table (recalc.md, our choice). The caller
    /// has checked that the moved lines stay on the grid.
    pub(crate) fn refuse_data_table_move(
        &self,
        sheet: u32,
        rows: bool,
        (start, end, delta): (i32, i32, i32),
    ) -> Result<(), String> {
        let (first, last) = (start.min(start + delta), end.max(end + delta));
        let ws = self.workbook.worksheet(sheet)?;
        let meets = |t: &DataTable| {
            let (a, b) = t.region_lines(rows);
            a <= last && first <= b
        };
        if ws.data_tables.iter().any(meets) {
            return Err(DATA_TABLE_PART.to_string());
        }
        Ok(())
    }

    /// Takes the merged ranges and the data tables of `sheet` out of it while a row or column
    /// edit moves its cells: the moves go through the write guard
    /// (`prepare_cell_for_user_input`), which refuses a covered cell of a merged range and a
    /// result cell of a data table. [`Model::restore_write_guards`] puts them back.
    pub(crate) fn take_write_guards(
        &mut self,
        sheet: u32,
    ) -> Result<(Vec<MergedCell>, Vec<DataTable>), String> {
        let ws = self.workbook.worksheet_mut(sheet)?;
        Ok((
            std::mem::take(&mut ws.merged_cells),
            std::mem::take(&mut ws.data_tables),
        ))
    }

    /// Puts the merged ranges back on `sheet` as they were (the caller remaps them), and the tables as the
    /// edit of `count` lines at `at` left them (see [`Model::refuse_data_table_split`]): moved,
    /// with their inputs moved or `#REF!`, or gone.
    pub(crate) fn restore_write_guards(
        &mut self,
        sheet: u32,
        (merged, tables): (Vec<MergedCell>, Vec<DataTable>),
        (rows, at, count): (bool, i32, i32),
    ) -> Result<(), String> {
        let ws = self.workbook.worksheet_mut(sheet)?;
        ws.merged_cells = merged;
        ws.data_tables = tables
            .iter()
            .filter_map(|t| t.displaced(rows, at, count))
            .collect();
        Ok(())
    }

    /// Refuses a write into a result cell of a data table, as Excel does.
    pub(crate) fn refuse_data_table_write(
        &self,
        sheet: u32,
        row: i32,
        column: i32,
    ) -> Result<(), String> {
        let at = crate::expressions::types::CellReferenceIndex { sheet, row, column };
        if self.data_table_at(at).is_some() {
            return Err(DATA_TABLE_PART.to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{DataTable, DataTableInput};

    /// Results B4:B6, column input D1: the region is A3:B6.
    fn table() -> DataTable {
        DataTable {
            range: (4, 2, 6, 2),
            row_input: None,
            column_input: Some(DataTableInput::Cell(1, 4)),
        }
    }

    #[test]
    fn row_edits_through_the_region_are_split() {
        let t = table();
        // Inserts at rows 3 (the header row) and 7 move or miss it; rows 4-6 cut through it.
        assert!(!t.split_by(true, 3, 1));
        assert!(t.split_by(true, 4, 1));
        assert!(t.split_by(true, 6, 2));
        assert!(!t.split_by(true, 7, 1));
        // Deletes of rows 3, 5 or 4:6 cut it; 3:6 and 2:7 delete it whole; 1 misses it.
        assert!(t.split_by(true, 3, -1));
        assert!(t.split_by(true, 5, -1));
        assert!(t.split_by(true, 4, -3));
        assert!(!t.split_by(true, 3, -4));
        assert!(!t.split_by(true, 2, -6));
        assert!(!t.split_by(true, 1, -1));
        // Columns A (header) and B (results).
        assert!(!t.split_by(false, 1, 1));
        assert!(t.split_by(false, 2, 1));
        assert!(!t.split_by(false, 3, 1));
        assert!(t.split_by(false, 1, -1));
        assert!(t.split_by(false, 2, -1));
        assert!(!t.split_by(false, 1, -2));
        assert!(!t.split_by(false, 4, -1));
    }

    #[test]
    fn edits_around_the_region_move_the_table_and_its_inputs() {
        let t = table();
        let moved = |range, input| DataTable {
            range,
            row_input: None,
            column_input: Some(input),
        };
        assert_eq!(
            t.displaced(true, 1, 1),
            Some(moved((5, 2, 7, 2), DataTableInput::Cell(2, 4)))
        );
        assert_eq!(
            t.displaced(true, 2, 1),
            Some(moved((5, 2, 7, 2), DataTableInput::Cell(1, 4)))
        );
        assert_eq!(
            t.displaced(false, 1, 1),
            Some(moved((4, 3, 6, 3), DataTableInput::Cell(1, 5)))
        );
        assert_eq!(
            t.displaced(false, 3, 1),
            Some(moved((4, 2, 6, 2), DataTableInput::Cell(1, 5)))
        );
        assert_eq!(
            t.displaced(true, 1, -1),
            Some(moved((3, 2, 5, 2), DataTableInput::Deleted))
        );
        assert_eq!(
            t.displaced(false, 4, -1),
            Some(moved((4, 2, 6, 2), DataTableInput::Deleted))
        );
        assert_eq!(t.displaced(true, 3, -4), None);
        assert_eq!(
            t.displaced(true, 2, crate::constants::LAST_ROW),
            None,
            "pushed off the sheet"
        );
        assert_eq!(t.displaced(false, 1, -2), None);
    }
}
