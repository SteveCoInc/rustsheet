//! Elapsed `[h]`/`[m]`/`[s]` token text and fractional seconds (`s.000`, `[s].000`).
// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::cast_possible_truncation)]
// LINT-DEBT-END

use super::date_clock::SECONDS_PER_DAY_F64;
use super::parser::TextToken;
use crate::constants::MAXIMUM_DATE_SERIAL_NUMBER;
use crate::number_format::significant_15;

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
/// half-step serial ([`half_step`]), the clock cut field by field ([`field_split`]) and the
/// fraction digits read from the remainder of a second at 15 significant digits
/// ([`fraction_digits`]), so float error decides a tie: `TEXT(1+0.25/86400,"[h]:mm.0")` is
/// "24:00.2" (probes/t4323-fraction-ties, -carry-ties, t4332-fraction-fresh,
/// t4370-fraction-fresh, number-formats.md "Fraction digits of a second at a tie"). Whole seconds
/// round the value.
pub(super) struct FractionalSeconds {
    /// What the elapsed counts truncate: the value rounded to whole seconds, or with fraction
    /// digits the half-step serial `y` itself ([`ElapsedBase`]).
    elapsed_base: ElapsedBase,
    /// The whole days the clock and date read.
    days: f64,
    /// The units within that day the clock reads, whole.
    day_units: f64,
    /// The shown fraction digits as one number (`376` for `.376`), below `scale`.
    fraction: f64,
    scale: f64,
    digits: usize,
}

