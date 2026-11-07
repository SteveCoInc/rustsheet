#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn computation() {
    let i2 = "=-C2*(1+D2)^E2-F2*((D2+1)*((1+D2)^E2-1))/D2";

    let mut model = new_empty_model();
    model.set_a1("C2", "1");
    model.set_a1("D2", "2");
    model.set_a1("E2", "3");
    model.set_a1("F2", "4");

    model.set_a1("I2", i2);

    model.evaluate();

    assert_eq!(model.get_text_a1("I2"), "-183");
    assert_eq!(model.get_formula_a1("I2"), i2);
}

#[test]
fn format_as_currency() {
    let mut model = new_empty_model();
    model.set_a1("C2", "1");
    model.set_a1("D2", "2");
    model.set_a1("E2", "3");
    model.set_a1("F2", "4");

    model.set_a1("I2", "=FV(D2,E2,F2,C2,1)");

    model.evaluate();

    assert_eq!(model.get_text_a1("I2"), "-$183.00");
}
