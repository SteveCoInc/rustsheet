//! [`Workbook::pivot_table`]: a read-only view of one pivot table (SPEC-0032 "API", T-4178), its
//! place, its source and its fields; and [`Workbook::pivot_table_cells`], the cells the engine's
//! grid writer would write for it. The view reads metadata only, so it costs no layout.

use rustsheet_engine::pivot::{
    AxisField, PivotBudget, PivotComputeError, PivotTable, SourceCells, VALUES,
};
use rustsheet_engine::types::Workbook as EngineWorkbook;

use crate::edit::PivotError;
use crate::refs::{CellRef, RangeRef, SheetId};
use crate::value::Value;
use crate::workbook::Workbook;

/// A pivot table as [`Workbook::pivot_table`] reads it. It is a snapshot: a later edit does
/// not change it. Its cells are [`Workbook::pivot_table_cells`]'s, computed only on request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PivotView {
    name: String,
    sheet: SheetId,
    location: RangeRef,
    source: Option<RangeRef>,
    row_fields: Vec<String>,
    column_fields: Vec<String>,
    data_fields: Vec<String>,
}

impl PivotView {
    /// The pivot's name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The sheet the pivot is on.
    #[must_use]
    pub fn sheet(&self) -> SheetId {
        self.sheet
    }

    /// The pivot's location, as saved with it: the cells of its last layout, page area
    /// excluded, or its anchor alone when it collapsed to `#SPILL!`. A loaded pivot not yet
    /// refreshed keeps the location its file gives.
    #[must_use]
    pub fn location(&self) -> RangeRef {
        self.location
    }

    /// Its cache's source range; `None` when the source is not a worksheet range of this
    /// workbook (a Table, a defined name or a connection of a loaded pivot, or a sheet that is
    /// gone).
    #[must_use]
    pub fn source(&self) -> Option<RangeRef> {
        self.source
    }

    /// The row fields' names, outer first; the data fields' own axis field of a pivot with
    /// several is `Values`.
    #[must_use]
    pub fn row_fields(&self) -> &[String] {
        &self.row_fields
    }

    /// The column fields' names, outer first, as [`PivotView::row_fields`].
    #[must_use]
    pub fn column_fields(&self) -> &[String] {
        &self.column_fields
    }

    /// The data fields' names, the captions their cells show (`Sum of Sales`), in order.
    #[must_use]
    pub fn data_fields(&self) -> &[String] {
        &self.data_fields
    }
}

/// The names of the axis fields `fields` of `pivot` over `workbook`'s caches: each cache
/// field's, or `Values` for the data fields' own axis field.
fn axis_names(workbook: &EngineWorkbook, pivot: &PivotTable, fields: &[AxisField]) -> Vec<String> {
    let cache = workbook.pivot_caches.get(pivot.cache as usize);
    fields
        .iter()
        .map(|field| match *field {
            AxisField::Values => VALUES.to_string(),
            AxisField::Field(index) => cache
                .and_then(|c| c.fields.get(index as usize))
                .map(|f| f.name.clone())
                .unwrap_or_default(),
        })
        .collect()
}

/// The index of the first pivot named `name` on `sheet` in `workbook`: the one every call that
/// takes a sheet and a name means (SPEC-0032 "API").
pub(crate) fn pivot_position(
    workbook: &EngineWorkbook,
    sheet: SheetId,
    name: &str,
) -> Option<usize> {
    (workbook.pivot_tables.iter()).position(|p| p.sheet_id == sheet.get() && p.name == name)
}

impl Workbook {
    /// The pivot table `name` on `sheet`, or `None` when the sheet has none of that name (or,
    /// for a loaded pivot, its saved location is outside the sheet). It reads the pivot's
    /// definition only: no layout is computed ([`Workbook::pivot_table_cells`] computes one).
    #[must_use]
    pub fn pivot_table(&self, sheet: SheetId, name: &str) -> Option<PivotView> {
        let model = &self.model;
        let workbook = &model.workbook;
        let pivot = workbook
            .pivot_tables
            .get(pivot_position(workbook, sheet, name)?)?;
        let cache = workbook.pivot_caches.get(pivot.cache as usize);
        let area = pivot.location;
        let location = RangeRef::new(sheet, area.top, area.left, area.bottom, area.right).ok()?;
        let source = (cache.and_then(|c| c.source.as_ref()))
            .and_then(|source| model.resolve(source))
            .and_then(|found| {
                let id = workbook.worksheets.get(found.sheet as usize)?.sheet_id;
                let a = found.area;
                RangeRef::new(SheetId::new(id), a.top, a.left, a.bottom, a.right).ok()
            });
        Some(PivotView {
            name: pivot.name.clone(),
            sheet,
            location,
            source,
            row_fields: axis_names(workbook, pivot, &pivot.row_fields),
            column_fields: axis_names(workbook, pivot, &pivot.column_fields),
            data_fields: (pivot.data_fields.iter())
                .map(|d| d.caption.clone())
                .collect(),
        })
    }

    /// The cells the grid writer would write for the pivot `name` on `sheet` if it laid the
    /// pivot out again now from its definition and cache, without reading its source again
    /// (`rustsheet_engine::pivot`'s `plan_pivot_render`, which changes nothing): its layout's
    /// cells, or `#SPILL!` at its anchor when the layout would meet a cell it does not own or
    /// pass the sheet's edge. By row and within a row by column; an empty cell is left out.
    /// They can differ from what the grid holds: a loaded pivot keeps its saved cells until it
    /// is refreshed. The computation is bounded as a refresh is (one pivot's `PivotBudget`), so
    /// a large pivot costs a layout's work and memory.
    ///
    /// # Errors
    /// [`PivotError::UnknownPivot`] when the sheet has no pivot of that name,
    /// [`PivotError::Limit`] past a limit (SPEC-0032 "Untrusted-input limits"), and the other
    /// [`PivotError`]s of a definition that does not lay out (a merged range in its own area,
    /// [`PivotError::Refresh`]; one that does not match its cache, [`PivotError::Compute`]).
    pub fn pivot_table_cells(
        &self,
        sheet: SheetId,
        name: &str,
    ) -> Result<Vec<(CellRef, Value)>, PivotError> {
        let table = pivot_position(&self.model.workbook, sheet, name)
            .ok_or_else(|| PivotError::UnknownPivot(name.to_string()))?;
        let plan = (self.model).plan_pivot_render(table, &mut PivotBudget::default())?;
        let mut cells = Vec::new();
        for placement in plan.placements {
            for cell in placement.cells {
                let at = CellRef::new(sheet, cell.row, cell.column)
                    .map_err(|_| PivotError::Compute(PivotComputeError::PastSheet))?;
                cells.push((at, Value::from_pivot_cell(cell.value)));
            }
        }
        cells.sort_by_key(|(cell, _)| (cell.row, cell.col));
        Ok(cells)
    }
}
