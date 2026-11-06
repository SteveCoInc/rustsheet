#![allow(clippy::unwrap_used)]

use super::text;
use crate::{
    formatter::format::{format_number, FormatContext},
    locale::get_locale,
};

// Expected texts are Excel's displayed text in eval/suites/gen-m10 (excel-graph, T-1611), cells
// named by case-id prefix; docs/excel-semantics/number-formats.md "Fraction formats".

#[test]
fn numerator_is_rounded_and_never_reduced() {
    assert_eq!(text(1.5, "# ?/4"), "1 2/4"); // 039accab F7
    assert_eq!(text(36.5, "# ?/10"), "36 5/10"); // 43980980 P2
    assert_eq!(text(0.125, "# ?/4"), " 1/4"); // 039accab F12: 0.5 rounds up
}

#[test]
fn numerator_rounds_at_fifteen_digits() {
    // 12.345 * 100 is 1234.4999999999998 (05c1c646 D11, 43980980 P16).
    assert_eq!(text(12.345, "?/100"), "1235/100");
    assert_eq!(text(12.345, "# ?/10"), "12 3/10");
}

#[test]
fn full_numerator_carries_into_the_integer() {
    assert_eq!(text(1.75, "# ?/2"), "2    "); // bfb1a4b0 S16
    assert_eq!(text(45000.75, "# ?/2"), "45001    "); // 8e073c5b I13
    assert_eq!(text(0.9999999999, "# ?/4"), "1    "); // 039accab F3
}

#[test]
fn zero_numerator_blanks_the_fraction() {
    assert_eq!(text(1E-5, "# ?/4"), "0    "); // 039accab F4
    assert_eq!(text(61.0, "# ??/16"), "61      "); // 05c1c646 A5
    assert_eq!(text(0.0, "# ?/2"), "0    "); // 43980980 N5
}

#[test]
fn zero_integer_is_left_out_under_sharp() {
    assert_eq!(text(0.5, "# ?/4"), " 2/4"); // 039accab F8
    assert_eq!(text(0.125, "# ?/10"), " 1/10"); // 43980980 P10
}

#[test]
fn minus_leads() {
    assert_eq!(text(-0.5, "# ?/4"), "- 2/4"); // 9a8b7d68 A1
    assert_eq!(text(-1.0 / 7.0, "# ??/16"), "-  2/16"); // 05c1c646 A14
    assert_eq!(text(-1.0 / 7.0, "# ?/2"), "-0    "); // bfb1a4b0 S4
}

#[test]
fn question_mark_pads_the_numerator() {
    assert_eq!(text(1.5, "# ??/16"), "1  8/16"); // 05c1c646 A3
    assert_eq!(text(2.0 / 3.0, "# ??/16"), " 11/16"); // 05c1c646 A19
}

#[test]
fn no_integer_part_puts_the_value_in_the_numerator() {
    assert_eq!(text(1.5, "?/100"), "150/100"); // 05c1c646 D3
    assert_eq!(text(61.0, "?/100"), "6100/100"); // 05c1c646 D5
    assert_eq!(text(-1.0 / 7.0, "?/100"), "-14/100"); // 05c1c646 D14
    assert_eq!(text(0.0, "?/100"), "0/100"); // bfbcd16c Y17
}

#[test]
fn large_integer_part() {
    // TEXT(1E+15,"# ?/4"), 039accab T!A6.
    assert_eq!(text(1E15, "# ?/4"), "1000000000000000    ");
}

#[test]
fn placeholder_denominator_is_the_last_convergent() {
    // 14/99 is closer to PI's fraction part, but 1/7 is the last convergent that fits.
    assert_eq!(text(std::f64::consts::PI, "# ??/??"), "3  1/7 "); // 4c6a49fe G13
    assert_eq!(text(std::f64::consts::PI, "# ???/???"), "3  16/113"); // 05c1c646 C18
                                                                      // 71/90 is closer, the binary fraction part is 0.78900000000430737...
    assert_eq!(text(123_456.789, "# ??/??"), "123456 15/19"); // 20512106 L16
    assert_eq!(text(12.345, "# ???/???"), "12  69/200"); // 05c1c646 C11
    assert_eq!(text(12.345, "?/?"), "37/3"); // 43980980 Z16
}

