#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn arguments() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=SUMSQ()");
    model.set_a1("A2", "=SUMSQ(2)");
    model.set_a1("A3", "=SUMSQ(1, 2)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A2"), *"4");
    assert_eq!(model.get_text_a1("A3"), *"5");
}

// ── whole-column range and a spill beyond the used area ─────────────────────

// A whole-column range is clipped to the sheet's used area before it is
// walked. Inside an anchor, the column may be spilled on demand during the
// walk, past the bound the walk was clipped to. The skipped part is on
// record, so the spill restarts the pass and the second run sees it all.
#[test]
fn whole_column_sees_a_spill_beyond_the_used_area() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=SEQUENCE(1,1,SUMSQ(D:D))");
    model.set_a1("C1", "=SEQUENCE(3,1,10)");
    model.set_a1("D1", "=SEQUENCE(5)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "55");
    assert_eq!(model.evaluation.restarts_in_last_evaluation, 1);

    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "55");
    assert_eq!(model.evaluation.restarts_in_last_evaluation, 0);
}

#[test]
fn typed_text_and_booleans_count_but_range_ones_do_not() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=SUMSQ(TRUE,TRUE,FALSE)");
    model.set_a1("A2", "=SUMSQ(\"2\",\"3\",4)");
    model.set_a1("B1", "=TRUE");
    model.set_a1("B2", "x");
    model.set_a1("B3", "5");
    model.set_a1("A3", "=SUMSQ(B1:B3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"2");
    assert_eq!(model.get_text_a1("A2"), *"29");
    assert_eq!(model.get_text_a1("A3"), *"25");
}
