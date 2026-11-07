#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn simple_example() {
    let mut model = new_empty_model();

    model.set_a1("A1", "220");
    model.set_a1("B1", "=A1*10%");
    model.set_a1("C1", "=SIN(A1)%");

    model.evaluate();

    assert_eq!(model.get_formula_a1("B1"), *"=A1*10%");
    assert_eq!(model.get_text_a1("B1"), *"22");
    assert_eq!(model.get_formula_a1("C1"), *"=SIN(A1)%");
    assert_eq!(model.get_text_a1("C1"), *"0.000883987");
}

// A subnormal result is 0 (numbers.md "Subnormal results become 0", T-0988).
#[test]
fn subnormal_result_is_zero() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=2.2250738585072E-308%");
    model.set_a1("A2", "=1E-300%");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"0");
    assert_eq!(model.get_text_a1("A2"), *"1E-302");
}
