#![allow(clippy::unwrap_used)]

use super::text;
use crate::{
    formatter::format::{format_number, FormatContext},
    locale::{get_locale, Locale},
};

fn get_default_locale() -> &'static Locale {
    get_locale("en").unwrap()
}

#[test]
fn simple_test() {
    let ctx = FormatContext::new(get_default_locale());
    let format = "h:mm AM/PM";
    let value = 16.001_423_611_111_11; // =1/86400 => 12:02 AM
    let formatted = format_number(value, format, &ctx);
    assert_eq!(formatted.text, "12:02 AM");
}

#[test]
fn padded_vs_unpadded() {
    let ctx = FormatContext::new(get_default_locale());
    let padded_format = "hh:mm:ss AM/PM";
    let unpadded_format = "h:m:s AM/PM";
    let value = 0.25351851851851853; // => 6:05:04 AM (21904/(24*60*60)) where 21904 = 6 * 3600 + 5*60 + 4
    let formatted = format_number(value, padded_format, &ctx);
    assert_eq!(formatted.text, "06:05:04 AM");

    let formatted = format_number(value, unpadded_format, &ctx);
    assert_eq!(formatted.text, "6:5:4 AM");
}

#[test]
fn elapsed_hour() {
    let ctx = FormatContext::new(get_default_locale());
    let format = "[hh]:mm:ss";
    // 1.5 days = 36 hours
    let value = 1.5;
    let formatted = format_number(value, format, &ctx);
    assert_eq!(formatted.text, "36:00:00");
}

#[test]
fn elapsed_minute() {
    let ctx = FormatContext::new(get_default_locale());
    let format = "[mm]:ss";
    // 2 days = 2880 minutes
    let value = 2.0;
    let formatted = format_number(value, format, &ctx);
    assert_eq!(formatted.text, "2880:00");
}

#[test]
fn elapsed_second() {
    let ctx = FormatContext::new(get_default_locale());
    let format = "[ss]";
    // 0.5 days = 43200 seconds
    let value = 0.5;
    let formatted = format_number(value, format, &ctx);
    assert_eq!(formatted.text, "43200");
}

#[test]
fn elapsed_hour_padded() {
    let ctx = FormatContext::new(get_default_locale());
    let format = "[hh]:mm:ss";
    // 0.1 days = 2.4 hours
    let value = 0.1;
    let formatted = format_number(value, format, &ctx);
    assert_eq!(formatted.text, "02:24:00");
}

/// gen-m10 and gen-m5 (T-1731): a clock token over a serial past 9999-12-31 is unshowable, a lone
/// elapsed token is not.
#[test]
fn clock_past_last_date_is_unshowable() {
    let ctx = FormatContext::new(get_default_locale());
    for format in ["h:mm", "[h]:mm", "[m]:ss", "h AM/PM", "h:mm:ss AM/PM"] {
        let formatted = format_number(1_234_567_890.0, format, &ctx);
        assert!(formatted.unshowable, "{format}");
    }
    let formatted = format_number(1e15, "[s]", &ctx);
    assert!(!formatted.unshowable);
    assert_eq!(formatted.text, "86400000000000000000");
}

#[test]
fn minutes_before_seconds_without_hour() {
    // `mm` directly before `:ss` is minutes even with no hour in the code
    // (gen-m10 text cells, excel-graph 2026-10-16, T-1733).
    let ctx = FormatContext::new(get_default_locale());
    for (value, format, expected) in [
        (1000.0, "mm:ss", "00:00"),
        (123_456.789, "mm:ss", "56:10"),
        (1.5, "mm:ss", "00:00"),
        // Unprobed in Excel, same rule.
        (123_456.789, "m:ss", "56:10"),
    ] {
        assert_eq!(
            format_number(value, format, &ctx).text,
            expected,
            "{format}"
        );
    }
}

// `[h]` and `[m]` without a seconds token count from the serial rounded to the second, as the
// clock does (probes/elapsed-no-seconds-boundary A1-A8, A20, T-2385).
#[test]
fn elapsed_without_seconds_rounds_to_the_second() {
    let ctx = FormatContext::new(get_default_locale());
    let short = |hours: f64, seconds: f64| (hours - seconds / 3600.0) / 24.0;
    for (value, format, text) in [
        (short(3.0, 0.4), "[h]", "3"),
        (short(3.0, 0.4), "[h]:mm", "3:00"),
        (short(3.0, 0.4), "[m]", "180"),
        (short(3.0, 0.6), "[h]", "2"),
        (short(3.0, 0.6), "[h]:mm", "2:59"),
        (short(3.0, 0.6), "[m]", "179"),
        ((5.0 - 0.4 / 60.0) / 1440.0, "[m]", "5"),
    ] {
        assert_eq!(format_number(value, format, &ctx).text, text, "{format}");
    }
}

