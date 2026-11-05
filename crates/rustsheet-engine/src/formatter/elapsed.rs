//! Elapsed `[h]`/`[m]`/`[s]` token text and fractional seconds (`s.000`, `[s].000`).

use super::date_clock::SECONDS_PER_DAY_F64;
use super::parser::TextToken;
use crate::constants::MAXIMUM_DATE_SERIAL_NUMBER;

/// The unit of an elapsed `[h]`/`[m]`/`[s]` token (or of a clock `h`/`m`/`s`), smallest first so
/// the derived order ranks them.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum ElapsedUnit {
    Second,
    Minute,
    Hour,
}

/// A non-negative serial counted in the shown fraction of a second (`.0`, `.00`, `.000`), so the
/// clock, elapsed and fraction tokens agree: `[h]:m:s.000` on 3.14159 is "75:23:53.376",
/// `[hh]:mm:ss.0` "75:23:53.4", `[s].000` "271433.376" (poi/ElapsedFormatTests A2-A7,
/// number-formats.md "Elapsed-time tokens"). With fraction digits every token truncates the
/// half-step register ([`half_step`]), whose float error decides a tie:
/// `TEXT(1+0.25/86400,"[h]:mm.0")` is "24:00.2" (probes/t4323-fraction-ties, -carry-ties,
/// number-formats.md "Fraction digits of a second at a tie"). Whole seconds round the value.
pub(super) struct FractionalSeconds {
    /// The seconds the elapsed counts truncate: the value rounded to whole seconds, or with
    /// fraction digits the half-step register's seconds, unfloored.
    elapsed_seconds: f64,
    /// The whole days the clock and date read.
    days: f64,
    /// The units within that day the clock and the fraction digits read, whole.
    day_units: f64,
    scale: f64,
    digits: usize,
}

impl FractionalSeconds {
    /// `None` for more digits than a format may show. The caller has rejected serials past
    /// 9999-12-31, so the units stay far below 2^53 and count exactly.
    pub(super) fn new(value: f64, digits: usize) -> Option<Self> {
        let scale = [1.0, 10.0, 100.0, 1000.0].get(digits).copied()?;
        let (elapsed_seconds, days, day_units) = if digits == 0 {
            // Whole seconds keep the rounded value: only fraction digits have the tie evidence.
            let units = (value * SECONDS_PER_DAY_F64).round();
            (
                units,
                (units / SECONDS_PER_DAY_F64).floor(),
                units % SECONDS_PER_DAY_F64,
            )
        } else {
            // The counts read the same register as the clock, so a tie just short of a whole
            // minute or hour carries in neither: `TEXT(1+3599.95/86400,"[h]:mm:ss.0")` is
            // "24:59:59.9", `TEXT(2+3599.95/86400,...)` "49:59:59.9" (probes/t4323-carry-ties).
            let (days, day_units) = half_step_split(value, scale);
            (
                half_step(value, scale) * SECONDS_PER_DAY_F64,
                days,
                day_units.floor(),
            )
        };
        Some(Self {
            elapsed_seconds,
            days,
            day_units,
            scale,
            digits,
        })
    }

    /// Hours, minutes and seconds of the time of day after rounding.
    pub(super) fn clock(&self) -> (f64, f64, f64) {
        let seconds = (self.day_units / self.scale).floor();
        (
            (seconds / 3600.0).floor(),
            (seconds % 3600.0 / 60.0).floor(),
            seconds % 60.0,
        )
    }

    /// The whole days after rounding: a time just short of midnight rolls into the next date.
    pub(super) fn days(&self) -> i64 {
        self.days as i64
    }

    /// The whole elapsed count of `unit` after rounding.
    fn elapsed(&self, unit: ElapsedUnit) -> f64 {
        let seconds = self.elapsed_seconds;
        match unit {
            ElapsedUnit::Hour => (seconds / 3600.0).floor(),
            ElapsedUnit::Minute => (seconds / 60.0).floor(),
            ElapsedUnit::Second => seconds.floor(),
        }
    }

