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

    model.set_a1("B1", "=AVEDEV(A2:A15)");
    model.evaluate();

    assert_eq!(model.get_text_a1("B1"), *"7.25");
}

#[test]
fn arguments() {
    let mut model = new_empty_model();

    model.set_a1("A1", "=AVEDEV()");
    model.set_a1("A2", "=AVEDEV(1)");
    model.set_a1("A3", "=AVEDEV(1, 2)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A2"), *"0");
    assert_eq!(model.get_text_a1("A3"), *"0.5");
}

// ── whole-column range and a spill beyond the used area ─────────────────────

// A whole-column range is clipped to the sheet's used area before it is
// walked. Inside an anchor, the column may be spilled on demand during the
// walk, past the bound the walk was clipped to. The skipped part is on
// record, so the spill restarts the pass and the second run sees it all.
#[test]
fn whole_column_sees_a_spill_beyond_the_used_area() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=SEQUENCE(1,1,AVEDEV(D:D))");
    model.set_a1("C1", "=SEQUENCE(3,1,10)");
    model.set_a1("D1", "=SEQUENCE(5)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "1.2");
    assert_eq!(model.evaluation.restarts_in_last_evaluation, 1);

    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "1.2");
    assert_eq!(model.evaluation.restarts_in_last_evaluation, 0);
}

#[test]
fn typed_booleans_and_no_numbers() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=AVEDEV(TRUE,FALSE)");
    model.set_a1("B1", "x");
    model.set_a1("A2", "=AVEDEV(B1:B2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"0.5");
    assert_eq!(model.get_text_a1("A2"), *"#NUM!");
}
