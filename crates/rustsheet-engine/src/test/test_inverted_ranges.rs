#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn sum_inverted_range_vertical() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("A3", "3");

    model.set_a1("B1", "=SUM(A1:A3)");
    model.set_a1("B2", "=SUM(A3:A1)");
    model.evaluate();

    // Formula should not be inverted
    assert_eq!(model.get_formula_a1("B2"), *"=SUM(A1:A3)");

    // Result check
    assert_eq!(model.get_text_a1("B1"), *"6");

    // Result should be the same
    assert_eq!(model.get_text_a1("B1"), model.get_text_a1("B2"));
}

#[test]
fn sum_inverted_range_horizontal() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("B1", "2");
    model.set_a1("C1", "3");

    model.set_a1("D1", "=SUM(A1:C1)");
    model.set_a1("D2", "=SUM(C1:A1)");
    model.evaluate();

    // Formula should not be inverted
    assert_eq!(model.get_formula_a1("D2"), *"=SUM(A1:C1)");

    // Result check
    assert_eq!(model.get_text_a1("D1"), *"6");

    // Result should be the same
    assert_eq!(model.get_text_a1("D1"), model.get_text_a1("D2"));
}

#[test]
fn sum_inverted_range_rectangular() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("B1", "2");
    model.set_a1("C1", "3");
    model.set_a1("A2", "4");
    model.set_a1("B2", "5");
    model.set_a1("C2", "6");

    model.set_a1("D1", "=SUM(A1:C2)"); // Default: top left to bottom right
    model.set_a1("D2", "=SUM(C2:A1)"); // Bottom right to top left
    model.set_a1("D3", "=SUM(C1:A2)"); // Top right to bottom left
    model.set_a1("D4", "=SUM(A2:C1)"); // Bottom left to top right
    model.evaluate();

    // Formula should not be inverted
    assert_eq!(model.get_formula_a1("D2"), *"=SUM(A1:C2)");
    assert_eq!(model.get_formula_a1("D3"), *"=SUM(A1:C2)");
    assert_eq!(model.get_formula_a1("D4"), *"=SUM(A1:C2)");

    // Result check
    assert_eq!(model.get_text_a1("D1"), *"21");

    // Result should be the same
    assert_eq!(model.get_text_a1("D1"), model.get_text_a1("D2"));
    assert_eq!(model.get_text_a1("D1"), model.get_text_a1("D3"));
    assert_eq!(model.get_text_a1("D1"), model.get_text_a1("D4"));
}

#[test]
fn sum_inverted_range_with_absolute_references() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("A3", "3");

    model.set_a1("B1", "=SUM(A$1:A3)");
    model.set_a1("B2", "=SUM(A3:A$1)");
    model.set_a1("B3", "=SUM(A$1:A$3)");
    model.set_a1("B4", "=SUM(A$3:A$1)");
    model.set_a1("B5", "=SUM($A1:$A3)");
    model.set_a1("B6", "=SUM($A3:$A1)");
    model.set_a1("B7", "=SUM($A$1:$A$3)");
    model.set_a1("B8", "=SUM($A$3:$A$1)");
    model.evaluate();

    // Formula should not be inverted
    assert_eq!(model.get_formula_a1("B2"), *"=SUM(A$1:A3)");
    assert_eq!(model.get_formula_a1("B3"), *"=SUM(A$1:A$3)");
    assert_eq!(model.get_formula_a1("B4"), *"=SUM(A$1:A$3)");
    assert_eq!(model.get_formula_a1("B5"), *"=SUM($A1:$A3)");
    assert_eq!(model.get_formula_a1("B6"), *"=SUM($A1:$A3)");
    assert_eq!(model.get_formula_a1("B7"), *"=SUM($A$1:$A$3)");
    assert_eq!(model.get_formula_a1("B8"), *"=SUM($A$1:$A$3)");

    // Result check
    assert_eq!(model.get_text_a1("B1"), *"6");

    // Result should be the same
    assert_eq!(model.get_text_a1("B1"), model.get_text_a1("B2"));
    assert_eq!(model.get_text_a1("B1"), model.get_text_a1("B3"));
    assert_eq!(model.get_text_a1("B1"), model.get_text_a1("B4"));
    assert_eq!(model.get_text_a1("B1"), model.get_text_a1("B5"));
    assert_eq!(model.get_text_a1("B1"), model.get_text_a1("B6"));
    assert_eq!(model.get_text_a1("B1"), model.get_text_a1("B7"));
    assert_eq!(model.get_text_a1("B1"), model.get_text_a1("B8"));
}

#[test]
fn inverted_range_with_blanks() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("A3", "3");

    model.set_a1("B1", "=SUM(A1:A4)");
    model.set_a1("B2", "=SUM(A4:A1)");

    model.set_a1("C1", "=COUNTA(A1:A4)"); // Counts non-blank cells
    model.set_a1("C2", "=COUNTA(A4:A1)");

    model.evaluate();

    // Formula should not be inverted
    assert_eq!(model.get_formula_a1("B2"), *"=SUM(A1:A4)");
    assert_eq!(model.get_formula_a1("C2"), *"=COUNTA(A1:A4)");

    // Result check
    assert_eq!(model.get_text_a1("B1"), *"6");
    assert_eq!(model.get_text_a1("C1"), *"3");

    // Result should be the same
    assert_eq!(model.get_text_a1("B1"), model.get_text_a1("B2"));
    assert_eq!(model.get_text_a1("C1"), model.get_text_a1("C2"));
}