// poi/DateFormatTests.xlsx Tests!A11, A16, A30, A35 (desktop Excel); noon and midnight are hand expectations that follow `AM/PM` (no Excel cell).
#[test]
fn short_am_pm_marker_keeps_its_case() {
    assert_eq!(text(17816.607951388887, "hh:mm:ss a/p"), "02:35:27 p");
    assert_eq!(text(17816.607951388887, "HH:MM:SS A/P"), "02:35:27 P");
    assert_eq!(text(36191.1702084375, "hh:mm:ss a/p"), "04:05:06 a");
    assert_eq!(text(36191.1702084375, "HH:MM:SS A/P"), "04:05:06 A");
    assert_eq!(text(0.5, "hh:mm:ss a/p"), "12:00:00 p");
    assert_eq!(text(0.0, "hh:mm:ss a/p"), "12:00:00 a");
}

// DateFormatTests A13 and A18: lowercase `am/pm` still prints uppercase.
#[test]
fn long_am_pm_marker_prints_uppercase() {
    assert_eq!(text(0.6079, "hh:mm:ss am/pm"), "02:35:23 PM");
}

// probes/negative-date-sections-1900 and -1904 A9, A21, A22: two bracketed elapsed tokens are #VALUE!.
#[test]
fn two_bracketed_elapsed_tokens_are_invalid() {
    assert_eq!(text(1.5, "[h]:[mm]"), "#VALUE!");
    assert_eq!(text(0.5, "[mm]:[ss]"), "#VALUE!");
    // One bracketed token, with or without a bare `h` after it, stays valid (poi ElapsedFormatTests A10).
    assert_eq!(text(1.5, "[h]:mm"), "36:00");
}

// probes/text-elapsed-two-tokens A1, A9-A11, A13 (excel-graph).
#[test]
fn elapsed_token_before_am_pm_marker_is_invalid() {
    assert_eq!(text(1.5, "[ss] AM/PM"), "#VALUE!");
    assert_eq!(text(1.5, "[h] AM/PM"), "#VALUE!");
    assert_eq!(text(1.5, "[mm] AM/PM"), "#VALUE!");
    assert_eq!(text(1.5, "[s] A/P"), "#VALUE!");
}

/// The clock, elapsed counts and fraction digits truncate the half-step register, so its float error decides a tie
/// (probes/t4323-fraction-ties A11, A13, A14, A16, A60; docs/excel-semantics/number-formats.md "Fraction
/// digits of a second at a tie").
#[test]
fn fraction_digit_ties_follow_the_half_step_register() {
    let ctx = FormatContext::new(get_default_locale());
    let cases = [
        (1.0 + 0.25 / 86400.0, "[h]:mm.0", "24:00.2"),
        (1.0 + 0.55 / 86400.0, "[h]:mm.0", "24:00.5"),
        (1.0 + 0.05 / 86400.0, "[h]:mm.0", "24:00.0"),
        (1.0 + 0.35 / 86400.0, "[h]:mm.0", "24:00.4"),
        (45000.0 + 0.95 / 86400.0, "h:mm:ss.0", "0:00:00.9"),
        // The elapsed count reads the same register (probes/t4323-carry-ties A9, A17; sweep-2 A27).
        (1.0 + 3599.95 / 86400.0, "[h]:mm:ss.0", "24:59:59.9"),
        (2.0 + 3599.95 / 86400.0, "[h]:mm:ss.0", "49:59:59.9"),
        (3497.95 / 86400.0, "[s].0", "3497.9"),
    ];
    for (value, format, expected) in cases {
        assert_eq!(
            format_number(value, format, &ctx).text,
            expected,
            "{format}"
        );
    }
}

