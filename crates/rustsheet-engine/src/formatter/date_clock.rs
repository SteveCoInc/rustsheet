//! Parsing of "date clock" and clock text (`"1/2/2020 10:11:12.5"`, `"10:11"`) into serials,
//! shared by TIMEVALUE and the number coercion of text (`functions::date_and_time::time_text`)
//! and by typed input (`parse_formatted`). It sits in the formatter, below both, next to the
//! date text it builds on (T-1953). Excel: see docs/excel-semantics/dates.md#time-text.

use chrono::{NaiveTime, Timelike};

use super::date_text::{parse_date, CurrentYear};
use crate::{locale::Locale, types::DateSystem};

/// Seconds in a day.
pub(crate) const SECONDS_PER_DAY: i32 = 86_400;
/// [`SECONDS_PER_DAY`] as a float, the divisor of a clock serial.
pub(crate) const SECONDS_PER_DAY_F64: f64 = SECONDS_PER_DAY as f64;

/// [`parse_date_clock_serial`] in the workbook's date system. The date part counts from the
/// workbook's epoch, and a day before the 1904 epoch is no date (`"1904-01-02 10:00"+0` is
/// 1.4166666666666667, `"1903-12-31 10:00"+0` is #VALUE!; probes/date-system-1904-clock-text A1,
/// A7). The clock is added after the shift: summing in the 1900 system first loses low bits.
pub(crate) fn parse_date_clock_serial_in(
    text: &str,
    locale: &Locale,
    current_year: CurrentYear,
    date_system: DateSystem,
) -> Option<f64> {
    if is_letter_period_shape(text) {
        return parse_letter_period_serial_in(text, locale, current_year, date_system);
    }
    let (days, clock) = parse_date_clock_parts(text, locale, current_year)?;
    serial_in(days, clock, date_system)
}

/// `days` (a 1900-system date serial) shifted to `date_system`, plus `clock`.
fn serial_in(days: i32, clock: f64, date_system: DateSystem) -> Option<f64> {
    let days = date_system.serial_from_1900_serial(i64::from(days))?;
    Some(days as f64 + clock)
}

/// Typed `10 A. 7/1/2025` text (see [`letter_period_parts`]) as a serial in the workbook's date
/// system, the way [`parse_date_clock_serial_in`] reads it; the typed reader gives it its format.
/// In a 1904 workbook the no-date form keeps day 0 (`10 A. 5` is 49710.2763252315 in both
/// systems), a date shifts to the 1904 serial and must land on day 1 or later: `10 A. 7/1/2025`
/// is 93906.2763275463, `10 A. 1/1904` (day 0) and `10 A. 12/1903` stay text and are #VALUE! to
/// the formula readers (probes/t3920-typed-a-period-1904 A1-A5, excel-graph 2026-11-03).
pub(crate) fn parse_letter_period_serial_in(
    text: &str,
    locale: &Locale,
    current_year: CurrentYear,
    date_system: DateSystem,
) -> Option<f64> {
    let (date, clock) = letter_period_parts(text, locale, current_year)?;
    let days = match date {
        None => 0,
        Some(days) => date_system
            .serial_from_1900_serial(i64::from(days))
            .filter(|days| *days > 0)?,
    };
    Some(days as f64 + clock)
}

/// Whether `text` starts like `N A. F/D/Y` text (see [`letter_period_parts`]): a clock word of
/// digits and colons, a spaced `A`/`P`/`AM`/`PM` word, its period and (after optional spaces)
/// anything. Such text either reads that way or is #VALUE! to every formula reader: `10 A. 7-2025`,
/// `10 A. 5/1`, `10 A. 7/1`, `10 A. July 1, 2025` and `10 A. Jul-2025` are not `10 A.` followed
/// by a date, nor `1:2:3:4 A. 5` or `10 A. 5x` anything else (probes/t3920-typed-a-period-unprobed-edges
/// rows 15-23, t3948-letter-period-gate-edges, excel-graph 2026-11-03). `10 A.` with nothing
/// after the period is 10 AM.
fn is_letter_period_shape(text: &str) -> bool {
    let Some((clock, rest)) = text.split_once(' ') else {
        return false;
    };
    let Some((word, rest)) = rest.trim_start_matches(' ').split_once('.') else {
        return false;
    };
    clock.bytes().any(|b| b.is_ascii_digit())
        && clock.bytes().all(|b| b.is_ascii_digit() || b == b':')
        && am_pm_word(word).is_some()
        && !rest.trim_start_matches(' ').is_empty()
}