impl FractionalSeconds {
    /// `None` for more digits than a format may show. The caller has rejected serials past
    /// 9999-12-31, so the units stay far below 2^53 and count exactly.
    pub(super) fn new(value: f64, digits: usize) -> Option<Self> {
        let scale = [1.0, 10.0, 100.0, 1000.0].get(digits).copied()?;
        let (elapsed_base, days, day_units, fraction) = if digits == 0 {
            // Whole seconds keep the rounded value: only fraction digits have the tie evidence.
            let units = (value * SECONDS_PER_DAY_F64).round();
            (
                ElapsedBase::Seconds(units),
                (units / SECONDS_PER_DAY_F64).floor(),
                units % SECONDS_PER_DAY_F64,
                0.0,
            )
        } else {
            // The counts read the half-step serial on their own chain of products, apart from
            // the clock and the fraction digits ([`ElapsedBase::count`]), so each unit carries or
            // not on its own: `TEXT(1+3599.95/86400,"[h]:mm:ss.0")` is "24:59:59.9" and
            // `TEXT(45000+59.95/86400,"[s].0")` "3888000060.9" (probes/t4323-carry-ties,
            // t4466-count-hypotheses B18).
            let (days, day_units, second_rest) = field_split(value, scale);
            (
                ElapsedBase::HalfStep(half_step(value, scale)),
                days,
                day_units,
                fraction_digits(second_rest, digits),
            )
        };
        Some(Self {
            elapsed_base,
            days,
            day_units,
            fraction,
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
        self.elapsed_base.count(unit)
    }

    /// The fraction digits, zero-padded (`376`, `4`, `00`).
    pub(super) fn digits_text(&self) -> String {
        format!("{:0>width$.0}", self.fraction, width = self.digits)
    }
}

/// What an elapsed `[h]`/`[m]`/`[s]` count is read from.
#[derive(Clone, Copy)]
enum ElapsedBase {
    /// The value rounded to whole seconds (no fraction digits shown).
    Seconds(f64),
    /// The half-step serial `y = value + 0.5/(86400*scale)` ([`half_step`]), with fraction digits.
    HalfStep(f64),
}

impl ElapsedBase {
    /// The whole count of `unit`. With fraction digits Excel multiplies the half-step serial by
    /// 60, 60 and 24 in that order, each product rounded to a double, keeping as many factors as
    /// the unit needs, and truncates: `[h]` is `floor(y*24)`, `[m]` `floor((y*60)*24)` and `[s]`
    /// `floor(((y*60)*60)*24)`. That order's rounding decides a tie, so a count is not a function
    /// of `y*86400`, and the count and the shown fraction (cut apart, [`field_split`]) carry on
    /// their own: `TEXT(3733.95/86400+1*2^-57,"[s].0")` is "3733.0" while `+2*2^-57` is "3734.0",
    /// and `TEXT(60000+82799.95/86400,"[h]:mm:ss.0")` carries the hour alone (probes/
    /// t4466-count-hypotheses B29, B30, F33). It matches all 2,617 elapsed-count cells with fraction
    /// digits of the t4323, t4332, t4333, t4370, t4389 and t4466 probes, the hold-out and the fresh
    /// sweep probes/t4466-count-fresh drawn before it was found included; no constant is fitted.
    /// docs/excel-semantics/number-formats.md "Elapsed counts beside fraction digits".
    fn count(self, unit: ElapsedUnit) -> f64 {
        match self {
            Self::Seconds(seconds) => match unit {
                ElapsedUnit::Hour => (seconds / 3600.0).floor(),
                ElapsedUnit::Minute => (seconds / 60.0).floor(),
                ElapsedUnit::Second => seconds.floor(),
            },
            Self::HalfStep(y) => match unit {
                ElapsedUnit::Hour => (y * 24.0).floor(),
                ElapsedUnit::Minute => (y * 60.0 * 24.0).floor(),
                ElapsedUnit::Second => (y * 60.0 * 60.0 * 24.0).floor(),
            },
        }
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
/// The rewrite also drops the elapsed flag the parser gave a later `s`. The first hour token takes
/// the count even behind a minute token (`[s] m h` is "18 42 444"), else the first minute token
/// does. Evidence: probes/t4295-elapsed-with-month A1:D9, K1:K9 (excel-graph, 2026-11-05, T-4295);
/// probes/t4341-elapsed-second-forms (excel-graph, 2026-11-06, T-4341). A clock token before the
/// `[s]` stops the rewrite, and [`elapsed_after_larger_clock`] keeps `s [s] m` `#VALUE!` like
/// `s [s]`; a date token does not stop it (`d [s] m`); `[h] [s] m` and `[m] [s] h` are `#VALUE!`
/// through the parser's two-bracketed-elapsed rule.
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
    // The first hour token after `[s]` takes the count even behind a minute token, else the first
    // minute token does (`[s] m h` is "18 42 444", `[s] m m` "18 26640 1"; probes/t4341).
    let after = &tokens[start..];
    let Some(target) = after
        .iter()
        .position(|t| matches!(t, TextToken::Hour | TextToken::HourPadded))
        .or_else(|| {
            after
                .iter()
                .position(|t| matches!(t, TextToken::Minute | TextToken::MinutePadded))
        })
    else {
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
    if fraction_digits.is_some() {
        // With fraction digits the register is the remainder of a second that the clock's field
        // cut leaves, in units of the shown digit: `TEXT(3733.95/86400+1*2^-57,"[s].0 General")`
        // and `+2*2^-57` both end in "1.63425E-12" (23*2^-47 s times 10), which the serial cut
        // at once (`frac(y)*86400*scale`) cannot give. probes/t4466-count-hypotheses G1:G40,
        // B121:B152 (72 of 72 cells; the cut at once 53) and the fresh sweep
        // probes/t4466-register-fresh (120 of 120, drawn and predicted before its fill),
        // excel-graph 2026-11-08, T-4466.
        return field_split(value, scale).2 * scale;
    }
    half_step_split(value, scale).1 % 60.0
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

/// Under `.000` Excel cuts the half step's serial `y` as if 2^-72 days (1.83E-17 s) were added to
/// it first: a remainder up to about 1.8E-17 s below a millisecond tie rounds up, where T-4370's
/// 15-digit rounding alone allows 5E-18 s at 0.005 s: `TEXT(0.0045/86400-36*2^-77,
/// "[h]:mm:ss.000")` is "0:00:00.005", at -40 ulp ".004". The offset is in the serial, not in the
/// seconds: the 71 cells of probes/t4389-holdout and t4389-confirm after whole seconds that tell
/// the two apart all follow the serial. It moves `y` only below 2^-18 days (about 0.33 s); from
/// 2^-19 days it is half an ulp, which the addition rounds to even, and Excel follows that in 29
/// of the 40 probed cells there (t4389-confirm A1:A40; the rest are T-4419's). `.0` and `.00`
/// show no offset (`TEXT(0.005/86400-12*2^-77,"s.00")` is "0.00"). The constant is a fit: every
/// offset from 1.74E-17 to 1.86E-17 s scores the 538 `.000` cells of the t4323, t4332 and t4370
/// fraction cases, and 2^-72 days is the power of two inside that window. Hold-out
/// probes/t4389-holdout 368 of 368 (T-4370's model 337, the correctly rounded value 81); fresh
/// sweep probes/t4389-fraction-fresh 120 of 120 (42 and 39) and t4389-confirm 69 of 80 (51 and
/// 31), both drawn after the freeze (excel-graph 2026-11-05; docs/research/t4389).
/// number-formats.md "Fraction digits of a second at a tie".
/// FITTED(T-4389, 538 cells, offset 1.74E-17..1.86E-17 s, serial below 2^-18 days, `.000` only)
fn millisecond_offset(shifted: f64, scale: f64) -> f64 {
    /// 2^-72 days; from 2^-18 days up it is under half an ulp and the addition leaves `shifted`.
    const OFFSET: f64 = 1.0 / 4_722_366_482_869_645_213_696.0;
    // `scale` is exactly 10, 100 or 1000 (`10f64.powi`).
    if scale > 500.0 {
        shifted + OFFSET
    } else {
        shifted
    }
}

/// The whole days and the whole units of `1/scale` second within the day that the clock shows,
/// and the remainder in [0, 1) of a second the fraction digits read ([`fraction_digits`]): the
/// half step's serial `y` ([`half_step`]) cut field by field, as Excel does. `frac(y)` is
/// multiplied by 24, floored and its remainder kept, then the same with 60, 60 and `scale`, all
/// in plain doubles. Cutting `frac(y)*86400*scale` at once (the register of
/// [`half_step_split`]) truncates many ties below serial 1 that Excel rounds up:
/// `TEXT(0.015/86400,"s.00")` is "0.02" and `TEXT(21.2685/86400,"[h]:mm:ss.000")` "0:00:21.268",
/// and only the field-by-field cut gives both. It gets 134 of the 156 tie cells below serial 1
/// and 469 of 470 from 1 up in the five t4323 probes (the register 101 and 467), and 103 of 120
/// in the fresh sweep generated after it was frozen (probes/t4332-fraction-fresh, excel-graph
/// 2026-11-05; the register 72). T-4332, number-formats.md "Fraction digits of a second at a tie".
fn field_split(value: f64, scale: f64) -> (f64, f64, f64) {
    let shifted = millisecond_offset(half_step(value, scale), scale);
    let days = shifted.floor();
    let mut rest = shifted - days;
    let mut units = 0.0;
    let mut second_rest = 0.0;
    for (stage, field) in [24.0, 60.0, 60.0, scale].into_iter().enumerate() {
        rest *= field;
        let whole = rest.floor();
        units = units * field + whole;
        rest -= whole;
        if stage == 2 {
            // Cut to the whole second: what is left is the remainder of a second.
            second_rest = rest;
        }
    }
    (days, units, second_rest)
}

/// The fraction digits of a second that Excel shows, as one number below `10^digits`: the
/// remainder `rest` of a second left by [`field_split`]'s cut, rounded to 15 significant digits
/// (Excel's 15-digit cut, [`significant_15`]), then truncated to `digits` places. Below serial 1
/// this rounds up ties that the cut's own last stage truncates, `TEXT(0.0005/86400,
/// "[h]:mm:ss.000")` is "0:00:00.001" and `TEXT(3.55/86400,"[s].0")` "3.6", while one just below
/// the 15th digit stays truncated (`TEXT(20.65/86400,"[s].0")` is "20.6"). A remainder that rounds
/// to 1 shows zeros without carrying into the seconds, which are cut on their own:
/// `TEXT(41.95/86400-2*2^-64,"[s].0")` is "41.0" (probes/t4370-ulp-map-2 A417). The model scores
/// 743 of 746 cells of the t4323/t4332 fraction cases (the field-by-field digits 706), 739 of the
/// 802 ulp-map cells (554) and all 120 of the fresh sweep generated after it was frozen
/// (probes/t4370-fraction-fresh, excel-graph 2026-11-05; the field-by-field digits 54). T-4370,
/// number-formats.md "Fraction digits of a second at a tie".
fn fraction_digits(rest: f64, digits: usize) -> f64 {
    if rest <= 0.0 || !rest.is_finite() {
        return 0.0;
    }
    // rest15 = mantissa * 10^(exponent - 14) with exponent <= 0, so the shown digits are
    // mantissa / 10^(14 - exponent - digits), truncated; a remainder far below a unit gives 0.
    let (mantissa, exponent) = significant_15(rest, true);
    let shown = u32::try_from(14 - exponent - i64::try_from(digits).unwrap_or(0))
        .ok()
        .and_then(|drop| 10_u64.checked_pow(drop))
        .map_or(0, |unit| mantissa / unit);
    // A remainder that rounds to 1 wraps to zeros; `digits` is at most 3, so this is exact.
    let modulus = 10_u64.pow(u32::try_from(digits).unwrap_or(0));
    f64::from(u32::try_from(shown % modulus).unwrap_or(0))
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
        FractionalSeconds,
    };

    fn count(value: f64, digits: usize, unit: super::ElapsedUnit) -> String {
        let fraction = FractionalSeconds::new(value, digits);
        elapsed_text(value, false, unit, fraction.as_ref())
    }

    /// Beside fraction digits a count is the half-step serial times 60, 60 and 24 in that order,
    /// truncated (probes/t4323-fraction-ties A90, t4323-carry-ties A27, A31, t4466-count-hypotheses
    /// B29, B30, F33; T-4333).
    #[test]
    fn a_count_beside_a_fraction_multiplies_by_60_60_and_24() {
        let x = 45000.0 + 0.95 / 86400.0;
        assert_eq!(count(x, 1, Second), "3888000001");
        let x = 45000.0 + 59.95 / 86400.0;
        assert_eq!(count(x, 1, Second), "3888000060");
        assert_eq!(count(x, 1, Minute), "64800001");
        assert_eq!(count(x, 1, Hour), "1080000");
        // One ulp decides: 3733.95 s plus 1 or 2 times 2^-57 days.
        let tie = 3733.95 / 86400.0;
        assert_eq!(count(tie + 2f64.powi(-57), 1, Second), "3733");
        assert_eq!(count(tie + 2.0 * 2f64.powi(-57), 1, Second), "3734");
        // The hour carries alone (F33).
        assert_eq!(count(60000.0 + 82799.95 / 86400.0, 1, Hour), "1440023");
        // Ties Excel truncates stay truncated (t4323-carry-ties A9, t4323-fraction-sweep-2 A27).
        assert_eq!(count(1.0 + 3599.95 / 86400.0, 1, Hour), "24");
        assert_eq!(count(3497.95 / 86400.0, 1, Second), "3497");
        // Whole seconds round the value (no fraction digits).
        assert_eq!(count(3497.95 / 86400.0, 0, Second), "3498");
    }

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
