//! Date groups of a pivot field (`<rangePr groupBy startDate endDate>`, T-4225, T-4236, T-4246).
//!
//! One rule decides which group a date falls in. The xlsx importer, `Txn::add_pivot_table` and
//! GETPIVOTDATA all use it: the importer and `add_pivot_table` turn the grouped field into its
//! group items, and GETPIVOTDATA picks a group by its calendar value. The rule is Excel's own,
//! probed with excel-graph (probes/t4236-pivot-date-groups-no-blank;
//! docs/excel-semantics/web.md "Grouped pivot fields import as their group items").
// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
// LINT-DEBT-END

use std::collections::HashMap;

use bitcode::{Decode, Encode};
use thiserror::Error as ThisError;

use super::{CacheField, CacheItem};
use crate::formatter::date_clock::SECONDS_PER_DAY_F64 as SECONDS_PER_DAY;
use crate::formatter::dates::excel_serial_to_ymd;

/// What a date group groups by (ECMA-376 `groupBy`, all but `range`).
#[derive(Encode, Decode, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateGroupBy {
    /// The second of the minute.
    Seconds,
    /// The minute of the hour.
    Minutes,
    /// The hour of the day.
    Hours,
    /// The day of a leap year (`29-Feb` is always an item).
    Days,
    /// The month.
    Months,
    /// The quarter.
    Quarters,
    /// The year, counted from `startDate`'s.
    Years,
}

impl DateGroupBy {
    /// The `groupBy` attribute's date value; `None` for `range` and any unknown one.
    #[must_use]
    pub fn parse(text: &str) -> Option<DateGroupBy> {
        Some(match text {
            "seconds" => DateGroupBy::Seconds,
            "minutes" => DateGroupBy::Minutes,
            "hours" => DateGroupBy::Hours,
            "days" => DateGroupBy::Days,
            "months" => DateGroupBy::Months,
            "quarters" => DateGroupBy::Quarters,
            "years" => DateGroupBy::Years,
            _ => return None,
        })
    }

    /// How many group items the grouping has: the in-range ones plus the `<start` and `>end`
    /// items around them. `None` for years, whose count follows the range.
    fn fixed_item_count(self) -> Option<usize> {
        match self {
            DateGroupBy::Seconds | DateGroupBy::Minutes => Some(60 + 2),
            DateGroupBy::Hours => Some(24 + 2),
            // Day of the year on a leap-year calendar: 29-Feb is always an item (52379).
            DateGroupBy::Days => Some(366 + 2),
            DateGroupBy::Months => Some(12 + 2),
            DateGroupBy::Quarters => Some(4 + 2),
            DateGroupBy::Years => None,
        }
    }
}

/// A field grouped by dates from `start` to `end` (serials, 1900 date system). Its items are the
/// `<start` group, one per calendar value, and the `>end` group.
///
/// The fields are private so that only [`DateGroups::new`] builds one, but a decoded value skips
/// it: every method still checks `count` itself ([`DateGroups::last_inner`]) rather than trust it.
#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub struct DateGroups {
    /// What the field groups by.
    by: DateGroupBy,
    /// `startDate`, as a serial.
    start: f64,
    /// `endDate`, as a serial.
    end: f64,
    /// How many group items the field has, the end groups included.
    count: usize,
}

/// Why a field cannot be grouped by dates.
#[derive(ThisError, Debug, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub enum DateGroupError {
    /// The item count is not the grouping's, there are fewer than three items, or the end is
    /// before the start.
    #[error("{0} group items where the date grouping has another count")]
    Count(usize),
    /// A date or a bound is outside the calendar.
    #[error("a date group outside the calendar")]
    Calendar,
    /// A base value is text, a boolean or an error.
    #[error("a date group over a value that is not a date")]
    NotADate,
    /// A record points past the base field's items.
    #[error("a record outside the grouped field's items")]
    BadRecord,
}

