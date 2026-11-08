//! The General number format: fixed or E notation (T-0938).
// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
// LINT-DEBT-END

use crate::decimal_round::{round_decimal, RoundMode};

/// Characters General's fixed notation may use, the sign excluded and the decimal point counted:
/// "0." plus 9 decimals below 1 (poi/GeneralFormatTests A27 `1E-09` → "0.000000001", A61 `-1E-09`
/// → "-0.000000001"); from 1 on the integer digits take their share, so `123456.123456789` is
/// "123456.1235" (probes/text-general-ties-wide-fixed, docs/excel-semantics/number-formats.md).
const GENERAL_WIDTH: usize = 11;

/// Decimals the fixed form of `magnitude` (non-zero, below 1E+11) may keep.
fn fixed_decimals(magnitude: f64) -> usize {
    if magnitude < 1.0 {
        return GENERAL_WIDTH - 2;
    }
    // The integer part has `floor(log10) + 1` digits; the decimal point takes one more character.
    let integer_digits = magnitude.log10().floor() as usize + 1;
    GENERAL_WIDTH.saturating_sub(integer_digits + 1)
}

/// `value` under the General format, with "." as the decimal point.
///
/// General prints fixed notation when `1E-09 <= |value| < 1E+11` and the fixed form shows at least
/// as many significant digits as E notation would; otherwise it prints E notation. So `1E-09` is
/// "0.000000001" but `1E-10` is "1E-10", and `1.23456789012345E-05` is "1.23457E-05" because the
/// fixed "0.000012346" keeps 5 digits against 6 (probes/number-text-exponent-digits D8,
/// poi/GeneralFormatTests A13-A33; `docs/excel-semantics/number-formats.md`).
pub(crate) fn format_general(value: f64) -> String {
    if value == 0.0 {
        return "0".to_string();
    }
    let scientific = format_general_scientific(value);
    // Below 1E-09 General is E notation even when the fixed form would show the same digits:
    // `9.9999996E-10` is "1E-09", not "0.000000001" (probes/text-general-exponent-boundary A10).
    // From 1E+11 the integer part alone needs 12 characters (A18 `1E11` → "1E+11").
    if (1e-9..1e11).contains(&value.abs()) {
        let decimals = fixed_decimals(value.abs());
        // Excel rounds the 15-digit decimal half away from zero (probes/text-general-ties-wide-fixed
        // A18-A24: `12345678902.5` → "12345678903", `123456.12345` → "123456.1235"), as ROUND does.
        let rounded =
            round_decimal(value, decimals as f64, RoundMode::HalfAwayFromZero).unwrap_or(value);
        let untrimmed = format!("{rounded:.decimals$}");
        // Integer-only text ("10000000000") has no decimal point, so its zeros are not trailing.
        let fixed = if untrimmed.contains('.') {
            untrimmed.trim_end_matches('0').trim_end_matches('.')
        } else {
            &untrimmed
        };
        // Each form is credited with the digits of the value it has room for, not the digits left
        // after rounding trims its zeros: `1.5459954738616943E-06` is "1.546E-06" (the E form's
        // room is 6 digits, the fixed "0.000001546" has 4), while `1.2345E-05`, whose 5 digits
        // both forms hold, is "0.000012345" (probes/t4466-count-hypotheses G24, excel-graph
        // 2026-11-08; number-formats.md "`TEXT(x,"General")` uses an 11-character fixed-notation
        // limit").
        let digits = value_digits(value);
        // The fixed form's room runs from the value's own leading digit to the last decimal, so a
        // rounding that carries into a new digit gains none: `9.7E-09` has room for 1 digit and is
        // "9.7E-09", not "0.00000001" (probes/t4466-general-fixed-or-e A18, A47).
        let leading = value.abs().log10().floor() as i64;
        let room_fixed =
            usize::try_from(i64::try_from(decimals).unwrap_or(0) + leading + 1).unwrap_or(0);
        let shown_fixed = room_fixed.min(digits);
        let shown_scientific = scientific_room(value).min(digits);
        // A value that rounds up to 1E+11 needs 12 characters (`99999999999.6` → "1E+11").
        if rounded.abs() < 1e11 && shown_fixed >= shown_scientific {
            return fixed.to_string();
        }
    }
    scientific
}

