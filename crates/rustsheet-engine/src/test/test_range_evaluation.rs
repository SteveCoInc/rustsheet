#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_inverted_range_sum() {
    let mut model = new_empty_model();
    model.set_a1("B2", "1");
    model.set_a1("B3", "2");
    model.set_a1("B4", "3");

    model.set_a1("A1", "=SUM(B2:B4)");
    model.set_a1("A2", "=SUM(B4:B2)"); // inverted

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), "6");
    assert_eq!(model.get_text_a1("A2"), "6");
}

#[test]
fn test_mixed_absolute_relative_range() {
    let mut model = new_empty_model();
    model.set_a1("B2", "10");
    model.set_a1("B3", "20");
    model.set_a1("B4", "30");
    model.set_a1("B5", "40");

    model.set_a1("A5", "=SUM(B$2:B5)");
    model.set_a1("C5", "=SUM(B5:B$2)"); // inverted

    model.evaluate();

    assert_eq!(model.get_text_a1("A5"), "100");
    assert_eq!(model.get_text_a1("C5"), "100");
}

#[test]
fn test_mixed_absolute_range_after_sheet_rename() {
    let mut model = new_empty_model();
    model.set_a1("B2", "1");
    model.set_a1("B3", "2");
    model.set_a1("B4", "3");
    model.set_a1("B5", "4");

    model.set_a1("A5", "=SUM(B$2:B5)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A5"), "10");

    model.rename_sheet("Sheet1", "RenamedSheet").unwrap();
    model.evaluate();
    assert_eq!(model.get_text_a1("RenamedSheet!A5"), "10");
}

#[test]
fn test_inverted_range_absolute_column() {
    let mut model = new_empty_model();
    model.set_a1("B2", "5");
    model.set_a1("C2", "10");
    model.set_a1("D2", "15");

    model.set_a1("A1", "=SUM($B2:D2)");
    model.set_a1("A2", "=SUM(D2:$B2)"); // inverted

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), "30");
    assert_eq!(model.get_text_a1("A2"), "30");
}

#[test]
fn test_inverted_2d_range() {
    let mut model = new_empty_model();
    model.set_a1("B2", "1");
    model.set_a1("B3", "2");
    model.set_a1("C2", "3");
    model.set_a1("C3", "4");

    model.set_a1("A1", "=SUM(B2:C3)");
    model.set_a1("A2", "=SUM(C3:B2)"); // fully inverted
    model.set_a1("A3", "=SUM(B3:C2)"); // row inverted
    model.set_a1("A4", "=SUM(C2:B3)"); // column inverted

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), "10");
    assert_eq!(model.get_text_a1("A2"), "10");
    assert_eq!(model.get_text_a1("A3"), "10");
    assert_eq!(model.get_text_a1("A4"), "10");
}

#[test]
fn test_mixed_absolute_2d_range() {
    let mut model = new_empty_model();
    model.set_a1("B2", "1");
    model.set_a1("B3", "2");
    model.set_a1("B4", "3");
    model.set_a1("C2", "4");
    model.set_a1("C3", "5");
    model.set_a1("C4", "6");
    model.set_a1("D2", "7");
    model.set_a1("D3", "8");
    model.set_a1("D4", "9");

    model.set_a1("A4", "=SUM($B$2:D4)");
    model.set_a1("E4", "=SUM(D4:$B$2)"); // inverted

    model.evaluate();

    assert_eq!(model.get_text_a1("A4"), "45");
    assert_eq!(model.get_text_a1("E4"), "45");

    model.rename_sheet("Sheet1", "Data").unwrap();
    model.evaluate();

    assert_eq!(model.get_text_a1("Data!A4"), "45");
    assert_eq!(model.get_text_a1("Data!E4"), "45");
}