impl DateGroups {
    /// A grouping of `count` group items.
    ///
    /// # Errors
    /// [`DateGroupError::Count`] when `count` is not the grouping's (fewer than three, or not
    /// 62, 26, 368, 14 or 6 for seconds, minutes, hours, days, months and quarters) or `end` is
    /// before `start`; [`DateGroupError::Calendar`] for a bound outside the calendar.
    pub fn new(
        by: DateGroupBy,
        start: f64,
        end: f64,
        count: usize,
    ) -> Result<DateGroups, DateGroupError> {
        if count < 3 || by.fixed_item_count().is_some_and(|n| n != count) || start > end {
            return Err(DateGroupError::Count(count));
        }
        split_serial(start)?;
        split_serial(end)?;
        Ok(DateGroups {
            by,
            start,
            end,
            count,
        })
    }

    /// What the grouping groups by and its `startDate` and `endDate` serials, to build it again
    /// over refreshed values ([`crate::pivot::regroup`]).
    #[must_use]
    pub fn bounds(&self) -> (DateGroupBy, f64, f64) {
        (self.by, self.start, self.end)
    }

    /// The group item `value` falls in: 0 below the range, `count - 1` above it, and otherwise
    /// the calendar part counted from 1. A date is compared to `start` and `end` rounded to the
    /// second, as it is grouped: Excel's cache holds 2018-12-31 23:59:59.5 as 2019-01-01, so it
    /// goes in the first month of a group starting then, not below it
    /// (probes/t4236-pivot-date-groups-no-blank, web.md).
    ///
    /// # Errors
    /// [`DateGroupError::Calendar`] for a date outside the calendar;
    /// [`DateGroupError::Count`] for a grouping of fewer than three items (only a decoded one).
    pub fn item(&self, value: f64) -> Result<usize, DateGroupError> {
        let last_inner = self.last_inner().ok_or(DateGroupError::Count(self.count))?;
        let at = split_serial(value)?;
        if at < split_serial(self.start)? {
            return Ok(0);
        }
        if at > split_serial(self.end)? {
            return Ok(last_inner + 1);
        }
        let (year_day, year, seconds) = calendar_parts(value)?;
        let inner = match self.by {
            DateGroupBy::Years => {
                let (_, start, _) = calendar_parts(self.start)?;
                usize::try_from(year - start).map_or(1, |n| n + 1)
            }
            DateGroupBy::Seconds => seconds % 60 + 1,
            DateGroupBy::Minutes => seconds / 60 % 60 + 1,
            DateGroupBy::Hours => seconds / 3600 + 1,
            DateGroupBy::Days => year_day.leap_year_day,
            DateGroupBy::Months => year_day.month as usize,
            DateGroupBy::Quarters => (year_day.month as usize - 1) / 3 + 1,
        };
        Ok(inner.clamp(1, last_inner))
    }

    /// The group item GETPIVOTDATA's number item `value` picks: the inner group whose calendar
    /// value equals `value`'s integer part. That is the hour (0 to 23: `0` and `TIME(10,0,0)` are
    /// `12 AM`, `10` is `10 AM`), the leap-year day (1 to 366: `60` is `29-Feb`), the month (1 to
    /// 12), the quarter (1 to 4) or the year (`2018`, `2018.5`). A value with no such group (a
    /// month of `31`, a date serial in a month group) is `None` (`#REF!`), and so are the
    /// `<start` and `>end` groups, which have no calendar value
    /// (probes/t4236-pivot-date-groups-no-blank). A second or a minute is 0 to 59 (`:00` to
    /// `:59`), and the integer part is truncated toward zero, so `-0.5` is `12 AM` or `:00` and
    /// `-1` and `60` are `#REF!` (probes/t4269-pivot-time-group-number-items).
    #[must_use]
    pub fn number_item(&self, value: f64) -> Option<usize> {
        let last_inner = self.last_inner()?;
        if !value.is_finite() {
            return None;
        }
        let whole = value.trunc();
        let inner = match self.by {
            DateGroupBy::Seconds | DateGroupBy::Minutes => in_range(whole, 0.0, 59.0)? + 1,
            DateGroupBy::Hours => in_range(whole, 0.0, 23.0)? + 1,
            DateGroupBy::Days => in_range(whole, 1.0, 366.0)?,
            DateGroupBy::Months => in_range(whole, 1.0, 12.0)?,
            DateGroupBy::Quarters => in_range(whole, 1.0, 4.0)?,
            DateGroupBy::Years => {
                let (_, start, _) = calendar_parts(self.start).ok()?;
                // At most MAX_ITEMS groups, so the count is exact as an f64.
                in_range(whole - f64::from(start), 0.0, (last_inner - 1) as f64)? + 1
            }
        };
        (1..=last_inner).contains(&inner).then_some(inner)
    }

