//! The pivot table API (SPEC-0032 "API", T-4178): [`Txn::add_pivot_table`] creates a pivot over a
//! source range and lays it out, [`Txn::set_pivot_table`] replaces an existing pivot's fields and
//! options in place, and [`Txn::refresh_pivot_table`] reads its source again; the pivot lifecycle
//! (SPEC-0032 "Who refreshes, and when", T-4151) also refreshes `refreshOnLoad` pivots after a
//! loaded workbook's first recalculation. A source edit never refreshes a pivot. Every write goes
//! through the engine's grid writer (`rustsheet_engine::pivot`'s `plan_pivot_refresh`,
//! `plan_pivot_render`, `apply_pivot_refresh`), which owns a pivot's area and collapses it to
//! `#SPILL!`, and every cell it writes or clears is logged, so the edit's [`ChangeSet`] lists it
//! and a failed edit restores it.
//!
//! [`ChangeSet`]: super::ChangeSet

use rustsheet_engine::pivot::{
    keep_ungrouped, read_source, regroup, Aggregate, Axis, AxisField, CacheField, CacheItem,
    CacheSource, CellArea, DataField, DateGroupError, NumberGroupError, PivotBudget, PivotCache,
    PivotCacheError, PivotComputeError, PivotField, PivotRefresh, PivotRefreshError, PivotSwap,
    PivotTable, RangeGrouping, RegroupError, MAX_RECORD_VALUES,
};
use rustsheet_engine::Model;

pub use rustsheet_engine::pivot::DateGroupBy as PivotDateGroupBy;

use super::{Txn, Undo};
use crate::error::{EditError, RefError};
use crate::pivot_view::pivot_position;
use crate::refs::{CellRef, RangeRef, SheetId};
use crate::workbook::{engine_rc, sheet_index, Workbook};

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

/// A pivot table's definition, as [`Txn::add_pivot_table`] creates one and
/// [`Txn::set_pivot_table`] replaces one: one row field, an optional column field and one data
/// field over a source range. [`PivotTableInput::new`] fills in the defaults.
#[derive(Clone, Debug)]
pub struct PivotTableInput {
    /// The pivot's name, unique among the pivots of its sheet.
    pub name: String,
    /// The source range: field names (text) in its first row, one record per row below it.
    pub source: RangeRef,
    /// Where the pivot goes: its sheet is the pivot's sheet, and its top-left cell the pivot's
    /// anchor, from which the engine lays the pivot out (Excel's "Location" of a new pivot).
    /// The rest of the range is not read, except by the doc-hidden
    /// `Txn::register_pivot_table`, for which an ungrouped pivot's rendered cells cover it.
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
    /// A numeric range or date grouping of the row field, if any. A grouped pivot has no column
    /// field and no unused items.
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

impl PivotTableInput {
    /// A pivot named `name` over `source` at the anchor `location`, summarizing the field
    /// `value` by `function` per item of the row field `row`: no column field, the caption
    /// `<Function> of <field>` (`Sum of Sales`, the case builder's default; the name Excel's
    /// UI picks for a new data field is unprobed, T-2327: pivots.md "Caption attributes, empty
    /// and error captions, page captions, custom subtotal labels"), no grouping, no unused
    /// items and `showAll` off on both axes, as the case builder writes a pivot.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        source: RangeRef,
        location: RangeRef,
        row: impl Into<String>,
        value: impl Into<String>,
        function: PivotFunction,
    ) -> Self {
        let value = value.into();
        Self {
            name: name.into(),
            source,
            location,
            row: row.into(),
            column: None,
            caption: format!("{} of {value}", function.caption_name()),
            value,
            function,
            group: None,
            unused_items: Vec::new(),
            show_all: false,
            column_unused_items: Vec::new(),
            column_show_all: false,
        }
    }
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

/// Why a pivot table could not be created, set or refreshed ([`EditError::Pivot`]); the edit
/// then fails and the workbook is as it was. The pivot reads (`Workbook::pivot_table_cells`,
/// `Workbook::pivot_cell_records`) fail with it too.
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
    /// The pivot passes a limit: its cache's fields, records, items or record values (the
    /// workbook's included), or the work, result nodes or cells of its layout (SPEC-0032
    /// "Untrusted-input limits"). Excel would render it (DEVIATION, pivots.md "A pivot result is
    /// bounded in work and nodes").
    #[error("the pivot table passes a size or work limit")]
    Limit,
    /// The pivot's source is not a worksheet range of this workbook (its sheet is gone, or a
    /// loaded pivot reads a Table, a name or a connection).
    #[error("the pivot source is not a worksheet range of this workbook")]
    Source,
    /// The cache could not be built for another reason than a limit or the source (a record
    /// that does not match its fields, a value its grouping cannot group).
    #[error(transparent)]
    Cache(PivotCacheError),
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
    /// The pivot could not be computed or laid out for another reason than a limit (a
    /// definition that does not match its cache).
    #[error(transparent)]
    Compute(PivotComputeError),
    /// No pivot of that name on the sheet.
    #[error("no pivot table {0:?} on the sheet")]
    UnknownPivot(String),
    /// Another pivot of the sheet has that name: the API keeps a sheet's pivot names unique
    /// (SPEC-0032 "API"; Excel's own rule is unprobed).
    #[error("the sheet already has a pivot table {0:?}")]
    DuplicateName(String),
    /// [`Txn::set_pivot_table`] was given a location on another sheet or with another anchor:
    /// it edits a pivot where it is, it does not move it.
    #[error("setting a pivot table does not move it")]
    Moved,
    /// The pivot could not be refreshed: a field it shows left the source, or its fields would
    /// be renumbered under parts of its definition kept verbatim.
    #[error(transparent)]
    Refresh(PivotRefreshError),
    /// [`Workbook::pivot_cell_records`](crate::Workbook::pivot_cell_records) was given a cell
    /// that is none of the pivot's value cells: a label, a header, a page field, a cell outside
    /// the pivot or on another sheet, a cell of a line that holds no values, or any cell of a
    /// pivot with no data field or of one that collapses to `#SPILL!`.
    #[error("the cell is not a value cell of the pivot table")]
    NotAValueCell,
    /// Pivot parts the xlsx importer would skip, or past the workbook's pivot limits, and why:
    /// the doc-hidden `Workbook::load_pivot_parts`, harness support for the eval adapter.
    #[error("pivot parts: {0}")]
    Parts(String),
}

