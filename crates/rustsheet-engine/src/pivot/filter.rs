//! Pivot filters (SPEC-0032 "`PivotResult`" stage 1; T-4165): the typed `<filters>` of a pivot
//! definition and a field's legacy `AutoShow`, and the record filter that applies them.
//!
//! The importer reads [`PivotTable::filters`] and [`PivotField::auto_show`] from the parts of the
//! definition it keeps verbatim (`<filters>` and the `<pivotField>` attributes stay in
//! [`PivotTable::opaque`], which is what the export writes), as it reads `fillDownLabels`.
//!
//! [`apply`] runs after the page and hidden-item filter (`result::filter`) and keeps the records
//! of the items every filter keeps, so the subtotals and the grand total cover only the items
//! shown, for every filter kind (docs/excel-semantics/pivots.md "Filters on an inner field rank
//! per parent; totals never count what a filter removes", probes/t4164-pivot-filter-totals):
//! 1. **Item filters** first, on every row and column field: caption (label) and date filters,
//!    whose answer is a property of the item alone. Hidden items are already gone, so a Top-N
//!    ranks only what these kept, whatever the order of `<filters>` (pivots.md "Several filters
//!    on one field", probes/t4164-pivot-filter-order HT, HP, LT0, LT1, TL0, TL1, LP0, LP1, PL0,
//!    PL1).
//! 2. **Value filters**, Top-N included, field by field (row fields outer first, then column
//!    fields), each field's in `<filters>` order (VT, VT0 against TV1, TV0), then its `AutoShow`
//!    (UNVERIFIED, T-4799: an `AutoShow` beside a `<filters>` Top-N is unprobed). Each ranks the
//!    items of its field **within each item of the outer fields of its axis**, on the values of
//!    the records still kept (`ITop`, `IVal`, `IPct`; `OTop` for an outer field). The order between two
//!    fields' value filters, and inner-field filters beside a column field or several data
//!    fields, are unprobed (UNVERIFIED, T-4799).
//!
//! A filter on a field that is on no row or column axis is not applied (UNVERIFIED, T-4799).
//! Every step charges its work before doing it ([`charge`]): an item test its text comparisons
//! as their ticks (`area_budget::compared_units`) before it makes them, so a limit stops a pass
//! mid-way (CLAUDE.md directive 5).
//!
//! The typed model is [`model`]; the value filters and Top-N are `rank`.

use std::collections::{HashMap, HashSet};

use crate::area_budget::compared_units;
use crate::formatter::dates::excel_serial_to_ymd;
use crate::functions::criteria::WildcardPattern;
use crate::functions::util::collation::{
    cmp_text_pairwise, fold_case, restore_compared_ticks, take_compared_ticks, BYTE_TICKS,
    CHAR_TICKS, FOLD_TICKS,
};
use crate::text::Text;

use super::layout::item_text;
use super::result::{axis_fields, charge, units, PivotComputeError};
use super::sort::display_order;
use super::{AxisField, CacheItem, PivotCache, PivotTable};

mod model;
mod rank;

pub use model::{
    AutoShow, DateGroupItem, DynamicDate, FilterCondition, FilterCriteria, FilterOperator,
    PivotFilter, PivotFilterKind, RelativeUnit,
};
use rank::Ranking;
pub(super) use rank::ValueStep;

/// What [`apply`] keeps.
pub(super) struct Kept {
    /// The records, in record order.
    pub(super) records: Vec<u32>,
    /// Per cache field, the items a filter removed from the whole field (an item filter, or a
    /// value filter on the outermost field of an axis), as a bitset over its items; empty for a
    /// field no filter removed from. A `showAll` axis does not show them.
    pub(super) removed: Vec<Vec<bool>>,
}

impl Kept {
    /// Whether a filter removed item `item` of field `field` from the whole field.
    pub(super) fn is_removed(&self, field: u32, item: u32) -> bool {
        (self.removed.get(field as usize))
            .and_then(|items| items.get(item as usize))
            .copied()
            .unwrap_or(false)
    }
}

/// The context the filters read besides the pivot: today's date, as a serial day, for the
/// relative date filters (the workbook's clock, as TODAY reads it); `None` keeps every item for
/// such a filter.
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct FilterContext {
    pub(super) today: Option<f64>,
}

