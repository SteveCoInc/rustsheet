#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use crate::test::util::new_empty_model;
use crate::types::{Cell, FormulaValue};

#[test]
fn test_simple_error_propagation() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=1/0");
    model.set_a1("A2", "=2+A1");
    model.set_a1("A3", "=C2+A2");
    model.evaluate();
    match model.get_cell_a1("Sheet1!A3") {
        Cell::CellFormula {
            v: FormulaValue::Error { o, .. },
            ..
        } => {
            assert_eq!(o, "Sheet1!A1");
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_simple_errors() {
    let mut model = new_empty_model();
    model.set_a1("A1", "#CALC!");
    model.set_a1("A2", "#SPILL!");
    model.set_a1("A3", "#OTHER!");

    model.set_a1("B1", "=ISERROR(A1)");
    model.set_a1("B2", "=ISERROR(A2)");
    model.set_a1("B3", "=ISERROR(A3)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), "#CALC!");
    assert_eq!(model.get_text_a1("A2"), "#SPILL!");
    assert_eq!(model.get_text_a1("A3"), "#OTHER!");
    assert_eq!(model.get_text_a1("B1"), "TRUE");
    assert_eq!(model.get_text_a1("B2"), "TRUE");
    assert_eq!(model.get_text_a1("B3"), "FALSE");
}
