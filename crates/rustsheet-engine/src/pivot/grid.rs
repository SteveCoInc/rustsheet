//! The grid writer (SPEC-0032 "Rendering and the grid", T-4151): a pivot owns the cells of its
//! location and of the page area above it, and a refresh clears what it owned and writes its new
//! cells there as constants, so a formula that reads them needs no new dependency kind.
//!
//! A plan ([`Model::plan_pivot_refresh`], [`Model::plan_pivot_render`]) reads the model and
//! changes nothing; [`Model::apply_pivot_refresh`] writes it and swaps the refreshed cache and
//! pivots in, returning what they replaced ([`PivotSwap`]) for an undo. The rules:
//! - a layout that would meet a cell the pivot does not own (a stored value or formula outside
//!   its area) writes `#SPILL!` at the anchor and saves the location as that one cell, as Excel
//!   does (pivots.md "A pivot that would meet a foreign cell collapses to #SPILL!",
//!   probes/t4128-pivot-spike S). That another pivot's area is foreign too, even where it holds
//!   no cell, and that the page area counts as the location does, is UNVERIFIED (probe pending:
//!   T-4664);
//! - a refresh rebuilds the cache from the source ([`refresh`]) and lays out again every pivot
//!   over it (UNVERIFIED that Excel refreshes the other pivots of a shared cache, T-4664); a
//!   pivot's field indices follow its fields by name ([`field_map`]);
//! - who refreshes and when is the caller's: the facade refreshes `refreshOnLoad` caches after a
//!   loaded workbook's first recalculation and on `Txn::refresh_pivot_table`, never on a source
//!   edit (pivots.md "Pivots without refreshOnLoad, and source edits", UNVERIFIED, T-4586).
//!
//! Every pivot is laid out within its own [`MAX_PIVOT_WORK`] and the result limits, and walks of
//! an area cost the cells stored there, never the area (an untrusted location can be the whole
//! sheet).

use std::collections::HashSet;

use thiserror::Error as ThisError;

use super::layout::{lay_out, render, PivotCell, RenderedCell};
use super::refresh::{field_map, refresh};
use super::result::{compute, PivotComputeError, MAX_PIVOT_WORK};
use super::{
    AxisField, CellArea, PivotCache, PivotCacheError, PivotField, PivotGeometry, PivotTable,
    MAX_RECORD_VALUES,
};
use crate::expressions::token::Error;
use crate::model::Model;
use crate::types::Cell;

/// Why a pivot could not be refreshed or laid out; the grid and the pivot are then as they were.
#[derive(ThisError, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PivotRefreshError {
    /// The cache could not be read again from its source (no worksheet source, a limit).
    #[error(transparent)]
    Cache(#[from] PivotCacheError),
    /// The pivot could not be computed or laid out (a work or size limit, a malformed pivot).
    #[error(transparent)]
    Compute(#[from] PivotComputeError),
    /// A field the pivot shows is no longer in the source. What Excel does then is UNVERIFIED
    /// (probe pending: T-4664).
    #[error("the pivot field {0:?} is no longer in the source")]
    FieldGone(String),
    /// The refresh would renumber the fields of a pivot that keeps parts of its definition
    /// verbatim ([`PivotTable::opaque`]), whose field indices nothing can remap.
    #[error("a refresh would renumber the fields of a pivot with unread parts")]
    FieldsMoved,
    /// The cache, the pivot or its sheet is not in the workbook.
    #[error("no such pivot, cache or sheet")]
    Missing,
}

/// One pivot as a plan lays it out on its sheet.
#[derive(Debug, Clone, PartialEq)]
pub struct PivotPlacement {
    /// The pivot's index in [`Workbook::pivot_tables`](crate::types::Workbook::pivot_tables).
    pub table: usize,
    /// The pivot's sheet index.
    pub sheet: u32,
    /// The pivot as it will be: field indices over the new cache, and the new location and
    /// geometry.
    pub pivot: PivotTable,
    /// The 0-based (row, column) of each cell the pivot owned that it no longer writes, emptied.
    pub cleared: Vec<(u32, u32)>,
    /// The cells it writes.
    pub cells: Vec<RenderedCell>,
    /// Whether the layout met a foreign cell, so the pivot collapsed to `#SPILL!`.
    pub spilled: bool,
}

/// What a refresh or render writes: [`Model::apply_pivot_refresh`]'s input.
#[derive(Debug, Clone, PartialEq)]
pub struct PivotRefresh {
    /// The cache's index.
    pub cache_index: u32,
    /// The refreshed cache; `None` when the plan only lays the pivot out again.
    pub cache: Option<PivotCache>,
    /// Each pivot laid out.
    pub placements: Vec<PivotPlacement>,
}

impl PivotRefresh {
    /// Every cell the plan writes or empties as 0-based (sheet index, row, column), for a
    /// caller's undo log.
    pub fn touched(&self) -> impl Iterator<Item = (u32, u32, u32)> + '_ {
        self.placements.iter().flat_map(|p| {
            let cleared = p
                .cleared
                .iter()
                .map(move |&(row, column)| (p.sheet, row, column));
            let written = p.cells.iter().map(move |c| (p.sheet, c.row, c.column));
            cleared.chain(written)
        })
    }
}