    /// The fraction digits, zero-padded (`376`, `4`, `00`).
    pub(super) fn digits_text(&self) -> String {
        format!(
            "{:0>width$.0}",
            self.day_units % self.scale,
            width = self.digits
        )
    }
}

/// The clock, day and elapsed-count inputs of one date format, all from the serial rounded to the
/// shown second or fraction digits (`0.9999999999` is "0:00" under `h:mm` and "1900-01-01" under
/// `yyyy-mm-dd`; gen-m10, T-1732).
pub(super) struct DisplayClock {
    /// Hours, minutes and seconds of the time of day.
    pub(super) hms: (f64, f64, f64),
    /// Whole days after rounding, so a time just short of midnight rolls into the next date.
    pub(super) days: i64,
    /// What the elapsed tokens count from, rounded like the clock whether or not seconds are
    /// shown: `[h]` on 0.4 s short of 3 h is "3", on 0.6 s short "2" (probes/elapsed-no-seconds-boundary,
    /// T-2385). `None` past 9999-12-31.
    pub(super) elapsed: Option<FractionalSeconds>,
}

impl DisplayClock {
    /// `value` is a non-negative serial. Past 9999-12-31 only a lone elapsed token formats and
    /// nothing is rounded here.
    pub(super) fn new(value: f64, fraction_digits: Option<usize>) -> Self {
        let digits = match fraction_digits {
            Some(digits) => Some(digits),
            None if value < f64::from(MAXIMUM_DATE_SERIAL_NUMBER) + 1.0 => Some(0),
            None => None,
        };
        let rounded = digits.and_then(|digits| FractionalSeconds::new(value, digits));
        let Some(rounded) = rounded else {
            let time_fract = value.fract();
            let hours = (time_fract * 24.0).floor();
            let minutes = ((time_fract * 24.0 - hours) * 60.0).floor();
            let seconds = ((((time_fract * 24.0 - hours) * 60.0) - minutes) * 60.0).round();
            return Self {
                hms: (hours, minutes, seconds),
                days: value as i64,
                elapsed: None,
            };
        };
        Self {
            hms: rounded.clock(),
            days: rounded.days(),
            elapsed: Some(rounded),
        }
    }
}

/// Whether the token is an elapsed `[h]`, `[m]` or `[s]` count, padded or not.
pub(super) fn is_elapsed(token: &TextToken) -> bool {
    matches!(
        token,
        TextToken::ElapsedHour
            | TextToken::ElapsedHourPadded
            | TextToken::ElapsedMinute
            | TextToken::ElapsedMinutePadded
            | TextToken::ElapsedSecond
            | TextToken::ElapsedSecondPadded
    )
}

/// The unit and padding of an elapsed `[h]`, `[m]` or `[s]` token, `None` for any other token.
pub(super) fn elapsed_token(token: &TextToken) -> Option<(ElapsedUnit, bool)> {
    match token {
        TextToken::ElapsedHour => Some((ElapsedUnit::Hour, false)),
        TextToken::ElapsedHourPadded => Some((ElapsedUnit::Hour, true)),
        TextToken::ElapsedMinute => Some((ElapsedUnit::Minute, false)),
        TextToken::ElapsedMinutePadded => Some((ElapsedUnit::Minute, true)),
        TextToken::ElapsedSecond => Some((ElapsedUnit::Second, false)),
        TextToken::ElapsedSecondPadded => Some((ElapsedUnit::Second, true)),
        _ => None,
    }
}

/// The unit of a clock `h`, `m` (as minutes) or `s` token.
fn clock_unit(token: &TextToken) -> Option<ElapsedUnit> {
    match token {
        TextToken::Hour | TextToken::HourPadded => Some(ElapsedUnit::Hour),
        TextToken::Minute | TextToken::MinutePadded => Some(ElapsedUnit::Minute),
        TextToken::Second | TextToken::SecondPadded => Some(ElapsedUnit::Second),
        _ => None,
    }
}

