//! [`Txn::add_pivot_table`]: test support that registers a pivot over a source range (SPEC-0015).

use rustsheet_engine::pivot::{
    compute, lay_out, read_source, regroup, render, Aggregate, Axis, AxisField, CacheField,
    CacheItem, CacheSource, CellArea, DataField, DateGroupError, NumberGroupError, PivotCache,
    PivotCacheError, PivotCell, PivotComputeError, PivotField, PivotTable, RangeGrouping,
    RegroupError, MAX_PIVOT_WORK, MAX_RECORD_VALUES,
};
use rustsheet_engine::Model;

pub use rustsheet_engine::pivot::DateGroupBy as PivotDateGroupBy;

use super::{Input, Txn, Undo};
use crate::error::{EditError, RefError};
use crate::refs::RangeRef;
use crate::value::{ErrorKind, Value};
use crate::workbook::sheet_index;

/// A data field's summary function.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[non_exhaustive]
pub enum PivotFunction {
    /// `sum`.
    Sum,
    /// `count`.
    Count,
    /// `average`.
    Average,
    /// `max`.
    Max,
    /// `min`.
    Min,
}

impl PivotFunction {
    /// The function named `name` in a spec (`sum`, `count`, `average`, `max`, `min`).
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "sum" => Self::Sum,
            "count" => Self::Count,
            "average" => Self::Average,
            "max" => Self::Max,
            "min" => Self::Min,
            _ => return None,
        })
    }

    /// The word a new data field's caption starts with (`Sum of Sales`). Oracle evidence covers `Sum`
    /// and `Average` only (probes/pivot-rendered-cells); the others follow the builder.
    #[must_use]
    pub fn caption_name(self) -> &'static str {
        match self {
            Self::Sum => "Sum",
            Self::Count => "Count",
            Self::Average => "Average",
            Self::Max => "Max",
            Self::Min => "Min",
        }
    }
}

impl From<PivotFunction> for Aggregate {
    fn from(function: PivotFunction) -> Self {
        match function {
            PivotFunction::Sum => Aggregate::Sum,
            PivotFunction::Count => Aggregate::Count,
            PivotFunction::Average => Aggregate::Average,
            PivotFunction::Max => Aggregate::Max,
            PivotFunction::Min => Aggregate::Min,
        }
    }
}

/// A pivot table to register: one row field, an optional column field and one data field over a
/// source range. Test support (`Txn::add_pivot_table`), not public API.
#[derive(Clone, Debug)]
pub struct PivotTableInput {
    /// The pivot's name.
    pub name: String,
    /// The source range, field names (text) in its first row, constants below.
    pub source: RangeRef,
    /// The cells the rendered pivot covers; its sheet is the pivot's sheet.
    pub location: RangeRef,
    /// The row field's name.
    pub row: String,
    /// The column field's name, if any.
    pub column: Option<String>,
    /// The data field's source field name.
    pub value: String,
    /// How the data field summarizes.
    pub function: PivotFunction,
    /// The data field's caption (`Sum of Sales`).
    pub caption: String,
    /// A numeric range or date grouping of the row field, if any. A grouped pivot is laid out by the
    /// engine (Excel lays one out itself on load): its rendered cells are written from
    /// `location`'s top-left cell, which is all of `location` that is read.
    pub group: Option<PivotRangeGroup>,
    /// Text items of the row field that no record holds, appended to its cache items after the
    /// records' own (a `<sharedItems>` entry no record uses). Only on an ungrouped row field.
    pub unused_items: Vec<String>,
    /// The row field's `showAll`: whether the pivot shows its items that no record holds.
    pub show_all: bool,
    /// `unused_items` for the column field (T-4311).
    pub column_unused_items: Vec<String>,
    /// The column field's `showAll`.
    pub column_show_all: bool,
}

/// A row field grouped by numbers or dates (`<fieldGroup>` with `<rangePr>`), grouped as the xlsx
/// importer groups one (`rustsheet_engine::pivot::group_by_numbers`, `group_by_dates`).
#[derive(Clone, Debug, PartialEq)]
pub struct PivotRangeGroup {
    /// What a date group groups by; `None` for numbers (ECMA-376's `range`).
    pub by: Option<PivotDateGroupBy>,
    /// `startNum`, or `startDate` as a serial.
    pub start: f64,
    /// `endNum`, or `endDate` as a serial.
    pub end: f64,
    /// `groupInterval` (1 for a date group).
    pub interval: f64,
    /// The `<groupItems>` labels, in order.
    pub items: Vec<String>,
}

