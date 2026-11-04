//! Grouped pivot cache fields (`<fieldGroup>`, T-4225).
//!
//! A grouped field shows group items (`<groupItems>`: `Qtr1`, `Jan`, `0-9`, ...) instead of its own
//! values. Excel stores no record values for a grouped field: each record's group is derived from
//! the base field's value, by a range (`<rangePr>`: numbers in steps, or dates by seconds, minutes,
//! hours, days, months, quarters or years) or by a list (`<discretePr>`: one group index per base
//! shared item). A grouping field Excel adds (`databaseField="0"`, `Quarters`, `Years`) derives
//! from its `base` field; a base field grouped in place (`par` set, `base` itself) shows its groups
//! in place of its values. The importer turns both into ordinary cache fields whose items are the
//! group items and whose records are each record's group, so the engine model and GETPIVOTDATA
//! need nothing new: a pivot field's `<item x>` indexes the group items, as Excel writes it.
//! Which group a date falls in is ECMA-376's layout, read off Excel-authored caches
//! (corpus-bench 52379, 59932), and UNVERIFIED against Excel's own grouping; a number follows
//! Excel's probed rule, [`group_by_numbers`] (T-4235; docs/excel-semantics/web.md "Grouped pivot
//! fields import as their group items").

use rustsheet_engine::formatter::dates::{carried_day, excel_serial_to_ymd, time_of_day_parts};
use rustsheet_engine::pivot::{
    group_by_numbers, CacheField, CacheItem, PivotCache, PivotCacheError, MAX_ITEMS,
    MAX_RECORD_VALUES,
};

use super::{skip, Skip};

/// What a `<rangePr groupBy>` groups by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GroupBy {
    /// Numbers in steps of `groupInterval` (`range`, the default).
    Range,
    Seconds,
    Minutes,
    Hours,
    Days,
    Months,
    Quarters,
    Years,
}

impl GroupBy {
    /// The `groupBy` attribute; `None` for an unknown one.
    pub(super) fn parse(text: Option<&str>) -> Option<GroupBy> {
        Some(match text.unwrap_or("range") {
            "range" => GroupBy::Range,
            "seconds" => GroupBy::Seconds,
            "minutes" => GroupBy::Minutes,
            "hours" => GroupBy::Hours,
            "days" => GroupBy::Days,
            "months" => GroupBy::Months,
            "quarters" => GroupBy::Quarters,
            "years" => GroupBy::Years,
            _ => return None,
        })
    }

    /// How many group items a date grouping has: the in-range ones plus the `<start` and `>end`
    /// items around them. `None` for numbers and years, whose count follows the range.
    fn fixed_item_count(self) -> Option<usize> {
        match self {
            GroupBy::Seconds | GroupBy::Minutes => Some(60 + 2),
            GroupBy::Hours => Some(24 + 2),
            // Day of the year on a leap-year calendar: 29-Feb is always an item (52379).
            GroupBy::Days => Some(366 + 2),
            GroupBy::Months => Some(12 + 2),
            GroupBy::Quarters => Some(4 + 2),
            GroupBy::Range | GroupBy::Years => None,
        }
    }
}

/// A `<rangePr>`: numbers or dates from `start` to `end`, in steps of `interval` (numbers only).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct RangeGroup {
    pub(super) by: GroupBy,
    pub(super) start: f64,
    pub(super) end: f64,
    pub(super) interval: f64,
}

/// One `<fieldGroup>` as read from the definition.
#[derive(Debug, Default)]
pub(super) struct FieldGroup {
    /// The cache field (in `<cacheFields>` order) the group belongs to.
    pub(super) field: u32,
    /// The database field its values come from.
    pub(super) base: u32,
    pub(super) range: Option<RangeGroup>,
    /// `<discretePr>`: per base shared item, its group index; `None` without one.
    pub(super) discrete: Option<Vec<u32>>,
    /// `<groupItems>`.
    pub(super) items: Vec<CacheItem>,
}