/// Applies `table`'s filters to `records` (see the module documentation).
pub(super) fn apply(
    table: &PivotTable,
    cache: &PivotCache,
    records: Vec<u32>,
    context: FilterContext,
    work: &mut u64,
) -> Result<Kept, PivotComputeError> {
    let mut kept = Kept {
        records,
        removed: Vec::new(),
    };
    let axes = [&table.row_fields, &table.column_fields];
    // The filters by field, in `<filters>` order, and the fields on an axis: one pass each.
    charge(
        work,
        units(table.filters.len().saturating_add(table.fields.len())),
    )?;
    // A timeline on a field replaces the pivot's own filters on it (probes/t4767-pivot-timelines
    // TX: its dateBetween filter; UNVERIFIED, T-4962, for the other kinds); `result::filter`
    // applies the timeline.
    let filters: Vec<&PivotFilter> = (table.filters.iter())
        .filter(|f| (table.fields.get(f.field as usize)).is_none_or(|p| p.timeline.is_none()))
        .collect();
    let mut by_field: HashMap<u32, Vec<&PivotFilter>> = HashMap::new();
    for &filter in &filters {
        by_field.entry(filter.field).or_default().push(filter);
    }
    if by_field.is_empty() && table.fields.iter().all(|f| f.auto_show.is_none()) {
        return Ok(kept);
    }
    let mut on_axis = vec![false; table.fields.len()];
    for field in axes.iter().flat_map(|axis| axis.iter()) {
        if let AxisField::Field(index) = field {
            if let Some(slot) = on_axis.get_mut(*index as usize) {
                *slot = true;
            }
        }
    }
    // 1. Item filters, in `<filters>` order (they commute).
    for &filter in &filters {
        let on = on_axis.get(filter.field as usize).copied().unwrap_or(false);
        if filter.kind.on_items() && on {
            item_filter(filter, cache, context, &mut kept, work)?;
        }
    }
    // 2. Value filters, field by field.
    for (index, axis) in axes.iter().enumerate() {
        let across = (axes.iter().enumerate())
            .any(|(other, a)| other != index && a.iter().any(|f| matches!(f, AxisField::Field(_))));
        let fields = axis_fields(axis);
        for (depth, &field) in fields.iter().enumerate() {
            let filters = by_field.get(&field).map_or(&[][..], Vec::as_slice);
            let auto_show = table.fields.get(field as usize).and_then(|f| f.auto_show);
            let ranks: Vec<(Ranking, u32)> = (filters.iter())
                .filter_map(|filter| Some((Ranking::of(filter)?, filter.measure)))
                .chain(auto_show.map(|a| (Ranking::auto_show(a), a.measure)))
                .collect();
            if ranks.is_empty() {
                continue;
            }
            if kept.records.is_empty() {
                return Ok(kept);
            }
            // The field's display order (a label sort's included), once for all its value
            // filters, as the axis makes it (`AxisColumns::new`): it breaks a Top-N's ties.
            let cache_field = cache
                .fields
                .get(field as usize)
                .ok_or(PivotComputeError::Malformed)?;
            let order = display_order(table, cache_field, field, work)?;
            let parents = fields.get(..depth).unwrap_or_default();
            for (rank, measure) in ranks {
                let step = ValueStep {
                    table,
                    cache,
                    field,
                    parents,
                    order: &order,
                    measure,
                    // An item of an outer field, or across the other axis's items, is a total.
                    total: depth + 1 < fields.len() || across,
                    within: &[],
                };
                step.run(&rank, &mut kept, work)?;
            }
        }
    }
    Ok(kept)
}

/// The records of `field` in `cache`, or `Malformed`.
fn column(cache: &PivotCache, field: u32) -> Result<&[u32], PivotComputeError> {
    cache
        .fields
        .get(field as usize)
        .map(|f| f.records.as_slice())
        .ok_or(PivotComputeError::Malformed)
}

/// `text` as a number, if it is one.
fn number(text: &str) -> Option<f64> {
    text.trim().parse::<f64>().ok()
}

/// Marks `item` of `field` removed from the whole field.
fn mark_removed(kept: &mut Kept, field: u32, item: u32, items: usize) {
    let field = field as usize;
    if kept.removed.len() <= field {
        kept.removed.resize(field + 1, Vec::new());
    }
    if let Some(bits) = kept.removed.get_mut(field) {
        if bits.len() < items {
            bits.resize(items, false);
        }
        if let Some(bit) = bits.get_mut(item as usize) {
            *bit = true;
        }
    }
}