    /// The index of the last inner group, `count - 2`; `None` for fewer than three items, which
    /// only a decoded grouping can have ([`DateGroups::new`] refuses them).
    fn last_inner(&self) -> Option<usize> {
        self.count.checked_sub(2).filter(|&n| n >= 1)
    }
}

/// `value` as a whole number when it is from `low` to `high`.
fn in_range(value: f64, low: f64, high: f64) -> Option<usize> {
    // `value` is a whole number from `low` >= 0 to `high`, so the cast is exact.
    (low..=high).contains(&value).then_some(value as usize)
}

/// Field `base` grouped by `groups`, `labels` being its `<groupItems>` labels: the grouped
/// field's items are the labels and each record's item is the group its date falls in
/// ([`DateGroups::item`]). A blank value goes in item 0, the label 59932 gives `(blank)`.
/// Which item Excel uses is UNVERIFIED: a refresh drops the date grouping of a field holding a
/// blank (T-4236), so no probe can say. The one grouping step for dates, shared by the xlsx
/// importer (`apply_groups`) and `Txn::add_pivot_table`.
///
/// # Errors
/// [`DateGroupError`] for a value that is not a date or is outside the calendar, or a bad record.
pub fn group_by_dates(
    base: &CacheField,
    groups: DateGroups,
    labels: Vec<CacheItem>,
) -> Result<CacheField, DateGroupError> {
    if labels.len() != groups.count {
        return Err(DateGroupError::Count(labels.len()));
    }
    let map = base
        .items
        .iter()
        .map(|item| match item {
            CacheItem::Missing => Ok(0),
            CacheItem::Number(v) | CacheItem::Date(v) => {
                // At most MAX_ITEMS labels, so the item index fits a u32.
                u32::try_from(groups.item(*v)?).map_err(|_| DateGroupError::Calendar)
            }
            _ => Err(DateGroupError::NotADate),
        })
        .collect::<Result<Vec<u32>, _>>()?;
    let records = base
        .records
        .iter()
        .map(|&i| map.get(i as usize).copied())
        .collect::<Option<Vec<u32>>>()
        .ok_or(DateGroupError::BadRecord)?;
    Ok(CacheField {
        name: base.name.clone(),
        items: labels,
        records,
        number_groups: None,
        date_groups: Some(groups),
        added_group: None,
    })
}

