//! The undo of the edits whose diffs carry cells, records and areas that are written straight into
//! the sheet. A diff list from a peer (`UserModel::apply_external_diffs`) is untrusted (CLAUDE.md
//! directive 5, T-2896), so each of these is checked against the model as it is when the diff
//! applies, with the checks a loaded workbook passes (`workbook_check`), before anything is written.

use crate::{
    cf_types::ConditionalFormatting,
    constants::COLUMN_WIDTH_FACTOR,
    cut_paste::cf_sqref_anchor,
    expressions::{parser::DefinedNameS, utils::check_block},
    formula_rewrite::RewrittenCell,
    types::{ArrayKind, Cell, DataTable, MergedCell, Style},
    user_model::history::{ColumnData, Diff, RowData},
    workbook_check::{check_cell, check_row_record, check_style},
    UserModel,
};

/// A block as (`row`, `column`) of its top left cell and (`width`, `height`).
type Block = ((i32, i32), (i32, i32));

impl UserModel {
    /// The cell an undo restores at (`row`, `column`) of `sheet` for `cell`, once checked, and
    /// counted into the cells this call restores (`restored_spills`, T-3426).
    ///
    /// A dynamic anchor comes back claiming only itself, as `Model::reset_dynamic_array_spills`
    /// leaves one: its spill cells are not in the diff, since evaluation rebuilds them, so the
    /// range it had when the diff was made claims cells nobody restores, and nothing must walk it.
    pub(super) fn restored_cell(
        &mut self,
        sheet: u32,
        row: i32,
        column: i32,
        cell: &Cell,
    ) -> Result<Cell, String> {
        let formulas = self.model.workbook.worksheet(sheet)?.shared_formulas.len();
        check_cell(
            row,
            column,
            cell,
            self.model.workbook.styles.cell_xfs.len(),
            formulas,
        )?;
        let mut cell = cell.clone();
        if let Cell::ArrayFormula {
            kind: ArrayKind::Dynamic,
            r,
            ..
        } = &mut cell
        {
            *r = (1, 1);
        }
        self.restored_spills.add(&cell);
        Ok(cell)
    }

    /// Undoes an array formula entered over the block: clears it and restores `old_values`.
    pub(super) fn undo_array_value(
        &mut self,
        sheet: u32,
        (corner, size): Block,
        old_values: &[Vec<Option<Cell>>],
    ) -> Result<(), String> {
        check_block(corner.0, corner.1, size.0, size.1)?;
        let mut restored = Vec::new();
        for (r, c, cell) in restored_cells((corner, size), old_values) {
            restored.push((r, c, self.restored_cell(sheet, r, c, cell)?));
        }
        // Clear the cells of the array formula (anchor and spill cells); only those that exist,
        // since the block can be the whole sheet.
        let ws = self.model.workbook.worksheet_mut(sheet)?;
        for (r, c) in ws.sheet_data.places_in_block(corner, size) {
            let _ = ws.cell_clear_contents(r, c);
        }
        // Restore all cells that existed before the array formula was placed.
        for (r, c, cell) in restored {
            ws.update_cell(r, c, cell)?;
        }
        Ok(())
    }

    /// Undoes a clear of the block's contents, or of everything when `old_style` holds the styles
    /// the cells had: restores `old_value` (and those styles).
    pub(super) fn undo_range_clear(
        &mut self,
        sheet: u32,
        (corner, size): Block,
        old_value: &[Vec<Option<Cell>>],
        old_style: Option<&Vec<Vec<Style>>>,
    ) -> Result<(), String> {
        check_block(corner.0, corner.1, size.0, size.1)?;
        let mut restored = Vec::new();
        for (r, c, cell) in restored_cells((corner, size), old_value) {
            let cell = self.restored_cell(sheet, r, c, cell)?;
            let style = match old_style {
                // The offsets are not negative: `restored_cells` starts at the corner.
                Some(styles) => Some(
                    usize::try_from(r - corner.0)
                        .ok()
                        .zip(usize::try_from(c - corner.1).ok())
                        .and_then(|(i, j)| styles.get(i)?.get(j))
                        .ok_or_else(|| format!("cell R{r}C{c} is restored without its style"))?,
                ),
                None => None,
            };
            restored.push((r, c, cell, style));
        }
        for (r, c, cell, style) in restored {
            let worksheet = self.model.workbook.worksheet_mut(sheet)?;
            worksheet.update_cell(r, c, cell)?;
            if let Some(style) = style {
                self.model.set_cell_style(sheet, r, c, style)?;
            }
        }
        Ok(())
    }