/// A caption or date filter: each item of the field is kept or not by itself.
fn item_filter(
    filter: &PivotFilter,
    cache: &PivotCache,
    context: FilterContext,
    kept: &mut Kept,
    work: &mut u64,
) -> Result<(), PivotComputeError> {
    let cache_field = cache
        .fields
        .get(filter.field as usize)
        .ok_or(PivotComputeError::Malformed)?;
    let items = &cache_field.items;
    let test = ItemTest::compile(filter, work)?;
    // The collation notes the ticks of what it compares; they are charged up front here
    // (`ItemTest::ticks`), so the ticks noted are dropped rather than left for a later formula.
    let pending = take_compared_ticks();
    let keeps = test.keeps_items(items, context, work);
    take_compared_ticks();
    restore_compared_ticks(pending);
    let keeps = keeps?;
    for (index, keep) in (0u32..).zip(&keeps) {
        if !keep {
            mark_removed(kept, filter.field, index, items.len());
        }
    }
    charge(work, units(kept.records.len()))?;
    let records = &cache_field.records;
    let mut malformed = false;
    kept.records.retain(|&record| {
        match records
            .get(record as usize)
            .and_then(|&item| keeps.get(item as usize))
        {
            Some(&keep) => keep,
            None => {
                malformed = true;
                false
            }
        }
    });
    if malformed {
        return Err(PivotComputeError::Malformed);
    }
    Ok(())
}

/// A caption condition compiled once per filter: its operator, its value as text, and for
/// `equal` and `notEqual` the value as a wildcard pattern.
struct CaptionTest {
    operator: FilterOperator,
    value: Text,
    pattern: Option<WildcardPattern>,
}

/// A caption or date filter compiled once, outside the item loop.
enum ItemTest<'a> {
    /// Keeps every item.
    Every,
    /// Keeps no item.
    Nothing,
    /// A values list, its values case-folded.
    Values {
        values: HashSet<String>,
        blank: bool,
        dates: &'a [DateGroupItem],
    },
    /// Caption conditions.
    Caption { and: bool, tests: Vec<CaptionTest> },
    /// Date conditions, each value parsed (`None` when it is no number).
    Date {
        and: bool,
        conditions: Vec<(FilterOperator, Option<f64>)>,
        whole_day: bool,
    },
    /// A month, quarter or relative period.
    Dynamic(DynamicDate),
}

impl<'a> ItemTest<'a> {
    /// `filter`'s test, its texts folded and its numbers parsed once, the folding charged first.
    fn compile(filter: &'a PivotFilter, work: &mut u64) -> Result<Self, PivotComputeError> {
        Ok(match &filter.criteria {
            FilterCriteria::Everything => Self::Every,
            // A Top-N criterion under a caption or date type is unprobed (UNVERIFIED, T-4799).
            FilterCriteria::Top10 { .. } => Self::Every,
            FilterCriteria::Nothing => Self::Nothing,
            FilterCriteria::Dynamic(dynamic) => Self::Dynamic(*dynamic),
            FilterCriteria::Values {
                values,
                blank,
                dates,
            } => {
                let bytes = values
                    .iter()
                    .map(String::len)
                    .fold(0, usize::saturating_add);
                let ticks = units(bytes).saturating_mul(FOLD_TICKS);
                charge(
                    work,
                    compared_units(ticks).saturating_add(units(values.len())),
                )?;
                Self::Values {
                    values: values.iter().map(|v| fold_case(v)).collect(),
                    blank: *blank,
                    dates,
                }
            }
            FilterCriteria::Custom { and, conditions } => {
                charge(work, units(conditions.len()))?;
                if filter.kind == PivotFilterKind::Caption {
                    let tests = (conditions.iter())
                        .map(|condition| {
                            let value = Text::from(condition.value.as_str());
                            let pattern = matches!(
                                condition.operator,
                                FilterOperator::Equal | FilterOperator::NotEqual
                            )
                            .then(|| WildcardPattern::new(&value));
                            CaptionTest {
                                operator: condition.operator,
                                value,
                                pattern,
                            }
                        })
                        .collect();
                    Self::Caption { and: *and, tests }
                } else {
                    Self::Date {
                        and: *and,
                        conditions: (conditions.iter())
                            .map(|c| (c.operator, number(&c.value)))
                            .collect(),
                        whole_day: filter.whole_day,
                    }
                }
            }
        })
    }