/// Hour-only "N AM" / "N PM" text (hour 0-12, no minutes) as a time-of-day serial: `"0 PM"` is
/// noon and `"0 AM"` midnight, `"13 PM"` is #VALUE! (probes/value-clock-text rows 9-11, 28).
pub(crate) fn parse_hour_only_am_pm(text: &str) -> Option<f64> {
    let (hour_str, is_pm) = parse_simple_am_pm(text)?;
    let hour = hour_str.parse::<u32>().ok()?;
    if hour > 12 {
        return None;
    }
    let time = NaiveTime::from_hms_opt(hour_from_12h(hour, is_pm), 0, 0)?;
    Some(time.num_seconds_from_midnight() as f64 / SECONDS_PER_DAY_F64)
}

/// "Date clock" text (`"1/2/2020 10:11:12.5"`, `"July 1, 2026 6 PM"`) as the unwrapped serial:
/// date serial plus clock serial. The clock is the last word, or the last two when they end in
/// AM/PM, since date text has spaces of its own (probes/date-text-grammar C30, C31, C55).
/// The sum is a 1900-system serial; a 1904 reader uses [`parse_date_clock_serial_in`] (via
/// `time_text::parse_time_serial` or TIMEVALUE) instead.
pub(crate) fn parse_date_clock_serial(
    text: &str,
    locale: &Locale,
    current_year: CurrentYear,
) -> Option<f64> {
    let (days, clock) = parse_date_clock_parts(text, locale, current_year)?;
    Some(f64::from(days) + clock)
}

/// The 1900-system date serial (not shifted for a 1904 workbook) and the clock fraction of "date clock" text.
///
/// The clock may also come first: `"10:00 7/1/2025"`, `"10 AM 7/1/2025"`, `"10:00 July 1, 2025"`
/// are the same date-times (probes/date-text-shape-survey row 113, date-text-shape-survey-2
/// rows 1-13), but not a clock on both sides (`"10:00 7/1/2025 10:00"`, row 8). The date and the
/// clock are split at a space character only: a tab or a no-break space between them is #VALUE!
/// (probes/date-text-shape-survey row 90, date-text-shape-survey-2 rows 31-32).
fn parse_date_clock_parts(
    text: &str,
    locale: &Locale,
    current_year: CurrentYear,
) -> Option<(i32, f64)> {
    if is_letter_period_shape(text) {
        let (date, clock) = letter_period_parts(text, locale, current_year)?;
        return Some((date.unwrap_or(0), clock));
    }
    // `date` comes with the spaces between it and the clock already cut.
    let split = |date: &str, clock: &str| {
        let clock = trim_spaces(clock);
        let clock = parse_clock_text(clock).or_else(|| parse_hour_only_am_pm(clock))?;
        let (days, _) = parse_date(date, locale, current_year).ok()?;
        Some((days, clock))
    };
    // The clock is the last word or the last two (`"6 PM"`), else the first word or two.
    let mut date_end = text.len();
    for _ in 0..2 {
        let Some(end) = text[..date_end].trim_end_matches(' ').rfind(' ') else {
            break;
        };
        date_end = end;
        let date = text[..date_end].trim_end_matches(' ');
        if let Some(parts) = split(date, &text[date_end..]) {
            return Some(parts);
        }
    }
    // Number coercion does not trim, so a space before a leading clock stays an error.
    if text.starts_with(' ') {
        return None;
    }
    let mut clock_end = 0;
    for _ in 0..2 {
        // Skip the spaces after the first clock word.
        let start =
            clock_end + (text[clock_end..].len() - text[clock_end..].trim_start_matches(' ').len());
        clock_end = start + text[start..].find(' ')?;
        let date = text[clock_end..].trim_start_matches(' ');
        if let Some(parts) = split(date, &text[..clock_end]) {
            return Some(parts);
        }
    }
    None
}

/// Seconds under every `N A. F/D/Y` reading (see [`letter_period_parts`]): 2^32 less 22, decoded
/// from probes/date-text-typed-clock-first-a-period A1 (`10 A. 7/1/2025` is 95368.2763275463) and
/// t3907-typed-clock-a-period-shapes A16 (`0 A. 7/1/2025`).
const LETTER_PERIOD_BASE_SECONDS: u64 = (1 << 32) - 22;

