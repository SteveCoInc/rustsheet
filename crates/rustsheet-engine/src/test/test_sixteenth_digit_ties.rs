#![allow(clippy::unwrap_used)]

//! An exact tie at the 16th significant digit splits by use (T-4848; numbers.md "A 16th-digit
//! tie splits by use"): comparisons cut it away from zero, text cuts it toward zero
//! (probes/t4848-sixteenth-digit-ties, excel-graph, 2026-11-07).

use crate::number_format::to_excel_precision_str;
use crate::test::util::new_empty_model;

#[test]
fn comparisons_break_a_sixteenth_digit_tie_away_from_zero() {
    let mut model = new_empty_model();
    model.set_a1("B1", "=1E+15+25");
    model.set_a1("B2", "=1E+14+2.5");
    model.set_a1("B3", "=-(1E+15+25)");
    model.set_a1("C1", "=B1=(1E+15+30)");
    model.set_a1("C2", "=B1=(1E+15+20)");
    model.set_a1("C3", "=B2=(1E+14+3)");
    model.set_a1("C4", "=B3=-(1E+15+30)");
    model.set_a1("C5", "=B1<(1E+15+30)");
    model.set_a1("C6", "=COUNTIF(B1,1E+15+30)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), *"TRUE");
    assert_eq!(model.get_text_a1("C2"), *"FALSE");
    assert_eq!(model.get_text_a1("C3"), *"TRUE");
    assert_eq!(model.get_text_a1("C4"), *"TRUE");
    assert_eq!(model.get_text_a1("C5"), *"FALSE");
    assert_eq!(model.get_text_a1("C6"), *"1");
}

#[test]
fn text_breaks_a_sixteenth_digit_tie_toward_zero() {
    let mut model = new_empty_model();
    model.set_a1("B1", "=1E+15+15");
    model.set_a1("C1", "=EXACT(B1,1E+15+10)");
    model.set_a1("C2", "=EXACT(B1,1E+15+20)");
    model.set_a1("C3", "=B1&\"\"");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), *"TRUE");
    assert_eq!(model.get_text_a1("C2"), *"FALSE");
    assert_eq!(model.get_text_a1("C3"), *"1000000000000010");
    // The cell text and a printed literal cut the same way.
    assert_eq!(to_excel_precision_str(1e15 + 15.0), "1000000000000010");
    assert_eq!(to_excel_precision_str(-(1e14 + 1.5)), "-100000000000001");
}