/// The cache and pivots a refresh replaced, as [`Model::apply_pivot_refresh`] returns them;
/// [`Model::restore_pivot_swap`] puts them back.
#[derive(Debug, Clone, PartialEq)]
pub struct PivotSwap {
    cache_index: u32,
    cache: Option<PivotCache>,
    tables: Vec<(usize, PivotTable)>,
}

/// The cells a pivot owns: its location and its page area (pivots.md "Page fields render above
/// the location": `rowPageCount` rows ending one row above the location, three columns per page
/// column but the last one's gap).
fn owned_areas(location: CellArea, geometry: &PivotGeometry) -> [Option<CellArea>; 2] {
    let (rows, columns) = (geometry.row_page_count, geometry.column_page_count);
    let page = (rows > 0 && columns > 0)
        .then(|| {
            let top = location.top.checked_sub(rows.checked_add(1)?)?;
            let right = location
                .left
                .checked_add(columns.checked_mul(3)?.checked_sub(2)?)?;
            Some(CellArea {
                top,
                left: location.left,
                bottom: location.top.checked_sub(2)?,
                right,
            })
        })
        .flatten();
    [Some(location), page]
}

fn contains(area: &CellArea, row: u32, column: u32) -> bool {
    (area.top..=area.bottom).contains(&row) && (area.left..=area.right).contains(&column)
}

fn overlaps(a: &CellArea, b: &CellArea) -> bool {
    a.top <= b.bottom && b.top <= a.bottom && a.left <= b.right && b.left <= a.right
}

/// `pivot` over `new`, a refresh of its cache `old`: each field index follows its field by name
/// ([`field_map`]), a field new to the source joins on no axis, and a field gone from it is
/// dropped unless the pivot shows it ([`PivotRefreshError::FieldGone`]).
fn remap(
    pivot: &PivotTable,
    old: &PivotCache,
    new: &PivotCache,
) -> Result<PivotTable, PivotRefreshError> {
    let map = field_map(old, new);
    let identity = map.iter().zip(0u32..).all(|(&to, from)| to == Some(from));
    if identity {
        let mut pivot = pivot.clone();
        for field in new.fields.iter().skip(pivot.fields.len()) {
            pivot.fields.push(PivotField::new(None, field.items.len()));
        }
        return Ok(pivot);
    }
    if !pivot.opaque.is_empty() {
        return Err(PivotRefreshError::FieldsMoved);
    }
    let to = |from: u32| {
        map.get(from as usize).copied().flatten().ok_or_else(|| {
            let name = old.fields.get(from as usize).map(|f| f.name.clone());
            PivotRefreshError::FieldGone(name.unwrap_or_default())
        })
    };
    let axis = |fields: &[AxisField]| -> Result<Vec<AxisField>, PivotRefreshError> {
        fields
            .iter()
            .map(|field| match *field {
                AxisField::Field(from) => to(from).map(AxisField::Field),
                AxisField::Values => Ok(AxisField::Values),
            })
            .collect()
    };
    let mut out = pivot.clone();
    out.row_fields = axis(&pivot.row_fields)?;
    out.column_fields = axis(&pivot.column_fields)?;
    for page in &mut out.page_fields {
        page.field = to(page.field)?;
    }
    for data in &mut out.data_fields {
        data.field = to(data.field)?;
        // A base field gone from the source falls back to the first field, the schema's default.
        data.base_field = u32::try_from(data.base_field)
            .ok()
            .and_then(|from| to(from).ok())
            .and_then(|to| i32::try_from(to).ok())
            .unwrap_or(0);
    }
    out.fields = new
        .fields
        .iter()
        .zip(0u32..)
        .map(|(field, index)| {
            let from = map.iter().position(|&to| to == Some(index));
            from.and_then(|from| pivot.fields.get(from).cloned())
                .unwrap_or_else(|| PivotField::new(None, field.items.len()))
        })
        .collect();
    Ok(out)
}

