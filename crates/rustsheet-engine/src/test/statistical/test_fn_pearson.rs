#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_chisq_test_smoke() {
    let mut model = new_empty_model();
    model.set_a1("A2", "48");
    model.set_a1("A3", "32");
    model.set_a1("A4", "12");
    model.set_a1("A5", "1");
    model.set_a1("A6", "'13");
    model.set_a1("A7", "TRUE");
    model.set_a1("A8", "1");
    model.set_a1("A9", "13");
    model.set_a1("A10", "15");

    model.set_a1("B2", "55");
    model.set_a1("B3", "34");
    model.set_a1("B4", "13");
    model.set_a1("B5", "blah");
    model.set_a1("B6", "13");
    model.set_a1("B7", "1");
    model.set_a1("B8", "TRUE");
    model.set_a1("B9", "'14");
    model.set_a1("B10", "16");

    model.set_a1("C1", "=PEARSON(A2:A10, B2:B10)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), *"0.998381439");
}
