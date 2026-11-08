#![allow(clippy::unwrap_used)]

use crate::cell::CellValue;
use crate::test::util::new_empty_model;

#[test]
fn fn_arguments() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=CSC()");
    model.set_a1("A2", "=SEC()");
    model.set_a1("A3", "=COT()");

    model.set_a1("A4", "=CSCH()");
    model.set_a1("A5", "=SECH()");
    model.set_a1("A6", "=COTH()");

    model.set_a1("A7", "=ACOT()");
    model.set_a1("A8", "=ACOTH()");

    model.set_a1("B1", "=CSC(1, 2)");
    model.set_a1("B2", "=SEC(1, 2)");
    model.set_a1("B3", "=COT(1, 2)");

    model.set_a1("B4", "=CSCH(1, 2)");
    model.set_a1("B5", "=SECH(1, 2)");
    model.set_a1("B6", "=COTH(1, 2)");

    model.set_a1("B7", "=ACOT(1, 2)");
    model.set_a1("B8", "=ACOTH(1, 2)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A3"), *"#ERROR!");

    assert_eq!(model.get_text_a1("A4"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A5"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A6"), *"#ERROR!");

    assert_eq!(model.get_text_a1("A7"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A8"), *"#ERROR!");

    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B3"), *"#ERROR!");

    assert_eq!(model.get_text_a1("B4"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B5"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B6"), *"#ERROR!");

    assert_eq!(model.get_text_a1("B7"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B8"), *"#ERROR!");
}

#[test]
fn reciprocals_divide_like_the_operator() {
    // probes/trig-reciprocal-special-angles (excel-graph): 1 over the rounded TAN, SIN or COS,
    // rounded to 64 bits and then to a double, like `1/TAN(PI()/4)`.
    let mut model = new_empty_model();
    model.set_a1("A1", "=COT(PI()/4)");
    model.set_a1("A2", "=CSC(PI()/6)");
    model.set_a1("A3", "=SEC(PI()/3)");
    model.set_a1("A4", "=COT(3*PI()/4)");
    model.evaluate();
    let bits = |c: &str| match model.get_cell_value_by_ref(&format!("Sheet1!{c}")) {
        Ok(CellValue::Number(n)) => n.to_bits(),
        other => panic!("expected number, got {other:?}"),
    };
    assert_eq!(bits("A1"), 1.0_f64.to_bits());
    assert_eq!(bits("A2"), 2.0_f64.to_bits());
    assert_eq!(bits("A3"), 1.999_999_999_999_999_6_f64.to_bits());
    assert_eq!(bits("A4"), (-0.999_999_999_999_999_8_f64).to_bits());
}

/// ACOT of a negative-zero double is PI()/2 like ACOT(0) (T-2376).
#[test]
fn acot_of_negative_zero() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=ACOT(-(0))");
    model.set_a1("A2", "=ACOT(CEILING.PRECISE(-0.0001))");
    model.set_a1("A3", "=ACOT(INT(-0.0))");
    model.set_a1("A4", "=ACOT(0)");
    model.evaluate();
    for c in ["A1", "A2", "A3", "A4"] {
        match model.get_cell_value_by_ref(&format!("Sheet1!{c}")) {
            Ok(CellValue::Number(n)) => {
                assert_eq!(n.to_bits(), std::f64::consts::FRAC_PI_2.to_bits(), "{c}");
            }
            other => panic!("expected number, got {other:?}"),
        }
    }
}