    /// Undoes a delete of the rows from `row`: inserts them again and restores `old_data`.
    pub(super) fn undo_delete_rows(
        &mut self,
        sheet: u32,
        row: i32,
        old_data: &[RowData],
    ) -> Result<(), String> {
        let count = restored_count(old_data.len())?;
        check_block(row, 1, 1, count)?;
        let styles = self.model.workbook.styles.cell_xfs.len();
        let mut restored = Vec::with_capacity(old_data.len());
        for (r, row_data) in (row..row + count).zip(old_data) {
            if let Some(record) = &row_data.row {
                check_row_record(record, styles)?;
            }
            let mut cells = Vec::with_capacity(row_data.data.len());
            for (c, cell) in &row_data.data {
                cells.push((*c, self.restored_cell(sheet, r, *c, cell)?));
            }
            restored.push((r, row_data.row.clone(), cells));
        }
        self.model.insert_rows(sheet, row, count)?;
        let worksheet = self.model.workbook.worksheet_mut(sheet)?;
        for (r, record, cells) in restored {
            if let Some(record) = record {
                worksheet.rows.push(record);
            }
            worksheet.sheet_data.set_row(r, cells);
        }
        Ok(())
    }

    /// Undoes a delete of the columns from `column`: inserts them again and restores `old_data`.
    pub(super) fn undo_delete_columns(
        &mut self,
        sheet: u32,
        column: i32,
        old_data: &[ColumnData],
    ) -> Result<(), String> {
        let count = restored_count(old_data.len())?;
        check_block(1, column, count, 1)?;
        let styles = self.model.workbook.styles.cell_xfs.len();
        let mut restored = Vec::new();
        for (c, col_data) in (column..column + count).zip(old_data) {
            if let Some(style) = col_data.column.as_ref().and_then(|col| col.style) {
                check_style(style, styles).map_err(|e| format!("column {c} {e}"))?;
            }
            for (r, cell) in &col_data.data {
                restored.push((*r, c, self.restored_cell(sheet, *r, c, cell)?));
            }
        }
        self.model.insert_columns(sheet, column, count)?;
        let worksheet = self.model.workbook.worksheet_mut(sheet)?;
        for (r, c, cell) in restored {
            worksheet.update_cell(r, c, cell)?;
        }
        for (c, col_data) in (column..column + count).zip(old_data) {
            if let Some(col) = &col_data.column {
                let width = col.width * COLUMN_WIDTH_FACTOR;
                worksheet.set_column_width_and_style(c, width, col.hidden, col.style)?;
            }
        }
        Ok(())
    }
}

impl UserModel {
    /// Returns a snapshot diff of the merged cells of the sheet when the
    /// current list differs from `old_merged_cells`, and nothing otherwise.
    ///
    /// Structural actions (insert, delete or move of rows and columns) displace
    /// merged ranges on their own when they are (re)played, but their undo
    /// cannot always reconstruct the original ranges (a fully deleted merge is
    /// gone, deleting rows above a merge and re-inserting them displaces it).
    /// The snapshot has `old_value == new_value` — a no-op when applied — and
    /// must go *before* the structural diff in the list, so that on undo
    /// (replayed in reverse) it runs last and restores the exact original list.
    pub(super) fn merged_cells_snapshot_diffs(
        &self,
        sheet: u32,
        old_merged_cells: Vec<MergedCell>,
    ) -> Result<Vec<Diff>, String> {
        if self.model.get_merged_cells(sheet)? == old_merged_cells.as_slice() {
            Ok(vec![])
        } else {
            Ok(vec![Diff::SetMergedCells {
                sheet,
                old_value: old_merged_cells.clone(),
                new_value: old_merged_cells,
            }])
        }
    }

