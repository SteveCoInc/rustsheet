//! Refreshing a pivot cache from its worksheet source (SPEC-0032 "Refresh", T-4141).
//!
//! A refresh reads the source cells through [`SourceCells`] and builds a new cache with the one
//! builder, [`CacheBuilder`] (directive 3); the caller swaps it in only on success, so a refresh
//! past a limit or over a source that does not resolve leaves the cache as it was (SPEC-0032
//! "Untrusted-input limits"). The rules are Excel's, from docs/excel-semantics/pivots.md:
//! - field names come from the header row, a repeated name numbered and a blank one `Field<n>`
//!   ([`field_names`]);
//! - an item is found by [`ItemKey::identity`]: exact for numbers and types, text ignoring case;
//! - the old cache's items keep their slots and order, even with no record left, and new items
//!   follow in source order ([`FieldReader`]);
//! - the `sharedItems` flags are derived from the items ([`SharedItemsFlags`]), so they always
//!   agree with them;
//! - a field's saved range grouping is built again over the new values ([`regroup`]).
//!
//! A source is a worksheet range; a defined name or a Table source resolves to its range in
//! T-4506 and refreshes as [`PivotCacheError::Source`] until then.

use std::collections::HashMap;

use thiserror::Error as ThisError;

use super::{
    group_by_dates, group_by_numbers, ungroup_dates, CacheBuilder, CacheField, CacheItem,
    CacheSource, CellArea, DateGroupBy, DateGroupError, DateGroups, ItemKey, NumberGroupError,
    PivotCache, PivotCacheError, RecordValue, MAX_FIELDS, MAX_ITEMS, MAX_RECORDS,
    MAX_RECORD_VALUES,
};
use crate::calc_result::CalcResult;
use crate::constants::{LAST_COLUMN, LAST_ROW};
use crate::expressions::parser::parse_range;
use crate::expressions::types::CellReferenceIndex;
use crate::formatter::lexer::is_likely_date_number_format;
use crate::functions::util::collation::fold_case;
use crate::model::Model;
use crate::number_format::get_num_fmt;

/// The cells a cache's source names, as values: what a refresh reads. An implementation reads
/// stored values only, never evaluating a formula and never reaching a file or the network
/// (directive 5), so a refresh is ordered after the recalculation whose values it reads.
pub trait SourceCells {
    /// The area `source` names, or `None` when it names nothing this workbook holds (an unknown
    /// sheet, a range that is not one).
    fn resolve(&self, source: &CacheSource) -> Option<SourceArea>;

    /// The value of the 0-based cell (`row`, `column`) of sheet `sheet`, typed as a cache holds
    /// it: an empty cell is [`CacheItem::Missing`] and a number shown in a date format is a
    /// [`CacheItem::Date`].
    fn value(&self, sheet: u32, row: u32, column: u32) -> CacheItem;
}

/// A resolved source: an area of one sheet, its first row the header row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceArea {
    /// The sheet's index.
    pub sheet: u32,
    /// The 0-based cells, header row included.
    pub area: CellArea,
}

/// `Model` reads the evaluated cell values (SPEC-0032 "Refresh").
impl SourceCells for Model {
    fn resolve(&self, source: &CacheSource) -> Option<SourceArea> {
        let sheet = self.get_sheet_index_by_name(&source.sheet_name)?;
        let (c1, r1, c2, r2) = parse_range(&source.range).ok()?;
        // 1-based and inside the sheet, or no area.
        let line = |n: i32, last: i32| (1..=last).contains(&n).then(|| n.unsigned_abs() - 1);
        let (top, bottom) = (line(r1.min(r2), LAST_ROW)?, line(r1.max(r2), LAST_ROW)?);
        let (left, right) = (
            line(c1.min(c2), LAST_COLUMN)?,
            line(c1.max(c2), LAST_COLUMN)?,
        );
        Some(SourceArea {
            sheet,
            area: CellArea {
                top,
                left,
                bottom,
                right,
            },
        })
    }