/// Whether a `.0` right after `token` is the fraction of the second: after any hour, minute or
/// second token, clock or elapsed (probes/t3575-fraction-after-elapsed A1-A12).
pub(super) fn takes_second_fraction(token: &TextToken) -> bool {
    clock_unit(token).is_some() || elapsed_token(token).is_some()
}

/// Whether an elapsed token stands after a clock token of its own unit or a larger one, which
/// makes the section `#VALUE!` for any value: `h [h]`, `h [m]`, `h [s]`, `h:mm [h]`, `mm:ss [m]`,
/// `mm:ss [s]`, `s [s]`, `s.0 [s]`, `h d [h]`. A smaller clock unit first is valid (`s [h]`,
/// `ss [m]`, `mm:ss [h]`), and so is the elapsed token first (`[h] h`, where the `h` counts too).
/// Evidence: probes/t3419-elapsed-after-date-general A5, A23; t3419-elapsed-after-date-b A1-A3,
/// A7, A8, A15, A23; t3419-elapsed-after-clock-c (excel-graph, 2026-10-31, T-3419).
pub(super) fn elapsed_after_larger_clock(tokens: &[TextToken]) -> bool {
    let mut largest_clock = None;
    for token in tokens {
        if let Some(unit) = clock_unit(token) {
            largest_clock = largest_clock.max(Some(unit));
        } else if let Some((unit, _)) = elapsed_token(token) {
            if largest_clock >= Some(unit) {
                return true;
            }
        }
    }
    false
}

/// Excel moves the elapsed count off `[s]`/`[ss]` onto the first hour or minute token after it:
/// `[s] m` prints like `s [m]`, the seconds field then the elapsed minutes counted from the date
/// section's register ([`date_section_register`], which a clock token in front selects). So
/// `TEXT(1.7/24+0.3/1440,"[s] m")` is "18 26640", `"[s] h"` "18 444", `"[s] m s"` "18 26640 18".
/// The rewrite also drops the elapsed flag the parser gave a later `s`. Evidence:
/// probes/t4295-elapsed-with-month A1:D9, K1:K9 (excel-graph, 2026-11-05, T-4295); `[ss]`, and a
/// second hour or minute token after the first, are UNVERIFIED (not probed). A clock token before
/// the `[s]` leaves the section as parsed, so [`elapsed_after_larger_clock`] keeps `s [s] m` and
/// `h [s] m` `#VALUE!` like `s [s]` (UNVERIFIED with the `m` after; not probed).
pub(super) fn move_elapsed_second(tokens: &mut [TextToken]) {
    let Some(start) = tokens
        .iter()
        .position(|t| matches!(t, TextToken::ElapsedSecond | TextToken::ElapsedSecondPadded))
    else {
        return;
    };
    if tokens[..start].iter().any(|t| clock_unit(t).is_some()) {
        return;
    }
    let Some(target) = tokens[start..].iter().position(|t| {
        matches!(
            t,
            TextToken::Hour | TextToken::HourPadded | TextToken::Minute | TextToken::MinutePadded
        )
    }) else {
        return;
    };
    for token in tokens.iter_mut() {
        *token = match token {
            TextToken::ElapsedSecond => TextToken::Second,
            TextToken::ElapsedSecondPadded => TextToken::SecondPadded,
            _ => continue,
        };
    }
    let moved = &mut tokens[start + target];
    *moved = match moved {
        TextToken::Hour => TextToken::ElapsedHour,
        TextToken::HourPadded => TextToken::ElapsedHourPadded,
        TextToken::Minute => TextToken::ElapsedMinute,
        _ => TextToken::ElapsedMinutePadded,
    };
}

/// Whether the token reads the serial's date or time of day (`d`, `mmm`, `h`, `ss`, `AM/PM`, an
/// era): anything in a date section but literals, `General`, elapsed counts and fraction digits.
pub(super) fn is_date_or_clock(token: &TextToken) -> bool {
    !(token.is_literal()
        || is_elapsed(token)
        || matches!(
            token,
            TextToken::Raw
                | TextToken::General
                | TextToken::Digit(_)
                | TextToken::Period
                | TextToken::SecondFraction(_)
        ))
}

