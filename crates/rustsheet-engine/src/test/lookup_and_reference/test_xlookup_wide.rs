#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// docs/excel-semantics/lookup.md "XLOOKUP: return_array may be wider than the lookup vector".
fn model() -> crate::Model {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("A3", "3");
    model.set_a1("B1", "a");
    model.set_a1("B2", "b");
    model.set_a1("B3", "c");
    model.set_a1("C1", "x");
    model.set_a1("C2", "y");
    model.set_a1("C3", "z");
    model
}

#[test]
fn wide_return_range_gives_the_matched_row() {
    let mut model = model();
    model.set_a1("E1", "=XLOOKUP(2,A1:A3,B1:C3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("E1"), *"b");
    assert_eq!(model.get_text_a1("F1"), *"y");
}

#[test]
fn different_row_count_is_value_error() {
    let mut model = model();
    model.set_a1("E1", "=XLOOKUP(2,A1:A3,B1:C2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("E1"), *"#VALUE!");
}

#[test]
fn single_column_return_range_is_scalar() {
    let mut model = model();
    model.set_a1("E1", "=XLOOKUP(3,A1:A3,C1:C3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("E1"), *"z");
    assert_eq!(model.get_text_a1("F1"), *"");
}

// Task T-1126: probes/isref-reference-lookups H10 (an array lookup_array, a range return_array).
#[test]
fn array_lookup_array_picks_from_the_return_range() {
    let mut model = model();
    model.set_a1("E1", "=XLOOKUP(2,{1;2;3},B1:B3)");
    model.set_a1("E2", "=ISREF(XLOOKUP(2,{1;2;3},B1:B3))");
    model.set_a1("E3", "=XLOOKUP(9,{1;2;3},B1:B3,\"none\")");
    model.evaluate();
    assert_eq!(model.get_text_a1("E1"), *"b");
    assert_eq!(model.get_text_a1("E2"), *"TRUE");
    assert_eq!(model.get_text_a1("E3"), *"none");
}

#[test]
fn array_lookup_array_with_a_mismatched_return_range_is_value_error() {
    let mut model = model();
    model.set_a1("E1", "=XLOOKUP(3,{1;2;3},B1:B2)");
    model.set_a1("E2", "=XLOOKUP(3,{1;2;3},5)");
    model.evaluate();
    assert_eq!(model.get_text_a1("E1"), *"#VALUE!");
    assert_eq!(model.get_text_a1("E2"), *"#VALUE!");
}

#[test]
fn one_cell_lookup_vector_takes_its_orientation_from_the_return() {
    // probes/xlookup-row-column-picks E9, E23 (T-0962).
    let mut model = model();
    model.set_a1("E1", "=XLOOKUP(1,A1:A1,B1:C1)");
    model.set_a1("E3", "=XLOOKUP(1,A1:A1,B1:B3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("E1"), *"a");
    assert_eq!(model.get_text_a1("F1"), *"x");
    assert_eq!(model.get_text_a1("E3"), *"a");
    assert_eq!(model.get_text_a1("E4"), *"b");
    assert_eq!(model.get_text_a1("E5"), *"c");
}

#[test]
fn array_returns_give_the_matched_row_or_column_and_at_intersects() {
    // probes/xlookup-row-column-picks E4, E13, E28, E31 (T-0962).
    let mut model = model();
    model.set_a1(
        "E1",
        "=XLOOKUP(2,A1:A3,{\"p\",\"q\";\"r\",\"s\";\"t\",\"u\"})",
    );
    model.set_a1("E3", "=XLOOKUP(\"b\",{\"a\",\"b\"},{1,2;3,4})");
    model.set_a1("E6", "=@XLOOKUP(2,A1:A3,B1:C3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("E1"), *"r");
    assert_eq!(model.get_text_a1("F1"), *"s");
    assert_eq!(model.get_text_a1("E3"), *"2");
    assert_eq!(model.get_text_a1("E4"), *"4");
    assert_eq!(model.get_text_a1("E6"), *"#VALUE!");
}
