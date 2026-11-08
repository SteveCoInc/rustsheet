#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn iserr_lifts_over_range_and_excludes_na() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=NA()");
    model.set_a1("A2", "=1/0");
    model.set_a1("A3", "5");
    model.set_a1("B1", "=ISERR(A1:A3)");
    model.set_a1("C1", "=ISNA(A1:A3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("B1"), *"FALSE");
    assert_eq!(model.get_text_a1("B2"), *"TRUE");
    assert_eq!(model.get_text_a1("B3"), *"FALSE");
    assert_eq!(model.get_text_a1("C1"), *"TRUE");
    assert_eq!(model.get_text_a1("C2"), *"FALSE");
}

#[test]
fn full_sheet_range_is_bounded() {
    let mut model = new_empty_model();
    // DEVIATION (decision 0014): Excel spills this; we refuse above the shared array bound.
    model.set_a1("A1", "=ISBLANK(A:XFD)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
}

#[test]
fn parity_counts_a_value_within_1e_10_below_an_integer_as_that_integer() {
    // probes/iseven-isodd-rounding-threshold and -rounding-window (excel-graph, T-2382).
    let mut model = new_empty_model();
    model.set_a1("A1", "=ISODD(7-1E-10)");
    model.set_a1("A2", "=ISODD(7-1.01E-10)");
    model.set_a1("A3", "=ISODD(-(7-1E-10))");
    model.set_a1("A4", "=ISEVEN(POWER(49,0.5-1E-15))");
    model.set_a1("A5", "=ISODD(1E+15-0.25)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"TRUE");
    assert_eq!(model.get_text_a1("A2"), *"FALSE");
    assert_eq!(model.get_text_a1("A3"), *"TRUE");
    assert_eq!(model.get_text_a1("A4"), *"FALSE");
    assert_eq!(model.get_text_a1("A5"), *"TRUE");
}
