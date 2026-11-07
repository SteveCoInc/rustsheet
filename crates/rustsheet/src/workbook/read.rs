//! The read path of a [`Workbook`]: values, cells, formatted text and extents (moved out of
//! workbook.rs, T-4669).

use std::sync::Arc;

use rustsheet_engine::{LAST_COLUMN, LAST_ROW};

use super::{engine_rc, CellInfo, Workbook};
use crate::error::RefError;
use crate::refs::{CellRef, RangeRef, SheetId, Target};
use crate::value::{lone_text, Array, Value};

impl Workbook {
    /// Resolves `target` to its sheet index and the top-left cell in engine (1-based) coordinates.
    fn top_left<'a>(
        &self,
        target: impl Into<Target<'a>>,
    ) -> Result<(RangeRef, u32, i32, i32), RefError> {
        let range = self.resolve_range(target.into())?;
        let sheet = self.sheet_index(range.sheet)?;
        let (row, col) = engine_rc(range.top, range.left).ok_or(RefError::OutOfBounds)?;
        Ok((range, sheet, row, col))
    }

    fn cell_at(&self, sheet: u32, row: u32, col: u32) -> Result<Value, RefError> {
        let (r, c) = engine_rc(row, col).ok_or(RefError::OutOfBounds)?;
        let ws = self
            .model
            .workbook
            .worksheet(sheet)
            .map_err(|_| RefError::UnknownSheet(sheet.to_string()))?;
        Ok(ws.cell(r, c).map_or(Value::Empty, |cell| {
            Value::from_cell(cell, &self.model.workbook.shared_strings)
        }))
    }

    /// The exact UTF-16 code units of a text value that holds a lone surrogate.
    ///
    /// Excel text is a sequence of UTF-16 units and keeps an unpaired surrogate when a cut
    /// splits a pair, which a Rust `str` cannot hold: [`Value::Text`] is the lossy view (each
    /// lone unit as U+FFFD). This returns `Some` exactly when the value of the cell (the top-left
    /// cell of a range) is text holding a lone surrogate, and `None` for every other value.
    ///
    /// # Errors
    /// [`RefError`] when the target does not resolve.
    pub fn text_utf16<'a>(&self, r: impl Into<Target<'a>>) -> Result<Option<Arc<[u16]>>, RefError> {
        let (range, sheet, _, _) = self.top_left(r)?;
        let (row, col) = engine_rc(range.top, range.left).ok_or(RefError::OutOfBounds)?;
        let ws = self
            .model
            .workbook
            .worksheet(sheet)
            .map_err(|_| RefError::UnknownSheet(sheet.to_string()))?;
        Ok(ws
            .cell(row, col)
            .and_then(lone_text)
            .map(|t| t.units().collect()))
    }

    /// The value of a cell, or of the top-left cell of a range.
    ///
    /// # Errors
    /// [`RefError`] when the target does not resolve.
    pub fn value<'a>(&self, r: impl Into<Target<'a>>) -> Result<Value, RefError> {
        let (range, sheet, _, _) = self.top_left(r)?;
        self.cell_at(sheet, range.top, range.left)
    }

    /// The values of a range as a dense row-major [`Array`].
    ///
    /// # Errors
    /// [`RefError::TooLarge`] when the range area exceeds `max_read_cells` (checked before
    /// allocating), otherwise as [`Self::value`].
    pub fn values<'a>(&self, r: impl Into<Target<'a>>) -> Result<Array, RefError> {
        let range = self.resolve_range(r.into())?;
        let sheet = self.sheet_index(range.sheet)?;
        // Re-validate: `RangeRef` has public fields, so a caller can build an inverted literal.
        let range = RangeRef::new(
            range.sheet,
            range.top,
            range.left,
            range.bottom,
            range.right,
        )?;
        let rows = u64::from(range.bottom - range.top) + 1;
        let cols = u64::from(range.right - range.left) + 1;
        let area = rows.checked_mul(cols).ok_or(RefError::TooLarge)?;
        if area > self.max_read_cells {
            return Err(RefError::TooLarge);
        }
        let mut values = Vec::with_capacity(usize::try_from(area).map_err(|_| RefError::TooLarge)?);
        for row in range.top..=range.bottom {
            for col in range.left..=range.right {
                values.push(self.cell_at(sheet, row, col)?);
            }
        }
        let rows = usize::try_from(rows).map_err(|_| RefError::TooLarge)?;
        let cols = usize::try_from(cols).map_err(|_| RefError::TooLarge)?;
        Array::new(rows, cols, values).map_err(|_| RefError::TooLarge)
    }

    /// A cell's value, formula and number format (top-left cell of a range).
    ///
    /// # Errors
    /// [`RefError`] when the target does not resolve.
    pub fn cell<'a>(&self, r: impl Into<Target<'a>>) -> Result<CellInfo, RefError> {
        let (range, sheet, row, col) = self.top_left(r)?;
        let formula = self
            .model
            .get_cell_formula(sheet, row, col)
            .map_err(RefError::Invalid)?;
        let format = self
            .model
            .get_style_for_cell(sheet, row, col)
            .map_err(RefError::Invalid)?
            .num_fmt;
        Ok(CellInfo {
            value: self.cell_at(sheet, range.top, range.left)?,
            formula,
            number_format: (!format.eq_ignore_ascii_case("general")).then_some(format),
        })
    }

    /// The cell's text as Excel displays it (top-left cell of a range).
    ///
    /// # Errors
    /// [`RefError`] when the target does not resolve.
    pub fn formatted<'a>(&self, r: impl Into<Target<'a>>) -> Result<String, RefError> {
        let (_, sheet, row, col) = self.top_left(r)?;
        self.model
            .get_formatted_cell_value(sheet, row, col)
            .map_err(RefError::Invalid)
    }

    /// The bounding rectangle of stored cells, `None` for an empty sheet.
    ///
    /// # Errors
    /// [`RefError::UnknownSheet`] when `sheet` does not exist.
    pub fn used_range(&self, sheet: SheetId) -> Result<Option<RangeRef>, RefError> {
        let index = self.sheet_index(sheet)?;
        let ws = self
            .model
            .workbook
            .worksheet(index)
            .map_err(|_| RefError::UnknownSheet(sheet.get().to_string()))?;
        let d = ws.dimension();
        // `dimension` reports A1:A1 for an empty sheet.
        if (d.min_row, d.min_column, d.max_row, d.max_column) == (1, 1, 1, 1)
            && ws.cell(1, 1).is_none()
        {
            return Ok(None);
        }
        debug_assert!(d.max_row <= LAST_ROW && d.max_column <= LAST_COLUMN);
        let to_u32 = |v: i32| u32::try_from(v - 1).map_err(|_| RefError::OutOfBounds);
        RangeRef::new(
            sheet,
            to_u32(d.min_row)?,
            to_u32(d.min_column)?,
            to_u32(d.max_row)?,
            to_u32(d.max_column)?,
        )
        .map(Some)
    }

    /// Every stored cell of `sheet` (values, formulas and spill members), by row and within a
    /// row by column. Cheap for a sparse sheet whatever its [`Self::used_range`].
    ///
    /// # Errors
    /// [`RefError::UnknownSheet`] when `sheet` does not exist.
    pub fn stored_cells(&self, sheet: SheetId) -> Result<Vec<CellRef>, RefError> {
        let index = self.sheet_index(sheet)?;
        let ws = self
            .model
            .workbook
            .worksheet(index)
            .map_err(|_| RefError::UnknownSheet(sheet.get().to_string()))?;
        ws.sheet_data
            .cells()
            .map(|(row, col, _)| {
                let to_u32 = |v: i32| u32::try_from(v - 1).map_err(|_| RefError::OutOfBounds);
                CellRef::new(sheet, to_u32(row)?, to_u32(col)?)
            })
            .collect()
    }
}