    /// Per item of `items`, whether the test keeps it; each item's work is charged before it is
    /// tested.
    fn keeps_items(
        &self,
        items: &[CacheItem],
        context: FilterContext,
        work: &mut u64,
    ) -> Result<Vec<bool>, PivotComputeError> {
        charge(work, units(items.len()))?;
        let mut keeps = Vec::with_capacity(items.len());
        for item in items {
            let keep = match self {
                Self::Every => true,
                Self::Nothing => false,
                Self::Dynamic(dynamic) => match serial(item) {
                    Some(serial) => dynamic.covers(serial, context.today),
                    // An item with no date is in no period (UNVERIFIED, T-4799); with no clock a
                    // relative period keeps every item, as it does a date.
                    None => dynamic.keeps_all(context.today),
                },
                Self::Date {
                    and,
                    conditions,
                    whole_day,
                } => {
                    let mut results = (conditions.iter())
                        .map(|&(operator, value)| date_holds(operator, value, item, *whole_day));
                    if *and {
                        results.all(|h| h)
                    } else {
                        results.any(|h| h)
                    }
                }
                Self::Values {
                    values,
                    blank,
                    dates,
                } => {
                    charge(work, units(dates.len()))?;
                    if matches!(item, CacheItem::Missing) {
                        *blank
                    } else if serial(item).is_some_and(|s| dates.iter().any(|d| d.covers(s))) {
                        true
                    } else {
                        // A date item's label is its serial (`item_text`); a values list over
                        // dates is unprobed (UNVERIFIED, T-4799).
                        let label = item_text(item);
                        let ticks = units(label.len()).saturating_mul(FOLD_TICKS);
                        charge(work, compared_units(ticks))?;
                        values.contains(&fold_case(&label))
                    }
                }
                Self::Caption { and, tests } => {
                    let label = Text::from(item_text(item));
                    let ticks = (tests.iter())
                        .map(|test| test.ticks(&label))
                        .fold(0, u64::saturating_add);
                    charge(work, compared_units(ticks))?;
                    let mut results = tests.iter().map(|test| test.holds(&label));
                    if *and {
                        results.all(|h| h)
                    } else {
                        results.any(|h| h)
                    }
                }
            };
            keeps.push(keep);
        }
        Ok(keeps)
    }
}

impl CaptionTest {
    /// The ticks of testing `label`, at most: a wildcard match reads the label once per
    /// pattern unit when it backtracks, a comparison each text once by the full collation.
    fn ticks(&self, label: &Text) -> u64 {
        let label_len = units(label.len());
        match &self.pattern {
            Some(_) => label_len
                .saturating_mul(units(self.value.len()).max(1))
                .saturating_mul(BYTE_TICKS),
            None => label_len
                .saturating_add(units(self.value.len()))
                .saturating_mul(CHAR_TICKS),
        }
    }

    /// The condition on `label` (`item_text`, a number as General shows it): `equal` and
    /// `notEqual` match the value as a wildcard pattern (`Ap*`, `*e`, `*an*`), the order
    /// operators compare texts as Excel's `<` does; both ignore case (pivots.md "Pivot sorts and
    /// filters as Excel for the web renders them": `CBwU`, `CCoU`, `CEqL`, `CGtU`). How a number
    /// item compares is unprobed (UNVERIFIED, T-4799).
    fn holds(&self, label: &Text) -> bool {
        match (&self.pattern, self.operator) {
            (Some(pattern), FilterOperator::NotEqual) => !pattern.matches(label),
            (Some(pattern), _) => pattern.matches(label),
            (None, operator) => operator.holds(cmp_text_pairwise(label, &self.value)),
        }
    }
}

/// The serial of a number or date item.
fn serial(item: &CacheItem) -> Option<f64> {
    match item {
        CacheItem::Number(n) | CacheItem::Date(n) => Some(*n),
        _ => None,
    }
}

