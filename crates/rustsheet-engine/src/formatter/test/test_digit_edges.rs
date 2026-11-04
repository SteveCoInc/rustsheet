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

#[test]
fn commas_after_the_decimal_point_neither_group_nor_scale() {
    // probes/t3598-decimal-part-commas.
    assert_eq!(text(1_234_567.89, "0.00|#,##0"), "1234567.89|0");
    assert_eq!(text(1_234_567.89, "0.0,,0"), "1234567.89");
    assert_eq!(text(1_234_567.89, "#,##0.0,0"), "1,234,567.89");
    assert_eq!(text(1_234_567.89, "0.00|,0"), "1234567.89|,0");
    assert_eq!(text(1_234_567.89, "0.00|0,"), "1234.56|8");
}