/// A clock, a spaced `A.`/`P.` (also `AM.`/`PM.`, any case), then digits and an optional date:
/// `10 A. 7/1/2025`, `10:00 P. 7/1/2025`, `10 A. 5`. Excel reads it as no clock it otherwise
/// knows: the digits right after the period are a fraction of a second (`.7`; rounded to the
/// millisecond half up, `.2025` is 0.203 s), a `/`, `-` or space after them starts a date that
/// must read on its own (`1/2025` is January 2025, `1/25` 25 January of the current year,
/// `Jul/2025`; none is day 0), a bare `N` is N minutes (below 60) and `H:M` hours and minutes,
/// and all of that sits on [`LETTER_PERIOD_BASE_SECONDS`], one second more for `P`. Typed, the
/// value takes `mm:ss.0`; VALUE and `0+` read it whole, DATEVALUE its day and TIMEVALUE its
/// fraction (probes/date-text-typed-clock-first-a-period, t3907-typed-clock-a-period-shapes,
/// t3907-typed-clock-a-period-edges, excel-graph 2026-11-03; docs/excel-semantics/dates.md
/// "Typed clock-first date-times"). `AM.` reads as `A.` and an `H:M` hour from 24 stays text
/// (probes/t3920-typed-a-period-unprobed-edges rows 1-14). Returns the date's 1900 serial (`None`
/// with no date) and the clock part.
fn letter_period_parts(
    text: &str,
    locale: &Locale,
    current_year: CurrentYear,
) -> Option<(Option<i32>, f64)> {
    let (clock, rest) = text.split_once(' ')?;
    let clock_seconds = match clock.split_once(':') {
        Some((hour, minute)) => {
            let (hour, minute) = (parse_clock_component(hour)?, parse_clock_component(minute)?);
            // `24:00 A. 7/1/2025`, `25:00 A. 5` and `10:60 A. 7/1/2025` stay text (t3920 rows 8-14).
            (hour < 24 && minute < 60).then_some(hour * 3600 + minute * 60)?
        }
        // `60 A. 7/1/2025` and `99 A. 7/1/2025` stay text (edges A10, A23).
        None => Some(parse_clock_component(clock)?)
            .filter(|minute| *minute < 60)?
            .checked_mul(60)?,
    };
    let (word, rest) = rest.trim_start_matches(' ').split_once('.')?;
    let is_pm = am_pm_word(word)?;
    let rest = rest.trim_start_matches(' ');
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    if digits == 0 {
        return None;
    }
    let (fraction, tail) = rest.split_at(digits);
    if let Some(after_colon) = tail.strip_prefix(':') {
        return colon_tail_parts(clock, is_pm, fraction, after_colon, locale, current_year);
    }
    let tail = tail.trim_end_matches(' ');
    let days = if tail.is_empty() {
        None
    } else {
        let date = tail.strip_prefix(['/', '-', ' '])?;
        Some(parse_date(trim_spaces(date), locale, current_year).ok()?.0)
    };
    let whole = LETTER_PERIOD_BASE_SECONDS + u64::from(is_pm) + u64::from(clock_seconds);
    // Below 2^53 ms, so exact, and divided as `parse_clock_text` divides.
    let total_ms = whole * 1000 + u64::from(fraction_digits_to_ms(fraction));
    Some((days, total_ms as f64 / 1000.0 / SECONDS_PER_DAY_F64))
}