#[test]
fn sum_inverted_range_with_errors() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("A3", "3");
    model.set_a1("A4", "=1/0");

    model.set_a1("B1", "=SUM(A1:A4)");
    model.set_a1("B2", "=SUM(A4:A1)");
    model.evaluate();

    // Formula should not be inverted
    assert_eq!(model.get_formula_a1("B2"), *"=SUM(A1:A4)");

    // Result check
    assert_eq!(model.get_text_a1("B1"), *"#DIV/0!");

    // Result should be the same
    assert_eq!(model.get_text_a1("B1"), model.get_text_a1("B2"));
}

#[test]
fn other_functions() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("A3", "3");

    model.set_a1("B1", "=AVERAGE(A1:A3)");
    model.set_a1("B2", "=AVERAGE(A3:A1)");

    model.set_a1("C1", "=COUNT(A1:A3)");
    model.set_a1("C2", "=COUNT(A3:A1)");

    model.set_a1("D1", "=INDEX(A1:A3, 2)");
    model.set_a1("D2", "=INDEX(A3:A1, 2)");

    model.set_a1("E1", "=MATCH(2, A1:A3)");
    model.set_a1("E2", "=MATCH(2, A3:A1)");

    model.set_a1("F1", "=SUMIF(A1:A3, \">1\")");
    model.set_a1("F2", "=SUMIF(A3:A1, \">1\")");

    model.set_a1("G1", "=CONCAT(A1:A3)");
    model.set_a1("G2", "=CONCAT(A3:A1)");

    model.set_a1("H1", "=ROWS(A1:A3)");
    model.set_a1("H2", "=ROWS(A3:A1)");

    model.set_a1("I1", "=COLUMNS(A1:A3)");
    model.set_a1("I2", "=COLUMNS(A3:A1)");

    model.evaluate();

    // AVERAGE
    assert_eq!(model.get_formula_a1("B2"), *"=AVERAGE(A1:A3)"); // Formula should not be inverted
    assert_eq!(model.get_text_a1("B1"), *"2"); // Result check
    assert_eq!(model.get_text_a1("B1"), model.get_text_a1("B2")); // Result should be the same

    // COUNT
    assert_eq!(model.get_formula_a1("C2"), *"=COUNT(A1:A3)"); // Formula should not be inverted
    assert_eq!(model.get_text_a1("C1"), *"3"); // Result check
    assert_eq!(model.get_text_a1("C1"), model.get_text_a1("C2")); // Result should be the same

    // INDEX
    assert_eq!(model.get_formula_a1("D2"), *"=INDEX(A1:A3, 2)"); // Formula should not be inverted
    assert_eq!(model.get_text_a1("D1"), *"2"); // Result check
    assert_eq!(model.get_text_a1("D1"), model.get_text_a1("D2")); // Result should be the same

    // MATCH
    assert_eq!(model.get_formula_a1("E2"), *"=MATCH(2, A1:A3)"); // Formula should not be inverted
    assert_eq!(model.get_text_a1("E1"), *"2"); // Result check
    assert_eq!(model.get_text_a1("E1"), model.get_text_a1("E2")); // Result should be the same

    // SUMIF
    assert_eq!(model.get_formula_a1("F2"), *"=SUMIF(A1:A3, \">1\")"); // Formula should not be inverted
    assert_eq!(model.get_text_a1("F1"), *"5"); // Result check
    assert_eq!(model.get_text_a1("F1"), model.get_text_a1("F2")); // Result should be the same

    // CONCATENATE
    assert_eq!(model.get_formula_a1("G2"), *"=CONCAT(A1:A3)"); // Formula should not be inverted
    assert_eq!(model.get_text_a1("G1"), *"123"); // Result check
    assert_eq!(model.get_text_a1("G1"), model.get_text_a1("G2")); // Result should be the same

    // ROWS
    assert_eq!(model.get_formula_a1("H2"), *"=ROWS(A1:A3)"); // Formula should not be inverted
    assert_eq!(model.get_text_a1("H1"), *"3"); // Result check
    assert_eq!(model.get_text_a1("H1"), model.get_text_a1("H2")); // Result should be the same

    // COLUMNS
    assert_eq!(model.get_formula_a1("I2"), *"=COLUMNS(A1:A3)"); // Formula should not be inverted
    assert_eq!(model.get_text_a1("I1"), *"1"); // Result check
    assert_eq!(model.get_text_a1("I1"), model.get_text_a1("I2")); // Result should be the same
}

#[test]
fn mixed_data_types() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "abc");
    model.set_a1("A3", "TRUE");
    model.set_a1("A4", "2");

    model.set_a1("B1", "=SUM(A1:A4)");
    model.set_a1("B2", "=SUM(A4:A1)");

    model.set_a1("C1", "=COUNT(A1:A4)");
    model.set_a1("C2", "=COUNT(A4:A1)");

    model.set_a1("D1", "=COUNTIF(A1:A4, \"TRUE\")");
    model.set_a1("D2", "=COUNTIF(A4:A1, \"TRUE\")");

    model.evaluate();

    // SUM
    assert_eq!(model.get_formula_a1("B2"), *"=SUM(A1:A4)"); // Formula should not be inverted
    assert_eq!(model.get_text_a1("B1"), *"3"); // Result check
    assert_eq!(model.get_text_a1("B1"), model.get_text_a1("B2")); // Result should be the same

    // COUNT
    assert_eq!(model.get_formula_a1("C2"), *"=COUNT(A1:A4)"); // Formula should not be inverted
    assert_eq!(model.get_text_a1("C1"), *"2"); // Result check
    assert_eq!(model.get_text_a1("C1"), model.get_text_a1("C2")); // Result should be the same

    // COUNTIF
    assert_eq!(model.get_formula_a1("D2"), *"=COUNTIF(A1:A4, \"TRUE\")"); // Formula should not be inverted
    assert_eq!(model.get_text_a1("D1"), *"1"); // Result check
    assert_eq!(model.get_text_a1("D1"), model.get_text_a1("D2")); // Result should be the same
}
