//! ISO 8601 text of `t="d"` cells (Strict OOXML) → Excel serial.

use chrono::{NaiveDate, NaiveTime, Timelike};
use rustsheet_engine::formatter::dates::real_date_to_serial;
use rustsheet_engine::types::DateSystem;

const SECONDS_PER_DAY: f64 = 86_400.0;

/// The serial Excel gives a `t="d"` cell holding `text` (`YYYY-MM-DD`, optionally followed by
/// `Thh:mm:ss[.fff]` and a trailing `Z`), or `hh:mm:ss[.fff]` alone (a fraction of a day); `None` if
/// it is not ISO 8601, before 1900-01-01 or after 9999-12-31 (only the first date bound is probed).
/// Evidence: fixtures/t-d-iso-date-cells (1990-01-01 = 32874, 1990-01-01T12:00:00 = 32874.5,
/// 1900-02-28 = 59, 1900-03-01 = 61), t-d-z-suffix, t-d-fractional-seconds, t-d-1900-02-29 (60),
/// t-d-time-only (0.5); docs/excel-semantics/xlsx-storage.md. A date before 1900-01-01 reads as
/// `#VALUE!` (t-d-pre-1900), so the importer turns `None` into that error. Excel rejects a
/// workbook holding other unparsable `t="d"` text, so those have no Excel behavior to match.
#[must_use]
pub fn iso8601_to_serial(text: &str) -> Option<f64> {
    let text = text.trim();
    if !text.contains('-') {
        return time_fraction(text);
    }
    let (date_text, time_text) = split_date_time(text);
    // Excel's 1900 leap-year bug: serial 60 is the nonexistent 1900-02-29 (t-d-1900-02-29).
    let serial = if date_text == "1900-02-29" {
        60.0
    } else {
        let date = NaiveDate::parse_from_str(date_text, "%Y-%m-%d").ok()?;
        f64::from(real_date_to_serial(date)?)
    };
    let Some(time_text) = time_text else {
        return Some(serial);
    };
    Some(serial + time_fraction(time_text)?)
}

/// The serial a `t="d"` cell or a pivot cache `<d>` item holding `text` stands for in a workbook
/// of `date_system`: `iso8601_to_serial` moved to that system. In a 1904 workbook `2024-01-01` is
/// 43830, a time of day alone keeps its day fraction, and a date before 1904-01-01 has no serial,
/// which a cell reads as `#VALUE!` (fixtures/t-d-1904, t-d-1904-epoch, t-d-1904-time-only,
/// t-d-1904-pre-epoch, t-d-1904-1900-01-01; docs/excel-semantics/xlsx-storage.md).
#[must_use]
pub fn iso8601_to_date_system_serial(text: &str, date_system: DateSystem) -> Option<f64> {
    iso8601_to_serial(text).and_then(|s| date_system.serial_from_imported_1900_serial(s))
}

/// The serial a pivot cache `<d>` item holding `text` stands for in a workbook of `date_system`.
/// Excel writes a cache date below serial 61 on the OLE epoch, 1899-12-30 plus the serial in days,
/// not on the cell calendar: `1899-12-30T00:07:23` is 0.005127..., `1900-01-01T16:11:48` is
/// 2.6748611111111114 (corpus-bench 52379, source cell FCR!E241) and `1899-12-30T00:00:00` is 0
/// (corpus-bench 59932, group label `<1/0/1900`); docs/excel-semantics/xlsx-storage.md "Pivot cache
/// dates below serial 61 use the 1899-12-30 epoch" (T-3182). From 1900-03-01 (serial 61) the two
/// calendars agree, so later dates, and every date of a 1904 workbook (unprobed), take the cell
/// rule [`iso8601_to_date_system_serial`].
#[must_use]
pub fn pivot_cache_date_serial(text: &str, date_system: DateSystem) -> Option<f64> {
    if date_system == DateSystem::Excel1900 {
        if let Some(serial) = ole_serial_before_march_1900(text.trim()) {
            return Some(serial);
        }
    }
    iso8601_to_date_system_serial(text, date_system)
}

