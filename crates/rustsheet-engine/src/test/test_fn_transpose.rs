#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn transpose_column_to_row() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("A3", "3");
    model.set_a1("B1", "=TRANSPOSE(A1:A3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("B1"), "1");
    assert_eq!(model.get_text_a1("C1"), "2");
    assert_eq!(model.get_text_a1("D1"), "3");
    assert_eq!(model.get_text_a1("B2"), "");
}

#[test]
fn transpose_row_to_column() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("B1", "2");
    model.set_a1("C1", "3");
    model.set_a1("A2", "=TRANSPOSE(A1:C1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A2"), "1");
    assert_eq!(model.get_text_a1("A3"), "2");
    assert_eq!(model.get_text_a1("A4"), "3");
    assert_eq!(model.get_text_a1("B2"), "");
}

#[test]
fn transpose_2d_matrix() {
    let mut model = new_empty_model();
    // 2×3 matrix:
    // 1 2 3
    // 4 5 6
    model.set_a1("A1", "1");
    model.set_a1("B1", "2");
    model.set_a1("C1", "3");
    model.set_a1("A2", "4");
    model.set_a1("B2", "5");
    model.set_a1("C2", "6");
    model.set_a1("E1", "=TRANSPOSE(A1:C2)");
    model.evaluate();
    // Result should be 3×2:
    // 1 4
    // 2 5
    // 3 6
    assert_eq!(model.get_text_a1("E1"), "1");
    assert_eq!(model.get_text_a1("F1"), "4");
    assert_eq!(model.get_text_a1("E2"), "2");
    assert_eq!(model.get_text_a1("F2"), "5");
    assert_eq!(model.get_text_a1("E3"), "3");
    assert_eq!(model.get_text_a1("F3"), "6");
    assert_eq!(model.get_text_a1("G1"), "");
}

#[test]
fn transpose_wrong_arg_count() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=TRANSPOSE()");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#ERROR!");
}

// A single value transposes to that value, not a 1x1 array, so a lifting function over it does
// not lift; a 1x1 array, constant or computed, stays an array (probes/t2455-transpose-single-value
// A12:A16, A70:A72, docs/excel-semantics/text.md, T-2455).
#[test]
fn transpose_of_a_single_value_is_that_value() {
    let mut model = new_empty_model();
    model.set_a1("B1", "5");
    model.set_a1("B2", "a,b");
    model.set_a1("A1", r#"=COLUMNS(TEXTSPLIT(TRANSPOSE("a,b"),","))"#);
    model.set_a1("A2", r#"=COLUMNS(TEXTSPLIT(TRANSPOSE(B2),","))"#);
    model.set_a1("A3", r#"=COLUMNS(TEXTSPLIT(TRANSPOSE(B2:B2),","))"#);
    model.set_a1("A4", "=ISREF(TRANSPOSE(B1))");
    model.set_a1("A5", r#"=COLUMNS(TEXTSPLIT(TRANSPOSE({"a,b"}),","))"#);
    model.set_a1(
        "A6",
        r#"=COLUMNS(TEXTSPLIT(TRANSPOSE(IF(TRUE,{"a,b"})),","))"#,
    );
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "2");
    assert_eq!(model.get_text_a1("A2"), "2");
    assert_eq!(model.get_text_a1("A3"), "2");
    assert_eq!(model.get_text_a1("A4"), "FALSE");
    assert_eq!(model.get_text_a1("A5"), "1");
    assert_eq!(model.get_text_a1("A6"), "1");
}