impl Model {
    /// The plan that refreshes the cache at `cache_index` from its source and lays out again
    /// every pivot over it. Changes nothing.
    ///
    /// # Errors
    /// [`PivotRefreshError`]: the cache does not refresh, or a pivot over it does not lay out.
    pub fn plan_pivot_refresh(&self, cache_index: u32) -> Result<PivotRefresh, PivotRefreshError> {
        let caches = &self.workbook.pivot_caches;
        let old = caches
            .get(cache_index as usize)
            .ok_or(PivotRefreshError::Missing)?;
        // The record values the other caches hold count against the workbook's limit.
        let others: usize = (0u32..)
            .zip(caches)
            .filter(|&(i, _)| i != cache_index)
            .map(|(_, c)| (c.record_count as usize).saturating_mul(c.fields.len()))
            .fold(0, usize::saturating_add);
        let new = refresh(old, self, MAX_RECORD_VALUES.saturating_sub(others))?;
        let mut placements = Vec::new();
        for (table, pivot) in self.workbook.pivot_tables.iter().enumerate() {
            if pivot.cache == cache_index {
                let remapped = remap(pivot, old, &new)?;
                placements.push(self.place(table, remapped, &new)?);
            }
        }
        Ok(PivotRefresh {
            cache_index,
            cache: Some(new),
            placements,
        })
    }

    /// The plan that lays out again the pivot at index `table` over its cache as it is, without
    /// refreshing it. Changes nothing.
    ///
    /// # Errors
    /// [`PivotRefreshError`]: the pivot does not lay out.
    pub fn plan_pivot_render(&self, table: usize) -> Result<PivotRefresh, PivotRefreshError> {
        let pivot = self
            .workbook
            .pivot_tables
            .get(table)
            .ok_or(PivotRefreshError::Missing)?;
        let cache = self
            .workbook
            .pivot_caches
            .get(pivot.cache as usize)
            .ok_or(PivotRefreshError::Missing)?;
        Ok(PivotRefresh {
            cache_index: pivot.cache,
            cache: None,
            placements: vec![self.place(table, pivot.clone(), cache)?],
        })
    }

    /// `pivot` (the pivot at index `table`, over `cache`) laid out from its anchor, the top-left
    /// cell of its location: its cells, or `#SPILL!` there when they would meet a foreign cell.
    fn place(
        &self,
        table: usize,
        mut pivot: PivotTable,
        cache: &PivotCache,
    ) -> Result<PivotPlacement, PivotRefreshError> {
        let sheet = self
            .workbook
            .worksheets
            .iter()
            .position(|w| w.sheet_id == pivot.sheet_id)
            .and_then(|i| u32::try_from(i).ok())
            .ok_or(PivotRefreshError::Missing)?;
        let mut work = MAX_PIVOT_WORK;
        let result = compute(&pivot, cache, &mut work)?;
        let layout = lay_out(&pivot, cache, &result, &mut work)?;
        let mut cells = render(&pivot, cache, &result, &layout)?;
        let owned = owned_areas(pivot.location, &pivot.geometry);
        let new = owned_areas(layout.location, &layout.geometry);
        let spilled = self.meets_foreign(sheet, table, &pivot, &owned, &new);
        let anchor = CellArea {
            bottom: pivot.location.top,
            right: pivot.location.left,
            ..pivot.location
        };
        if spilled {
            cells = vec![RenderedCell {
                row: anchor.top,
                column: anchor.left,
                value: PivotCell::Error(Error::SPILL),
            }];
            pivot.location = anchor;
            pivot.geometry = PivotGeometry::default();
        } else {
            pivot.location = layout.location;
            pivot.geometry = layout.geometry;
        }
        let written: HashSet<(u32, u32)> = cells.iter().map(|c| (c.row, c.column)).collect();
        let mut cleared = Vec::new();
        for area in owned.iter().flatten() {
            for place in self.stored_in(sheet, area) {
                if !written.contains(&place) && !cleared.contains(&place) {
                    cleared.push(place);
                }
            }
        }
        Ok(PivotPlacement {
            table,
            sheet,
            pivot,
            cleared,
            cells,
            spilled,
        })
    }

    /// Whether the areas `new` of the pivot at index `table` meet a cell it does not own: a
    /// stored value or formula outside `owned`, or another pivot's area on the sheet.
    fn meets_foreign(
        &self,
        sheet: u32,
        table: usize,
        pivot: &PivotTable,
        owned: &[Option<CellArea>; 2],
        new: &[Option<CellArea>; 2],
    ) -> bool {
        let is_owned = |(row, column): (u32, u32)| {
            owned
                .iter()
                .flatten()
                .any(|area| contains(area, row, column))
        };
        let other_pivot = self
            .workbook
            .pivot_tables
            .iter()
            .enumerate()
            .filter(|&(i, other)| i != table && other.sheet_id == pivot.sheet_id)
            .flat_map(|(_, other)| owned_areas(other.location, &other.geometry))
            .flatten()
            .any(|area| new.iter().flatten().any(|n| overlaps(n, &area)));
        other_pivot
            || new.iter().flatten().any(|area| {
                self.stored_in(sheet, area)
                    .into_iter()
                    .any(|place| !is_owned(place))
            })
    }