impl From<PivotCacheError> for PivotError {
    fn from(error: PivotCacheError) -> Self {
        match error {
            PivotCacheError::TooManyFields
            | PivotCacheError::TooManyRecords
            | PivotCacheError::TooManyItems
            | PivotCacheError::TooManyRecordValues => Self::Limit,
            PivotCacheError::Source => Self::Source,
            error => Self::Cache(error),
        }
    }
}

impl From<PivotComputeError> for PivotError {
    fn from(error: PivotComputeError) -> Self {
        match error {
            PivotComputeError::Limit => Self::Limit,
            error => Self::Compute(error),
        }
    }
}

impl From<PivotRefreshError> for PivotError {
    fn from(error: PivotRefreshError) -> Self {
        match error {
            PivotRefreshError::Cache(error) => error.into(),
            PivotRefreshError::Compute(error) => error.into(),
            error => Self::Refresh(error),
        }
    }
}

/// How [`roll_back`](super::roll_back) undoes a pivot change.
pub(super) enum PivotUndo {
    /// `add_pivot_table`: the lengths of the workbook's pivot caches and tables before it.
    Added { caches: usize, tables: usize },
    /// A refresh or layout: the cache and pivots it replaced.
    Swapped(Box<PivotSwap>),
    /// `set_pivot_table`: the pivot at index `table` as it was, the cache it replaced at the
    /// pivot's cache index (`None` when a new cache was appended instead), and the number of
    /// caches before it.
    Replaced {
        table: usize,
        pivot: Box<PivotTable>,
        cache: Option<Box<PivotCache>>,
        caches: usize,
    },
}

impl PivotUndo {
    pub(super) fn roll_back(self, model: &mut Model) {
        match self {
            Self::Added { caches, tables } => {
                model.workbook.pivot_caches.truncate(caches);
                model.workbook.pivot_tables.truncate(tables);
            }
            Self::Replaced {
                table,
                pivot,
                cache,
                caches,
            } => {
                let workbook = &mut model.workbook;
                if let Some((slot, cache)) = cache
                    .and_then(|c| Some((workbook.pivot_caches.get_mut(pivot.cache as usize)?, c)))
                {
                    *slot = *cache;
                }
                workbook.pivot_caches.truncate(caches);
                if let Some(slot) = workbook.pivot_tables.get_mut(table) {
                    *slot = *pivot;
                }
            }
            Self::Swapped(swap) => {
                model.restore_pivot_swap(*swap);
            }
        }
    }
}