/// `N A. F:G[ DATE]`, a colon after the letter-period digits: every unit moves up one place. The
/// word's constant counts in minutes (60 x (2^32 - 22) s for `A`, 60 s more for `P`), the clock `N`
/// in hours, and the digits `F` once as whole seconds and once as the fraction
/// ([`fraction_as_seconds`]); `G` is read and ignored. So `10 A. 5:30` is 60 x (2^32 - 22) s plus
/// 10 h and 5.5 s (2982616.579230324), `05:30` adds 5.05 s and `12:00` 12.12 s, and a date after a space
/// adds its serial (probes/t3957-letter-period-colon-tail rows 1-4, 6, 8, 13-18, 23, 24;
/// docs/excel-semantics/dates.md "Typed clock-first date-times"). These stay text: an `H:M` clock
/// or `N` from 24 (rows 5, 20, 21), a second colon or a fraction after `G` (rows 7, 22) and `F` of
/// three digits (row 19). `F` and `G` past two digits are unprobed beyond `100:30` and kept text.
fn colon_tail_parts(
    clock: &str,
    is_pm: bool,
    fraction: &str,
    after_colon: &str,
    locale: &Locale,
    current_year: CurrentYear,
) -> Option<(Option<i32>, f64)> {
    let hour =
        Some(parse_clock_component(clock)?).filter(|hour| *hour < 24 && !clock.contains(':'))?;
    let ignored = after_colon.bytes().take_while(u8::is_ascii_digit).count();
    if fraction.len() > 2 || !(1..=2).contains(&ignored) {
        return None;
    }
    let tail = after_colon[ignored..].trim_end_matches(' ');
    let days = if tail.is_empty() {
        None
    } else {
        let date = tail.strip_prefix(' ')?;
        Some(parse_date(trim_spaces(date), locale, current_year).ok()?.0)
    };
    let (seconds, ms) = fraction_as_seconds(fraction);
    let base = 60 * (LETTER_PERIOD_BASE_SECONDS + u64::from(is_pm)) + u64::from(hour) * 3600;
    // `seconds` is at most 99 (two digits), so the sum stays an exact integer below 2^53 ms.
    let total_ms = (base * 1000) as f64 + seconds * 1000.0 + f64::from(ms);
    Some((days, total_ms / 1000.0 / SECONDS_PER_DAY_F64))
}

/// Whether `word` is an `AM`/`A` (false) or `PM`/`P` (true) clock word, in any case.
pub(crate) fn am_pm_word(word: &str) -> Option<bool> {
    if word.eq_ignore_ascii_case("pm") || word.eq_ignore_ascii_case("p") {
        Some(true)
    } else if word.eq_ignore_ascii_case("am") || word.eq_ignore_ascii_case("a") {
        Some(false)
    } else {
        None
    }
}

/// Time text is trimmed and split on the space character only: a tab, a no-break space or an
/// ideographic space anywhere is #VALUE! (`"10:00\t"`, `"10:00\u{a0}AM"`, `"\u{3000}10:00"`:
/// probes/value-clock-text rows 38, 42-45).
pub(crate) fn trim_spaces(text: &str) -> &str {
    text.trim_matches(' ')
}

/// Largest value of one clock component: Excel takes at most four digits
/// (`TIMEVALUE("9999:59:59")` parses, `"10000:00:00"` is #VALUE!).
const MAX_CLOCK_COMPONENT: u32 = 9_999;

/// 2^53: below it every millisecond total is an exact `f64`.
const MAX_EXACT_MS: f64 = 9_007_199_254_740_992.0;

/// One `H`, `M` or `S` component of clock text: 1-4 ASCII digits, surrounding spaces allowed,
/// a leading zero only in a 2-digit component (`"10:05"` parses, `"010:11:12"` and `"10:011:12"`
/// are #VALUE!).
fn parse_clock_component(part: &str) -> Option<u32> {
    let part = trim_spaces(part);
    let digits_ok = (1..=4).contains(&part.len()) && part.bytes().all(|b| b.is_ascii_digit());
    if !digits_ok || (part.len() > 2 && part.starts_with('0')) {
        return None;
    }
    part.parse::<u32>()
        .ok()
        .filter(|v| *v <= MAX_CLOCK_COMPONENT)
}

/// Milliseconds of a fractional-seconds digit string (ASCII digits only), rounded half-up on the decimal digits:
/// `.1234` is 123 ms, `.4995` is 500 ms, `.9999` is 1000 ms (probes timevalue-fractional-seconds*).
fn fraction_digits_to_ms(digits: &str) -> u32 {
    let digit = |i: usize| digits.as_bytes().get(i).map_or(0, |b| u32::from(b - b'0'));
    let ms = digit(0) * 100 + digit(1) * 10 + digit(2);
    if digit(3) >= 5 {
        ms + 1
    } else {
        ms
    }
}

