#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_minifs_arguments() {
    let mut model = new_empty_model();

    // Incorrect number of arguments
    model.set_a1("A1", "=MINIFS()");
    model.set_a1("A2", "=MINIFS(B2:B9)");
    model.set_a1("A3", "=MINIFS(B2:B9,C2:C9)");
    model.set_a1("A4", "=MINIFS(B2:B9,C2:C9,\"=A*\",D2:D9)");

    // Correct (Sum everything in column 'B' if column 'C' starts with "A")
    model.set_a1("A5", "=MINIFS(B2:B9,C2:C9,\"=A*\")");

    // Data
    model.set_a1("B2", "5");
    model.set_a1("B3", "4");
    model.set_a1("B4", "15");
    model.set_a1("B5", "22");
    model.set_a1("B6", "=NA()");
    model.set_a1("C2", "Apples");
    model.set_a1("C3", "Bananas");
    model.set_a1("C4", "Almonds");
    model.set_a1("C5", "Yoni");
    model.set_a1("C6", "Mandarin");

    model.evaluate();

    // Error (Incorrect number of arguments)
    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A3"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A4"), *"#ERROR!");

    // Correct
    assert_eq!(model.get_text_a1("A5"), *"5");
}

// ── whole-column range and a spill beyond the used area ─────────────────────

// A whole-column range is clipped to the sheet's used area before it is
// walked. Inside an anchor, the column may be spilled on demand during the
// walk, past the bound the walk was clipped to. The skipped part is on
// record, so the spill restarts the pass and the second run sees it all.
#[test]
fn test_fn_minifs_whole_column_sees_a_spill_beyond_the_used_area() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=SEQUENCE(1,1,MINIFS(D:D,D:D,\">2\"))");
    model.set_a1("C1", "=SEQUENCE(3,1,10)");
    model.set_a1("D1", "=SEQUENCE(5)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "3");
    assert_eq!(model.evaluation.restarts_in_last_evaluation, 1);

    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "3");
    assert_eq!(model.evaluation.restarts_in_last_evaluation, 0);
}