    fn value(&self, sheet: u32, row: u32, column: u32) -> CacheItem {
        // Rows and columns within the sheet fit an i32 (`resolve`); anything else is a blank.
        let (Ok(row), Ok(column)) = (i32::try_from(row + 1), i32::try_from(column + 1)) else {
            return CacheItem::Missing;
        };
        let Some(cell) = self
            .workbook
            .worksheet(sheet)
            .ok()
            .and_then(|w| w.cell(row, column))
        else {
            return CacheItem::Missing;
        };
        let reference = CellReferenceIndex { sheet, row, column };
        match self.get_cell_value(cell, reference) {
            CalcResult::Number(n) if self.is_date_style(cell.get_style()) => CacheItem::Date(n),
            CalcResult::Number(n) => CacheItem::Number(n),
            CalcResult::String(text) => CacheItem::Text(text.into_lossy_string()),
            CalcResult::Boolean(b) => CacheItem::Boolean(b),
            CalcResult::Error { error, .. } => CacheItem::Error(error),
            // A stored cell value is never a reference, an array or a lambda.
            _ => CacheItem::Missing,
        }
    }
}

impl Model {
    /// Whether the cell style at `index` shows numbers as dates (`is_likely_date_number_format`),
    /// which Excel caches as dates: probes/t4582-pivot-item-edges J3 and J4 (`m/d/yyyy`,
    /// `m/d/yyyy h:mm`) do not match the cached number items.
    fn is_date_style(&self, index: i32) -> bool {
        let styles = &self.workbook.styles;
        usize::try_from(index)
            .ok()
            .and_then(|i| styles.cell_xfs.get(i))
            .is_some_and(|xf| {
                is_likely_date_number_format(&get_num_fmt(xf.num_fmt_id, &styles.num_fmts))
            })
    }
}

/// `cache` refreshed from `source`: the cache [`read_source`] reads, each field whose old
/// version had a range grouping grouped again ([`regroup`]). `workbook_values_left` is how many
/// record values the workbook may still hold without this cache's old ones. The caller swaps the
/// result in only on success.
///
/// # Errors
/// [`read_source`]'s, and [`PivotCacheError::Group`] when a field's new values do not fit its
/// grouping (UNVERIFIED what Excel does then; grouped fields are PV3).
pub fn refresh(
    cache: &PivotCache,
    source: &dyn SourceCells,
    workbook_values_left: usize,
) -> Result<PivotCache, PivotCacheError> {
    let mut fresh = read_source(cache, source, workbook_values_left)?.finish();
    let old = fields_by_name(cache);
    for field in &mut fresh.fields {
        let Some(prior) = old.get(&fold_case(&field.name)) else {
            continue;
        };
        let grouping = match (&prior.number_groups, prior.date_groups) {
            (Some(g), _) => RangeGrouping::Numbers {
                start: g.start,
                end: g.end,
                interval: g.interval,
            },
            (None, Some(g)) => {
                let (by, start, end) = g.bounds();
                RangeGrouping::Dates { by, start, end }
            }
            (None, None) => continue,
        };
        // A grouped field's items are its labels, in cache order, which `NumberGroups::new`
        // and `group_by_dates` take back unchanged.
        *field = regroup(field, &grouping, &prior.items).map_err(|_| PivotCacheError::Group)?;
    }
    Ok(fresh)
}