/// Below serial 1 the clock and the fraction digits are cut from the half-step serial field by field (24, 60, 60,
/// then the fraction), so Excel rounds some ties up and truncates others (probes/t4323-fraction-ties A92,
/// t4323-fraction-sweep-2 A3, A4, t4323-fraction-holdout A5, t4332-fraction-fresh A6; T-4332).
#[test]
fn fraction_digit_ties_below_one_are_cut_field_by_field() {
    let ctx = FormatContext::new(get_default_locale());
    let cases = [
        (0.015 / 86400.0, "s.00", "0.02"),
        (21.2685 / 86400.0, "[h]:mm:ss.000", "0:00:21.268"),
        (28506.75 / 86400.0, "mm:ss.0", "55:06.7"),
        (20.65 / 86400.0, "[s].0", "20.6"),
        (8134.55 / 86400.0, "mm:ss.0", "15:34.5"),
    ];
    for (value, format, expected) in cases {
        assert_eq!(
            format_number(value, format, &ctx).text,
            expected,
            "{format}"
        );
    }
}

/// The fraction digits read the cut's remainder of a second at 15 significant digits, so a tie within half a unit
/// of the 15th digit rounds up, and a remainder that rounds to 1 shows zeros without carrying into the seconds
/// (probes/t4323-fraction-ties A121, A128, t4323-fraction-holdout A100, t4332-fraction-fresh A8,
/// t4370-ulp-map-2 A417, A418, t4370-fraction-fresh; T-4370).
#[test]
fn fraction_digits_read_the_remainder_at_fifteen_digits() {
    let ctx = FormatContext::new(get_default_locale());
    let cases = [
        (0.0005 / 86400.0, "[h]:mm:ss.000", "0:00:00.001"),
        (0.0075 / 86400.0, "[h]:mm:ss.000", "0:00:00.008"),
        (3.55 / 86400.0, "[s].0", "3.6"),
        (1.45 / 86400.0, "h:mm:ss.0", "0:00:01.5"),
        (0.055 / 86400.0, "s.00", "0.06"),
        // Just below the 15th digit stays truncated.
        (20.65 / 86400.0, "[s].0", "20.6"),
        (2.05 / 86400.0, "[s].0", "2.0"),
        // The digits wrap and the seconds stay: 2 ulp below 41.95 s is "41.0", 1 ulp below "42.0".
        (41.95 / 86400.0 - 2.0 * 2f64.powi(-64), "[s].0", "41.0"),
        (41.95 / 86400.0 - 2f64.powi(-64), "[s].0", "42.0"),
    ];
    for (value, format, expected) in cases {
        assert_eq!(
            format_number(value, format, &ctx).text,
            expected,
            "{value:e} {format}"
        );
    }
}

/// Under `.000` the serial is cut as if 2^-72 days were added after the half step, so a tie up to
/// about 1.8E-17 s below rounds up; after whole seconds the addition rounds away, and `.00` has no
/// offset (probes/t4370-ulp-map-2 A76, A75, t4389-holdout A272; T-4389).
#[test]
fn millisecond_ties_take_the_serial_offset() {
    let ctx = FormatContext::new(get_default_locale());
    let cases = [
        (
            0.0045 / 86400.0 - 36.0 * 2f64.powi(-77),
            "[h]:mm:ss.000",
            "0:00:00.005",
        ),
        (
            0.0045 / 86400.0 - 40.0 * 2f64.powi(-77),
            "[h]:mm:ss.000",
            "0:00:00.004",
        ),
        (
            5.0605 / 86400.0 - 2f64.powi(-67),
            "[h]:mm:ss.000",
            "0:00:05.060",
        ),
        (0.005 / 86400.0 - 12.0 * 2f64.powi(-77), "s.00", "0.00"),
    ];
    for (value, format, expected) in cases {
        assert_eq!(
            format_number(value, format, &ctx).text,
            expected,
            "{value:e} {format}"
        );
    }
}

// probes/t4295-elapsed-with-month A1, D1, K1 (excel-graph): `[s]` before an hour or minute token
// hands it the elapsed count, counted from the date section's register (18.5 days on 1h42m18s).
#[test]
fn elapsed_second_moves_onto_a_later_hour_or_minute() {
    let value = 1.7 / 24.0 + 0.3 / 1440.0;
    assert_eq!(text(value, "[s] m"), "18 26640");
    assert_eq!(text(value, "[s] h"), "18 444");
    assert_eq!(text(value, "[s] m s"), "18 26640 18");
    assert_eq!(text(value, "[s] d"), "6138 0");
    // A clock token before `[s]` keeps the section as parsed: `s [s]` is #VALUE! (T-3419).
    assert_eq!(text(value, "s [s] m"), "#VALUE!");
}