/// Field `base` as Excel's refresh leaves a date-grouped field that holds a blank: the grouping
/// is dropped, each date rounded to the second ([`split_serial`], the rule [`DateGroups::item`]
/// groups by) is its own [`CacheItem::Date`], and the blank stays [`CacheItem::Missing`]. The
/// items are in record (first-appearance) order, not sorted, so dates within a second of each
/// other are one item at the first one's place (probes/t4236-pivot-date-groups pivot M:
/// `... 43876, (blank), 43631`; web.md "Grouped pivot fields import as their group items",
/// T-4260). Used by `Txn::add_pivot_table`; the importer keeps a saved grouping (59932).
///
/// # Errors
/// [`DateGroupError`] for a value that is not a date or is outside the calendar, or a bad record.
pub fn ungroup_dates(base: &CacheField) -> Result<CacheField, DateGroupError> {
    let rounded = base
        .items
        .iter()
        .map(|item| match item {
            CacheItem::Missing => Ok(None),
            CacheItem::Number(v) | CacheItem::Date(v) => split_serial(*v).map(Some),
            _ => Err(DateGroupError::NotADate),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut items = Vec::new();
    let mut seen: HashMap<Option<(i64, usize)>, u32> = HashMap::new();
    let mut records = Vec::with_capacity(base.records.len());
    for &record in &base.records {
        let key = *rounded
            .get(record as usize)
            .ok_or(DateGroupError::BadRecord)?;
        let index = if let Some(&index) = seen.get(&key) {
            index
        } else {
            // At most one item per base item, so the index fits a u32 as theirs do.
            let index = u32::try_from(items.len()).map_err(|_| DateGroupError::BadRecord)?;
            items.push(match key {
                None => CacheItem::Missing,
                Some((day, seconds)) => {
                    CacheItem::Date(day as f64 + seconds as f64 / SECONDS_PER_DAY)
                }
            });
            seen.insert(key, index);
            index
        };
        records.push(index);
    }
    Ok(CacheField {
        name: base.name.clone(),
        items,
        records,
        number_groups: None,
        date_groups: None,
        added_group: None,
    })
}

/// The calendar parts of a date that groups read.
struct YearDay {
    month: u32,
    /// The day's position in a leap year, 1 to 366 (1-Jan is 1, 29-Feb is 60, 1-Mar is 61).
    leap_year_day: usize,
}

/// The month and leap-year day of `serial`'s date, its year, and its time of day in whole
/// seconds. A time rounds to the nearest second, and 24:00:00 is the next day.
fn calendar_parts(serial: f64) -> Result<(YearDay, i32, usize), DateGroupError> {
    const BEFORE_MONTH: [usize; 12] = [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335];
    let (day, seconds) = split_serial(serial)?;
    let (year, month, date) = excel_serial_to_ymd(day).ok_or(DateGroupError::Calendar)?;
    let before = BEFORE_MONTH
        .get((month as usize).wrapping_sub(1))
        .copied()
        .unwrap_or(0);
    Ok((
        YearDay {
            month,
            leap_year_day: before + (date as usize).max(1),
        },
        year,
        seconds,
    ))
}

/// A serial's whole day and its time of day in seconds, as Excel's pivot cache rounds a date to
/// the second: the serial divided by the double `1/86400` (not multiplied by 86400), rounded half
/// up, so 23:59:59.5 is the next day. This is not HOUR's rule (`time_of_day_parts`, dates.md): a
/// 2019 date 2.2e-7 s below 10:59:59.5 is 11:00:00 here and 10:59:59 to HOUR. Chosen among 13
/// candidates, then confirmed by two sweeps written after the freeze, 180 of 180 points
/// (probes/t4261-pivot-half-second, -fresh, -fresh-2, -fresh-3; web.md, T-4261).
fn split_serial(serial: f64) -> Result<(i64, usize), DateGroupError> {
    // Pivot dates are 1900-01-00 to 9999-12-31, about 3e6 days: well inside an i64.
    if !(0.0..=1e7).contains(&serial) {
        return Err(DateGroupError::Calendar);
    }
    // Divide, do not multiply by 86400: the two differ by an ulp at noise points (the unit test).
    // At most 8.64e11 seconds: exact in an f64 and an i64.
    let total = (serial / (1.0 / SECONDS_PER_DAY) + 0.5).floor() as i64;
    let day_seconds = SECONDS_PER_DAY as i64;
    Ok((total / day_seconds, (total % day_seconds) as usize))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests")]

    use super::*;

    const JAN_1_2019: f64 = 43466.0;

    /// GETPIVOTDATA number items (probes/t4236-pivot-date-groups-no-blank, web.md).
    #[test]
    fn a_number_item_picks_the_group_of_its_calendar_value() {
        let hours = DateGroups::new(DateGroupBy::Hours, JAN_1_2019, JAN_1_2019 + 1.0, 26).unwrap();
        assert_eq!(hours.number_item(0.0), Some(1));
        assert_eq!(hours.number_item(10.0 / 24.0), Some(1));
        assert_eq!(hours.number_item(10.0), Some(11));
        assert_eq!(hours.number_item(24.0), None);
        assert_eq!(hours.number_item(JAN_1_2019), None);
        let days = DateGroups::new(DateGroupBy::Days, 1.0, 43831.0, 368).unwrap();
        assert_eq!(days.number_item(1.0), Some(1));
        assert_eq!(days.number_item(60.0), Some(60));
        assert_eq!(days.number_item(61.0), Some(61));
        assert_eq!(days.number_item(0.0), None);
        assert_eq!(days.number_item(JAN_1_2019), None);
        let months =
            DateGroups::new(DateGroupBy::Months, JAN_1_2019, JAN_1_2019 + 365.0, 14).unwrap();
        assert_eq!(months.number_item(1.0), Some(1));
        for value in [31.0, 32.0, 60.0, JAN_1_2019, 0.0, -1.0, f64::NAN] {
            assert_eq!(months.number_item(value), None, "{value}");
        }
        let quarters =
            DateGroups::new(DateGroupBy::Quarters, JAN_1_2019, JAN_1_2019 + 364.0, 6).unwrap();
        assert_eq!(quarters.number_item(1.0), Some(1));
        assert_eq!(quarters.number_item(5.0), None);
        // 2017-01-01 to 2020-12-31: items <1/1/2017, 2017, 2018, 2019, 2020, >12/31/2020.
        let years = DateGroups::new(DateGroupBy::Years, 42736.0, 44196.0, 6).unwrap();
        assert_eq!(years.number_item(2018.0), Some(2));
        assert_eq!(years.number_item(2018.5), Some(2));
        assert_eq!(years.number_item(2017.0), Some(1));
        assert_eq!(years.number_item(2020.0), Some(4));
        assert_eq!(years.number_item(2021.0), None);
        assert_eq!(years.number_item(2016.0), None);
        assert_eq!(years.number_item(42736.0), None);
    }

    /// Number items on second, minute and hour groups: 0 to 59 (0 to 23), the integer part
    /// truncated toward zero (probes/t4269-pivot-time-group-number-items, web.md).
    #[test]
    fn a_time_number_item_truncates_toward_zero() {
        for by in [DateGroupBy::Seconds, DateGroupBy::Minutes] {
            let g = DateGroups::new(by, JAN_1_2019, JAN_1_2019 + 1.0, 62).unwrap();
            for (value, item) in [(0.0, 1), (5.0, 6), (59.0, 60), (59.999, 60), (1.5, 2)] {
                assert_eq!(g.number_item(value), Some(item), "{by:?} {value}");
            }
            for value in [-0.5, -0.0001, 0.5, 5.0 / 86400.0] {
                assert_eq!(g.number_item(value), Some(1), "{by:?} {value}");
            }
            for value in [60.0, 61.0, -1.0, JAN_1_2019] {
                assert_eq!(g.number_item(value), None, "{by:?} {value}");
            }
        }
        let hours = DateGroups::new(DateGroupBy::Hours, JAN_1_2019, JAN_1_2019 + 1.0, 26).unwrap();
        assert_eq!(hours.number_item(-0.5), Some(1));
        assert_eq!(hours.number_item(-1.0), None);
    }

    /// A decoded grouping skips `new`'s count check: fewer than three items is an error, never
    /// an underflow.
    #[test]
    fn a_grouping_of_too_few_items_never_underflows() {
        for count in 0..3 {
            for by in [DateGroupBy::Hours, DateGroupBy::Years] {
                let g = DateGroups {
                    by,
                    start: JAN_1_2019,
                    end: JAN_1_2019 + 1.0,
                    count,
                };
                assert_eq!(g.item(JAN_1_2019), Err(DateGroupError::Count(count)));
                assert_eq!(g.number_item(0.0), None);
                assert_eq!(g.number_item(2019.0), None);
            }
        }
    }

    /// A dropped grouping over a blank: second-rounded dates in first-appearance order, the
    /// blank kept at its record's place (probes/t4236-pivot-date-groups pivot M, web.md).
    #[test]
    fn an_ungrouped_date_field_rounds_to_the_second_in_record_order() {
        let base = CacheField {
            name: "D1".into(),
            // 2019-02-28 23:59:59.4 and .5, 2018-06-01, a blank, 2019-01-01 00:00.
            items: vec![
                CacheItem::Number(43_496.999_993_055_56),
                CacheItem::Number(43_496.999_994_212_965),
                CacheItem::Number(43252.0),
                CacheItem::Missing,
                CacheItem::Number(43497.0),
            ],
            records: vec![2, 0, 1, 3, 4, 2],
            number_groups: None,
            date_groups: None,
            added_group: None,
        };
        let field = ungroup_dates(&base).unwrap();
        assert_eq!(
            field.items,
            vec![
                CacheItem::Date(43252.0),
                CacheItem::Date(43496.0 + 86399.0 / 86400.0),
                CacheItem::Date(43497.0),
                CacheItem::Missing,
            ]
        );
        assert_eq!(field.records, vec![0, 1, 2, 3, 2, 0]);
        let text = CacheField {
            items: vec![CacheItem::Text("x".into())],
            records: vec![0],
            ..base
        };
        assert_eq!(ungroup_dates(&text), Err(DateGroupError::NotADate));
    }

    /// The cache's second rounding divides by the double 1/86400 (probes/t4261-pivot-half-second,
    /// -fresh, -fresh-2): a hair below the half rounds up where the product would not.
    #[test]
    fn a_noise_half_second_rounds_as_the_pivot_cache_does() {
        // 10:59:59.5 written in 2019, 2.2e-7 s below the half: 11:00:00 (t4236 no-blank H).
        assert_eq!(
            split_serial(43_466.458_327_546_294).unwrap(),
            (43466, 39600)
        );
        // 12:34:56.5 less 2.6e-7 s on day 25006: 12:34:57, though serial*86400 is ...96.4999995.
        assert_eq!(
            split_serial(25_006.524_265_046_293).unwrap(),
            (25006, 45297)
        );
        // A 1900 date 2.2e-7 s below 10:59:59.5 stays 10:59:59 (t4261-pivot-half-second A10).
        assert_eq!(split_serial(66.458_327_546_293_75).unwrap(), (66, 39599));
        // 23:59:59.5 is the next day.
        assert_eq!(split_serial(43_465.999_994_212_965).unwrap(), (43466, 0));
    }

    /// The record-to-group rule (probes/t4236-pivot-date-groups-no-blank, web.md).
    #[test]
    fn a_date_goes_in_its_second_rounded_group() {
        let months =
            DateGroups::new(DateGroupBy::Months, JAN_1_2019, JAN_1_2019 + 365.0, 14).unwrap();
        let second = 1.0 / 86400.0;
        // 2019-01-31 23:59:59.4 is Jan, .5 is Feb.
        let jan_31 = JAN_1_2019 + 30.0;
        assert_eq!(months.item(jan_31 + 1.0 - 0.6 * second).unwrap(), 1);
        assert_eq!(months.item(jan_31 + 1.0 - 0.5 * second).unwrap(), 2);
        // 2018-12-31 23:59:59.5 is Jan of a group starting 2019-01-01.
        assert_eq!(months.item(JAN_1_2019 - 0.5 * second).unwrap(), 1);
        assert_eq!(months.item(JAN_1_2019 - 1.0).unwrap(), 0);
        assert_eq!(months.item(JAN_1_2019 + 365.0).unwrap(), 1);
        assert_eq!(months.item(JAN_1_2019 + 365.5).unwrap(), 13);
        let e = DateGroups::new(DateGroupBy::Months, 1.0, 366.0, 14).unwrap();
        for (serial, item) in [
            (0.5, 0),
            (31.0, 1),
            (32.0, 2),
            (60.0, 2),
            (60.75, 2),
            (61.0, 3),
        ] {
            assert_eq!(e.item(serial).unwrap(), item, "{serial}");
        }
    }

    #[test]
    fn a_grouping_of_the_wrong_count_is_refused() {
        assert_eq!(
            DateGroups::new(DateGroupBy::Months, 1.0, 366.0, 13),
            Err(DateGroupError::Count(13))
        );
        assert_eq!(
            DateGroups::new(DateGroupBy::Years, 2.0, 1.0, 3),
            Err(DateGroupError::Count(3))
        );
        assert_eq!(
            DateGroups::new(DateGroupBy::Years, -1.0, 1.0, 3),
            Err(DateGroupError::Calendar)
        );
    }
}
