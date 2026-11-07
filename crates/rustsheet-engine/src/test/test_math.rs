#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_sqrt_arguments() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=SQRT(4)");
    model.set_a1("A2", "=SQRT()");
    model.set_a1("A3", "=SQRT(4, 4)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"2");
    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A3"), *"#ERROR!");
}

#[test]
fn test_fn_sqrtpi_arguments() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=SQRTPI()");
    model.set_a1("A2", "=SQRTPI(4, 4)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");
}

// Excel: probes/min-max-value-args; docs/excel-semantics/coercion.md (T-0217).
#[test]
fn test_fn_min_max_text_value_arguments() {
    let mut model = new_empty_model();
    model.set_a1("B1", "a");
    model.set_a1("B2", "5");
    model.set_a1("A1", "=MIN(\"a\")");
    model.set_a1("A2", "=MAX(CONCATENATE(\"x\"))");
    model.set_a1("A3", "=MIN(\"2\")");
    model.set_a1("A4", "=MAX(B1:B2)");
    model.set_a1("A5", "=MIN(B1)");
    model.set_a1("A6", "=MIN(IFERROR(B1,1))");
    model.set_a1("A7", "=MIN(TRUE,5)");
    model.set_a1("A8", "=MAX(-5,)");
    model.set_a1("A9", "=MIN({TRUE,5})");
    model.set_a1("A10", "=MAX({\"9\",-1})");
    model.set_a1("A11", "=MAX(IF(TRUE,B1),B2)");
    model.set_a1("A12", "=MIN(IFERROR(B9,7),3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A2"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A3"), *"2");
    assert_eq!(model.get_text_a1("A4"), *"5");
    assert_eq!(model.get_text_a1("A5"), *"0");
    assert_eq!(model.get_text_a1("A6"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A7"), *"1");
    assert_eq!(model.get_text_a1("A8"), *"0");
    assert_eq!(model.get_text_a1("A9"), *"5");
    assert_eq!(model.get_text_a1("A10"), *"-1");
    assert_eq!(model.get_text_a1("A11"), *"5");
    assert_eq!(model.get_text_a1("A12"), *"0");
}