impl Txn<'_> {
    /// Creates the pivot table `input` and lays it out from the top-left cell of
    /// `input.location`, its anchor. Its cache is the source as it holds now, read as Excel's
    /// refresh reads it (field names, item typing and identity). The layout's cells are written
    /// as constants, the anchor's own content replaced (the API's choice, pivots.md "A pivot
    /// created through the API"). When the layout would meet a stored value or formula the
    /// pivot does not own, the pivot collapses to `#SPILL!` at its anchor, as Excel's does
    /// (pivots.md "A pivot that would meet a foreign cell collapses to #SPILL!",
    /// probes/t4128-pivot-spike S); it collapses the same way when it would meet a Table, a
    /// merged range or another pivot, or pass the sheet's edge, which are the grid writer's own
    /// readings (pivots.md "The grid writer's own readings", UNVERIFIED). A merged range inside
    /// the pivot's own area refuses the edit instead. The edit's
    /// [`ChangeSet`](super::ChangeSet) lists every cell written, and the transaction
    /// recalculates, so GETPIVOTDATA and formulas reading the pivot's cells see it.
    ///
    /// # Errors
    /// [`EditError::Ref`] for an unknown sheet or a range outside the sheet, and
    /// [`EditError::Pivot`]: a malformed source or field ([`PivotError::Header`],
    /// [`PivotError::UnknownField`], [`PivotError::RepeatedField`], [`PivotError::NoRecords`],
    /// a grouping that cannot apply), a name another pivot of the sheet has
    /// ([`PivotError::DuplicateName`]), a pivot past a limit ([`PivotError::Limit`]), or a
    /// merged range in its own area ([`PivotError::Refresh`]). A pivot that fails is not added
    /// and nothing is written.
    pub fn add_pivot_table(&mut self, input: &PivotTableInput) -> Result<(), EditError> {
        if self
            .pivot_index(input.location.sheet, &input.name)
            .is_some()
        {
            return Err(PivotError::DuplicateName(input.name.clone()).into());
        }
        let table = self.push_pivot(input, true)?;
        self.lay_out(table)
    }

    /// Registers the pivot table `input` without laying it out: an ungrouped pivot's
    /// `location` is its whole area, and its rendered cells are the caller's to write (the eval
    /// adapter writes a spec case's baked cells, SPEC-0015 "Spec cases in the eval adapter"); a
    /// grouped one is laid out as [`Txn::add_pivot_table`] lays it out (T-4235). Harness
    /// support, not public API.
    ///
    /// # Errors
    /// As [`Txn::add_pivot_table`], but for [`PivotError::DuplicateName`].
    #[doc(hidden)]
    pub fn register_pivot_table(&mut self, input: &PivotTableInput) -> Result<(), EditError> {
        let grouped = input.group.is_some();
        let table = self.push_pivot(input, grouped)?;
        if grouped {
            self.lay_out(table)?;
        }
        Ok(())
    }

    /// Replaces the fields and options of the pivot `name` on `sheet` with `input`, in place:
    /// its cache is read again from `input.source`, and the pivot is laid out again from its
    /// anchor, the cells it owned and no longer writes cleared, as a refresh lays it out
    /// ([`Txn::refresh_pivot_table`]). The pivot takes `input.name`. What a loaded pivot's
    /// definition held beyond `input` (styles, formats, page fields, options) is dropped, and
    /// other pivots that shared its cache keep the old one. The edit's
    /// [`ChangeSet`](super::ChangeSet) lists every cell written or cleared.
    ///
    /// # Errors
    /// [`EditError::Ref`] for an unknown sheet, and [`EditError::Pivot`]: no pivot `name` on
    /// `sheet` ([`PivotError::UnknownPivot`]), a location on another sheet or with another
    /// anchor ([`PivotError::Moved`]), a new name another pivot of the sheet has
    /// ([`PivotError::DuplicateName`]), any error of [`Txn::add_pivot_table`], and the engine
    /// refusing a cell (a merged cell in the pivot's area). The pivot and its cells are then
    /// as they were.
    pub fn set_pivot_table(
        &mut self,
        sheet: SheetId,
        name: &str,
        input: &PivotTableInput,
    ) -> Result<(), EditError> {
        sheet_index(self.model, sheet)?;
        let index = self
            .pivot_index(sheet, name)
            .ok_or_else(|| PivotError::UnknownPivot(name.to_string()))?;
        let old = (self.model.workbook.pivot_tables.get(index)).ok_or(RefError::OutOfBounds)?;
        let anchor = (old.location.top, old.location.left);
        if input.location.sheet != sheet || (input.location.top, input.location.left) != anchor {
            return Err(PivotError::Moved.into());
        }
        if (self.pivot_index(sheet, &input.name)).is_some_and(|other| other != index) {
            return Err(PivotError::DuplicateName(input.name.clone()).into());
        }
        let (location, geometry, old_cache) = (old.location, old.geometry, old.cache);
        let workbook = &self.model.workbook;
        let shared = (workbook.pivot_tables.iter().enumerate())
            .any(|(i, p)| i != index && p.cache == old_cache);
        let in_place = !shared && (old_cache as usize) < workbook.pivot_caches.len();
        let (cache, mut table) = self.build_pivot(input, in_place.then_some(old_cache))?;
        // The old location and geometry are the area the grid writer clears what it no longer
        // writes from; the layout replaces them.
        table.location = location;
        table.geometry = geometry;
        let workbook = &mut self.model.workbook;
        let caches = workbook.pivot_caches.len();
        if in_place {
            table.cache = old_cache;
        }
        // The pivot's slot is fetched before anything changes, so no error can return between
        // a change and its undo entry below.
        let slot = (workbook.pivot_tables.get_mut(index)).ok_or(RefError::OutOfBounds)?;
        let pivot = Box::new(std::mem::replace(slot, table));
        let replaced = match workbook.pivot_caches.get_mut(old_cache as usize) {
            Some(slot) if in_place => Some(Box::new(std::mem::replace(slot, cache))),
            _ => {
                workbook.pivot_caches.push(cache);
                None
            }
        };
        self.sheet_op = true;
        self.undo.push(Undo::Pivot(PivotUndo::Replaced {
            table: index,
            pivot,
            cache: replaced,
            caches,
        }));
        self.lay_out(index)
    }

    /// Refreshes the pivot `name` on `sheet` from its source as it holds now: its cache is read
    /// again, and every pivot over that cache is laid out again from its anchor, its old cells
    /// cleared and its new ones written as constants, or collapsed to `#SPILL!` at the anchor when
    /// the layout would meet a cell it does not own (pivots.md "A pivot that would meet a foreign
    /// cell collapses to #SPILL!", probes/t4128-pivot-spike S). Nothing else refreshes a pivot:
    /// editing its source does not, nor does a recalculation
    /// (probes/t4586-pivot-refresh-on-load C). The edit's [`ChangeSet`](super::ChangeSet) lists
    /// every cell written or cleared, and the transaction recalculates, so GETPIVOTDATA and
    /// formulas reading the pivot's cells see it.
    ///
    /// # Errors
    /// [`EditError::Ref`] for an unknown sheet, [`EditError::Pivot`] for an unknown pivot
    /// ([`PivotError::UnknownPivot`]) or one that does not refresh ([`PivotError::Limit`],
    /// [`PivotError::Source`], [`PivotError::Refresh`], ...), and the engine refusing a cell (a
    /// merged or array-formula cell in the pivot's area). The pivot and its cells are then as
    /// they were.
    pub fn refresh_pivot_table(&mut self, sheet: SheetId, name: &str) -> Result<(), EditError> {
        sheet_index(self.model, sheet)?;
        let cache = (self.pivot_index(sheet, name))
            .and_then(|i| self.model.workbook.pivot_tables.get(i))
            .map(|p| p.cache)
            .ok_or_else(|| PivotError::UnknownPivot(name.to_string()))?;
        let plan = self
            .model
            .plan_pivot_refresh(cache, &mut PivotBudget::default())
            .map_err(PivotError::from)?;
        self.write_pivots(plan)
    }

    /// The index of the first pivot named `name` on `sheet`.
    fn pivot_index(&self, sheet: SheetId, name: &str) -> Option<usize> {
        pivot_position(&self.model.workbook, sheet, name)
    }

    /// Builds `input` and appends its cache and pivot, logged for undo; `anchor_only` keeps
    /// only the top-left cell of its location, the anchor a layout starts from. Returns the
    /// pivot's index.
    fn push_pivot(
        &mut self,
        input: &PivotTableInput,
        anchor_only: bool,
    ) -> Result<usize, EditError> {
        let (cache, mut table) = self.build_pivot(input, None)?;
        if anchor_only {
            table.location.bottom = table.location.top;
            table.location.right = table.location.left;
        }
        let (caches, tables) = (
            self.model.workbook.pivot_caches.len(),
            self.model.workbook.pivot_tables.len(),
        );
        self.model.workbook.pivot_caches.push(cache);
        self.model.workbook.pivot_tables.push(table);
        self.sheet_op = true;
        self.undo
            .push(Undo::Pivot(PivotUndo::Added { caches, tables }));
        Ok(tables)
    }

    /// Lays out the pivot at index `table` over its cache as it is, from its anchor. A grouped
    /// pivot registered for the eval adapter is laid out too: Excel lays one out itself on load
    /// (probes/t4228-pivot-range-groups reads it back), and the case builder renders none
    /// (SPEC-0015 "Spec cases in the eval adapter", T-4235).
    fn lay_out(&mut self, table: usize) -> Result<(), EditError> {
        let plan = self
            .model
            .plan_pivot_render(table, &mut PivotBudget::default())
            .map_err(PivotError::from)?;
        self.write_pivots(plan)
    }

    /// Writes `plan` through the grid writer, logging every cell it clears or writes first so
    /// the transaction's undo restores them and its change set lists them.
    fn write_pivots(&mut self, plan: PivotRefresh) -> Result<(), EditError> {
        for (sheet, row, col) in plan.touched() {
            let id = (self.model.workbook.worksheet(sheet))
                .map_err(EditError::Engine)?
                .get_sheet_id();
            let (r, c) = engine_rc(row, col).ok_or(RefError::OutOfBounds)?;
            let sheet_id = SheetId::new(id);
            self.log_cell(sheet, (id, r, c), CellRef::new(sheet_id, row, col)?)?;
        }
        self.sheet_op = true;
        let swap = (self.model.apply_pivot_refresh(plan)).map_err(|error| match error {
            PivotRefreshError::Write(message) => EditError::from_engine_message(message, None),
            error => PivotError::from(error).into(),
        })?;
        self.undo
            .push(Undo::Pivot(PivotUndo::Swapped(Box::new(swap))));
        Ok(())
    }

    /// The cache and pivot of `input`, the pivot over a cache appended to the workbook's. The
    /// record values of the workbook's other caches count against the limit, all but the one at
    /// index `replacing`, which the new cache is to replace.
    fn build_pivot(
        &self,
        input: &PivotTableInput,
        replacing: Option<u32>,
    ) -> Result<(PivotCache, PivotTable), EditError> {
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
        let used: usize = (model.workbook.pivot_caches.iter().zip(0u32..))
            .filter(|&(_, i)| Some(i) != replacing)
            .map(|(c, _)| (c.record_count as usize).saturating_mul(c.fields.len()))
            .fold(0, usize::saturating_add);
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
                // Grouped in place, it keeps its own values for the export (T-4734).
                let base = std::mem::take(field);
                let regrouped = grouped(&base, group)?;
                *field = keep_ungrouped(base, regrouped);
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
}