/// Whole seconds plus milliseconds of the `f` in `H:M.f:S` clock text: the digits are read twice,
/// once as whole seconds and once as the fraction (`"10:11.25:12"` is 10:11 plus 25.25 s,
/// `"10:11.0123:12"` plus 123.012 s; probes/clock-text-oddities*). The whole seconds are
/// accumulated digit by digit in an `f64` (`w * 10 + d`), as Excel does: past 18 digits that
/// differs from the correctly rounded value (probes/clock-text-long-fraction2 B4, B5).
fn fraction_as_seconds(digits: &str) -> (f64, u32) {
    let whole = digits
        .bytes()
        .fold(0.0, |w, b| w * 10.0 + f64::from(b - b'0'));
    (whole, fraction_digits_to_ms(digits))
}

/// Whether at most one clock component is out of its usual range (hours from 24, minutes or
/// whole seconds from 60, before millisecond rounding): `"10:61"`, `"0:24:60"`, `"24:59:59"`,
/// `"0:59:9999.9999"` and `"0:11.9999:1"` parse, while `"24:60"`, `"0:60:60"`, `"24:0:60"`,
/// `"60:60.5"` and `"0:9998.60:1"` are #VALUE! (probes/clock-text-oddities4-7).
fn at_most_one_out_of_range(hour: u32, minute: u32, second: f64) -> bool {
    [hour >= 24, minute >= 60, second >= 60.0]
        .iter()
        .filter(|out| **out)
        .count()
        <= 1
}

/// Serial value of clock text in Excel's en-US grammar, before any wrap to one day:
/// `H:M`, `H:M:S[.f]`, `M:S.f` (two components with a fraction are minutes and seconds) or the
/// odd `H:M.f:S` (see [`fraction_as_seconds`]; `S` is checked and then ignored), spaces allowed
/// around `:` and `.`, one trailing `:` ignored (so `H:` alone is an hour), components up to
/// 9999 and at most one of them out of its usual range (see [`at_most_one_out_of_range`]), an
/// optional ` AM`/` PM`/` A`/` P` suffix after whitespace (then the hour is at most 12 and
/// minutes and whole seconds at most 59). The value is
/// `(total_ms / 1000) / 86400`, which matches Excel bit for bit (`"10:11:12.5"` is
/// 0.4244502314814815). Excel: see docs/excel-semantics/dates.md#time-text.
pub(crate) fn parse_clock_text(text: &str) -> Option<f64> {
    let text = trim_spaces(text);
    let (body, suffix_is_pm) = match text.rsplit_once(' ') {
        Some((body, word)) if ["am", "a"].iter().any(|s| word.eq_ignore_ascii_case(s)) => {
            (body, Some(false))
        }
        Some((body, word)) if ["pm", "p"].iter().any(|s| word.eq_ignore_ascii_case(s)) => {
            (body, Some(true))
        }
        _ => (text, None),
    };
    // `"10:11:"` is 10:11 and `"10:11:12.5:"` is 10:11:12.5, but `"10:11.5::"` is #VALUE!
    // (probes/clock-text-oddities3 A11-A14).
    let body = trim_spaces(body);
    let (body, bare_hour) = match body.strip_suffix(':') {
        // `"12:"` is 12:00 (`"25:"` is 25 hours) without an AM/PM suffix; `"12: PM"` is #VALUE!
        // (probes/t2352-time-text-trailing-colon A1-A5, A16).
        // `H:M:` before an AM/PM suffix is #VALUE! too (`"10:30: PM"`, `"10:30:45: PM"`:
        // probes/t4602-clock-text-colon-am-pm-and-spaces A1-A25).
        Some(_) if suffix_is_pm.is_some() => return None,
        Some(stripped) => (stripped, true),
        None => (body, false),
    };
    let (head, fraction, odd_tail) = match body.split_once('.') {
        Some((head, rest)) => {
            let (fraction, odd_tail) = match rest.split_once(':') {
                Some((fraction, tail)) => (fraction, Some(tail)),
                None => (rest, None),
            };
            let fraction = trim_spaces(fraction);
            if !fraction.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            (head, Some(fraction), odd_tail)
        }
        None => (body, None, None),
    };
    let parts = head
        .split(':')
        .map(parse_clock_component)
        .collect::<Option<Vec<u32>>>()?;
    let has_fraction_digits = fraction.is_some_and(|f| !f.is_empty());
    let ((hour, minute), (second, ms)) = match (parts.as_slice(), fraction, odd_tail) {
        ([h, m], Some(f), Some(tail)) if has_fraction_digits => {
            parse_clock_component(tail)?;
            ((*h, *m), fraction_as_seconds(f))
        }
        (_, _, Some(_)) => return None,
        ([h, m, s], ..) => (
            (*h, *m),
            (f64::from(*s), fraction.map_or(0, fraction_digits_to_ms)),
        ),
        ([m, s], Some(f), None) if has_fraction_digits => {
            ((0, *m), (f64::from(*s), fraction_digits_to_ms(f)))
        }
        ([h, m], ..) => ((*h, *m), (0.0, 0)),
        ([h], None, None) if bare_hour => ((*h, 0), (0.0, 0)),
        _ => return None,
    };
    if !at_most_one_out_of_range(hour, minute, second) {
        return None;
    }
    let hour = match suffix_is_pm {
        Some(is_pm) => {
            if hour > 12 || minute > 59 || second > 59.0 {
                return None;
            }
            hour_from_12h(hour, is_pm)
        }
        None => hour,
    };
    let clock_seconds = f64::from(hour * 3600 + minute * 60);
    let total_ms = clock_seconds * 1000.0 + second * 1000.0 + f64::from(ms);
    // Exact (and bit for bit Excel's) while below 2^53 ms: at most 9999 h, 9999 min and 10^12 s.
    // A longer `H:M.f:S` fraction makes `second` a rounded double and Excel adds the parts as
    // `(clock + whole) + ms / 1000` (probes/clock-text-long-fraction2 B2-B5).
    let seconds = if total_ms < MAX_EXACT_MS {
        total_ms / 1000.0
    } else {
        (clock_seconds + second) + f64::from(ms) / 1000.0
    };
    Some(seconds / SECONDS_PER_DAY_F64)
}

