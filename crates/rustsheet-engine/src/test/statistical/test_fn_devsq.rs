#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn arguments_smoke_test() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=DEVSQ()");
    model.set_a1("A2", "=DEVSQ(1, 2, 3)");
    model.set_a1("A3", "=DEVSQ(1, )");
    model.set_a1("A4", "=DEVSQ(1,   , 3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A2"), *"2");
    assert_eq!(model.get_text_a1("A3"), *"0.5"); // empty counts as 0 (probes/stat-order-floor-batch A299, T-1748)
    assert_eq!(model.get_text_a1("A4"), *"4.666666667");
}

#[test]
fn ranges() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=DEVSQ(A2:A8)");
    model.set_a1("A2", "4");
    model.set_a1("A3", "5");
    model.set_a1("A4", "8");
    model.set_a1("A5", "7");
    model.set_a1("A6", "11");
    model.set_a1("A7", "4");
    model.set_a1("A8", "3");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"48");
}

#[test]
fn arrays() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=DEVSQ({1, 2, 3})");
    model.set_a1("A2", "=DEVSQ({1; 2; 3})");
    model.set_a1("A3", "=DEVSQ({1, 2; 3, 4})");
    model.set_a1("A4", "=DEVSQ({1, 2; 3, 4; 5, 6})");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"2");
    assert_eq!(model.get_text_a1("A2"), *"2");
    assert_eq!(model.get_text_a1("A3"), *"5");
    assert_eq!(model.get_text_a1("A4"), *"17.5");
}

// ── whole-column range and a spill beyond the used area ─────────────────────

// A whole-column range is clipped to the sheet's used area before it is
// walked. Inside an anchor, the column may be spilled on demand during the
// walk, past the bound the walk was clipped to. The skipped part is on
// record, so the spill restarts the pass and the second run sees it all.
#[test]
fn whole_column_sees_a_spill_beyond_the_used_area() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=SEQUENCE(1,1,DEVSQ(D:D))");
    model.set_a1("C1", "=SEQUENCE(3,1,10)");
    model.set_a1("D1", "=SEQUENCE(5)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "10");
    assert_eq!(model.evaluation.restarts_in_last_evaluation, 1);

    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "10");
    assert_eq!(model.evaluation.restarts_in_last_evaluation, 0);
}

#[test]
fn typed_text_booleans_and_no_numbers() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=DEVSQ(\"1\",\"2\",2)");
    model.set_a1("B1", "x");
    model.set_a1("B2", "=TRUE");
    model.set_a1("A2", "=DEVSQ(B1:B2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"0.666666667");
    // Excel: no numbers at all is #NUM!, not #DIV/0!.
    assert_eq!(model.get_text_a1("A2"), *"#NUM!");
}