impl Workbook {
    /// Refreshes the `refreshOnLoad` pivots of a loaded workbook (`Workbook::recalculate`), one
    /// cache after another in workbook order, within one [`PivotBudget`] for them all. Each cache
    /// is planned against the grid as the caches before it left it, so its source reads their
    /// pivots' new cells and its pivots meet their new areas. A cache whose plan fails (a source
    /// that does not resolve, such as a sheet-scoped name, T-5034; a limit; a field gone) keeps
    /// its pivots' saved cells. Until a
    /// plan succeeds nothing is edited; from the first one on, the writes are one edit with no
    /// value diff, which recalculates once more.
    pub(crate) fn refresh_pivots_on_load(&mut self) {
        let mut budget = PivotBudget::default();
        let caches = (self.model.workbook.pivot_caches.iter()).zip(0u32..);
        let due: Vec<u32> = (caches.filter(|(c, _)| c.refresh_on_load))
            .map(|(_, i)| i)
            .collect();
        let mut rest = due.into_iter();
        let Some(first) = rest
            .by_ref()
            .find_map(|i| self.model.plan_pivot_refresh(i, &mut budget).ok())
        else {
            return;
        };
        let track = std::mem::replace(&mut self.track_changes, false);
        // A plan's areas hold no merged range, Table or array formula, so a write fails only for
        // a plan that does not match the workbook, which nothing here can cause. Should one fail
        // all the same, the edit rolls back and every pivot keeps its saved cells: a load does not
        // fail for a pivot, so the error is dropped on purpose.
        let written = self.edit(|tx| {
            tx.write_pivots(first)?;
            for cache in rest {
                if let Ok(plan) = tx.model.plan_pivot_refresh(cache, &mut budget) {
                    tx.write_pivots(plan)?;
                }
            }
            Ok(())
        });
        drop(written);
        self.track_changes = track;
    }
}

mod parts;

pub use parts::PivotParts;

#[cfg(test)]
mod tests;
