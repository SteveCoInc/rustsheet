//! The structural edit as the printer applies it: the displacement and what it does to the tables
//! of the edited sheet (SPEC-0016 "Structural edits").

use super::range_shrink::delete_corners;
use super::DisplaceData;

/// A structural edit as the printer applies it (SPEC-0016 "Structural edits").
#[derive(Clone, Copy)]
pub(crate) struct Displacement<'a> {
    /// The displacement.
    pub(crate) data: &'a DisplaceData,
    /// The lowercase names of the tables that follow the edit, whose structured references keep
    /// their text (T-2297).
    pub(crate) kept_tables: &'a [String],
    /// The data blocks of the tables that grow with a row insert at their first data row or at
    /// their totals row (T-2321).
    pub(crate) grown_edges: &'a [DataBlock],
    /// The table columns a column delete removes, as lowercase (table, column) names: a
    /// structured reference to one of them prints `#REF!` (T-2339).
    pub(crate) deleted_columns: &'a [(String, String)],
    /// The tables (lowercase) a column delete removes whole: they print `#REF!` (T-3733).
    pub(crate) deleted_tables: &'a [String],
    /// The tables (lowercase) a column insert or delete left of them moves whole, with no
    /// change to their column list: a `#This Row` reference to one keeps its text (T-4680).
    pub(crate) moved_tables: &'a [String],
    /// Whether a row edit leaves a wholly relative single-cell reference where it is. Set for
    /// defined names, whose relative references Excel does not move under a row insert or delete
    /// (references.md "Relative references in a name", T-4747, T-4760). Ranges, column edits,
    /// mixed references and a deleted referenced row are unprobed and displace as before (T-4875).
    pub(crate) keep_relative: bool,
}

/// The data rows `rows` and columns `columns` of a table on `sheet`, before the edit.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DataBlock {
    pub(crate) sheet: u32,
    pub(crate) rows: (i32, i32),
    pub(crate) columns: (i32, i32),
}

impl Displacement<'_> {
    /// A displacement that no table follows.
    pub(super) const fn plain(data: &DisplaceData) -> Displacement<'_> {
        Displacement {
            data,
            kept_tables: &[],
            grown_edges: &[],
            deleted_columns: &[],
            deleted_tables: &[],
            moved_tables: &[],
            keep_relative: false,
        }
    }

    /// Whether this edit leaves a wholly relative single reference (`all_relative`) on row `row`
    /// where it is, under [`Displacement::keep_relative`]: only a row insert, or a row delete that
    /// does not take `row`, is probed (T-4760); a deleted row displaces as before (T-4875).
    pub(super) fn keeps(&self, all_relative: bool, row: i32) -> bool {
        let DisplaceData::Row {
            row: edited, delta, ..
        } = *self.data
        else {
            return false;
        };
        let deletes_row = delta < 0 && row >= edited && row < edited.saturating_sub(delta);
        self.keep_relative && all_relative && !deletes_row
    }

    /// The displacements of the two corners of the plain range `rows` x `columns` on sheet
    /// `sheet_index` where a row or column edit does not displace them one by one; `None` for any other
    /// range, whose corners displace as usual.
    ///
    /// A row delete that takes a range's first or last rows, but not all of them, shrinks it to
    /// the rows left (a column delete likewise, `range_shrink.rs`; plain-range-delete-row-first and -last A1:A4, excel-graph, T-2338:
    /// `=SUM(S!C4:C7)` and `=SUM(S!$C$4:$C$7)` become `S!C4:C6` either way), where displacing the
    /// deleted corner alone would give `#REF!`. Only one-row deletes are probed; a wider delete
    /// across an edge shrinks the same way (hand, T-2851).
    ///
    /// Quirk (T-2197, probes/table-edits): a range in the data block of a table that grows with a
    /// row insert at an edge, and touching that edge, grows with the table, where an ordinary
    /// insert at a range's first row shifts it and one just below it leaves it. `=SUM(S!C4:C7)`
    /// onto data rows 4-7 becomes `=SUM(S!C4:C8)` both for the insert above row 4
    /// (table-insert-row-above-first-data A7) and above the totals row 8
    /// (table-insert-row-above-totals A7). Only that exact range is probed; a range inside the
    /// block touching the edge is assumed to grow the same way (hand, unprobed: T-2846).
    pub(super) fn range_corners(
        &self,
        sheet_index: u32,
        rows: (i32, i32),
        columns: (i32, i32),
    ) -> Option<(DisplaceData, DisplaceData)> {
        if let Some(shrunk) = delete_corners(self.data, sheet_index, rows, columns) {
            return Some(shrunk);
        }
        let DisplaceData::Row { sheet, row, delta } = *self.data else {
            return None;
        };
        // A row delete that shrinks nothing leaves the corners to displace as usual.
        if sheet != sheet_index || delta < 0 {
            return None;
        }
        // The corner on row `from` moved to row `to`.
        let shift_to = |from: i32, to: i32| DisplaceData::Row {
            sheet,
            row: from.min(to),
            delta: to - from,
        };
        let block = self.grown_edges.iter().find(|block| {
            block.sheet == sheet_index
                && block.rows.0 <= rows.0
                && rows.1 <= block.rows.1
                && block.columns.0 <= columns.0
                && columns.1 <= block.columns.1
        })?;
        // The first row stays (at or above the insert) and the last grows by `delta`.
        let at_edge = (row == block.rows.0 && rows.0 == row)
            || (row == block.rows.1 + 1 && rows.1 == block.rows.1);
        at_edge.then(|| (DisplaceData::None, shift_to(rows.1, rows.1 + delta)))
    }
}
