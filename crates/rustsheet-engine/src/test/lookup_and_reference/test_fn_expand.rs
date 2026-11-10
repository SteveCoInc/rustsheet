#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_expand_add_rows() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("C1", "=EXPAND(A1:A2,4)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), "1");
    assert_eq!(model.get_text_a1("C2"), "2");
    assert_eq!(model.get_text_a1("C3"), "#N/A");
    assert_eq!(model.get_text_a1("C4"), "#N/A");
}

#[test]
fn test_expand_add_rows_and_cols_with_pad() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("C1", "=EXPAND(A1:A2,3,2,0)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), "1");
    assert_eq!(model.get_text_a1("D1"), "0");
    assert_eq!(model.get_text_a1("C2"), "2");
    assert_eq!(model.get_text_a1("D2"), "0");
    assert_eq!(model.get_text_a1("C3"), "0");
    assert_eq!(model.get_text_a1("D3"), "0");
}

#[test]
fn test_expand_shrink_rows_error() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("A3", "3");
    model.set_a1("C1", "=EXPAND(A1:A3,2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), "#VALUE!");
}

#[test]
fn test_expand_same_size() {
    let mut model = new_empty_model();
    model.set_a1("A1", "7");
    model.set_a1("B1", "8");
    model.set_a1("D1", "=EXPAND(A1:B1,1,2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("D1"), "7");
    assert_eq!(model.get_text_a1("E1"), "8");
}