/// Turns the grouped fields of `cache` (its database fields only, as the records part holds them)
/// into group-item fields: each group replaces its field's items and records, and each name of
/// `grouping_fields` (the `databaseField="0"` fields, in order after the database fields) becomes
/// a field of its own. `values_left` is the workbook's record-value budget.
pub(super) fn apply_groups(
    cache: &mut PivotCache,
    groups: &[FieldGroup],
    grouping_fields: Vec<String>,
    values_left: usize,
) -> Result<(), Skip> {
    let database_fields = cache.fields.len();
    let total = database_fields + grouping_fields.len();
    if (cache.record_count as usize).saturating_mul(total) > values_left.min(MAX_RECORD_VALUES) {
        return Err(PivotCacheError::TooManyRecordValues.into());
    }
    let mut grouped = vec![false; total];
    for group in groups {
        let Some(seen) = grouped.get_mut(group.field as usize) else {
            return skip("a fieldGroup past the cache's fields");
        };
        if *seen {
            return skip("a field grouped twice");
        }
        *seen = true;
        if group.base as usize >= database_fields {
            return skip("a fieldGroup whose base is not a database field");
        }
    }
    if grouped.iter().skip(database_fields).any(|g| !g) {
        return skip("a grouping field without a fieldGroup");
    }
    // Every group reads its base's own values, so all are derived before any field changes.
    let mut derived = Vec::with_capacity(groups.len());
    let mut items = cache.fields.iter().map(|f| f.items.len()).sum::<usize>();
    for group in groups {
        let Some(base) = cache.fields.get(group.base as usize) else {
            return skip("a fieldGroup whose base is not a database field");
        };
        let field = group_field(base, group)?;
        items = items.saturating_add(field.items.len());
        if items > MAX_ITEMS {
            return Err(PivotCacheError::TooManyItems.into());
        }
        derived.push((group.field as usize, field));
    }
    derived.sort_by_key(|(index, _)| *index);
    let mut names = grouping_fields.into_iter();
    for (index, field) in derived {
        if let Some(existing) = cache.fields.get_mut(index) {
            existing.items = field.items;
            existing.records = field.records;
            existing.number_groups = field.number_groups;
        } else {
            // Sorted, and every grouping field has exactly one group, so this is the next one.
            let Some(name) = names.next() else {
                return skip("a grouping field without a name");
            };
            cache.fields.push(CacheField { name, ..field });
        }
    }
    Ok(())
}

/// The group items of `group` and, per record of `base`, its group.
fn group_field(base: &CacheField, group: &FieldGroup) -> Result<CacheField, Skip> {
    let count = group.items.len();
    if count == 0 {
        return skip("a fieldGroup without groupItems");
    }
    // A numeric range: Excel's own rule, which relabels a `<groupItems>` list of the wrong length
    // rather than refusing it (`group_by_numbers`; probes/t4228-pivot-range-groups pivot 3).
    if let (Some(range), None) = (&group.range, &group.discrete) {
        if range.by == GroupBy::Range {
            let bounds = [range.start, range.end, range.interval];
            return Ok(group_by_numbers(base, bounds, group.items.clone())?);
        }
    }
    // A blank base value goes in item 0: Excel's pivot fields list exactly the group items, with
    // no blank item of their own (59932: 14, 6 and 123 items over a field holding a blank record),
    // and 59932 labels its first month item "(blank)". Which item Excel uses is UNVERIFIED (T-4228).
    let mut map = Vec::with_capacity(base.items.len());
    for (index, item) in base.items.iter().enumerate() {
        let target = match (item, &group.discrete, &group.range) {
            (_, Some(discrete), _) => match discrete.get(index) {
                Some(&k) if (k as usize) < count => k,
                _ => return skip("a base item without a valid discrete group"),
            },
            (CacheItem::Missing, None, Some(_)) => 0,
            (CacheItem::Number(v) | CacheItem::Date(v), None, Some(range)) => {
                group_index(*v, range, count)?
            }
            (_, None, Some(_)) => return skip("a range group over a value that is not a number"),
            (_, None, None) => return skip("a fieldGroup without rangePr or discretePr"),
        };
        map.push(target);
    }
    let records = base
        .records
        .iter()
        .map(|&i| map.get(i as usize).copied())
        .collect::<Option<Vec<u32>>>()
        .ok_or(PivotCacheError::BadRecord)?;
    Ok(CacheField {
        name: String::new(),
        items: group.items.clone(),
        records,
        number_groups: None,
    })
}

