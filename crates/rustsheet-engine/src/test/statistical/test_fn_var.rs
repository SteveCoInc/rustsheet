#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn smoke_test() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=STDEV.P(10, 12, 23, 23, 16, 23, 21)");
    model.set_a1("A2", "=STDEV.S(10, 12, 23, 23, 16, 23, 21)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"5.174505793");

    assert_eq!(model.get_text_a1("A2"), *"5.589105048");
}

#[test]
fn numbers() {
    let mut model = new_empty_model();

    model.set_a1("A2", "24");
    model.set_a1("A3", "25");
    model.set_a1("A4", "27");
    model.set_a1("A5", "23");
    model.set_a1("A6", "45");
    model.set_a1("A7", "23.5");
    model.set_a1("A8", "34");
    model.set_a1("A9", "23");
    model.set_a1("A10", "23");
    model.set_a1("A11", "TRUE");
    model.set_a1("A12", "'23");
    model.set_a1("A13", "Text");
    model.set_a1("A14", "FALSE");
    model.set_a1("A15", "45");

    model.set_a1("B1", "=VAR.P(A2:A15)");
    model.set_a1("B2", "=VAR.S(A2:A15)");
    model.set_a1("B3", "=VARA(A2:A15)");
    model.set_a1("B4", "=VARPA(A2:A15)");
    model.evaluate();

    assert_eq!(model.get_text_a1("B1"), *"71.9625");
    assert_eq!(model.get_text_a1("B2"), *"79.95833333");
    assert_eq!(model.get_text_a1("B3"), *"240.2486264");
    assert_eq!(model.get_text_a1("B4"), *"223.0880102");
}

// ── whole-column range and a spill beyond the used area ─────────────────────

// A whole-column range is clipped to the sheet's used area before it is
// walked. Inside an anchor, the column may be spilled on demand during the
// walk, past the bound the walk was clipped to. The skipped part is on
// record, so the spill restarts the pass and the second run sees it all.
#[test]
fn whole_column_sees_a_spill_beyond_the_used_area() {
    let cases = [
        ("VAR.P(D:D)", "2"),
        ("VAR.S(D:D)", "2.5"),
        ("VARA(D:D)", "2.5"),
        ("VARPA(D:D)", "2"),
    ];
    for (formula, expected) in cases {
        let mut model = new_empty_model();
        model.set_a1("A1", &format!("=SEQUENCE(1,1,{formula})"));
        model.set_a1("C1", "=SEQUENCE(3,1,10)");
        model.set_a1("D1", "=SEQUENCE(5)");
        model.evaluate();
        assert_eq!(model.get_text_a1("A1"), expected, "{formula}");
        assert_eq!(model.evaluation.restarts_in_last_evaluation, 1, "{formula}");
    }
}

#[test]
fn typed_text_and_booleans() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=VAR(\"1\",\"2\")");
    model.set_a1("A2", "=VARP(\"1\",\"2\")");
    model.set_a1("A3", "=VAR(TRUE,FALSE,TRUE)");
    model.set_a1("B1", "=TRUE");
    model.set_a1("B2", "x");
    model.set_a1("A4", "=VAR(B1:B2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"0.5");
    assert_eq!(model.get_text_a1("A2"), *"0.25");
    assert_eq!(model.get_text_a1("A3"), *"0.333333333");
    assert_eq!(model.get_text_a1("A4"), *"#DIV/0!");
}