#[test]
fn placeholder_denominator_without_integer_is_improper() {
    assert_eq!(text(36.5, "?/?"), "73/2"); // 039accab E18
    assert_eq!(text(0.0, "?/?"), "0/1"); // 43980980 Z5
    assert_eq!(text(-1.5, "??/??"), "- 3/2 "); // 039accab U1
}

#[test]
fn placeholder_denominator_padding_and_blanks() {
    assert_eq!(text(1.5, "# ???/???"), "1   1/2  "); // 05c1c646 C3
    assert_eq!(text(999.9999, "# ?/?"), "1000    "); // 0e3d8eba F5
    assert_eq!(text(1.0 / 3.0, "0 ?/?"), "0 1/3"); // 0e3d8eba W4
    assert_eq!(text(-1.0 / 7.0, "0 ?/?"), "-0 1/7"); // 71ad5f93 G7
}

#[test]
fn sharp_numerator_drops_the_separator() {
    assert_eq!(text(0.5, "# #/#"), "1/2"); // 039accab K8
    assert_eq!(text(1_234_567_890.0, "# #/#"), "1234567890"); // 039accab K2
    assert_eq!(text(1.5, "# #/#"), "1 1/2");
}

// The next tests are desktop Excel's cached values in poi/NumberFormatTests.xlsx (Tests!A<row>,
// `=TEXT(C<row>, B<row>)`), with `|` literals around each code.

#[test]
fn literals_around_the_slash() {
    assert_eq!(text(23.75, r"|#\:#=/=#|"), "|23:3=/=4|"); // A134
    assert_eq!(text(-23.75, r"|#\:#=/=#|"), "-|23:3=/=4|"); // A135
    assert_eq!(text(23.75, r"|#\:? ?=/=#|"), "|2:3 3=/=4|"); // A149
    assert_eq!(text(3.75, "|#_#/#|"), "|15 /4|"); // A165
    assert_eq!(text(0.0, "|#_#/#|"), "|0 /1|"); // A169
    assert_eq!(text(-3.75, "|#_?=/=?|"), "-|15 =/=4|"); // A193
}

#[test]
fn zero_padded_denominator_pads_on_the_left() {
    assert_eq!(text(23.75, r"|#\:? ?0#/000"), "|2:3  03/004"); // A160
    assert_eq!(text(0.0, r"|#\:? ?0#/000"), "|:0  00/001"); // A162
    assert_eq!(text(1.0, r"|#\:? ?0#/000"), "|:1  00/001"); // A163
}

#[test]
fn blank_fraction_is_spaces_unless_all_sharp() {
    assert_eq!(text(1.0, r"|#\:#=/=#|"), "|1|"); // A252
    assert_eq!(text(1.0, r"|#\:#=/=?|"), "|1      |"); // A253
    assert_eq!(text(1.0, r"|#\:#=/=0|"), "|1|"); // A254
    assert_eq!(text(1.0, r"|#\:?=/=#|"), "|1      |"); // A255
    assert_eq!(text(1.0, r"|?\:#=/=#|"), "|1      |"); // A261
    assert_eq!(text(1.0, r"|0\:#=/=#|"), "|1|"); // A270
    assert_eq!(text(0.0, r"|?\:#=/=0|"), "|0      |"); // A209
}

#[test]
fn zero_numerator_prints_under_a_zero_placeholder() {
    assert_eq!(text(1.0, r"|#\:0=/=#|"), "|1:0=/=1|"); // A258
    assert_eq!(text(0.0, r"|#\:0=/=#|"), "|0=/=1|"); // A204
    assert_eq!(text(0.0, r"|?\:0=/=#|"), "|0:0=/=1|"); // A213
    assert_eq!(text(0.0, r"|0\:0=/=0|"), "|0:0=/=1|"); // A224
}