/// The first step of [`refresh`]: a builder holding `cache`'s source read again, every field
/// ungrouped, for a caller that adds items before finishing it (`Txn::add_pivot_table`'s unused
/// items). Fields are matched to `cache`'s by name, ignoring case (UNVERIFIED: a field Excel
/// matches by position is unprobed); a matched ungrouped field keeps its items, in their order,
/// and new items follow in source order (pivots.md "Items show in the order the pivot field
/// lists them; new items are appended", probes/t4134-pivot-items I2, I3,
/// t4582-pivot-item-edges J2).
///
/// # Errors
/// [`PivotCacheError::Source`] when `cache` has no source or it does not resolve,
/// [`PivotCacheError::Header`] for a header that is not text or blank, and the cache limits,
/// checked on the source's size before anything is read.
pub fn read_source(
    cache: &PivotCache,
    source: &dyn SourceCells,
    workbook_values_left: usize,
) -> Result<CacheBuilder, PivotCacheError> {
    let spec = cache.source.as_ref().ok_or(PivotCacheError::Source)?;
    let SourceArea { sheet, area } = source.resolve(spec).ok_or(PivotCacheError::Source)?;
    let (Some(width), Some(height)) = (
        area.right.checked_sub(area.left),
        area.bottom.checked_sub(area.top),
    ) else {
        return Err(PivotCacheError::Source);
    };
    let (width, records) = (width as usize + 1, height as usize);
    if width > MAX_FIELDS {
        return Err(PivotCacheError::TooManyFields);
    }
    if records > MAX_RECORDS {
        return Err(PivotCacheError::TooManyRecords);
    }
    if width.saturating_mul(records) > workbook_values_left.min(MAX_RECORD_VALUES) {
        return Err(PivotCacheError::TooManyRecordValues);
    }
    let headers = (area.left..=area.right).map(|column| source.value(sheet, area.top, column));
    let names = field_names(headers)?;
    let old = fields_by_name(cache);
    let mut items = 0;
    let mut readers: Vec<FieldReader> = names
        .iter()
        .map(|name| {
            let prior = old
                .get(&fold_case(name))
                .copied()
                .filter(|f| f.number_groups.is_none() && f.date_groups.is_none());
            items += prior.map_or(0, |f| f.items.len());
            FieldReader::new(prior, records)
        })
        .collect();
    for row in area.top.saturating_add(1)..=area.bottom {
        for (reader, column) in readers.iter_mut().zip(area.left..=area.right) {
            reader.push(source.value(sheet, row, column), &mut items)?;
        }
    }
    let mut builder = CacheBuilder::new(cache.source.clone(), workbook_values_left);
    for name in names {
        builder.add_field(name)?;
    }
    for (reader, field) in readers.iter_mut().zip(0u32..) {
        for item in std::mem::take(&mut reader.items) {
            builder.add_shared_item(field, item)?;
        }
    }
    for record in 0..records {
        let values = readers
            .iter()
            .map(|r| r.records.get(record).copied().map(RecordValue::Shared))
            .collect::<Option<Vec<_>>>()
            .ok_or(PivotCacheError::BadRecord)?;
        builder.push_record(values)?;
    }
    Ok(builder)
}

/// `cache`'s fields by their case-folded name, the first of a repeated name.
fn fields_by_name(cache: &PivotCache) -> HashMap<String, &CacheField> {
    let mut by_name = HashMap::with_capacity(cache.fields.len());
    for field in &cache.fields {
        by_name.entry(fold_case(&field.name)).or_insert(field);
    }
    by_name
}

/// The cache field names of a source's `headers`, as Excel names them on refresh
/// (probes/t4585-pivot-field-names, whose headers Region, Sales, `="Sales"`, `=""`, `="region"`
/// and `="Sales"&""` save as Region, Sales, Sales2, Field4, region2 and Sales3; pivots.md "Pivot
/// field names for duplicate and blank source headers"):
/// - a header equal to an earlier one ignoring case gets the next number of that name, from 2,
///   and keeps its own casing;
/// - a blank header is `Field<n>`, n its 1-based position in the source. Only empty text (`=""`)
///   in the 4th column of a source starting at column A is evidenced, so whether n is the
///   position in the source or the sheet's column number is UNVERIFIED, and so is an empty
///   header cell, named here as empty text is (probe pending: T-4627);
/// - a numbered or `Field<n>` name that meets a real header (a source that already has `Sales2`
///   or `Field4`) is not made unique: UNVERIFIED (probe pending: T-4627).
///
/// A number, boolean or error header is refused: how Excel names it is unprobed.
fn field_names(headers: impl Iterator<Item = CacheItem>) -> Result<Vec<String>, PivotCacheError> {
    let mut seen: HashMap<String, u32> = HashMap::new();
    headers
        .zip(0u32..)
        .map(|(header, column)| match header {
            CacheItem::Missing => Ok(format!("Field{}", column + 1)),
            CacheItem::Text(name) if name.is_empty() => Ok(format!("Field{}", column + 1)),
            CacheItem::Text(name) => {
                let count = seen.entry(fold_case(&name)).or_insert(0);
                *count += 1;
                Ok(if *count == 1 {
                    name
                } else {
                    format!("{name}{count}")
                })
            }
            _ => Err(PivotCacheError::Header { column }),
        })
        .collect()
}