/// Excel's one value register in a date section: what `General` prints and what an elapsed token
/// after a date or clock token counts from (the parser lets one `General` in; a digit placeholder
/// beside it still makes the section a number one and `#VALUE!`). With elapsed tokens only and no
/// fraction digits it is the value's magnitude: `TEXT(5.25,"[h] General")` is "126 5.25". Once a
/// date or clock token or fraction digits split the serial, wherever they stand, it is the last
/// component of the serial after Excel's half-unit rounding step (`x + 0.5/86400` with no fraction
/// digits, `x + 0.5/864000` with `.0`), with that step's float error left in: without fraction
/// digits the seconds within the minute (`TEXT(5,"d General")` is "5 0.5",
/// `TEXT(45000,"General d")` "0.499999803 15", `TEXT(12345.678,"d General")` "18 19.69999995"),
/// with them the shown fraction in its own units (`TEXT(12345.678,"s.0 General")` is
/// "19.2 2.500000331", `TEXT(0.99999,"s.00 General")` "59.14 14.1", `TEXT(5,"[s].0 General")`
/// "432000.0 0.5"). `value` is non-negative. Evidence: probes/t3378-general-in-date-section,
/// probes/t3419-elapsed-after-date-general A12, A15, A16 (excel-graph, 2026-10-31, T-3378, T-3419);
/// number-formats.md "`General` in a date section".
pub(super) fn date_section_register(
    value: f64,
    tokens: &[TextToken],
    fraction_digits: Option<usize>,
) -> f64 {
    if fraction_digits.is_none() && !tokens.iter().any(is_date_or_clock) {
        return value;
    }
    let scale = 10f64.powi(i32::try_from(fraction_digits.unwrap_or(0)).unwrap_or(0));
    // The parser only makes `.0`, `.00` and `.000`, so with fraction digits `scale` is 10 or more.
    let modulus = if fraction_digits.is_some() {
        scale
    } else {
        60.0
    };
    half_step_split(value, scale).1 % modulus
}

/// The serial moved up half a unit of `1/scale` second: `value + 0.5/(86400*scale)`.
fn half_step(value: f64, scale: f64) -> f64 {
    value + 0.5 / (SECONDS_PER_DAY_F64 * scale)
}

/// Excel's half-unit rounding step on a non-negative serial: `y = value + 0.5/(86400*scale)`,
/// split into its whole days and the units of `1/scale` second within the day, `frac(y)*86400*
/// scale`, unfloored and with the step's float error left in. The `General` register
/// ([`date_section_register`]) prints those units; the clock and the fraction digits of a second
/// truncate them, so at a tie the float error of `frac(y)` decides: `1+0.25/86400` under
/// `[h]:mm.0` gives 2.99999999997 units and shows ".2" (probes/t4323-fraction-ties: all 100
/// cells with a serial of 1 or more, excel-graph 2026-11-05, T-4323).
fn half_step_split(value: f64, scale: f64) -> (f64, f64) {
    let shifted = half_step(value, scale);
    let days = shifted.floor();
    (days, (shifted - days) * SECONDS_PER_DAY_F64 * scale)
}

/// `x` rounded to `1/scale`; a magnitude whose scaled value overflows is already integral, and
/// Excel does not turn it into 0 (probes/elapsed-huge-h-1900: 2.08E303 days in `[s]` is #VALUE!).
fn snap(x: f64, scale: f64) -> f64 {
    let scaled = x * scale;
    if scaled.is_finite() {
        scaled.round() / scale
    } else {
        x
    }
}

