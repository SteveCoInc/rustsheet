//! Date groups of a pivot field (`<rangePr groupBy startDate endDate>`, T-4225, T-4236, T-4246).
//!
//! One rule decides which group a date falls in. The xlsx importer, `Txn::add_pivot_table` and
//! GETPIVOTDATA all use it: the importer and `add_pivot_table` turn the grouped field into its
//! group items, and GETPIVOTDATA picks a group by its calendar value. The rule is Excel's own,
//! probed with excel-graph (probes/t4236-pivot-date-groups-no-blank;
//! docs/excel-semantics/web.md "Grouped pivot fields import as their group items").

use bitcode::{Decode, Encode};
use thiserror::Error as ThisError;

use super::{CacheField, CacheItem};
use crate::formatter::dates::{carried_day, excel_serial_to_ymd, time_of_day_parts};

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
#[derive(Encode, Decode, Debug, PartialEq, Clone, Copy)]
pub struct DateGroups {
    /// What the field groups by.
    pub by: DateGroupBy,
    /// `startDate`, as a serial.
    pub start: f64,
    /// `endDate`, as a serial.
    pub end: f64,
    /// How many group items the field has, the end groups included.
    pub count: usize,
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

    /// The group item `value` falls in: 0 below the range, `count - 1` above it, and otherwise
    /// the calendar part counted from 1. A date is compared to `start` and `end` rounded to the
    /// second, as it is grouped: Excel's cache holds 2018-12-31 23:59:59.5 as 2019-01-01, so it
    /// goes in the first month of a group starting then, not below it
    /// (probes/t4236-pivot-date-groups-no-blank, web.md).
    ///
    /// # Errors
    /// [`DateGroupError::Calendar`] for a date outside the calendar.
    pub fn item(&self, value: f64) -> Result<usize, DateGroupError> {
        let last = self.count - 1;
        let at = split_serial(value)?;
        if at < split_serial(self.start)? {
            return Ok(0);
        }
        if at > split_serial(self.end)? {
            return Ok(last);
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
        Ok(inner.clamp(1, last - 1))
    }

    /// The group item GETPIVOTDATA's number item `value` picks: the inner group whose calendar
    /// value equals `value`'s integer part. That is the hour (0 to 23: `0` and `TIME(10,0,0)` are
    /// `12 AM`, `10` is `10 AM`), the leap-year day (1 to 366: `60` is `29-Feb`), the month (1 to
    /// 12), the quarter (1 to 4) or the year (`2018`, `2018.5`). A value with no such group (a
    /// month of `31`, a date serial in a month group) is `None` (`#REF!`), and so are the
    /// `<start` and `>end` groups, which have no calendar value
    /// (probes/t4236-pivot-date-groups-no-blank). Seconds and minutes (0 to 59) are UNVERIFIED:
    /// no probe groups by them.
    #[must_use]
    pub fn number_item(&self, value: f64) -> Option<usize> {
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
                in_range(whole - f64::from(start), 0.0, (self.count - 3) as f64)? + 1
            }
        };
        (1..self.count - 1).contains(&inner).then_some(inner)
    }
}

/// `value` as a whole number when it is from `low` to `high`.
fn in_range(value: f64, low: f64, high: f64) -> Option<usize> {
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

/// A serial's whole day and its time of day in seconds, split as YEAR, MONTH, HOUR, MINUTE and
/// SECOND split it (`carried_day`, `time_of_day_parts`): the time rounds to the nearest second
/// and 23:59:59.5 or later is the next day, as Excel's pivot cache rounds a date (T-4236).
fn split_serial(serial: f64) -> Result<(i64, usize), DateGroupError> {
    // Pivot dates are 1900-01-00 to 9999-12-31, about 3e6 days: well inside an i64.
    if !(0.0..=1e7).contains(&serial) {
        return Err(DateGroupError::Calendar);
    }
    let day = carried_day(serial) as i64;
    let [hour, minute, second] = time_of_day_parts(serial);
    let seconds = (hour * 3600.0 + minute * 60.0 + second) as usize;
    Ok((day, seconds))
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