    /// Returns a snapshot diff of the data tables of the sheet when the edit changed them from
    /// `old_tables` (taken before it), and nothing otherwise. Like
    /// [`Self::merged_cells_snapshot_diffs`] it has `old_value == new_value` and goes *before*
    /// the edit's diff, so that undo restores the exact tables (T-4940).
    pub(super) fn data_tables_snapshot_diffs(
        &self,
        sheet: u32,
        old_tables: Vec<DataTable>,
    ) -> Result<Vec<Diff>, String> {
        if self.model.workbook.worksheet(sheet)?.data_tables == old_tables {
            return Ok(vec![]);
        }
        Ok(vec![Diff::SetDataTables {
            sheet,
            old_value: old_tables.clone(),
            new_value: old_tables,
        }])
    }

    /// What a row or column delete on `sheet` is about to rewrite past what re-inserting the lines
    /// undoes: the sheet's conditional formats and every defined name (T-4702). Taken before the
    /// delete; [`Self::rewritten_snapshot_diffs`] turns it into diffs after it.
    pub(super) fn rewrite_snapshot(
        &self,
        sheet: u32,
    ) -> Result<(Vec<ConditionalFormatting>, Vec<DefinedNameS>), String> {
        let formats = self
            .model
            .workbook
            .worksheet(sheet)?
            .conditional_formatting
            .clone();
        Ok((formats, self.model.workbook.get_defined_names_with_scope()))
    }

    /// One diff per conditional format of `sheet` and per defined name the delete changed, each
    /// with `old == new` set to its state before the delete, as [`Diff::SetMergedCells`] does: a
    /// no-op on redo, which runs before the delete rewrites them again, and on undo, which runs
    /// after the lines are inserted again, the exact text before the delete (T-4702). A rule or
    /// name is matched by its index, which a delete does not change.
    pub(super) fn rewritten_snapshot_diffs(
        &self,
        sheet: u32,
        (formats, names): (Vec<ConditionalFormatting>, Vec<DefinedNameS>),
    ) -> Result<Vec<Diff>, String> {
        let mut diffs = Vec::new();
        let now = &self.model.workbook.worksheet(sheet)?.conditional_formatting;
        for (index, (before, after)) in formats.into_iter().zip(now).enumerate() {
            if before.range != after.range || before.cf_rule != after.cf_rule {
                diffs.push(Diff::UpdateConditionalFormatting {
                    sheet,
                    index: u32::try_from(index).map_err(|_| "Too many conditional formats")?,
                    old_range: before.range.clone(),
                    old_rule: Box::new(before.cf_rule.clone()),
                    old_priority: before.priority,
                    new_range: before.range,
                    new_rule: Box::new(before.cf_rule),
                });
            }
        }
        let now = self.model.workbook.get_defined_names_with_scope();
        for ((name, scope, before), (_, _, after)) in names.into_iter().zip(now) {
            if before != after {
                diffs.push(Diff::UpdateDefinedName {
                    name: name.clone(),
                    scope,
                    old_formula: before.clone(),
                    new_name: name,
                    new_scope: scope,
                    new_formula: before,
                });
            }
        }
        Ok(diffs)
    }