/// Elapsed `[h]`/`[m]`/`[s]` count for a non-negative serial `value` (a negative one is formatted
/// as its magnitude after a leading minus, see the `ParsePart::Date` arm). `[s]` rounds to nearest
/// (1.7 s is "2", 0.9 s of a negative is "-1") and `[h]`/`[m]` truncate (1.7 min is "1", 0.3 h of
/// a negative is "-0"), in both date systems (probes/elapsed-tiny-negatives-1900,
/// elapsed-fractions-1900, T-1338, T-1346). Noise is absorbed first: `[s]` snaps the seconds at
/// 1e-12 (1.5/86400 days is "2" although it is 1.4999999999999998 s, yet 0.49999999999 s stays
/// "0"), `[h]`/`[m]` snap them to the millisecond ((3-1E-7) h is "3"). Those snaps are fitted to
/// the probes, not documented. Excel has no 32-bit limit (T-1362) and no i64 one (T-1375).
/// Up to 9999-12-31 the count comes from `fraction` instead, the serial rounded to the shown
/// second (T-1732, T-2385), so the snaps only serve a lone token past it.
pub(super) fn elapsed_text(
    value: f64,
    padded: bool,
    unit: ElapsedUnit,
    fraction: Option<&FractionalSeconds>,
) -> String {
    let secs = value * 86400.0;
    let n = match (fraction, unit) {
        (Some(fraction), unit) => fraction.elapsed(unit),
        (None, ElapsedUnit::Second) => snap(secs, 1e12).round(),
        // Excel counts minutes and hours from the serial itself once the seconds overflow.
        (None, ElapsedUnit::Minute) if secs.is_infinite() => (value * 1440.0).floor(),
        (None, ElapsedUnit::Hour) if secs.is_infinite() => (value * 24.0).floor(),
        (None, ElapsedUnit::Minute) => (snap(secs, 1e3) / 60.0).floor(),
        (None, ElapsedUnit::Hour) => (snap(secs, 1e3) / 3600.0).floor(),
    };
    // Excel: a count that overflows f64 prints as 0, and from 1e15 up the digits are the first 15
    // significant ones followed by zeros (docs/excel-semantics/number-formats.md, "Elapsed tokens
    // beyond the i64 range"; probes/elapsed-huge-*).
    let (sign, mag) = if n.is_sign_negative() {
        ("-", -n)
    } else {
        ("", n)
    };
    let digits = count_digits(mag);
    if padded {
        format!("{sign}{digits:0>2}")
    } else {
        format!("{sign}{digits}")
    }
}

/// Digits of a non-negative integral count: exact below 1e15, from there the first 15 significant
/// digits then zeros, and "0" for a count that overflowed f64 (probes/elapsed-huge-*).
fn count_digits(mag: f64) -> String {
    if !mag.is_finite() {
        "0".to_owned()
    } else if mag < 1e15 {
        format!("{mag:.0}")
    } else {
        let sci = format!("{mag:.14e}");
        let (mantissa, exp) = sci.split_once('e').unwrap_or((&sci, "0"));
        let exp: usize = exp.parse().unwrap_or(0);
        let mut d: String = mantissa.chars().filter(char::is_ascii_digit).collect();
        d.extend(std::iter::repeat_n('0', exp.saturating_sub(14)));
        d
    }
}

#[cfg(test)]
mod tests {
    use super::{
        elapsed_text,
        ElapsedUnit::{Hour, Minute, Second},
    };

    #[test]
    fn huge_counts_keep_fifteen_significant_digits() {
        assert_eq!(
            elapsed_text(1e15, false, Second, None),
            "86400000000000000000"
        );
        assert_eq!(
            elapsed_text(1e100, false, Hour, None),
            format!("24{}", "0".repeat(100))
        );
        assert_eq!(elapsed_text(1e10, false, Second, None), "864000000000000");
    }

    #[test]
    fn overflowed_counts_are_zero() {
        assert_eq!(elapsed_text(2.0807e303, false, Second, None), "0");
        assert_eq!(elapsed_text(1.5e307, false, Hour, None), "0");
        assert_eq!(elapsed_text(1.5e307, false, Minute, None), "0");
    }

    #[test]
    fn finite_huge_counts_do_not_collapse_to_zero() {
        assert!(elapsed_text(2.08e303, false, Second, None).len() > 300);
        assert!(elapsed_text(1e305, false, Minute, None).len() > 300);
    }
}
