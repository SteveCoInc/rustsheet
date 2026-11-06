//! [`Txn::add_pivot_table`]: test support that registers a pivot over a source range (SPEC-0015).

use rustsheet_engine::cell::CellValue;
use rustsheet_engine::formatter::lexer::is_likely_date_number_format;
use rustsheet_engine::pivot::{
    group_by_dates, group_by_numbers, layout_extent, row_layout, ungroup_dates, Aggregate, Axis,
    AxisField, CacheBuilder, CacheField, CacheItem, CacheSource, CellArea, DataField,
    DateGroupError, DateGroups, LayoutCell, NumberGroupError, PivotCache, PivotCacheError,
    PivotField, PivotTable, RecordValue, MAX_RECORD_VALUES,
};
use rustsheet_engine::Model;

pub use rustsheet_engine::pivot::DateGroupBy as PivotDateGroupBy;

use super::{Input, Txn, Undo};
use crate::error::{EditError, RefError};
use crate::refs::RangeRef;
use crate::value::{ErrorKind, Value};
use crate::workbook::{engine_rc, sheet_index};

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

/// Field `field` grouped by `group`, as the xlsx importer groups one (`group_by_numbers`,
/// `group_by_dates`), or left ungrouped where Excel's refresh drops the grouping. `holds_dates`
/// says whether the field's source cells hold a number in a date format.
fn grouped(
    field: &CacheField,
    group: &PivotRangeGroup,
    holds_dates: bool,
) -> Result<CacheField, PivotError> {
    let labels: Vec<CacheItem> = group.items.iter().cloned().map(CacheItem::Text).collect();
    Ok(match group.by {
        None => {
            // Excel's refresh drops a numeric grouping over a blank and over any date cell,
            // among numbers or alone: each value is its own item, in record order, the blank
            // `(blank)` (probes/t4287-pivot-group-whole-end pivot S,
            // t4287-pivot-group-dates-as-numbers, t4316-pivot-group-dates-alone; web.md "Grouped
            // pivot fields import as their group items"). The xlsx importer keeps a saved
            // grouping: only this refresh-like path drops one.
            if field.items.contains(&CacheItem::Missing) || holds_dates {
                return Ok(field.clone());
            }
            group_by_numbers(field, [group.start, group.end, group.interval], &labels)?
        }
        Some(by) => {
            // Excel's refresh drops a date grouping over a blank (probes/t4236-pivot-date-groups,
            // web.md): each second-rounded date is its own item.
            if field.items.contains(&CacheItem::Missing) {
                return Ok(ungroup_dates(field)?);
            }
            let groups = DateGroups::new(by, group.start, group.end, labels.len())?;
            group_by_dates(field, groups, labels)?
        }
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
        let value_at = |row: u32, col: u32| -> Result<CellValue, EditError> {
            let (r, c) = engine_rc(row, col).ok_or(RefError::OutOfBounds)?;
            model
                .get_cell_value_by_index(source_sheet, r, c)
                .map_err(EditError::Engine)
        };
        let used: usize = model
            .workbook
            .pivot_caches
            .iter()
            .map(|c| c.record_count as usize * c.fields.len())
            .sum();
        let mut builder = CacheBuilder::new(
            Some(CacheSource {
                sheet_name,
                range: input.source.to_string(),
            }),
            MAX_RECORD_VALUES.saturating_sub(used),
        );
        let mut names = Vec::new();
        for col in input.source.left..=input.source.right {
            let CellValue::String(name) = value_at(input.source.top, col)? else {
                let at = crate::refs::CellRef::new(input.source.sheet, input.source.top, col)?;
                return Err(EditError::Pivot(PivotError::Header(at.to_string())));
            };
            builder.add_field(name.clone()).map_err(PivotError::from)?;
            names.push(name);
        }
        if input.source.bottom == input.source.top {
            return Err(PivotError::NoRecords.into());
        }
        for row in input.source.top + 1..=input.source.bottom {
            let mut record = Vec::with_capacity(names.len());
            for col in input.source.left..=input.source.right {
                record.push(RecordValue::Inline(match value_at(row, col)? {
                    CellValue::None => CacheItem::Missing,
                    CellValue::String(s) => CacheItem::Text(s),
                    CellValue::Number(n) => CacheItem::Number(n),
                    CellValue::Boolean(b) => CacheItem::Boolean(b),
                }));
            }
            builder.push_record(record).map_err(PivotError::from)?;
        }
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
        // Exact-case match: the builder gives the header's own spelling. Excel's GETPIVOTDATA matches
        // field names ignoring case (web.md); this is not that rule.
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
            let holds_dates = group.by.is_none() && self.holds_dates(input, row)?;
            if let Some(field) = cache.fields.get_mut(row as usize) {
                *field = grouped(field, group, holds_dates)?;
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

    /// Whether the record cells of source column `field` hold a number shown in a date format
    /// (`is_likely_date_number_format`), which Excel caches as a date.
    fn holds_dates(&self, input: &PivotTableInput, field: u32) -> Result<bool, EditError> {
        let model: &Model = self.model;
        let sheet = sheet_index(model, input.source.sheet)?;
        let col = input.source.left + field;
        for row in input.source.top + 1..=input.source.bottom {
            let (r, c) = engine_rc(row, col).ok_or(RefError::OutOfBounds)?;
            if !matches!(
                model.get_cell_value_by_index(sheet, r, c),
                Ok(CellValue::Number(_))
            ) {
                continue;
            }
            let style = model
                .get_style_for_cell(sheet, r, c)
                .map_err(EditError::Engine)?;
            if is_likely_date_number_format(&style.num_fmt) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Writes the compact-form layout of grouped pivot `table` from `input.location`'s top-left
    /// cell ([`row_layout`]: the captions, each group that holds a record with its value, every
    /// group under the row field's `showAll`, and the grand total) and returns the area it covers. Excel lays out a grouped pivot itself on
    /// load (probes/t4228-pivot-range-groups reads it back), and the case builder renders none
    /// (SPEC-0015 "Spec cases in the eval adapter", T-4235).
    fn lay_out_grouped(
        &mut self,
        input: &PivotTableInput,
        cache: &PivotCache,
        table: &PivotTable,
    ) -> Result<CellArea, EditError> {
        let bad = || PivotError::Cache(PivotCacheError::BadRecord);
        let (Some(AxisField::Field(row)), Some(data)) =
            (table.row_fields.first(), table.data_fields.first())
        else {
            return Err(bad().into());
        };
        let show_all = table.fields.get(*row as usize).is_some_and(|f| f.show_all);
        let rows = row_layout(
            cache,
            [*row, data.field],
            data.function,
            &data.caption,
            show_all,
        )
        .ok_or_else(bad)?;
        let (sheet, top, left) = (
            input.location.sheet,
            input.location.top,
            input.location.left,
        );
        let (bottom, right) =
            layout_extent(top, left, u32::try_from(rows.len()).unwrap_or(u32::MAX));
        for (r, cells) in (top..).zip(rows) {
            for (c, cell) in (left..).zip(cells) {
                // Typed values: a label such as `1-1.1` is text, never parsed as a date.
                let value = match cell {
                    LayoutCell::Text(text) => Value::Text(text.into()),
                    LayoutCell::Number(n) => Value::Number(n),
                    LayoutCell::Error(error) => Value::Error(ErrorKind::from_engine(&error).into()),
                    LayoutCell::Blank => continue,
                };
                self.set(crate::refs::CellRef::new(sheet, r, c)?, Input::Value(value))?;
            }
        }
        Ok(CellArea {
            top,
            left,
            bottom,
            right,
        })
    }
}

#[cfg(test)]
mod tests;