/// Field `field` grouped by `group` as a refresh groups one (`rustsheet_engine::pivot::regroup`,
/// shared with the engine's refresh): a numeric grouping over a blank or a date, or a date
/// grouping over a blank, is dropped as Excel's refresh drops it. The xlsx importer keeps a
/// saved grouping: only this refresh-like path drops one.
fn grouped(field: &CacheField, group: &PivotRangeGroup) -> Result<CacheField, PivotError> {
    let labels: Vec<CacheItem> = group.items.iter().cloned().map(CacheItem::Text).collect();
    let grouping = match group.by {
        None => RangeGrouping::Numbers {
            start: group.start,
            end: group.end,
            interval: group.interval,
        },
        Some(by) => RangeGrouping::Dates {
            by,
            start: group.start,
            end: group.end,
        },
    };
    regroup(field, &grouping, &labels).map_err(|error| match error {
        RegroupError::Numbers(error) => PivotError::Group(error),
        RegroupError::Dates(error) => PivotError::DateGroup(error),
    })
}

/// The pivot fields of `cache`: `row` on the row axis with `showAll` `show_all`, `column` on
/// the column axis with `column_show_all`, the others on none.
fn pivot_fields(
    cache: &PivotCache,
    row: u32,
    column: Option<u32>,
    [show_all, column_show_all]: [bool; 2],
) -> Vec<PivotField> {
    cache
        .fields
        .iter()
        .zip(0u32..)
        .map(|(f, i)| {
            let axis = if i == row {
                Some(Axis::Row)
            } else if Some(i) == column {
                Some(Axis::Column)
            } else {
                None
            };
            let mut field = PivotField::new(axis, f.items.len());
            if i == row {
                field.show_all = show_all;
            } else if Some(i) == column {
                field.show_all = column_show_all;
            }
            field
        })
        .collect()
}