/// 24-hour value of a 12-hour clock hour (0-12): PM adds 12 to hours below 12 (`"0:11:12.5 PM"`
/// is 12:11:12.5, probes/timevalue-component-limits B16), AM maps 12 to 0.
fn hour_from_12h(hour: u32, is_pm: bool) -> u32 {
    match (is_pm, hour) {
        (false, 12) => 0,
        (true, h) if h < 12 => h + 12,
        (_, h) => h,
    }
}

/// Splits hour-only "N PM" / "N AM" text into the hour digits and whether it is PM. The suffix is
/// `AM`, `PM`, `A` or `P` in any case with an optional trailing `.` (`"6 P."`, `"6 a"`:
/// probes/value-clock-text rows 7, 25, 29-33).
fn parse_simple_am_pm(text: &str) -> Option<(&str, bool)> {
    let (hour_part, suffix) = text.rsplit_once(' ')?;

    let suffix = suffix.strip_suffix('.').unwrap_or(suffix);

    let is_pm = am_pm_word(suffix)?;

    let hour_part = trim_spaces(hour_part);

    if !hour_part.is_empty() && hour_part.chars().all(|c| c.is_ascii_digit()) {
        return Some((hour_part, is_pm));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{letter_period_parts, parse_date_clock_serial, parse_date_clock_serial_in};
    use crate::{locale::get_locale, types::DateSystem};

    fn serial(text: &str) -> Option<f64> {
        let locale = get_locale("en").ok()?;
        parse_date_clock_serial(text, locale, &|| Some(2026))
    }

    #[test]
    fn letter_period_text_sits_on_two_to_the_32_seconds() {
        // probes/t3907-typed-clock-a-period-shapes, -edges and date-text-typed-clock-first-a-period
        // (excel-graph 2026-11-03), compared at Excel's 15 significant digits.
        let cases = [
            ("10 A. 7/1/2025", 95_368.276_327_546_3),
            ("10:00 A. 7/1/2025", 95_368.686_049_768_53),
            ("10 P. 7/1/2025", 95_368.276_339_120_4),
            ("10 a. 7/1/2025", 95_368.276_327_546_3),
            ("10 PM. 7/1/2025", 95_368.276_339_120_4),
            ("0 A. 7/1/2025", 95_368.269_383_101_8),
            ("1:2 A. 7/1/2025", 95_368.312_438_657_4),
            ("10 A. 1/1/1900", 49_711.276_320_601_9),
            ("10 A. 5", 49_710.276_325_231_5),
            ("10 A. 2025-07-01", 95_914.276_321_794),
            ("10 A. 123456789/1/2025", 95_368.276_320_868_1),
            ("10 A. 7 1/1/2025", 95_368.276_327_546_3),
            ("10 A.7/1/2025", 95_368.276_327_546_3),
            ("10  A.  7/Jul/2025", 95_549.276_327_546_3),
            // `AM.` reads as `A.` (probes/t3920-typed-a-period-unprobed-edges rows 1-6).
            ("10 AM. 7/1/2025", 95_368.276_327_546_3),
            ("10:00 AM. 7/1/2025", 95_368.686_049_768_53),
            ("10 am. 7/1/2025", 95_368.276_327_546_3),
            ("10 AM. 5", 49_710.276_325_231_48),
            ("23:59 A. 7/1/2025", 95_369.268_688_657_41),
        ];
        for (text, expected) in cases {
            let got = serial(text).map(|v| format!("{v:.10}"));
            assert_eq!(got, Some(format!("{expected:.10}")), "{text}");
        }
    }

    /// Typed, these stay text (edges and shapes cases); the shapes whose period is followed by a
    /// digit are #VALUE! to the formula readers too, with no fallback to `10 A.` and a date
    /// (`letter_period_shapes_have_no_other_reading`).
    #[test]
    fn letter_period_text_needs_digits_a_date_tail_and_minutes_below_60() {
        let locale = get_locale("en").ok();
        for text in [
            "10 A.. 7/1/2025",
            "10 a.m. 7/1/2025",
            "A. 7/1/2025",
            "10 B. 7/1/2025",
            "10:00:30 A. 7/1/2025",
            "10 A. 7/1",
            "10 A. 7-2025",
            "10 A. 7/13/2025",
            "10 A. 7/0/2025",
            "10 A. 7/1/1899",
            "10 A. 7/1/2025 10:00",
            "60 A. 7/1/2025",
            "10 A. July 1, 2025",
            "24:00 A. 7/1/2025",
            "25:00 A. 5",
            "10:60 A. 7/1/2025",
        ] {
            let parts = locale.and_then(|l| letter_period_parts(text, l, &|| Some(2026)));
            assert_eq!(parts, None, "{text}");
        }
    }

    /// probes/t3920-typed-a-period-unprobed-edges rows 8-23 (excel-graph 2026-11-03): VALUE, `0+`,
    /// DATEVALUE and TIMEVALUE are #VALUE! on each, so the date-clock reader is `None`.
    #[test]
    fn letter_period_shapes_have_no_other_reading() {
        for text in [
            "24:00 A. 7/1/2025",
            "25:00 A. 7/1/2025",
            "100:00 A. 7/1/2025",
            "10 A. 7-2025",
            "60 A. 7/1/2025",
            "10 A. 5/1",
            "10:00:30 A. 7/1/2025",
            "10 A. 7/1/2025 10:00",
            "10 A. 7/13/2025",
            "10 A. 7/1",
            // probes/t3948-letter-period-gate-edges rows 1, 8, 13, 16.
            "1:2:3:4 A. 5",
            "10 A. 5x",
            "10 A. July 1, 2025",
            "10 A. Jul-2025",
        ] {
            assert_eq!(serial(text), None, "{text}");
        }
    }

    /// probes/t3920-typed-a-period-1904 A1-A5 (excel-graph 2026-11-03).
    #[test]
    fn letter_period_text_in_a_1904_workbook() {
        let serial_1904 = |text: &str| {
            let locale = get_locale("en").ok()?;
            parse_date_clock_serial_in(text, locale, &|| Some(2026), DateSystem::Excel1904)
                .map(|v| format!("{v:.10}"))
        };
        let fixed = |v: f64| Some(format!("{v:.10}"));
        assert_eq!(serial_1904("10 A. 5"), fixed(49_710.276_325_231_48));
        assert_eq!(serial_1904("10 A. 7/1/2025"), fixed(93_906.276_327_546_3));
        assert_eq!(
            serial_1904("10:00 P. 7/1/2025"),
            fixed(93_906.686_061_342_6)
        );
        assert_eq!(serial_1904("10 A. 1/1904"), None);
        assert_eq!(serial_1904("10 A. 12/1903"), None);
    }
}