/// One field's items and records as a refresh reads them.
struct FieldReader {
    /// The old items, then the new ones in source order.
    items: Vec<CacheItem>,
    /// Each item's index by [`ItemKey::identity`], the first of equal ones.
    keys: HashMap<ItemKey, u32>,
    /// Per old item, whether a source value has already relabelled it.
    relabelled: Vec<bool>,
    /// Per record, its item.
    records: Vec<u32>,
}

impl FieldReader {
    fn new(prior: Option<&CacheField>, records: usize) -> Self {
        let items = prior.map(|f| f.items.clone()).unwrap_or_default();
        let mut keys = HashMap::with_capacity(items.len());
        for (item, index) in items.iter().zip(0u32..) {
            keys.entry(ItemKey::identity(item)).or_insert(index);
        }
        FieldReader {
            relabelled: vec![false; items.len()],
            items,
            keys,
            records: Vec::with_capacity(records),
        }
    }

    /// Adds a record's `value`, finding its item or appending one; `items` counts the items over
    /// all fields against [`MAX_ITEMS`].
    ///
    /// An old text item a source value finds keeps its slot but takes that value's casing: in
    /// probes/t4582-pivot-item-edges J1 the cached `EAST` shows `east` (pivots.md "Item casing
    /// when cache and source disagree, and date items"). The first source value's casing is
    /// used; J1's `east` is also the lowercase and the ordinally greatest casing, so which of
    /// those decides is UNVERIFIED (probe pending: T-4612).
    fn push(&mut self, value: CacheItem, items: &mut usize) -> Result<(), PivotCacheError> {
        let key = ItemKey::identity(&value);
        let index = if let Some(&index) = self.keys.get(&key) {
            let slot = index as usize;
            if let (Some(done @ false), CacheItem::Text(_)) =
                (self.relabelled.get_mut(slot), &value)
            {
                *done = true;
                if let Some(item) = self.items.get_mut(slot) {
                    *item = value;
                }
            }
            index
        } else {
            if *items >= MAX_ITEMS {
                return Err(PivotCacheError::TooManyItems);
            }
            // At most MAX_ITEMS items, so the index fits a u32.
            let index =
                u32::try_from(self.items.len()).map_err(|_| PivotCacheError::TooManyItems)?;
            // A date item appended for a value no cached number matched: whether Excel saves it
            // as a `<d>` item is UNVERIFIED (probe pending: T-4612); it renders as its serial
            // either way (t4582 J3, J4).
            self.items.push(value);
            self.keys.insert(key, index);
            *items += 1;
            index
        };
        self.records.push(index);
        Ok(())
    }
}

/// A range grouping to build over a field's values ([`regroup`]).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RangeGrouping {
    /// By numbers from `start` to `end` in steps of `interval` ([`group_by_numbers`]).
    Numbers {
        /// `startNum`.
        start: f64,
        /// `endNum`.
        end: f64,
        /// `groupInterval`.
        interval: f64,
    },
    /// By dates from `start` to `end` ([`group_by_dates`]).
    Dates {
        /// What the field groups by.
        by: DateGroupBy,
        /// `startDate`, as a serial.
        start: f64,
        /// `endDate`, as a serial.
        end: f64,
    },
}