#[test]
fn separator_after_a_zero_integer() {
    assert_eq!(text(0.75, r"|#\:#=/=#|"), "|3=/=4|"); // A225
    assert_eq!(text(0.75, r"|#\:?=/=#|"), "| 3=/=4|"); // A228
    assert_eq!(text(0.75, r"|#\:0=/=#|"), "|3=/=4|"); // A231
    assert_eq!(text(0.75, r"|?\:#=/=#|"), "|  3=/=4|"); // A234
    assert_eq!(text(0.75, r"|0\:#=/=#|"), "|0:3=/=4|"); // A243
    assert_eq!(text(-0.75, r"|?\:0=/=0|"), "-|  3=/=4|"); // A296
    assert_eq!(text(0.75, r"|?\:?0=/=#|"), "|   3=/=4|"); // A161
}

/// Engine behavior, not Excel evidence: with no denominator after the literals the `/` is a
/// literal and the section is no fraction.
#[test]
fn slash_without_a_denominator_stays_a_literal() {
    assert_eq!(text(5.0, r#"0 /"x""#), "5 /x");
    assert_eq!(text(5.0, "0=/"), "5=/");
}

/// probes/fraction-numerator-overflow and -overflow2 (excel-graph, T-1740): with no integer part a
/// fixed numerator past 32767 and a placeholder value from 2147483647 on cannot be shown (TEXT
/// `#VALUE!`, displayed text 255 `#`); a whole part takes the rest.
#[test]
fn numerator_past_excels_bound_is_unshowable() {
    let ctx = FormatContext::new(get_locale("en").unwrap());
    let unshowable = |value: f64, format: &str| format_number(value, format, &ctx).unshowable;
    assert_eq!(text(327.67, "?/100"), "32767/100");
    assert!(unshowable(327.68, "?/100"));
    assert!(unshowable(-327.68, "?/100"));
    // 327.675 * 100 rounds to 32768 at 15 digits.
    assert!(unshowable(327.675, "?/100"));
    assert_eq!(text(32767.0, "?/1"), "32767/1");
    assert!(unshowable(32768.0, "?/1"));
    assert_eq!(text(327.68, "# ?/100"), "327 68/100");
    assert_eq!(text(2_147_483_646.0, "?/?"), "2147483646/1");
    assert!(unshowable(2_147_483_647.0, "?/?"));
    assert!(unshowable(-2_147_483_647.0, "??/??"));
    assert_eq!(text(1E15, "# ?/?"), "1000000000000000    ");
    assert_eq!(format_number(1000.0, "?/100", &ctx).text, "#VALUE!");
}

/// probes/fraction-format-features F9, F13 and I10 (excel-graph, T-2074): a negative value whose
/// numerator rounds to zero takes no minus when there is no integer part, and a comma between
/// integer placeholders groups the integer digits.
#[test]
fn zero_numerator_without_integer_has_no_minus_and_integer_groups() {
    assert_eq!(text(-0.001, "?/100"), "0/100");
    assert_eq!(text(-0.001, "?/00"), "0/01");
    assert_eq!(text(-1.0 / 7.0, "# ?/2"), "-0    ");
    assert_eq!(text(1234.5, "#,##0 ?/4"), "1,234 2/4");
    assert_eq!(text(1234567.5, "#,##0 ?/4"), "1,234,567 2/4");
}

/// probes/fraction-overflow-scaled (excel-graph, T-2522): a `,` right after the denominator
/// multiplies by 10^((-3n) mod 256); elsewhere it is a literal or vanishes.
#[test]
fn comma_after_denominator_scales_up_by_a_wrapped_exponent() {
    let ctx = FormatContext::new(get_locale("en").unwrap());
    let unshowable = |value: f64, format: &str| format_number(value, format, &ctx).unshowable;
    assert_eq!(text(1E-250, "?/?,"), "1000/1");
    assert_eq!(text(0.0, "?/?,"), "0/1");
    assert!(unshowable(0.5, "?/?,"));
    assert_eq!(text(1E-250, "?/?,,"), "1/1");
    assert_eq!(text(1E-251, "?/?,,"), "0/1");
    assert_eq!(text(1E-238, "?/?,,,"), "1000000000/1");
    assert_eq!(text(1E-250, "?/4,"), "4000/4");
    assert!(unshowable(2.0, "# ?/4,"));
    assert_eq!(format_number(2.0, "# ?/4,", &ctx).text, "#VALUE!");
    assert_eq!(text(1E-252, "?/?%,"), "0/1%,");
    assert_eq!(text(1E-250, "?/? ,"), "0/1 ,");
    assert_eq!(text(1234567.5, "0, ?/4"), "1234567 2/4");
    assert_eq!(text(0.5, "0%,"), "50%,");
    assert_eq!(text(21474836.469, "?/?%"), "2147483647/1%");
    assert!(unshowable(21474836.47, "?/?%"));
}

/// probes/fraction-large-fixed-denominator and -six-digit-denominator (excel-graph, T-2522): past
/// 32768 a fixed denominator is picked as for placeholders (up to 9999999) and prints the code's
/// digits cut to its length; six digits are #VALUE!.
#[test]
fn fixed_denominator_past_32768_is_picked() {
    let ctx = FormatContext::new(get_locale("en").unwrap());
    assert_eq!(text(0.5, "?/32768"), "16384/32768");
    assert_eq!(text(0.5, "?/32769"), "1/3    ");
    assert_eq!(text(0.8192, "?/40000"), "512/400  ");
    assert_eq!(text(0.8191874, "?/40000"), "4095937/40000");
    assert_eq!(text(1.0 / 9_999_999.0, "?/40000"), "1/40000");
    assert_eq!(text(1E-7, "?/40000"), "0/4    ");
    assert_eq!(text(0.5, "??/40000"), " 1/4    ");
    assert_eq!(text(0.5, "0/40000"), "1/4");
    assert_eq!(text(1.5, "# ?/40000"), "1 1/4    ");
    assert_eq!(format_number(0.5, "?/100000", &ctx).text, "#VALUE!");
}

/// probes/t3711-fraction-trailing-placeholders rows 5 and 9-19 and t4204-fraction-denominator-mixed
/// (excel-graph, T-4204): a denominator's unused placeholders print in place; leading `#`s take no
/// digit; a short denominator leaves out `0`s, then `#`s, then the rightmost `?`s.
#[test]
fn denominator_placeholders_print_in_place() {
    assert_eq!(text(12.5, "# ?/?0"), "12 1/20"); // t3711 A5
    assert_eq!(text(12.5, "# ?/??0"), "12 1/2 0"); // A9
    assert_eq!(text(12.125, "# ?/?0?"), "12 1/80 "); // C16
    assert_eq!(text(12.5, "# ?/??#"), "12 1/2 "); // A17
    assert_eq!(text(12.5, "# ?/?#"), "12 1/2"); // A12
    assert_eq!(text(12.5, "# ?/#0"), "12 1/2"); // A10
    assert_eq!(text(0.5, "# ?/#?"), " 1/2"); // B13
    assert_eq!(text(12.5, "# ?/0?"), "12 1/02"); // A11
    assert_eq!(text(12.125, "# ?/0#"), "12 1/08"); // C14
    assert_eq!(text(0.5, "# ?/00?"), " 1/002"); // B15
    assert_eq!(text(12.5, "# ?/#0#"), "12 1/02"); // A19
    assert_eq!(text(12.0, "# ?/?0"), "12     "); // B5
    assert_eq!(text(12.5, "# ?/#?0"), "12 1/20"); // t4204 A2
    assert_eq!(text(12.5, "# ?/?#?"), "12 1/2 "); // A4
    assert_eq!(text(12.5, "# ?/#?#"), "12 1/2"); // A5
    assert_eq!(text(12.125, "# ?/0??"), "12 1/08 "); // C8
    assert_eq!(text(12.5, "# ?/0#?"), "12 1/02"); // A6
    assert_eq!(text(0.123, "# ?/?0"), " 8/65"); // A9
    assert_eq!(text(std::f64::consts::PI, "# ?/?0"), "3 1/70"); // B9
    assert_eq!(text(0.3, "# ?/?0"), " 3/10"); // C9
    assert_eq!(text(std::f64::consts::PI, "# ?/#0"), "3 1/7"); // B10
    assert_eq!(text(std::f64::consts::PI, "# ?/0?"), "3 1/07"); // B13
    assert_eq!(text(0.123, "# ?/??0"), " 23/187"); // A11
    assert_eq!(text(0.3, "# ?/?0?"), " 3/100"); // C12
}

// probes/t4069-fixed-denominator-tail (T-4069): a placeholder after a fixed denominator makes
// Excel pick the fraction and print the code's digits in place of the picked denominator's.
#[test]
fn a_placeholder_after_a_fixed_denominator_picks_the_fraction() {
    for (value, code, expected) in [
        (12.5, "# ?/4|0", "12 1/4|0"),
        (12.5, "# ?/4", "12 2/4"),
        (12.5, "# ?/8|0", "12 1/8|0"),
        (12.5, "# ?/10|0", "12 1/1 |0"),
        (12.6, "# ?/10|0", "12 3/1 |0"),
        (12.375, "# ?/8|0", "12 3/8|0"),
        (12.5, "?/4|0", "25/4|0"),
        (12.0, "# ?/4|0", "12      "),
    ] {
        assert_eq!(text(value, code), expected, "{code} on {value}");
    }
}

// Placeholders after a fraction print what the denominator hands on: spilled and lacking digits,
// owed zeros and a due space (number-formats.md "Placeholders after a fraction"; excel-graph
// probes/t4043-separated-slots A12, A24, A36, A71, t4043-fixed-denominator-slots, T-4515).
#[test]
fn placeholders_after_a_fraction_take_what_the_denominator_hands_on() {
    assert_eq!(text(0.5, "# ?/4|?|?"), " 1/4| |0");
    assert_eq!(text(0.5, "# ?/4|?|?|#"), " 1/4| |0|00");
    assert_eq!(text(0.5, "# ?/16|?|?"), " 1/1 |0| ");
    assert_eq!(text(0.5, "# ?/100|#|?|?"), " 1/1  ||0|0");
    assert_eq!(text(0.0625, "# ?/4 ? 0"), " 1/4 6 00");
    assert_eq!(text(0.3, "# 0/4|0"), "3/40|0");
    assert_eq!(text(0.5, "# ?/16|#"), " 1/1 |");
    assert_eq!(text(0.3, "# ?/100|##"), " 3/10 |0");
    // A blank fraction's placeholder is as wide as what it would print.
    assert_eq!(text(0.0, "# ?/100|##"), "0       ");
}

// A `,` after the decimal point with a placeholder anywhere after it is inert; after a literal a
// run prints one `,`; after a fraction a `,` does not group (probes/t3598-decimal-part-commas A23,
// A24, A26, t3666-review-followups A7, A10, t4515-fraction-tail-holdout A18, A19, A20, T-4515).
#[test]
fn commas_after_the_point_or_a_fraction() {
    assert_eq!(text(1234567.89, "0.0,,0"), "1234567.89");
    assert_eq!(text(1234567.89, "0.,0"), "1234567.9");
    assert_eq!(text(1234567.89, "0.,"), "1235.");
    assert_eq!(text(1234567.89, "0.00|,0"), "1234567.89|,0");
    assert_eq!(text(1234567.891, "0.0|,,0"), "1234567.8|,9");
    assert_eq!(text(1234567.891, "0.00,|0"), "1234567.89|1");
    assert_eq!(text(1234567.891, "0.0,0,"), "1234.57");
    assert_eq!(text(1234567.89, "# ?/?|#,##0"), "1234567 8/9|0000");
}