/// `value` (non-zero) in General's E notation: "1.23457E-05", "1.2346E+100".
fn format_general_scientific(value: f64) -> String {
    // The mantissa rounds the 15-significant-digit decimal half away from zero, as the fixed form
    // does, not the binary value: `1.133925E-06`, stored just below the tie, is "1.13393E-06" and
    // `2.059915E-08` "2.05992E-08" (probes/t4466-general-fixed-or-e A82, A96, excel-graph 2026-11-08).
    let (mantissa15, mut exponent) = crate::number_format::significant_15(value.abs(), true);
    // One digit fewer from a decimal exponent magnitude of 99 on: "1.2346E+100" against
    // "1.23457E-05" (probes/number-text-exponent-digits D1:D15, numbers.md "Exponent width").
    let digits_for = |exponent: i64| if exponent.abs() >= 99 { 5_u32 } else { 6 };
    let round_to = |digits: u32| {
        let unit = 10_u64.pow(15 - digits);
        (mantissa15 + unit / 2) / unit
    };
    let mut digits = digits_for(exponent);
    let mut mantissa = round_to(digits);
    // Only rounding can reach 10^digits ("10."): it carries into the exponent, so 9.9999996E+98
    // prints "1E+99", not "10E+98" (probes/text-general-exponent-boundary A5-A8, A10).
    if mantissa >= 10_u64.pow(digits) {
        exponent += 1;
        digits = digits_for(exponent);
        mantissa = 10_u64.pow(digits - 1);
    }
    let text = mantissa.to_string();
    let (lead, rest) = text.split_at(1);
    let rest = rest.trim_end_matches('0');
    let sign = if value < 0.0 { "-" } else { "" };
    let point = if rest.is_empty() { "" } else { "." };
    let exponent_sign = if exponent < 0 { '-' } else { '+' };
    format!(
        "{sign}{lead}{point}{rest}E{exponent_sign}{:02}",
        exponent.abs()
    )
}

/// The significant digits of `value` (non-zero) at Excel's 15: `1.2345E-05` has 5,
/// `1.5459954738616943E-06` 14 ("1.54599547386169").
fn value_digits(value: f64) -> usize {
    let (mut mantissa, _) = crate::number_format::significant_15(value.abs(), true);
    let mut digits = 15;
    while digits > 1 && mantissa % 10 == 0 {
        mantissa /= 10;
        digits -= 1;
    }
    digits
}

/// The significant digits General's E form has room for: 6, or 5 from a decimal exponent
/// magnitude of 99 on ([`format_general_scientific`]).
fn scientific_room(value: f64) -> usize {
    if value.abs().log10().floor().abs() >= 99.0 {
        5
    } else {
        6
    }
}

/// The value a `General` token shows among literals. With no digit placeholder the minus leads the
/// literals (`"x"General` on -1.5 is "-x1.5", gen-m10 F!D19; T-2097), so the token shows |value|.
pub(super) fn lead_abs(value: f64, digit_count: i32) -> f64 {
    if digit_count == 0 {
        value.abs()
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::format_general;

    /// Each form is credited with the digits it has room for, capped by the value's own digits
    /// (probes/t4466-count-hypotheses G24; number-formats.md "`TEXT(x,"General")` uses an
    /// 11-character fixed-notation limit").
    #[test]
    fn fixed_or_e_compares_the_digits_each_form_has_room_for() {
        assert_eq!(format_general(1.545_995_473_861_694_3e-6), "1.546E-06");
        assert_eq!(format_general(1.2345e-5), "0.000012345");
        assert_eq!(format_general(1e-9), "0.000000001");
        assert_eq!(format_general(1.4e-9), "1.4E-09");
        assert_eq!(format_general(1.234_567_890_123_45e-5), "1.23457E-05");
        assert_eq!(format_general(0.000_123_456_789), "0.000123457");
        // A fixed rounding that carries into a new digit gains no room (t4466-general-fixed-or-e
        // A18, A47), and the E mantissa rounds the 15-digit decimal half away (A82, A96).
        assert_eq!(format_general(9.7e-9), "9.7E-09");
        assert_eq!(format_general(9.5e-9), "9.5E-09");
        assert_eq!(format_general(1.133_925e-6), "1.13393E-06");
        assert_eq!(format_general(2.059_915e-8), "2.05992E-08");
    }
}
