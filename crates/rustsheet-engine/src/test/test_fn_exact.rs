#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn fn_exact_args_number() {
    let mut model = new_empty_model();

    model.set_a1("A1", "=EXACT(1)");
    model.set_a1("A2", "=EXACT(1, 1, 1)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");
}

#[test]
fn fn_exact() {
    let mut model = new_empty_model();

    model.set_a1("A1", "=EXACT(2.3, 2.3)");
    model.set_a1("A2", r#"=EXACT(2.3, "2.3")"#);
    model.set_a1("A3", r#"=EXACT("Hello", "hello")"#);

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"TRUE");
    assert_eq!(model.get_text_a1("A2"), *"TRUE");
    assert_eq!(model.get_text_a1("A3"), *"FALSE");
}

#[test]
fn single_cells() {
    let mut model = new_empty_model();
    model.set_a1("A1", "Hello");
    model.set_a1("A2", "hello");
    model.set_a1("A3", "Hello");
    model.set_a1("B1", "=EXACT(A1, A2)");
    model.set_a1("B2", "=EXACT(A1, A3)");
    // An empty cell is the empty string
    model.set_a1("B3", "=EXACT(A4, \"\")");
    model.set_a1("B4", "=EXACT(TRUE, \"TRUE\")");
    model.set_a1("B5", "=EXACT(1/0, \"a\")");
    model.evaluate();

    assert_eq!(model.get_text_a1("B1"), *"FALSE");
    assert_eq!(model.get_text_a1("B2"), *"TRUE");
    assert_eq!(model.get_text_a1("B3"), *"TRUE");
    assert_eq!(model.get_text_a1("B4"), *"TRUE");
    assert_eq!(model.get_text_a1("B5"), *"#DIV/0!");
}

#[test]
fn range_and_text() {
    let mut model = new_empty_model();
    model.set_a1("A1", "he");
    model.set_a1("A2", "He");
    model.set_a1("A3", "HE");
    model.set_a1("C1", "=EXACT(A1:A4, \"He\")");
    model.set_a1("D1", "=EXACT(\"he\", A1:A4)");
    model.evaluate();

    assert_eq!(model.get_text_a1("C1"), *"FALSE");
    assert_eq!(model.get_text_a1("C2"), *"TRUE");
    assert_eq!(model.get_text_a1("C3"), *"FALSE");
    assert_eq!(model.get_text_a1("C4"), *"FALSE");
    assert_eq!(model.get_text_a1("C5"), *"");

    assert_eq!(model.get_text_a1("D1"), *"TRUE");
    assert_eq!(model.get_text_a1("D2"), *"FALSE");
    assert_eq!(model.get_text_a1("D3"), *"FALSE");
    assert_eq!(model.get_text_a1("D4"), *"FALSE");
}

#[test]
fn two_ranges() {
    let mut model = new_empty_model();
    model.set_a1("A1", "a");
    model.set_a1("A2", "b");
    model.set_a1("A3", "c");
    model.set_a1("B1", "a");
    model.set_a1("B2", "B");
    model.set_a1("B3", "c");
    model.set_a1("D1", "=EXACT(A1:A3, B1:B3)");
    // A column and a row make a table
    model.set_a1("F1", "=EXACT(A1:A2, {\"a\",\"b\",\"c\"})");
    model.evaluate();

    assert_eq!(model.get_text_a1("D1"), *"TRUE");
    assert_eq!(model.get_text_a1("D2"), *"FALSE");
    assert_eq!(model.get_text_a1("D3"), *"TRUE");

    assert_eq!(model.get_text_a1("F1"), *"TRUE");
    assert_eq!(model.get_text_a1("G1"), *"FALSE");
    assert_eq!(model.get_text_a1("H1"), *"FALSE");
    assert_eq!(model.get_text_a1("F2"), *"FALSE");
    assert_eq!(model.get_text_a1("G2"), *"TRUE");
    assert_eq!(model.get_text_a1("H2"), *"FALSE");
}

#[test]
fn ranges_of_different_sizes() {
    let mut model = new_empty_model();
    model.set_a1("A1", "a");
    model.set_a1("A2", "b");
    model.set_a1("A3", "c");
    model.set_a1("B1", "a");
    model.set_a1("B2", "b");
    model.set_a1("D1", "=EXACT(A1:A3, B1:B2)");
    model.evaluate();

    assert_eq!(model.get_text_a1("D1"), *"TRUE");
    assert_eq!(model.get_text_a1("D2"), *"TRUE");
    assert_eq!(model.get_text_a1("D3"), *"#N/A");
}

#[test]
fn numbers_and_errors_in_ranges() {
    let mut model = new_empty_model();
    model.set_a1("A1", "2.3");
    model.set_a1("A2", "=1/0");
    model.set_a1("A3", "0.1");
    model.set_a1("A4", "TRUE");
    model.set_a1("B1", "'2.3");
    model.set_a1("B2", "a");
    model.set_a1("B3", "=0.3-0.2");
    model.set_a1("B4", "'TRUE");
    model.set_a1("D1", "=EXACT(A1:A4, B1:B4)");
    model.evaluate();

    assert_eq!(model.get_text_a1("D1"), *"TRUE");
    assert_eq!(model.get_text_a1("D2"), *"#DIV/0!");
    // The same up to 15 digits
    assert_eq!(model.get_text_a1("D3"), *"TRUE");
    assert_eq!(model.get_text_a1("D4"), *"TRUE");
}

#[test]
fn array_inside_other_functions() {
    let mut model = new_empty_model();
    model.set_a1("A1", "the");
    model.set_a1("A2", "The");
    model.set_a1("A3", "THE");
    model.set_a1("A4", "The");
    model.set_a1("C1", "=SUM(EXACT(A1:A4, \"The\")*1)");
    model.set_a1("C2", "=SUMPRODUCT(EXACT(A1:A4, A2)*1)");
    model.evaluate();

    assert_eq!(model.get_text_a1("C1"), *"2");
    assert_eq!(model.get_text_a1("C2"), *"2");
}

#[test]
fn case_sensitive_lookup_in_whole_columns() {
    let mut model = new_empty_model();
    model.new_sheet();
    model.set_a1("Sheet2!A1", "the");
    model.set_a1("Sheet2!B1", "1169");
    model.set_a1("Sheet2!A2", "The");
    model.set_a1("Sheet2!B2", "464");
    model.set_a1("Sheet2!A3", "THE");
    model.set_a1("Sheet2!B3", "10970");
    model.set_a1("A1", "The");
    model.set_a1(
        "B1",
        "=IF(A1=\"\", \"\", FILTER(Sheet2!$B:$B, EXACT(Sheet2!$A:$A, A1)))",
    );
    model.set_a1(
        "B2",
        "=IF(A2=\"\", \"\", FILTER(Sheet2!$B:$B, EXACT(Sheet2!$A:$A, A2)))",
    );
    model.evaluate();

    assert_eq!(model.get_text_a1("B1"), *"464");
    assert_eq!(model.get_text_a1("B2"), *"");
}

#[test]
fn implicit_intersection() {
    let mut model = new_empty_model();
    model.set_a1("A1", "a");
    model.set_a1("A2", "b");
    model.set_a1("A3", "c");
    model.set_a1("C2", "=EXACT(@A1:A3, \"b\")");
    model.evaluate();

    assert_eq!(model.get_text_a1("C2"), *"TRUE");
    assert_eq!(model.get_text_a1("C3"), *"");
}