/// The date group item `value` falls in: 0 below the range, `count - 1` above it, and otherwise
/// the calendar part counted from 1.
fn group_index(value: f64, range: &RangeGroup, count: usize) -> Result<u32, Skip> {
    if let Some(expected) = range.by.fixed_item_count() {
        if count != expected {
            return skip(format!(
                "{count} group items where the grouping has {expected}"
            ));
        }
    }
    if count < 3 {
        return skip("a range group with fewer than three items");
    }
    let last = count - 1;
    if value < range.start {
        return Ok(0);
    }
    if value > range.end {
        return Ok(u32::try_from(last).unwrap_or(0));
    }
    let inner = match range.by {
        // Numbers group by `group_by_numbers` (`group_field`).
        GroupBy::Range => return skip("a numeric range group outside group_by_numbers"),
        GroupBy::Years => {
            let ((_, Some(year), _), (_, Some(start), _)) =
                (calendar_parts(value)?, calendar_parts(range.start)?)
            else {
                return skip("a date group outside the calendar");
            };
            usize::try_from(year - start).map_or(1, |n| n + 1)
        }
        by => {
            let (Some(year_day), _, seconds) = calendar_parts(value)? else {
                return skip("a date group outside the calendar");
            };
            match by {
                GroupBy::Seconds => seconds % 60 + 1,
                GroupBy::Minutes => seconds / 60 % 60 + 1,
                GroupBy::Hours => seconds / 3600 + 1,
                GroupBy::Days => year_day.leap_year_day,
                GroupBy::Months => year_day.month as usize,
                _ => (year_day.month as usize - 1) / 3 + 1,
            }
        }
    };
    Ok(u32::try_from(inner.clamp(1, last - 1)).unwrap_or(0))
}

/// The calendar parts of a date that groups read.
struct YearDay {
    month: u32,
    /// The day's position in a leap year, 1 to 366 (1-Jan is 1, 29-Feb is 60, 1-Mar is 61).
    leap_year_day: usize,
}

/// The year of `serial`'s date (`None` past the calendar), its month and leap-year day, and its
/// time of day in whole seconds. A time rounds to the nearest second, and 24:00:00 is the next day.
fn calendar_parts(serial: f64) -> Result<(Option<YearDay>, Option<i32>, usize), Skip> {
    let (day, seconds) = split_serial(serial)?;
    Ok(match excel_serial_to_ymd(day) {
        Some((year, month, date)) => {
            const BEFORE_MONTH: [usize; 12] =
                [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335];
            let before = BEFORE_MONTH.get(month as usize - 1).copied().unwrap_or(0);
            (
                Some(YearDay {
                    month,
                    leap_year_day: before + (date as usize).max(1),
                }),
                Some(year),
                seconds,
            )
        }
        None => (None, None, seconds),
    })
}

/// A serial's whole day and its time of day in seconds, split as YEAR, MONTH, HOUR, MINUTE and
/// SECOND split it (`carried_day`, `time_of_day_parts`): the time rounds to the nearest second
/// and 23:59:59.5 or later is the next day. That pivot groups split it so is UNVERIFIED (T-4228).
fn split_serial(serial: f64) -> Result<(i64, usize), Skip> {
    // Pivot dates are 1900-01-00 to 9999-12-31, about 3e6 days: well inside an i64.
    if !(0.0..=1e7).contains(&serial) {
        return skip("a date group outside the calendar");
    }
    let day = carried_day(serial) as i64;
    let [hour, minute, second] = time_of_day_parts(serial);
    let seconds = (hour * 3600.0 + minute * 60.0 + second) as usize;
    Ok((day, seconds))
}
