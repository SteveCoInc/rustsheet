#![allow(clippy::unwrap_used)]

//! SPEC-0003 rollout E1: Excel's newer error kinds in formulas and typed input.

use crate::expressions::token::Error;
use crate::test::util::new_empty_model;
use crate::types::CellType;

#[test]
fn getting_data_literal_evaluates_with_its_own_code() {
    // probes/error-literals-new A2, A4, A15 (oracle: excel-graph).
    let mut model = new_empty_model();
    model.set_a1("A1", "=ERROR.TYPE(#GETTING_DATA)");
    model.set_a1("A2", "=ISNA(#getting_data)");
    model.set_a1("A3", "=IFNA(#GETTING_DATA,\"y\")");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "8");
    assert_eq!(model.get_text_a1("A2"), "FALSE");
    assert_eq!(model.get_text_a1("A3"), "#GETTING_DATA");
}

#[test]
fn blocked_is_not_a_formula_literal() {
    // Excel refuses `=#BLOCKED!` (T-0175); it must not become a #BLOCKED! value.
    let mut model = new_empty_model();
    model.set_a1("A1", "=#BLOCKED!");
    model.evaluate();
    // A formula that does not parse evaluates to the engine's #ERROR!.
    assert_eq!(model.get_text_a1("A1"), "#ERROR!");
}

#[test]
fn typed_getting_data_is_an_error_and_typed_blocked_is_text() {
    // SPEC-0003's choice, UNVERIFIED until desktop evidence (T-1016).
    let mut model = new_empty_model();
    model.set_a1("A1", "#getting_data");
    model.set_a1("A2", "#BLOCKED!");
    model.evaluate();
    assert_eq!(model.get_cell_type(0, 1, 1), Ok(CellType::ErrorValue));
    assert_eq!(model.get_text_a1("A1"), "#GETTING_DATA");
    assert_eq!(model.get_cell_type(0, 2, 1), Ok(CellType::Text));
    assert_eq!(model.get_text_a1("A2"), "#BLOCKED!");
}

#[test]
fn busy_is_neither_a_formula_literal_nor_typed_input() {
    // Excel refuses `=#BUSY!` (syntax.md "Newer error literals"); typed `#BUSY!` stays text
    // (SPEC-0003, UNVERIFIED until desktop evidence, T-1016).
    let mut model = new_empty_model();
    model.set_a1("A1", "=#BUSY!");
    model.set_a1("A2", "#BUSY!");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#ERROR!");
    assert_eq!(model.get_cell_type(0, 2, 1), Ok(CellType::Text));
    assert_eq!(model.get_text_a1("A2"), "#BUSY!");
}

#[test]
fn stored_busy_is_an_ordinary_error_with_code_zero() {
    // probes/error-stored-constants S!B1:I1 (oracle: excel-graph).
    let mut model = new_empty_model();
    model.update_cell_with_error(0, 1, 1, Error::BUSY).unwrap();
    model.set_a1("B1", "=ERROR.TYPE(A1)");
    model.set_a1("C1", "=ISERROR(A1)");
    model.set_a1("D1", "=A1&\"\"");
    model.set_a1("E1", "=ISERR(A1)");
    model.set_a1("F1", "=ISNA(A1)");
    model.set_a1("G1", "=TYPE(A1)");
    model.set_a1("H1", "=ERROR.TYPE(A1)+100");
    model.set_a1("I1", "=IFNA(A1,\"y\")");
    model.evaluate();
    let expected = [
        ("A1", "#BUSY!"),
        ("B1", "0"),
        ("C1", "TRUE"),
        ("D1", "#BUSY!"),
        ("E1", "TRUE"),
        ("F1", "FALSE"),
        ("G1", "16"),
        ("H1", "100"),
        ("I1", "#BUSY!"),
    ];
    for (cell, text) in expected {
        assert_eq!(model.get_text_a1(cell), text, "{cell}");
    }
    assert_eq!(model.get_cell_type(0, 1, 4), Ok(CellType::ErrorValue));
}

#[test]
fn unique_keeps_busy_apart_from_null() {
    // Pins the engine: code 0 is its own `CellKey::Error` key (SPEC-0003 "Stored `#BUSY!`").
    // Whether Excel's UNIQUE keeps the two apart is UNVERIFIED (T-1228).
    let mut model = new_empty_model();
    model.update_cell_with_error(0, 1, 1, Error::BUSY).unwrap();
    model.update_cell_with_error(0, 2, 1, Error::NULL).unwrap();
    model.update_cell_with_error(0, 3, 1, Error::BUSY).unwrap();
    model.set_a1("B1", "=ROWS(UNIQUE(A1:A3))");
    model.set_a1("C1", "=UNIQUE(A1:A3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("B1"), "2");
    assert_eq!(model.get_text_a1("C1"), "#BUSY!");
    assert_eq!(model.get_text_a1("C2"), "#NULL!");
}