    /// The conditional format at `index` on `sheet` that an `UpdateConditionalFormatting` diff
    /// sets to `range`, or an error if there is none or `range` names no cell: a peer's diff is
    /// untrusted (CLAUDE.md directive 5), and a delete's undo writes these back (T-4702).
    pub(super) fn checked_cf_slot(
        &mut self,
        sheet: u32,
        index: u32,
        range: &str,
    ) -> Result<&mut ConditionalFormatting, String> {
        if cf_sqref_anchor(range).is_none() {
            return Err(format!(
                "Conditional formatting range '{range}' is not valid"
            ));
        }
        let worksheet = self.model.workbook.worksheet_mut(sheet)?;
        usize::try_from(index)
            .ok()
            .and_then(|i| worksheet.conditional_formatting.get_mut(i))
            .ok_or_else(|| format!("Conditional formatting index {index} out of bounds"))
    }

    /// The formula cells a delete of `count` rows (columns when not `rows`) from `first` on
    /// `sheet` rewrote, `rewritten` at their places before it, checked before its undo changes
    /// anything, since a peer's diff is untrusted (CLAUDE.md directive 5, T-4668): each passes
    /// [`Self::restored_cell`] and holds a formula, and the place the delete moved it to holds a
    /// formula now.
    pub(super) fn checked_rewritten_cells(
        &mut self,
        rewritten: &[RewrittenCell],
        (sheet, first, count, rows): (u32, i32, usize, bool),
    ) -> Result<Vec<RewrittenCell>, String> {
        let count = restored_count(count)?;
        let mut checked = Vec::with_capacity(rewritten.len());
        for entry in rewritten {
            let (row, column) = (entry.row, entry.column);
            let moved = |line: i32| match line.checked_sub(count) {
                _ if entry.sheet != sheet || line < first => Some(line),
                Some(now) if now >= first => Some(now),
                _ => None,
            };
            let now = if rows {
                moved(row).map(|r| (r, column))
            } else {
                moved(column).map(|c| (row, c))
            };
            let worksheet = self.model.workbook.worksheet(entry.sheet)?;
            let holds = now
                .and_then(|(r, c)| worksheet.cell(r, c))
                .is_some_and(Cell::has_formula);
            if !holds || !entry.cell.has_formula() {
                return Err(format!("Cell R{row}C{column} holds no formula to restore"));
            }
            let cell = self.restored_cell(entry.sheet, row, column, &entry.cell)?;
            checked.push(RewrittenCell {
                sheet: entry.sheet,
                row,
                column,
                cell,
            });
        }
        Ok(checked)
    }

    /// Puts back the cells [`Self::checked_rewritten_cells`] checked, once the undo has inserted
    /// the rows or columns again, so they are at their places before the delete.
    pub(super) fn restore_rewritten_cells(
        &mut self,
        checked: Vec<RewrittenCell>,
    ) -> Result<(), String> {
        for RewrittenCell {
            sheet,
            row,
            column,
            cell,
        } in checked
        {
            self.model
                .workbook
                .worksheet_mut(sheet)?
                .update_cell(row, column, cell)?;
        }
        Ok(())
    }
}

/// The number of rows or columns an undo restores, refused when it is no `i32`.
fn restored_count(len: usize) -> Result<i32, String> {
    i32::try_from(len).map_err(|_| "Too many rows or columns to restore".to_string())
}

/// The cells of `old_values`, laid over a block `check_block` accepted, with their places; the
/// cells past the block are not restored.
fn restored_cells(
    ((row, column), (width, height)): Block,
    old_values: &[Vec<Option<Cell>>],
) -> Vec<(i32, i32, &Cell)> {
    (row..row + height)
        .zip(old_values)
        .flat_map(|(r, cells)| {
            (column..column + width)
                .zip(cells)
                .filter_map(move |(c, cell)| cell.as_ref().map(|cell| (r, c, cell)))
        })
        .collect()
}

/// Where the undo of a move by `delta` starts and how far it moves back, refused when either
/// is no `i32`.
pub(super) fn moved_back(first: i32, delta: i32) -> Result<(i32, i32), String> {
    first
        .checked_add(delta)
        .zip(delta.checked_neg())
        .ok_or_else(|| "The move is outside the sheet".to_string())
}