    /// The 0-based places of the cells of `area` that hold a value or formula (a cell that only
    /// holds a style is empty), walking the cells stored there, never the area.
    fn stored_in(&self, sheet: u32, area: &CellArea) -> Vec<(u32, u32)> {
        let Ok(worksheet) = self.workbook.worksheet(sheet) else {
            return Vec::new();
        };
        let one_based = |n: u32| i32::try_from(n).ok().and_then(|n| n.checked_add(1));
        let (Some(top), Some(left), Some(bottom), Some(right)) = (
            one_based(area.top),
            one_based(area.left),
            one_based(area.bottom),
            one_based(area.right),
        ) else {
            return Vec::new();
        };
        let data = &worksheet.sheet_data;
        data.places_in_block((top, left), (right - left + 1, bottom - top + 1))
            .into_iter()
            .filter(|&(row, column)| {
                !matches!(data.cell(row, column), Some(Cell::EmptyCell { .. }))
            })
            .filter_map(|(row, column)| {
                Some((
                    u32::try_from(row - 1).ok()?,
                    u32::try_from(column - 1).ok()?,
                ))
            })
            .collect()
    }

    /// Writes `refresh`: empties each pivot's cleared cells, writes its cells as constants, and
    /// swaps in the refreshed cache and pivots, returning what they replaced. GETPIVOTDATA's
    /// indices are dropped, so the next recalculation reads the new pivots.
    ///
    /// # Errors
    /// The engine refusing a cell (one merged over, or inside an array formula). Cells may then
    /// be partly written and the pivots are as they were: the caller restores the cells from its
    /// undo log (the facade's `Txn` logs [`PivotRefresh::touched`] first).
    pub fn apply_pivot_refresh(&mut self, refresh: PivotRefresh) -> Result<PivotSwap, String> {
        let one_based = |n: u32| {
            i32::try_from(n)
                .ok()
                .and_then(|n| n.checked_add(1))
                .ok_or_else(|| "a pivot cell outside the sheet".to_string())
        };
        for placement in &refresh.placements {
            let sheet = placement.sheet;
            for &(row, column) in &placement.cleared {
                let (row, column) = (one_based(row)?, one_based(column)?);
                self.prepare_cell_for_user_input(sheet, row, column)?;
                self.workbook
                    .worksheet_mut(sheet)?
                    .cell_clear_contents(row, column)?;
            }
            for cell in &placement.cells {
                let (row, column) = (one_based(cell.row)?, one_based(cell.column)?);
                self.prepare_cell_for_user_input(sheet, row, column)?;
                match &cell.value {
                    PivotCell::Text(text) => self.update_cell_with_text(sheet, row, column, text),
                    PivotCell::Number(n) => self.update_cell_with_number(sheet, row, column, *n),
                    PivotCell::Error(error) => {
                        self.update_cell_with_error(sheet, row, column, error.clone())
                    }
                }?;
            }
        }
        let tables = refresh
            .placements
            .into_iter()
            .map(|p| (p.table, p.pivot))
            .collect();
        Ok(self.restore_pivot_swap(PivotSwap {
            cache_index: refresh.cache_index,
            cache: refresh.cache,
            tables,
        }))
    }

    /// Puts the cache and pivots of `swap` in place, returning the ones they replace; a swap of
    /// a pivot or cache the workbook no longer has is dropped.
    pub fn restore_pivot_swap(&mut self, swap: PivotSwap) -> PivotSwap {
        let workbook = &mut self.workbook;
        let cache = swap.cache.and_then(|cache| {
            let slot = workbook.pivot_caches.get_mut(swap.cache_index as usize)?;
            Some(std::mem::replace(slot, cache))
        });
        let tables = swap
            .tables
            .into_iter()
            .filter_map(|(index, table)| {
                let slot = workbook.pivot_tables.get_mut(index)?;
                Some((index, std::mem::replace(slot, table)))
            })
            .collect();
        self.evaluation.pivot_indexes.clear();
        PivotSwap {
            cache_index: swap.cache_index,
            cache,
            tables,
        }
    }
}

#[cfg(test)]
mod tests;
