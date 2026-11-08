// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![cfg_attr(test, expect(clippy::unreadable_literal))]
// LINT-DEBT-END
use super::text;

// T-1738: `?` pads an empty decimal with spaces and keeps the point on an integer (gen-m10,
// excel-graph 2026-10-16: 1000 is "1000.  ", 61 is "61.  ").
#[test]
fn question_decimals_keep_the_point_on_an_integer() {
    assert_eq!(text(1000.0, "?.??"), "1000.  ");
    assert_eq!(text(61.0, "?.??"), "61.  ");
}

// T-1738: `#%` of 0.0049 is "%": 0.49 rounds to 0 and `#` prints no zero (gen-m10).
#[test]
fn hash_percent_of_a_value_rounding_to_zero_is_the_sign_alone() {
    assert_eq!(text(0.0049, "#%"), "%");
}

// T-4595: a literal right after the decimal point prints after the point, not ahead of it
// (probes/t4515-fresh-sweep A15:D15, probes/t4595-literal-after-point A1:C4, excel-graph 2026-11-06).
#[test]
fn literal_right_after_the_point_prints_after_it() {
    assert_eq!(text(1234567.891, "0.,|0"), "1234567.|9");
    assert_eq!(text(-0.5, "0.,|0"), "-0.|5");
    assert_eq!(text(98765.4321, "0.,|0"), "98765.|4");
    assert_eq!(text(0.0, "0.,|0"), "0.|0");
    for (value, cells) in [
        (
            1234567.891,
            ["1234567.|9", "1234567.x9", "1234567.-9", "1234568.|"],
        ),
        (-0.5, ["-0.|5", "-0.x5", "-.-5", "-1.|"]),
        (0.0, ["0.|0", "0.x0", ".-", "0.|"]),
    ] {
        for (format, expected) in ["0.|0", "0.\"x\"0", "#.\\-#", "0.|"].iter().zip(cells) {
            assert_eq!(text(value, format), expected, "{format} on {value}");
        }
    }
}