/// A date condition on `item`'s serial against `value`: the item's date-time, or its day under
/// `useWholeDay` (pivots.md "useWholeDay": `WEqN`, `WEq1`, `WNt1`, `WOe1`, `WBt1`, `WEqT1`). An
/// item with no date, or a value that is no number, meets no condition, `notEqual` included
/// (UNVERIFIED, T-4799).
fn date_holds(
    operator: FilterOperator,
    value: Option<f64>,
    item: &CacheItem,
    whole_day: bool,
) -> bool {
    let (Some(serial), Some(value)) = (serial(item), value) else {
        return false;
    };
    let serial = if whole_day { serial.floor() } else { serial };
    serial
        .partial_cmp(&value)
        .is_some_and(|ordering| operator.holds(ordering))
}

/// The (year, month, day) of the day of `serial`.
fn ymd(serial: f64) -> Option<(i32, u32, u32)> {
    if !serial.is_finite() {
        return None;
    }
    // Within Excel's calendar (`excel_serial_to_ymd` refuses the rest), so the cast is exact.
    let day = serial.floor() as i64;
    excel_serial_to_ymd(day)
}

/// A month's index from year 0, to compare months and quarters across years.
fn month_index(year: i32, month: u32) -> i64 {
    i64::from(year) * 12 + i64::from(month) - 1
}

impl DynamicDate {
    /// Whether the filter keeps every item whatever its date: an unread type, or a relative
    /// period with no clock to read today from.
    fn keeps_all(self, today: Option<f64>) -> bool {
        match self {
            Self::Other => true,
            Self::Relative(..) | Self::YearToDate => today.is_none(),
            Self::Month(_) | Self::Quarter(_) => false,
        }
    }

    /// Whether the date `serial` is in the period, `today` being a serial day. A relative period
    /// compares the item's day with today's (probes/t4130-pivot-relative-date-filters, each
    /// keeping exactly its bucket). Weeks run Sunday to Saturday, en-US's (UNVERIFIED, T-4799: the
    /// probe's ±7 days do not show where a week starts).
    fn covers(self, serial: f64, today: Option<f64>) -> bool {
        let Some((year, month, _)) = ymd(serial) else {
            return false;
        };
        if self.keeps_all(today) {
            return true;
        }
        match self {
            Self::Month(m) => month == u32::from(m),
            Self::Quarter(q) => (month - 1) / 3 + 1 == u32::from(q),
            Self::Other => true,
            Self::Relative(unit, offset) => {
                let Some(today) = today else {
                    return true;
                };
                let Some((this_year, this_month, _)) = ymd(today) else {
                    return false;
                };
                let offset = i64::from(offset);
                let day = serial.floor();
                match unit {
                    RelativeUnit::Day => day == today + offset as f64,
                    RelativeUnit::Week => {
                        // Serial 1 (1900-01-01) is a Sunday in Excel's calendar.
                        let start = today - (today - 1.0).rem_euclid(7.0) + 7.0 * offset as f64;
                        (start..start + 7.0).contains(&day)
                    }
                    RelativeUnit::Month => {
                        month_index(year, month) == month_index(this_year, this_month) + offset
                    }
                    RelativeUnit::Quarter => {
                        month_index(year, month).div_euclid(3)
                            == month_index(this_year, this_month).div_euclid(3) + offset
                    }
                    RelativeUnit::Year => i64::from(year) == i64::from(this_year) + offset,
                }
            }
            Self::YearToDate => today.is_none_or(|today| {
                ymd(today).is_some_and(|(this_year, _, _)| year == this_year)
                    && serial.floor() <= today
            }),
        }
    }
}

impl DateGroupItem {
    /// Whether the date `serial` has this item's parts.
    fn covers(&self, serial: f64) -> bool {
        let Some((year, month, day)) = ymd(serial) else {
            return false;
        };
        let seconds = ((serial - serial.floor()) * 86_400.0).round();
        // Within a day, so it fits.
        let seconds = seconds as u32;
        let parts = [
            u32::try_from(year).ok() == u32::try_from(self.year).ok(),
            month == self.month,
            day == self.day,
            seconds / 3600 == self.hour,
            seconds / 60 % 60 == self.minute,
            seconds % 60 == self.second,
        ];
        parts
            .iter()
            .take(usize::from(self.parts.clamp(1, 6)))
            .all(|p| *p)
    }
}

#[cfg(test)]
mod tests;