/// Why [`regroup`] cannot group a field.
#[derive(ThisError, Debug, PartialEq, Eq, Clone)]
pub enum RegroupError {
    /// The numeric grouping cannot apply.
    #[error(transparent)]
    Numbers(#[from] NumberGroupError),
    /// The date grouping cannot apply.
    #[error(transparent)]
    Dates(#[from] DateGroupError),
}

/// Field `base`, read from its source, grouped by `grouping` with the `<groupItems>` `labels`,
/// or left ungrouped where Excel's refresh drops the grouping:
/// - a numeric grouping over a blank or over any date, among numbers or alone: each value is its
///   own item, in record order, the blank `(blank)` (probes/t4287-pivot-group-whole-end pivot S,
///   t4287-pivot-group-dates-as-numbers, t4316-pivot-group-dates-alone; web.md "Grouped pivot
///   fields import as their group items");
/// - a date grouping over a blank: each second-rounded date is its own item
///   ([`ungroup_dates`]; probes/t4236-pivot-date-groups).
///
/// The xlsx importer keeps a saved grouping as it is; only a refresh drops one.
///
/// # Errors
/// [`RegroupError`] when the grouping cannot apply (a value that is not a number or a date, a
/// bad range, a wrong label count).
pub fn regroup(
    base: &CacheField,
    grouping: &RangeGrouping,
    labels: &[CacheItem],
) -> Result<CacheField, RegroupError> {
    let blank = base.items.contains(&CacheItem::Missing);
    Ok(match *grouping {
        RangeGrouping::Numbers {
            start,
            end,
            interval,
        } => {
            let dates = base.items.iter().any(|i| matches!(i, CacheItem::Date(_)));
            if blank || dates {
                return Ok(base.clone());
            }
            group_by_numbers(base, [start, end, interval], labels)?
        }
        RangeGrouping::Dates { by, start, end } => {
            if blank {
                return Ok(ungroup_dates(base)?);
            }
            let groups = DateGroups::new(by, start, end, labels.len())?;
            group_by_dates(base, groups, labels.to_vec())?
        }
    })
}

/// A cache field's `<sharedItems>` flags, derived from its items so that they always agree with
/// them: Excel for the web refuses a cache whose flags contradict its items (pivots.md
/// "Shared-item flags Excel accepts for mixed fields"; T-4122 runs). Written by the export
/// (T-4176). Booleans and errors count as strings. A date counts as neither a string nor a
/// number: how Excel flags dates (`containsDate`, `containsNonDate`, `minDate`) is unprobed.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SharedItemsFlags {
    /// `containsSemiMixedTypes`: false only when the field has no string and no blank.
    pub contains_semi_mixed_types: bool,
    /// `containsString`: a text, boolean or error item.
    pub contains_string: bool,
    /// `containsMixedTypes`: strings beside numbers.
    pub contains_mixed_types: bool,
    /// `containsNumber`.
    pub contains_number: bool,
    /// `containsInteger`: every number is whole.
    pub contains_integer: bool,
    /// `containsBlank`.
    pub contains_blank: bool,
    /// `containsDate`.
    pub contains_date: bool,
    /// `minValue` and `maxValue` over the numbers, if any.
    pub number_range: Option<(f64, f64)>,
}

impl SharedItemsFlags {
    /// The flags of `field`'s items: the ones Excel writes (pivots.md), and the number flags as
    /// Excel-saved caches carry them (oss-engines.md §5; the case builder's `shared_items`).
    #[must_use]
    pub fn of(field: &CacheField) -> SharedItemsFlags {
        let mut flags = SharedItemsFlags::default();
        let mut whole = true;
        for item in &field.items {
            match item {
                CacheItem::Missing => flags.contains_blank = true,
                CacheItem::Text(_) | CacheItem::Boolean(_) | CacheItem::Error(_) => {
                    flags.contains_string = true;
                }
                CacheItem::Date(_) => flags.contains_date = true,
                CacheItem::Number(n) => {
                    flags.contains_number = true;
                    whole &= n.fract() == 0.0;
                    let (low, high) = flags.number_range.unwrap_or((*n, *n));
                    flags.number_range = Some((low.min(*n), high.max(*n)));
                }
            }
        }
        flags.contains_semi_mixed_types = flags.contains_string || flags.contains_blank;
        flags.contains_mixed_types = flags.contains_string && flags.contains_number;
        flags.contains_integer = flags.contains_number && whole;
        flags
    }
}

#[cfg(test)]
mod tests;