#[test]
fn test_range_with_both_corners_sheet_qualified() {
    let mut model = new_empty_model();
    model.rename_sheet("Sheet1", "Data").unwrap();
    model.set_a1("Data!B2", "1");
    model.set_a1("Data!B3", "2");
    model.set_a1("Data!C2", "3");
    model.set_a1("Data!C3", "4");
    model.set_a1("Data!E1", "=SUM(Data!B2:Data!C3)");
    model.set_a1("Data!E2", "=SUM(Data!B2:C3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("Data!E1"), "10");
    assert_eq!(model.get_text_a1("Data!E2"), "10");
    // The formula round-trips through stringify unchanged.
    assert_eq!(model.get_formula_a1("Data!E1"), "=SUM(Data!B2:Data!C3)");
}

#[test]
fn test_range_with_both_corners_quoted_sheet_qualified() {
    let mut model = new_empty_model();
    model.new_sheet();
    model.rename_sheet("Sheet2", "It's data").unwrap();
    model.set_user_input(1, 2, 2, "1".to_string()).unwrap();
    model.set_user_input(1, 3, 3, "4".to_string()).unwrap();
    model.set_a1("A1", "=SUM('It''s data'!B2:'It''s data'!C3)");
    model.set_a1("A2", "=SUM('It''s data'!B2:C3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), "5");
    assert_eq!(model.get_text_a1("A2"), "5");
}

#[test]
fn test_range_with_both_corners_reversed() {
    let mut model = new_empty_model();
    model.rename_sheet("Sheet1", "Data").unwrap();
    model.set_a1("Data!A1", "1");
    model.set_a1("Data!B1", "2");
    model.set_a1("Data!A2", "3");
    model.set_a1("Data!B2", "4");
    model.set_a1("Data!E1", "=SUM(Data!B2:Data!A1)");

    model.evaluate();

    assert_eq!(model.get_text_a1("Data!E1"), "10");
}

#[test]
fn test_array_arithmetic_coerces_text_elements() {
    let mut model = new_empty_model();
    model.set_a1("B1", "' 2 ");
    model.set_a1("B2", "abc");
    model.set_a1("A1", "=B1:B1/1");
    model.set_a1("A2", "=B1:B1*2.5");
    model.set_a1("A3", "=B1:B1/0");
    model.set_a1("A4", "=B2:B2+1");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), "2");
    assert_eq!(model.get_text_a1("A2"), "5");
    assert_eq!(model.get_text_a1("A3"), "#DIV/0!");
    assert_eq!(model.get_text_a1("A4"), "#VALUE!");
}

// gen-m1/4dc8ed1de05a61be Calc!A250: the error in the left range operand wins
// over the right operand's error (docs/excel-semantics/coercion.md, T-0137).
#[test]
fn test_range_operand_error_wins_over_right_scalar_error() {
    let mut model = new_empty_model();
    model.rename_sheet("Sheet1", "Data").unwrap();
    model.set_a1("Data!C9", "=NA()");
    model.set_a1("Data!D5", "=1/0");
    model.set_a1("A1", "=(Data!C1:C10 Data!B9:D9)/Data!D$5");
    model.set_a1("A2", "=Data!C9:C9/Data!D$5");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), "#N/A");
    assert_eq!(model.get_text_a1("A2"), "#N/A");
}

#[test]
fn test_sheet_qualified_malformed_corner_pairs_keep_text_and_are_name_errors() {
    let mut model = new_empty_model();
    model.rename_sheet("Sheet1", "Data").unwrap();
    model.new_sheet();
    model.rename_sheet("Sheet1", "My Data").unwrap();
    // T-0145: Excel gives #NAME? and keeps the formula as typed.
    let formulas = [
        "=SUM(Data!A:Data!B)",
        "=SUM(Data!A1:B)",
        "=SUM('My Data'!A:'My Data'!B1)",
    ];
    for (i, formula) in formulas.iter().enumerate() {
        model.set_a1(&format!("Data!D{}", i + 1), formula);
    }

    model.evaluate();

    for (i, formula) in formulas.iter().enumerate() {
        let cell = format!("Data!D{}", i + 1);
        assert_eq!(model.get_text_a1(&cell), "#NAME?", "{formula}");
        assert_eq!(model.get_formula_a1(&cell), *formula);
    }
}
