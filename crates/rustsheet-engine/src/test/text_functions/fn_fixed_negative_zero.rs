#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

/// FIXED prints no minus sign for a value that rounds to zero (docs/excel-semantics/text.md,
/// gen-m6/f94b5b8236742618 A24 for the first, probes/dollar-fixed-unichar-edges A49, A50 for the
/// decimals shapes).
#[test]
fn fixed_of_a_negative_value_rounding_to_zero_has_no_sign() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=FIXED(-2,-1)");
    model.set_a1("A2", "=FIXED(-2,-1,TRUE)");
    model.set_a1("A3", "=FIXED(-6,-1)");
    model.set_a1("A4", "=FIXED(-0.004,2)");
    model.set_a1("A5", "=FIXED(-0.4,0)");
    model.set_a1("A6", "=FIXED(-0.005,2)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"0");
    assert_eq!(model.get_text_a1("A2"), *"0");
    assert_eq!(model.get_text_a1("A3"), *"-10");
    assert_eq!(model.get_text_a1("A4"), *"0.00");
    assert_eq!(model.get_text_a1("A5"), *"0");
    // Not evidenced (UNVERIFIED): a value that rounds up to 0.01 keeps its sign.
    assert_eq!(model.get_text_a1("A6"), *"-0.01");
}