/// Why a pivot table could not be registered.
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PivotError {
    /// A header cell of the source is not text.
    #[error("pivot source header cell {0} is not text")]
    Header(String),
    /// A field name is not in the source's header row.
    #[error("pivot field {0:?} is not in the source header row")]
    UnknownField(String),
    /// The row, column and data fields are not three distinct (or two distinct) fields.
    #[error("a pivot's row, column and data fields must differ")]
    RepeatedField,
    /// The source has no data row.
    #[error("the pivot source has no data row")]
    NoRecords,
    /// The cache exceeds a limit.
    #[error(transparent)]
    Cache(#[from] PivotCacheError),
    /// The row field's grouping cannot apply (a value that is not a number, a bad range).
    #[error(transparent)]
    Group(#[from] NumberGroupError),
    /// The row field's date grouping cannot apply (a value that is not a date, a wrong item
    /// count).
    #[error(transparent)]
    DateGroup(#[from] DateGroupError),
    /// A grouped pivot with a column field: only a grouped row field is laid out.
    #[error("a grouped pivot has no column field")]
    GroupedWithColumn,
    /// Unused items on a grouped row field: their place among the group items is unprobed.
    #[error("a grouped pivot has no unused items")]
    GroupedWithUnusedItems,
    /// The grouped pivot could not be computed or laid out (past a work or size limit, or a
    /// definition that does not match its cache).
    #[error(transparent)]
    Compute(#[from] PivotComputeError),
}

impl Txn<'_> {
    /// Registers a pivot table over `input.source`, building its cache from the source cells as
    /// they hold now (the snapshot `refreshOnLoad` gives); the rendered cells are the caller's to
    /// write. Test support for the eval adapter, not public API, and not mirrored in the Python
    /// binding (SPEC-0015). A change to the workbook's pivots makes the transaction evaluate, so
    /// every GETPIVOTDATA cell is recalculated.
    ///
    /// # Errors
    /// [`EditError::Ref`] for an unknown sheet or a range outside the sheet,
    /// [`EditError::Pivot`] for a malformed source or field, or a cache over a limit.
    #[doc(hidden)]
    pub fn add_pivot_table(&mut self, input: &PivotTableInput) -> Result<(), EditError> {
        let (cache, mut table) = self.build_pivot(input)?;
        if input.group.is_some() {
            table.location = self.lay_out_grouped(input, &cache, &table)?;
        }
        let (caches, tables) = (
            self.model.workbook.pivot_caches.len(),
            self.model.workbook.pivot_tables.len(),
        );
        self.model.workbook.pivot_caches.push(cache);
        self.model.workbook.pivot_tables.push(table);
        self.sheet_op = true;
        self.undo.push(Undo::Pivot { caches, tables });
        Ok(())
    }

    fn build_pivot(
        &self,
        input: &PivotTableInput,
    ) -> Result<(rustsheet_engine::pivot::PivotCache, PivotTable), EditError> {
        let model: &Model = self.model;
        let source_sheet = sheet_index(model, input.source.sheet)?;
        let sheet_name = model
            .workbook
            .worksheet(source_sheet)
            .map_err(EditError::Engine)?
            .get_name();
        let pivot_sheet = sheet_index(model, input.location.sheet)?;
        let sheet_id = model
            .workbook
            .worksheet(pivot_sheet)
            .map_err(EditError::Engine)?
            .get_sheet_id();
        let used: usize = model
            .workbook
            .pivot_caches
            .iter()
            .map(|c| c.record_count as usize * c.fields.len())
            .sum();
        if input.source.bottom == input.source.top {
            return Err(PivotError::NoRecords.into());
        }
        // The cache is the source read as Excel's refresh reads it (field names, item typing
        // and identity), from no earlier cache: the snapshot `refreshOnLoad` gives.
        let new = PivotCache {
            source: Some(CacheSource {
                sheet_name,
                range: input.source.to_string(),
            }),
            ..PivotCache::default()
        };
        let mut builder = match read_source(&new, model, MAX_RECORD_VALUES.saturating_sub(used)) {
            Ok(builder) => builder,
            Err(PivotCacheError::Header { column }) => {
                let col = input.source.left + column;
                let at = crate::refs::CellRef::new(input.source.sheet, input.source.top, col)?;
                return Err(EditError::Pivot(PivotError::Header(at.to_string())));
            }
            Err(error) => return Err(PivotError::from(error).into()),
        };
        let names: Vec<String> = builder.fields().iter().map(|f| f.name.clone()).collect();
        if input.group.is_some() && !input.unused_items.is_empty() {
            return Err(PivotError::GroupedWithUnusedItems.into());
        }
        for (name, unused) in [
            (Some(&input.row), &input.unused_items),
            (input.column.as_ref(), &input.column_unused_items),
        ] {
            let index = name.and_then(|name| names.iter().position(|n| n == name));
            if let Some(index) = index.and_then(|i| u32::try_from(i).ok()) {
                for item in unused {
                    builder
                        .add_shared_item(index, CacheItem::Text(item.clone()))
                        .map_err(PivotError::from)?;
                }
            }
        }
        let mut cache = builder.finish();
        // Exact-case match on the cache's field names (a header's own spelling, a repeated one
        // numbered: `Sales2`). Excel's GETPIVOTDATA matches field names ignoring case (web.md);
        // this is not that rule.
        let field = |name: &str| {
            names
                .iter()
                .position(|n| n == name)
                .and_then(|i| u32::try_from(i).ok())
                .ok_or_else(|| PivotError::UnknownField(name.to_string()))
        };
        let row = field(&input.row)?;
        let column = input.column.as_deref().map(field).transpose()?;
        let value = field(&input.value)?;
        if Some(row) == column || row == value || column == Some(value) {
            return Err(PivotError::RepeatedField.into());
        }
        if let Some(group) = &input.group {
            if column.is_some() {
                return Err(PivotError::GroupedWithColumn.into());
            }
            if let Some(field) = cache.fields.get_mut(row as usize) {
                *field = grouped(field, group)?;
            }
        }
        let fields = pivot_fields(&cache, row, column, [input.show_all, input.column_show_all]);
        let location = CellArea {
            top: input.location.top,
            left: input.location.left,
            bottom: input.location.bottom,
            right: input.location.right,
        };
        let cache_index =
            u32::try_from(model.workbook.pivot_caches.len()).map_err(|_| RefError::OutOfBounds)?;
        let table = PivotTable {
            fields,
            row_fields: vec![AxisField::Field(row)],
            column_fields: column.map(AxisField::Field).into_iter().collect(),
            data_fields: vec![DataField {
                caption: input.caption.clone(),
                field: value,
                function: input.function.into(),
                ..DataField::default()
            }],
            ..PivotTable::new(input.name.clone(), sheet_id, location, cache_index)
        };
        Ok((cache, table))
    }

    /// Writes the cells of grouped pivot `table` over `cache` and returns the area they cover:
    /// its result, layout lines and cells come from the engine's one pivot pipeline
    /// (`compute`, `lay_out`, `render`; T-4149), so a compact pivot of one row field shows the
    /// captions, each group that holds a record with its value, every group under the row
    /// field's `showAll`, and the grand total. Excel lays out a grouped pivot itself on load
    /// (probes/t4228-pivot-range-groups reads it back), and the case builder renders none
    /// (SPEC-0015 "Spec cases in the eval adapter", T-4235).
    fn lay_out_grouped(
        &mut self,
        input: &PivotTableInput,
        cache: &PivotCache,
        table: &PivotTable,
    ) -> Result<CellArea, EditError> {
        let mut work = MAX_PIVOT_WORK;
        let result = compute(table, cache, &mut work).map_err(PivotError::from)?;
        let layout = lay_out(table, cache, &result, &mut work).map_err(PivotError::from)?;
        let cells = render(table, cache, &result, &layout).map_err(PivotError::from)?;
        for cell in cells {
            // Typed values: a label such as `1-1.1` is text, never parsed as a date.
            let value = match cell.value {
                PivotCell::Text(text) => Value::Text(text.into()),
                PivotCell::Number(n) => Value::Number(n),
                PivotCell::Error(error) => Value::Error(ErrorKind::from_engine(&error).into()),
            };
            let at = crate::refs::CellRef::new(input.location.sheet, cell.row, cell.column)?;
            self.set(at, Input::Value(value))?;
        }
        Ok(layout.location)
    }
}

#[cfg(test)]
mod tests;
