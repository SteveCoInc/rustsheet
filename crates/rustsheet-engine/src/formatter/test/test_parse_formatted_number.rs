#![allow(clippy::unwrap_used)]

use crate::types::DateSystem;
use crate::{
    formatter::format::parse_formatted_number,
    locale::{get_default_locale, get_locale},
};

const PARSE_ERROR_MSG: &str = "Could not parse number";

fn parse(input: &str, currencies: &[&str]) -> Result<(f64, Option<String>), String> {
    let locale = get_default_locale();
    parse_formatted_number(
        input,
        currencies,
        locale,
        &|| Some(2026),
        DateSystem::default(),
    )
}

fn parse_de(input: &str, currencies: &[&str]) -> Result<(f64, Option<String>), String> {
    let locale = get_locale("de").unwrap();
    parse_formatted_number(
        input,
        currencies,
        locale,
        &|| Some(2026),
        DateSystem::default(),
    )
}

fn parse_fr(input: &str, currencies: &[&str]) -> Result<(f64, Option<String>), String> {
    let locale = get_locale("fr").unwrap();
    parse_formatted_number(
        input,
        currencies,
        locale,
        &|| Some(2026),
        DateSystem::default(),
    )
}

#[test]
fn numbers() {
    // whole numbers
    assert_eq!(parse("400", &["$"]), Ok((400.0, None)));

    // decimal numbers
    assert_eq!(parse("4.456", &["$"]), Ok((4.456, None)));

    // scientific notation
    assert_eq!(
        parse("23e-12", &["$"]),
        Ok((2.3e-11, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("2.123456789e-11", &["$"]),
        Ok((2.123456789e-11, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("4.5E-9", &["$"]),
        Ok((4.5e-9, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("23e+2", &["$"]),
        Ok((2300.0, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("4.5E9", &["$"]),
        Ok((4.5e9, Some("0.00E+00".to_string())))
    );

    // negative numbers
    assert_eq!(parse("-400", &["$"]), Ok((-400.0, None)));
    assert_eq!(parse("-4.456", &["$"]), Ok((-4.456, None)));
    assert_eq!(
        parse("-23e-12", &["$"]),
        Ok((-2.3e-11, Some("0.00E+00".to_string())))
    );

    // trims space
    assert_eq!(parse(" 400  ", &["$"]), Ok((400.0, None)));
}

#[test]
fn percentage() {
    // whole numbers
    assert_eq!(parse("400%", &["$"]), Ok((4.0, Some("0%".to_string()))));
    // decimal numbers
    assert_eq!(
        parse("4.456$", &["$"]),
        Ok((4.456, Some("#,##0.00$".to_string())))
    );
    // Percentage in scientific notation will not be formatted as percentage
    assert_eq!(
        parse("23e-12%", &["$"]),
        Ok((23e-14, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("2.3E4%", &["$"]),
        Ok((230.0, Some("0.00E+00".to_string())))
    );
}

#[test]
fn percent_text_is_rounded_once() {
    // ironcalc-calc TEXT/T_VALUE_VALUETOTEXT.xlsx I4: VALUE("23.123%") is 0.23123, not
    // 23.123 / 100 = 0.23123000000000002 (coercion.md "Percent text is rounded once").
    assert_eq!(
        parse("23.123%", &["$"]),
        Ok((0.23123, Some("0.00%".to_string())))
    );
    assert_eq!(
        parse("-23.123%", &["$"]),
        Ok((-0.23123, Some("0.00%".to_string())))
    );
    assert_eq!(
        parse("0.07%", &["$"]),
        Ok((0.0007, Some("0.00%".to_string())))
    );
    assert_eq!(parse("0%", &["$"]), Ok((0.0, Some("0%".to_string()))));
    // 15 significant digits are kept before the shift.
    assert_eq!(
        parse("12345.6789012345678%", &["$"]),
        Ok((123.456789012345, Some("0.00%".to_string())))
    );
}

#[test]
fn currency() {
    // whole numbers
    assert_eq!(
        parse("400$", &["$"]),
        Ok((400.0, Some("#,##0$".to_string())))
    );
    // decimal numbers
    assert_eq!(
        parse("4.456$", &["$"]),
        Ok((4.456, Some("#,##0.00$".to_string())))
    );
    // A trailing `$` after E-notation is not a number (probes/t3334-criteria-operand-forms H39).
    assert!(parse("23e-12$", &["$"]).is_err());
    assert!(parse("2.3e-12$", &["$"]).is_err());
    assert_eq!(
        parse("€23e-12", &["€"]),
        Ok((2.3e-11, Some("0.00E+00".to_string())))
    );

    // switch side of currencies
    assert_eq!(
        parse("$400", &["$"]),
        Ok((400.0, Some("$#,##0_);[Red]($#,##0)".to_string())))
    );
    assert_eq!(
        parse("$4.456", &["$"]),
        Ok((4.456, Some("$#,##0.00_);[Red]($#,##0.00)".to_string())))
    );
    assert_eq!(
        parse("$23e-12", &["$"]),
        Ok((2.3e-11, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("$2.3e-12", &["$"]),
        Ok((2.3e-12, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("23e-12€", &["€"]),
        Ok((2.3e-11, Some("0.00E+00".to_string())))
    );
}

#[test]
fn negative_currencies() {
    assert_eq!(
        parse("-400$", &["$"]),
        Ok((-400.0, Some("#,##0$".to_string())))
    );
    assert_eq!(
        parse("-$400", &["$"]),
        Ok((-400.0, Some("$#,##0_);[Red]($#,##0)".to_string())))
    );
    assert_eq!(
        parse("$-400", &["$"]),
        Ok((-400.0, Some("$#,##0_);[Red]($#,##0)".to_string())))
    );
}

#[test]
fn errors() {
    // Strings are not numbers
    assert_eq!(parse("One", &["$"]), Err(PARSE_ERROR_MSG.to_string()));
    // Not partial parsing
    assert_eq!(parse("23 Hello", &["$"]), Err(PARSE_ERROR_MSG.to_string()));
    assert_eq!(parse("Hello 23", &["$"]), Err(PARSE_ERROR_MSG.to_string()));
    assert_eq!(parse("2 3", &["$"]), Err(PARSE_ERROR_MSG.to_string()));
    // Excel skips spaces after a sign: VALUE("- 5") is -5 (probes/value-sign-space-mixed-fraction-dash-date A1).
    assert_eq!(parse("- 23", &["$"]), Ok((-23.0, None)));
}

#[test]
fn errors_wrong_currency() {
    assert_eq!(parse("123€", &["$"]), Err(PARSE_ERROR_MSG.to_string()));
}

// German uses ',' as decimal separator and '.' as group separator
#[test]
fn numbers_de() {
    assert_eq!(parse_de("3,15", &["$"]), Ok((3.15, None)));
    assert_eq!(parse_de("-4,456", &["$"]), Ok((-4.456, None)));
    assert_eq!(
        parse_de("1.234", &["$"]),
        Ok((1234.0, Some("#,##0".to_string())))
    );
    assert_eq!(
        parse_de("1.234,56", &["$"]),
        Ok((1234.56, Some("#,##0.00".to_string())))
    );
    assert_eq!(
        parse_de("1.234.567", &["$"]),
        Ok((1234567.0, Some("#,##0".to_string())))
    );
}

#[test]
fn percentage_and_currency_de() {
    assert_eq!(
        parse_de("12,5%", &["$"]),
        Ok((0.125, Some("0.00%".to_string())))
    );
    assert_eq!(
        parse_de("1.234,50€", &["€"]),
        Ok((1234.5, Some("#,##0.00€".to_string())))
    );
}

#[test]
fn errors_de() {
    // en-US formatted numbers are not valid in the German locale
    assert_eq!(
        parse_de("1,234.56", &["$"]),
        Err(PARSE_ERROR_MSG.to_string())
    );
    // group separators must come in groups of three
    assert!(parse_de("1.23,4", &["$"]).is_err());
}

// French uses ',' as decimal separator and
// '\u{202f}' (narrow no-break space) as group separator
#[test]
fn numbers_fr() {
    assert_eq!(parse_fr("3,15", &["$"]), Ok((3.15, None)));
    // "1,000" is one, not one thousand
    assert_eq!(parse_fr("1,000", &["$"]), Ok((1.0, None)));
    assert_eq!(
        parse_fr("1\u{202f}234", &["$"]),
        Ok((1234.0, Some("#,##0".to_string())))
    );
    assert_eq!(
        parse_fr("1\u{202f}234,56", &["$"]),
        Ok((1234.56, Some("#,##0.00".to_string())))
    );
    assert_eq!(
        parse_fr("-12\u{202f}345\u{202f}678", &["$"]),
        Ok((-12345678.0, Some("#,##0".to_string())))
    );
}

#[test]
fn percentage_and_currency_fr() {
    assert_eq!(
        parse_fr("12,5%", &["$"]),
        Ok((0.125, Some("0.00%".to_string())))
    );
    assert_eq!(
        parse_fr("1\u{202f}234,50€", &["€"]),
        Ok((1234.5, Some("#,##0.00€".to_string())))
    );
}

#[test]
fn errors_fr() {
    // '.' is neither the decimal nor the group separator in French
    assert_eq!(parse_fr("1.000", &["$"]), Err(PARSE_ERROR_MSG.to_string()));
    // en-US formatted numbers are not valid in the French locale
    assert_eq!(
        parse_fr("1,234.56", &["$"]),
        Err(PARSE_ERROR_MSG.to_string())
    );
    // group separators must come in groups of three
    assert!(parse_fr("1\u{202f}23,4", &["$"]).is_err());
}

#[test]
fn long_dates_en_us() {
    assert_eq!(
        parse("03/02/2024", &["$"]),
        Ok((45353.0, Some("mm/dd/yyyy".to_string())))
    );
    assert_eq!(
        parse("3/02/2024", &["$"]),
        Ok((45353.0, Some("m/dd/yyyy".to_string())))
    );
    assert_eq!(
        parse("3/2/24", &["$"]),
        Ok((45353.0, Some("m/d/yy".to_string())))
    );

    assert_eq!(
        parse("02-10-1975", &["$"]),
        Ok((27435.0, Some("mm-dd-yyyy".to_string())))
    );
    assert_eq!(
        parse("2-10-1975", &["$"]),
        Ok((27435.0, Some("m-dd-yyyy".to_string())))
    );
    assert_eq!(
        parse("2-10-75", &["$"]),
        Ok((27435.0, Some("m-dd-yy".to_string())))
    );
}

#[test]
fn month_name_first_separated_dates_are_not_dates() {
    // Excel en-US: probes/date-text-grammar A15, A16, A59-A62 are #VALUE!.
    for text in [
        "Mar/02/2024",
        "March/02/2024",
        "Feb-10-1975",
        "February-10-1975",
    ] {
        assert!(parse(text, &["$"]).is_err(), "{text}");
    }
}

#[test]
fn month_name_dates_en_us() {
    // probes/date-text-grammar A1, A8, A10, A17.
    for text in ["July 1, 2026", "1 July 2026", "1-Jul-2026", "Jul 2026"] {
        assert!(
            matches!(parse(text, &["$"]), Ok((46204.0, Some(_)))),
            "{text}"
        );
    }
    // Number coercion does not trim date text (ironcalc-calc MROUND_TRUNC_INT.xlsx MROUND!C32).
    assert!(parse(" July 1, 2026 ", &["$"]).is_err());
}

#[test]
fn iso_dates() {
    assert_eq!(
        parse("2024/03/02", &["$"]),
        Ok((45353.0, Some("yyyy/mm/dd".to_string())))
    );
    assert_eq!(
        parse("2024/March/02", &["$"]),
        Err(PARSE_ERROR_MSG.to_string())
    );
}

#[test]
fn long_dates_with_dots() {
    // en-US's short date does not use dots, so dotted text is not a date
    // (probes/date-text-grammar A39/A40: "1.7.2026" and "7.1.2026" are #VALUE!).
    assert!(parse("03.02.2024", &["$"]).is_err());
}

// Excel: probes/paren-negative-text, docs/excel-semantics/coercion.md.
#[test]
fn parenthesised_number_is_negative() {
    assert_eq!(parse("(1)", &["$"]), Ok((-1.0, None)));
    assert_eq!(
        parse("(1,000)", &["$"]),
        Ok((-1000.0, Some("#,##0".to_string())))
    );
    assert!(parse("-(1)", &["$"]).is_err());
    assert!(parse("(-1)", &["$"]).is_err());
    assert!(parse("()", &["$"]).is_err());
}

// Excel: probes/paren-negative-variants (excel-graph, T-0277).
#[test]
fn parenthesised_number_with_padding_currency_percent() {
    assert_eq!(parse("( 1 )", &["$"]), Ok((-1.0, None)));
    assert_eq!(parse("(1.5)", &["$"]), Ok((-1.5, None)));
    assert_eq!(
        parse("($1)", &["$"]),
        Ok((-1.0, Some("$#,##0_);[Red]($#,##0)".to_string())))
    );
    assert_eq!(parse("(1%)", &["$"]), Ok((-0.01, Some("0%".to_string()))));
    assert!(parse("(-1)", &["$"]).is_err());
    assert!(parse("(+1)", &["$"]).is_err());
    assert!(parse("((1))", &["$"]).is_err());
    assert!(parse("(1)x", &["$"]).is_err());
    // probes/paren-number-text-forms: a sign after the currency and a trailing currency.
    assert!(parse("($-1)", &["$"]).is_err());
    assert!(parse("($ -1)", &["$"]).is_err());
    assert!(parse("(1$)", &["$"]).is_err());
    // probes/paren-number-text-currency-variants: only a trailing euro sign is accepted.
    assert!(parse("($+1)", &["$", "€"]).is_err());
    assert!(parse("(1 $)", &["$", "€"]).is_err());
    assert!(parse("(€-1)", &["$", "€"]).is_err());
    assert_eq!(
        parse("(1€)", &["$", "€"]),
        Ok((-1.0, Some("#,##0€".to_string())))
    );
}

/// Formats Excel picks for typed text (probes/typed-currency-percent-input, excel-graph).
#[test]
fn typed_currency_and_percent_formats() {
    let usd = |n: f64, decimals: bool| {
        let digits = if decimals { "#,##0.00" } else { "#,##0" };
        Ok((n, Some(format!("${digits}_);[Red](${digits})"))))
    };
    assert_eq!(parse("$1.50", &["$"]), usd(1.5, true));
    assert_eq!(parse("($1.50)", &["$"]), usd(-1.5, true));
    assert_eq!(parse("-$1", &["$"]), usd(-1.0, false));
    assert_eq!(parse("-$1.50", &["$"]), usd(-1.5, true));
    assert_eq!(parse("$1,234.5", &["$"]), usd(1234.5, true));
    // `-$1e3` is negative like `$-1e3`.
    assert_eq!(
        parse("-$1e3", &["$"]),
        Ok((-1000.0, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("1.5%", &["$"]),
        Ok((0.015, Some("0.00%".to_string())))
    );
    assert_eq!(
        parse("(1.5%)", &["$"]),
        Ok((-0.015, Some("0.00%".to_string())))
    );
    assert_eq!(parse("1,000%", &["$"]), Ok((10.0, Some("0%".to_string()))));
    assert_eq!(parse("-1%", &["$"]), Ok((-0.01, Some("0%".to_string()))));
}

#[test]
fn script_digits_are_mapped_only_by_the_coercion_helper() {
    use crate::formatter::format::ascii_script_digits;
    let map = |s: &str| ascii_script_digits(s).into_owned();
    assert_eq!(map("٣٫"), "3٫");
    assert_eq!(map("१२३"), "123");
    assert_eq!(map("๓.๕"), "3.5");
    assert_eq!(map("1٣"), "13");
    // The five blocks T-3446 added: Extended Arabic-Indic, Bengali, Lao, Tibetan, Myanmar
    // (probes/t3446-script-digits-forms rows 23:33, 45:55, 67:99).
    assert_eq!(map("۱۲۳"), "123");
    assert_eq!(map("১২৩"), "123");
    assert_eq!(map("໑໒໓"), "123");
    assert_eq!(map("༡༢༣"), "123");
    assert_eq!(map("၁၂၃"), "123");
    assert_eq!(
        map("\u{0669}\u{06F9}\u{096F}\u{09EF}\u{0E59}\u{0ED9}\u{0F29}\u{1049}"),
        "99999999"
    );
    // Full-width digits stay text (probes/t3276-collation-other-scripts Calc!H6).
    assert_eq!(map("３"), "３");
    // The typed-input parser does not map them yet (TODO(T-3891)).
    assert!(parse("٣", &[]).is_err());
}

// docs/excel-semantics/coercion.md "Group commas and a leading percent sign" (T-3811).
#[test]
fn group_commas_of_three_or_more_digits_and_a_leading_percent() {
    for (text, value) in [
        ("1,0000", 10000.0),
        ("1,000,0000", 10_000_000.0),
        ("1,0000.5", 10000.5),
        ("12,0000", 120_000.0),
        ("%1", 0.01),
        ("%-1", -0.01),
        ("1,0000%", 100.0),
    ] {
        assert_eq!(parse(text, &[]).unwrap().0, value, "{text}");
    }
    for text in [
        "1,", "1,00", ",1", "2,0", "1,000,00", "1,,000", "0,0000", "0,000", "00,000", "%1%", "2%%",
    ] {
        assert!(parse(text, &[]).is_err(), "{text}");
    }
}

/// Typed mixed-separator dates and clock-first date-times (probes/date-text-mixed-separator-typed,
/// excel-graph 2026-11-01; docs/excel-semantics/dates.md "Typed mixed-separator dates").
#[test]
fn typed_mixed_separator_date_formats() {
    let fmt = |s: &str| {
        let locale = get_default_locale();
        crate::formatter::format::parse_typed_number(
            s,
            &["$"],
            locale,
            &|| Some(2026),
            DateSystem::default(),
        )
        .map(|(v, f)| (v, f.unwrap_or_default()))
    };
    for s in ["2025/07-01", "2025-07/01", "7/1-25", "7/1-2025", "7-1/2025"] {
        assert_eq!(fmt(s).unwrap().1, "m/d/yyyy", "{s}");
    }
    for s in ["1-Jul/2025", "1/Jul-2025"] {
        assert_eq!(fmt(s).unwrap().1, "d-mmm-yy", "{s}");
    }
    // Pure separators keep their own formats.
    assert_eq!(fmt("2025/07/01").unwrap().1, "yyyy/mm/dd");
    assert_eq!(fmt("7/1/25").unwrap().1, "m/d/yy");
    let (v, f) = fmt("10:00 7/1/2025").unwrap();
    assert!((v - 45_839.416_666_666_7).abs() < 1e-9);
    assert_eq!(f, "m/d/yyyy h:mm");
    // A leading space keeps it text.
    assert!(fmt(" 10:00 7/1/2025").is_err());
    assert!(fmt(" 10:00 AM 7/1/2025").is_err());
    // Other clock-first shapes (probes/date-text-typed-clock-first-shapes, excel-graph 2026-11-03).
    for (s, value, format) in [
        ("10 AM 7/1/2025", 45_839.416_666_666_7, "m/d/yyyy h:mm"),
        ("10:00 PM 7/1/2025", 45_839.916_666_666_7, "m/d/yyyy h:mm"),
        ("9:00 7/1/2025", 45_839.375, "m/d/yyyy h:mm"),
        ("10:00:30 7/1/2025", 45_839.417_013_888_9, "m/d/yyyy h:mm"),
        ("10:00:30.5 7/1/2025", 45_839.417_019_675_9, "mm:ss.0"),
        ("10:00.5 7/1/2025", 45_839.006_950_231_5, "mm:ss.0"),
        ("24:00 7/1/2025", 45_840.0, ""),
        ("10:60 7/1/2025", 45_839.458_333_333_3, ""),
        ("10:00  7/1/2025", 45_839.416_666_666_7, "m/d/yyyy h:mm"),
        ("10:00 July 1, 2025", 45_839.416_666_666_7, "m/d/yyyy h:mm"),
        ("10:00 1-Jul-2025", 45_839.416_666_666_7, "m/d/yyyy h:mm"),
        ("10:00 2025-07-01", 45_839.416_666_666_7, "m/d/yyyy h:mm"),
        ("10:00 7/1", 46_204.416_666_666_7, "m/d/yyyy h:mm"),
        ("10:00 7/1/25", 45_839.416_666_666_7, "m/d/yyyy h:mm"),
        ("23:59 7/1-2025", 45_839.999_305_555_6, "m/d/yyyy h:mm"),
        ("10:00 7-1-2025", 45_839.416_666_666_7, "m/d/yyyy h:mm"),
        ("10:00 7/1/2025 ", 45_839.416_666_666_7, "m/d/yyyy h:mm"),
        ("10:00 1/Jul/2025", 45_839.416_666_666_7, "m/d/yyyy h:mm"),
        ("10:00 AM 7-1-2025", 45_839.416_666_666_7, "m/d/yyyy h:mm"),
    ] {
        let (v, f) = fmt(s).unwrap_or_else(|e| panic!("{s}: {e}"));
        assert!((v - value).abs() < 1e-9, "{s}: {v}");
        assert_eq!(f, format, "{s}");
    }
    // Untyped mixed shapes.
    assert_eq!(fmt("2025/7-1").unwrap().1, "m/d/yyyy");
    assert_eq!(fmt("1-Jul/25").unwrap().1, "d-mmm-yy");
}

/// Typed text and the format Excel picks (probes/typed-group-comma-leading-percent, excel-graph
/// 2026-11-02, raw `ENTERED` log in docs/research/typed-group-comma-leading-percent-2026-11-02.md,
/// T-3835): the same reader as VALUE, so values and formats follow the grouped digits and the percent.
#[test]
fn typed_group_comma_and_leading_percent_formats() {
    for (text, value, format) in [
        ("1,0000", 10000.0, "#,##0"),
        ("01,0000", 10000.0, "#,##0"),
        ("-1,0000", -10000.0, "#,##0"),
        ("1,000,0000", 10_000_000.0, "#,##0"),
        ("1,0000.5", 10000.5, "#,##0.00"),
        ("%1", 0.01, "0%"),
        ("%-1", -0.01, "0%"),
        ("% 1", 0.01, "0%"),
        ("%1.5", 0.015, "0.00%"),
        ("%1,0000", 100.0, "0%"),
        ("1,0000%", 100.0, "0%"),
    ] {
        assert_eq!(
            parse(text, &["$"]),
            Ok((value, Some(format.to_string()))),
            "{text}"
        );
    }
    // Stay text (General) when typed.
    for text in ["1,", "0,000", "0,0000", "1,00", "%1%"] {
        assert!(parse(text, &["$"]).is_err(), "{text}");
    }
}

/// A bare typed 24-hour clock is a number with Excel's clock format (probes/t3894-bare-24h-clock,
/// excel-graph 2026-11-03; docs/excel-semantics/dates.md).
#[test]
fn typed_bare_24_hour_clock() {
    let fmt = |s: &str| {
        crate::formatter::format::parse_typed_number(
            s,
            &["$"],
            get_default_locale(),
            &|| Some(2026),
            DateSystem::default(),
        )
        .map(|(v, f)| (v, f.unwrap_or_else(|| "General".to_string())))
    };
    let cases = [
        ("12:30", 0.520_833_333_333_333, "h:mm"),
        ("9:05", 0.378_472_222_222_222, "h:mm"),
        ("0:00", 0.0, "h:mm"),
        ("1:2", 0.043_055_555_555_555_6, "h:mm"),
        ("10:00:", 0.416_666_666_666_667, "h:mm"),
        ("10:00:30", 0.417_013_888_888_889, "h:mm:ss"),
        ("01:02:03", 0.043_090_277_777_777_8, "h:mm:ss"),
        ("24:00", 1.0, "[h]:mm:ss"),
        ("100:00", 4.166_666_666_666_67, "[h]:mm:ss"),
        ("10:60", 0.458_333_333_333_333, "General"),
        ("10:00:60", 0.417_361_111_111_111, "General"),
        ("10:00.5", 0.006_950_231_481_481_48, "mm:ss.0"),
        ("10:00:30.5", 0.417_019_675_925_926, "mm:ss.0"),
    ];
    for (text, value, format) in cases {
        let (v, f) = fmt(text).unwrap_or_else(|e| panic!("{text}: {e}"));
        assert!((v - value).abs() < 1e-12, "{text}: {v}");
        assert_eq!(f, format, "{text}");
    }
    assert!(fmt(":30").is_err());
    assert!(fmt(" 10:00").is_err());
}
