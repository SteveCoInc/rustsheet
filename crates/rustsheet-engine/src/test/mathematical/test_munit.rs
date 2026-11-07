#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_munit_2x2() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=MUNIT(2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "1");
    assert_eq!(model.get_text_a1("B1"), "0");
    assert_eq!(model.get_text_a1("A2"), "0");
    assert_eq!(model.get_text_a1("B2"), "1");
}

#[test]
fn test_munit_3x3() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=MUNIT(3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "1");
    assert_eq!(model.get_text_a1("B1"), "0");
    assert_eq!(model.get_text_a1("C1"), "0");
    assert_eq!(model.get_text_a1("A2"), "0");
    assert_eq!(model.get_text_a1("B2"), "1");
    assert_eq!(model.get_text_a1("C2"), "0");
    assert_eq!(model.get_text_a1("A3"), "0");
    assert_eq!(model.get_text_a1("B3"), "0");
    assert_eq!(model.get_text_a1("C3"), "1");
}

#[test]
fn test_munit_zero_error() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=MUNIT(0)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#VALUE!");
}

#[test]
fn test_munit_negative_error() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=MUNIT(-1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#VALUE!");
}

#[test]
fn test_munit_wrong_arg_count() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=MUNIT()");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#ERROR!");
}

#[test]
fn test_munit_very_large_dimension_error() {
    // A very large dimension must not OOM — it should return an error instead.
    let mut model = new_empty_model();
    model.set_a1("A1", "=MUNIT(1000000)");
    model.evaluate();
    // Sheet limit or size limit hit → error, not a panic/OOM.
    let result = model.get_text_a1("A1");
    assert!(
        result == "#VALUE!" || result == "#ERROR!",
        "expected an error for huge MUNIT, got {result}"
    );
}