/// `text`'s serial on the OLE epoch (1899-12-30 = 0) when its date is 1899-12-30 to 1900-02-28;
/// `None` otherwise (the 1900 cell calendar then reads it as Excel's caches do).
fn ole_serial_before_march_1900(text: &str) -> Option<f64> {
    let (date_text, time_text) = split_date_time(text);
    let date = NaiveDate::parse_from_str(date_text, "%Y-%m-%d").ok()?;
    let epoch = NaiveDate::from_ymd_opt(1899, 12, 30)?;
    let first_shared = NaiveDate::from_ymd_opt(1900, 3, 1)?;
    if date < epoch || date >= first_shared {
        return None;
    }
    let days = f64::from(i32::try_from((date - epoch).num_days()).ok()?);
    match time_text {
        Some(time) => Some(days + time_fraction(time)?),
        None => Some(days),
    }
}

/// `YYYY-MM-DD[Thh:mm:ss[.fff][Z]]` split into its date and its time (the `Z` dropped).
fn split_date_time(text: &str) -> (&str, Option<&str>) {
    match text.split_once('T') {
        Some((d, t)) => (d, Some(t.strip_suffix('Z').unwrap_or(t))),
        None => (text, None),
    }
}

/// `hh:mm:ss[.fff]` as a fraction of a day.
fn time_fraction(text: &str) -> Option<f64> {
    let time = NaiveTime::parse_from_str(text, "%H:%M:%S%.f").ok()?;
    let seconds = f64::from(time.num_seconds_from_midnight()) + f64::from(time.nanosecond()) / 1e9;
    Some(seconds / SECONDS_PER_DAY)
}

#[cfg(test)]
mod tests {
    use super::{iso8601_to_serial, pivot_cache_date_serial};
    use rustsheet_engine::types::DateSystem;

    /// Pivot cache dates below serial 61 are on the OLE epoch (T-3182): the cached items of
    /// corpus-bench 52379 (`1899-12-30T00:07:23`; `1900-01-01T16:11:48` for source cell FCR!E241 =
    /// 2.6748611111111114) and 59932 (`1899-12-30T00:00:00`, labelled `<1/0/1900`, serial 0).
    /// From 1900-03-01 a cache date is the cell serial.
    #[test]
    fn pivot_cache_dates_below_serial_61_use_the_ole_epoch() {
        let serial = |text| pivot_cache_date_serial(text, DateSystem::Excel1900);
        assert_eq!(serial("1899-12-30T00:07:23"), Some(443.0 / 86_400.0));
        assert_eq!(serial("1900-01-01T16:11:48"), Some(2.674_861_111_111_111_4));
        assert_eq!(serial("1899-12-30T00:00:00"), Some(0.0));
        assert_eq!(serial("1900-02-28"), Some(60.0));
        assert_eq!(serial("1900-03-01"), Some(61.0));
        assert_eq!(serial("2024-01-01"), Some(45292.0));
        assert_eq!(serial("1899-12-29"), None);
        // A cell keeps its calendar: 1899-12-31 has no serial, 1900-02-28 is 59.
        assert_eq!(iso8601_to_serial("1900-02-28"), Some(59.0));
    }

    #[test]
    fn matches_excel_serials() {
        assert_eq!(iso8601_to_serial("1990-01-01"), Some(32874.0));
        assert_eq!(iso8601_to_serial("1990-01-01T12:00:00"), Some(32874.5));
        assert_eq!(iso8601_to_serial("1990-01-01T12:00:00Z"), Some(32874.5));
        assert_eq!(iso8601_to_serial("1900-02-28"), Some(59.0));
        assert_eq!(iso8601_to_serial("1900-03-01"), Some(61.0));
        assert_eq!(iso8601_to_serial("1900-02-29"), Some(60.0));
        assert_eq!(iso8601_to_serial("1900-02-29T12:00:00"), Some(60.5));
        assert_eq!(iso8601_to_serial("12:00:00"), Some(0.5));
        assert_eq!(
            iso8601_to_serial("1990-01-01T12:00:00.5"),
            Some(32_874.500_005_787_035)
        );
    }

    #[test]
    fn rejects_unparsable_text() {
        assert_eq!(iso8601_to_serial("abc"), None);
        assert_eq!(iso8601_to_serial("1990-13-01"), None);
        assert_eq!(iso8601_to_serial("1899-12-30"), None);
        assert_eq!(iso8601_to_serial("1899-12-31"), None);
        assert_eq!(iso8601_to_serial("1990-01-01T25:00:00"), None);
    }
}
