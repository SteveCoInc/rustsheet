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
/// `group_by_dates`), or left ungrouped where Excel's refresh drops the grouping. `mixes_dates`
/// says whether the field's source cells hold numbers both in a date format and not.
fn grouped(
    field: &CacheField,
    group: &PivotRangeGroup,
    mixes_dates: bool,
) -> Result<CacheField, PivotError> {
    let labels: Vec<CacheItem> = group.items.iter().cloned().map(CacheItem::Text).collect();
    Ok(match group.by {
        None => {
            // Excel's refresh drops a numeric grouping over a blank, and over date cells among
            // numbers: each value is its own item, in record order, the blank `(blank)`
            // (probes/t4287-pivot-group-whole-end pivot S, t4287-pivot-group-dates-as-numbers;
            // web.md "Grouped pivot fields import as their group items").
            // A numeric grouping over date cells alone is unprobed and stays grouped. The xlsx
            // importer keeps a saved grouping: only this refresh-like path drops one.
            if field.items.contains(&CacheItem::Missing) || mixes_dates {
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
/// the column axis, the others on none.
fn pivot_fields(
    cache: &PivotCache,
    row: u32,
    column: Option<u32>,
    show_all: bool,
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
        let row_index = names.iter().position(|n| *n == input.row);
        if let Some(row) = row_index.and_then(|i| u32::try_from(i).ok()) {
            if input.group.is_some() && !input.unused_items.is_empty() {
                return Err(PivotError::GroupedWithUnusedItems.into());
            }
            for item in &input.unused_items {
                builder
                    .add_shared_item(row, CacheItem::Text(item.clone()))
                    .map_err(PivotError::from)?;
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
            let mixes_dates = group.by.is_none() && self.mixes_dates(input, row)?;
            if let Some(field) = cache.fields.get_mut(row as usize) {
                *field = grouped(field, group, mixes_dates)?;
            }
        }
        let fields = pivot_fields(&cache, row, column, input.show_all);
        let table = PivotTable {
            name: input.name.clone(),
            sheet_id,
            location: CellArea {
                top: input.location.top,
                left: input.location.left,
                bottom: input.location.bottom,
                right: input.location.right,
            },
            cache: u32::try_from(model.workbook.pivot_caches.len())
                .map_err(|_| RefError::OutOfBounds)?,
            fields,
            row_fields: vec![AxisField::Field(row)],
            column_fields: column.map(AxisField::Field).into_iter().collect(),
            data_fields: vec![DataField {
                caption: input.caption.clone(),
                field: value,
                function: input.function.into(),
            }],
        };
        Ok((cache, table))
    }

    /// Whether the record cells of source column `field` hold a number shown in a date format
    /// (`is_likely_date_number_format`), which Excel caches as a date, and a number that is not.
    fn mixes_dates(&self, input: &PivotTableInput, field: u32) -> Result<bool, EditError> {
        let model: &Model = self.model;
        let sheet = sheet_index(model, input.source.sheet)?;
        let col = input.source.left + field;
        let (mut dates, mut numbers) = (false, false);
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
                dates = true;
            } else {
                numbers = true;
            }
            if dates && numbers {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Writes the compact-form layout of grouped pivot `table` from `input.location`'s top-left
    /// cell ([`row_layout`]: the captions, each group that holds a record with its value, and
    /// the grand total) and returns the area it covers. Excel lays out a grouped pivot itself on
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
        let rows =
            row_layout(cache, *row, data.field, data.function, &data.caption).ok_or_else(bad)?;
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
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests")]

    use super::*;
    use crate::{Input, Workbook, WorkbookOptions};

    /// Region/Sales over A1:B4 and a pivot rendered at D1:E4.
    fn book() -> (Workbook, PivotTableInput) {
        let mut wb = Workbook::new(&WorkbookOptions::default()).unwrap();
        wb.edit(|tx| {
            for (a1, input) in [
                ("A1", Input::from("Region")),
                ("B1", Input::from("Sales")),
                ("A2", Input::from("East")),
                ("B2", Input::from(10.0)),
                ("A3", Input::from("West")),
                ("B3", Input::from(20.0)),
                ("A4", Input::from("East")),
                ("B4", Input::from(5.0)),
            ] {
                tx.set(a1, input)?;
            }
            Ok(())
        })
        .unwrap();
        let input = PivotTableInput {
            name: "Pv".into(),
            source: wb.range_ref("A1:B4").unwrap(),
            location: wb.range_ref("D1:E4").unwrap(),
            row: "Region".into(),
            column: None,
            value: "Sales".into(),
            function: PivotFunction::Sum,
            caption: "Sum of Sales".into(),
            group: None,
            unused_items: Vec::new(),
            show_all: false,
        };
        (wb, input)
    }

    #[test]
    fn the_cache_and_definition_are_built_from_the_source_cells() {
        let (mut wb, input) = book();
        wb.edit(|tx| tx.add_pivot_table(&input)).unwrap();
        let [cache] = &wb.model.workbook.pivot_caches[..] else {
            panic!("one cache");
        };
        assert_eq!(cache.record_count, 3);
        assert_eq!(cache.fields[0].name, "Region");
        assert_eq!(cache.fields[0].items.len(), 2);
        assert_eq!(cache.fields[0].records, vec![0, 1, 0]);
        let source = cache.source.as_ref().unwrap();
        assert_eq!(
            (source.sheet_name.as_str(), source.range.as_str()),
            ("Sheet1", "A1:B4")
        );
        let [pivot] = &wb.model.workbook.pivot_tables[..] else {
            panic!("one pivot");
        };
        assert_eq!(
            pivot.location,
            CellArea {
                top: 0,
                left: 3,
                bottom: 3,
                right: 4
            }
        );
        assert_eq!(pivot.row_fields, vec![AxisField::Field(0)]);
        assert_eq!(pivot.data_fields[0].field, 1);
        assert_eq!(pivot.fields[0].axis, Some(Axis::Row));
    }

    /// Unused items are appended to the row field's items after the records' own, and the row
    /// field takes the input's `showAll` (T-4276; probes/t4248-getpivotdata-recordless-item).
    #[test]
    fn unused_items_follow_the_records_items_and_show_all_is_the_row_fields() {
        let (mut wb, input) = book();
        let input = PivotTableInput {
            unused_items: vec!["North".into()],
            show_all: true,
            ..input
        };
        wb.edit(|tx| tx.add_pivot_table(&input)).unwrap();
        let [cache] = &wb.model.workbook.pivot_caches[..] else {
            panic!("one cache");
        };
        assert_eq!(
            cache.fields[0].items.last(),
            Some(&CacheItem::Text("North".into()))
        );
        assert!(!cache.fields[0].records.contains(&2));
        let pivot = &wb.model.workbook.pivot_tables[0];
        assert!(pivot.fields[0].show_all);
        assert_eq!(pivot.fields[0].hidden.len(), 1);
        // On a grouped row field their place among the group items is unprobed: refused.
        let grouped = PivotTableInput {
            group: Some(PivotRangeGroup {
                by: None,
                start: 0.0,
                end: 30.0,
                interval: 10.0,
                items: ["<0", "0-10", "10-20", "20-30", ">30"]
                    .map(String::from)
                    .to_vec(),
            }),
            ..input
        };
        let err = wb.edit(|tx| tx.add_pivot_table(&grouped)).unwrap_err();
        assert!(
            matches!(&err, EditError::Pivot(PivotError::GroupedWithUnusedItems)),
            "{err:?}"
        );
    }

    #[test]
    fn bad_pivots_are_typed_errors_and_leave_no_pivot() {
        let (mut wb, input) = book();
        for (change, expected) in [
            (
                PivotTableInput {
                    row: "Nope".into(),
                    ..input.clone()
                },
                PivotError::UnknownField("Nope".into()),
            ),
            (
                PivotTableInput {
                    value: "Region".into(),
                    ..input.clone()
                },
                PivotError::RepeatedField,
            ),
            (
                PivotTableInput {
                    source: wb.range_ref("A1:B1").unwrap(),
                    ..input.clone()
                },
                PivotError::NoRecords,
            ),
            (
                PivotTableInput {
                    source: wb.range_ref("B2:B4").unwrap(),
                    ..input.clone()
                },
                PivotError::Header("B2".into()),
            ),
        ] {
            let err = wb.edit(|tx| tx.add_pivot_table(&change)).unwrap_err();
            assert!(
                matches!(&err, EditError::Pivot(e) if *e == expected),
                "{err:?}"
            );
        }
        assert!(wb.model.workbook.pivot_tables.is_empty());
        assert!(wb.model.workbook.pivot_caches.is_empty());
    }

    #[test]
    fn a_failed_edit_removes_the_pivot() {
        let (mut wb, input) = book();
        let failed: Result<((), _), _> = wb.edit(|tx| {
            tx.add_pivot_table(&input)?;
            tx.add_pivot_table(&PivotTableInput {
                row: "Nope".into(),
                ..input.clone()
            })
        });
        assert!(failed.is_err());
        assert!(wb.model.workbook.pivot_tables.is_empty());
        assert!(wb.model.workbook.pivot_caches.is_empty());
    }

    /// A `Sum of S` pivot of row field `row` over `source`, grouped by `group`, laid out from
    /// `location`.
    fn grouped_input(
        source: RangeRef,
        location: RangeRef,
        row: &str,
        group: PivotRangeGroup,
    ) -> PivotTableInput {
        PivotTableInput {
            name: "Pv".into(),
            source,
            location,
            row: row.into(),
            column: None,
            value: "S".into(),
            function: PivotFunction::Sum,
            caption: "Sum of S".into(),
            group: Some(group),
            unused_items: Vec::new(),
            show_all: false,
        }
    }

    /// Rows 1 to 6 of column `col`.
    fn column(wb: &Workbook, col: &str) -> Vec<Value> {
        (1..=6)
            .map(|row| wb.value(format!("{col}{row}").as_str()).unwrap())
            .collect()
    }

    /// A pivot grouped by 0.1 from 0 to 1 is laid out as Excel shows it (only the groups holding
    /// a record), and GETPIVOTDATA reads a group by its label or its lower bound
    /// (probes/t4228-pivot-range-groups).
    #[test]
    fn a_grouped_pivot_is_laid_out_and_answers_by_label_and_lower_bound() {
        let mut wb = Workbook::new(&WorkbookOptions::default()).unwrap();
        let items: Vec<String> = ["<0", "0-0.1", "0.1-0.2", "0.2-0.3", "0.3-0.4", "0.4-0.5"]
            .into_iter()
            .chain(["0.5-0.6", "0.6-0.7", "0.7-0.8", "0.8-0.9", "0.9-1", ">1"])
            .map(String::from)
            .collect();
        let (source, location) = (wb.range_ref("A1:B4").unwrap(), wb.range_ref("D1").unwrap());
        wb.edit(|tx| {
            for (a1, input) in [
                ("A1", Input::from("N")),
                ("B1", Input::from("S")),
                ("A2", Input::from(0.3)),
                ("B2", Input::from(1.0)),
                ("A3", Input::from(0.999_999_999_999_999_9)),
                ("B3", Input::from(2.0)),
                ("A4", Input::from(1.0)),
                ("B4", Input::from(4.0)),
                ("F1", Input::from("=GETPIVOTDATA(\"S\",D1,\"N\",0.3)")),
                (
                    "F2",
                    Input::from("=GETPIVOTDATA(\"S\",D1,\"N\",\"0.1-0.2\")"),
                ),
                ("F3", Input::from("=GETPIVOTDATA(\"S\",D1,\"N\",1)")),
                ("F4", Input::from("=GETPIVOTDATA(\"S\",D1,\"N\",0.35)")),
            ] {
                tx.set(a1, input)?;
            }
            tx.add_pivot_table(&grouped_input(
                source,
                location,
                "N",
                PivotRangeGroup {
                    by: None,
                    start: 0.0,
                    end: 1.0,
                    interval: 0.1,
                    items,
                },
            ))
        })
        .unwrap();
        let text = |s: &str| Value::Text(s.into());
        assert_eq!(
            column(&wb, "D"),
            [
                text("Row Labels"),
                text("0.3-0.4"),
                text("0.9-1"),
                text(">1"),
                text("Grand Total"),
                Value::Empty
            ]
        );
        assert_eq!(
            column(&wb, "E")[..5],
            [
                text("Sum of S"),
                1.0.into(),
                4.0.into(),
                2.0.into(),
                7.0.into()
            ]
        );
        let ref_error = |cell: &str| matches!(wb.value(cell).unwrap(), Value::Error(_));
        assert_eq!(wb.value("F1").unwrap(), 1.0.into());
        assert!(ref_error("F2"), "a group without a record is #REF!");
        assert_eq!(wb.value("F3").unwrap(), 2.0.into());
        assert!(ref_error("F4"), "0.35 is no group's lower bound");
        let [pivot] = &wb.model.workbook.pivot_tables[..] else {
            panic!("one pivot");
        };
        assert_eq!(
            pivot.location,
            CellArea {
                top: 0,
                left: 3,
                bottom: 4,
                right: 4
            }
        );
    }

    /// A numeric grouping over a blank, or over date cells among numbers, is dropped as Excel's
    /// refresh drops it: each value is its own item in record order and the blank is `(blank)`
    /// (probes/t4287-pivot-group-whole-end pivot S, t4287-pivot-group-dates-as-numbers pivot P).
    #[test]
    fn a_numeric_grouping_over_a_blank_or_mixed_dates_is_dropped() {
        for date_cell in [None, Some("A3")] {
            let mut wb = Workbook::new(&WorkbookOptions::default()).unwrap();
            let (source, location) = (wb.range_ref("A1:B5").unwrap(), wb.range_ref("D1").unwrap());
            wb.edit(|tx| {
                for (a1, input) in [
                    ("A1", Input::from("N")),
                    ("B1", Input::from("S")),
                    ("A2", Input::from(-1.0)),
                    ("B2", Input::from(1.0)),
                    ("A3", Input::from(4.0)),
                    ("B3", Input::from(2.0)),
                    ("B4", Input::from(4.0)),
                    ("A5", Input::from(7.0)),
                    ("B5", Input::from(8.0)),
                ] {
                    tx.set(a1, input)?;
                }
                if let Some(a1) = date_cell {
                    tx.set("A4", Input::from(10.0))?;
                    tx.set_number_format(a1, "yyyy-mm-dd")?;
                }
                tx.add_pivot_table(&grouped_input(
                    source,
                    location,
                    "N",
                    PivotRangeGroup {
                        by: None,
                        start: 0.0,
                        end: 9.0,
                        interval: 3.0,
                        items: ["<x", "x", ">x"].map(String::from).to_vec(),
                    },
                ))
            })
            .unwrap();
            let text = |s: &str| Value::Text(s.into());
            let third = if date_cell.is_some() {
                10.0.into()
            } else {
                text("(blank)")
            };
            assert_eq!(
                column(&wb, "D")[..6],
                [
                    text("Row Labels"),
                    (-1.0).into(),
                    4.0.into(),
                    third,
                    7.0.into(),
                    text("Grand Total"),
                ],
                "{date_cell:?}"
            );
        }
    }

    /// A date grouping over a blank is dropped as Excel's refresh drops it: each second-rounded
    /// date is its own item in record order, the blank is `(blank)`, and GETPIVOTDATA takes a
    /// date serial or `"(blank)"` but no group label (probes/t4236-pivot-date-groups pivot B).
    #[test]
    fn a_date_grouping_over_a_blank_is_dropped() {
        let mut wb = Workbook::new(&WorkbookOptions::default()).unwrap();
        let (source, location) = (wb.range_ref("A1:B5").unwrap(), wb.range_ref("D1").unwrap());
        wb.edit(|tx| {
            for (a1, input) in [
                ("A1", Input::from("D")),
                ("B1", Input::from("S")),
                ("B2", Input::from(1.0)),
                ("A3", Input::from(43590.0)),
                ("B3", Input::from(2.0)),
                ("A4", Input::from(43_465.999_994_212_965)),
                ("B4", Input::from(4.0)),
                ("A5", Input::from(43590.0)),
                ("B5", Input::from(8.0)),
                ("F1", Input::from("=GETPIVOTDATA(\"S\",D1,\"D\",43590)")),
                (
                    "F2",
                    Input::from("=GETPIVOTDATA(\"S\",D1,\"D\",\"(blank)\")"),
                ),
                ("F3", Input::from("=GETPIVOTDATA(\"S\",D1,\"D\",43466)")),
                ("F4", Input::from("=GETPIVOTDATA(\"S\",D1,\"D\",\"May\")")),
            ] {
                tx.set(a1, input)?;
            }
            let months = ["<1/1/2019", "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul"]
                .into_iter()
                .chain(["Aug", "Sep", "Oct", "Nov", "Dec", ">12/31/2019"]);
            tx.add_pivot_table(&grouped_input(
                source,
                location,
                "D",
                PivotRangeGroup {
                    by: Some(PivotDateGroupBy::Months),
                    start: 43466.0,
                    end: 43830.0,
                    interval: 1.0,
                    items: months.map(String::from).collect(),
                },
            ))
        })
        .unwrap();
        let text = |s: &str| Value::Text(s.into());
        assert_eq!(
            column(&wb, "D"),
            [
                text("Row Labels"),
                text("(blank)"),
                43590.0.into(),
                43466.0.into(),
                text("Grand Total"),
                Value::Empty
            ]
        );
        assert_eq!(
            column(&wb, "E")[..5],
            [
                text("Sum of S"),
                1.0.into(),
                10.0.into(),
                4.0.into(),
                15.0.into()
            ]
        );
        assert_eq!(wb.value("F1").unwrap(), 10.0.into());
        assert_eq!(wb.value("F2").unwrap(), 1.0.into());
        assert_eq!(wb.value("F3").unwrap(), 4.0.into());
        assert!(matches!(wb.value("F4").unwrap(), Value::Error(_)));
    }
}
