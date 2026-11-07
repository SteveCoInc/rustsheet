#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn numbers() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=MAX(1, 5, 3)");
    model.set_a1("A2", "=MIN(1, 5, 3)");
    model.set_a1("A3", "=MAX(-1, -5)");
    model.set_a1("A4", "=MIN(-1, -5)");
    // Nothing to compare
    model.set_a1("A5", "=MAX(C1:C5)");
    model.set_a1("A6", "=MIN(C1:C5)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"5");
    assert_eq!(model.get_text_a1("A2"), *"1");
    assert_eq!(model.get_text_a1("A3"), *"-1");
    assert_eq!(model.get_text_a1("A4"), *"-5");
    assert_eq!(model.get_text_a1("A5"), *"0");
    assert_eq!(model.get_text_a1("A6"), *"0");
}

#[test]
fn text_and_booleans_in_ranges_are_ignored() {
    let mut model = new_empty_model();
    model.set_a1("A1", "-2");
    model.set_a1("A2", "text");
    model.set_a1("A3", "TRUE");
    model.set_a1("A4", "-10");
    model.set_a1("A5", "'100");
    model.set_a1("A6", "=\"\"");
    model.set_a1("C1", "=MAX(A1:A6)");
    model.set_a1("C2", "=MIN(A1:A6)");
    // Also when the reference is a single cell
    model.set_a1("C3", "=MAX(A2)");
    model.set_a1("C4", "=MAX(A2, -3)");
    model.set_a1("C5", "=MIN(A3, 3)");
    model.set_a1("C6", "=MAX(A5, 3)");
    model.set_a1("C7", "=MAX(A6, -3)");
    model.set_a1("C8", "=MAX(A7, -3)");
    model.evaluate();

    assert_eq!(model.get_text_a1("C1"), *"-2");
    assert_eq!(model.get_text_a1("C2"), *"-10");
    assert_eq!(model.get_text_a1("C3"), *"0");
    assert_eq!(model.get_text_a1("C4"), *"-3");
    assert_eq!(model.get_text_a1("C5"), *"3");
    assert_eq!(model.get_text_a1("C6"), *"3");
    assert_eq!(model.get_text_a1("C7"), *"-3");
    assert_eq!(model.get_text_a1("C8"), *"-3");
}

#[test]
fn text_and_booleans_in_arrays_are_ignored() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=MAX({-1, \"a\", TRUE, -7})");
    model.set_a1("A2", "=MIN({3, \"a\", FALSE, 7})");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"-1");
    assert_eq!(model.get_text_a1("A2"), *"3");
}

#[test]
fn text_as_argument() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=MAX(\"\")");
    model.set_a1("A2", "=MIN(\"\")");
    model.set_a1("A3", "=MAX(\"abc\", 1)");
    model.set_a1("A4", "=MIN(1, \"abc\")");
    // Text that reads as a number is a number
    model.set_a1("A5", "=MAX(\"3\", 2)");
    model.set_a1("A6", "=MIN(\"-3\", 2)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A2"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A3"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A4"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A5"), *"3");
    assert_eq!(model.get_text_a1("A6"), *"-3");
}

#[test]
fn boolean_as_argument() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=MAX(TRUE, 0.5)");
    model.set_a1("A2", "=MIN(FALSE, 3)");
    model.set_a1("A3", "=MAX(-1, FALSE)");
    model.set_a1("A4", "=MIN(TRUE, 3)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"1");
    assert_eq!(model.get_text_a1("A2"), *"0");
    assert_eq!(model.get_text_a1("A3"), *"0");
    assert_eq!(model.get_text_a1("A4"), *"1");
}

#[test]
fn text_from_a_function() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=MAX(LEFT(\"abc\", 1))");
    model.set_a1("A2", "=MIN(IF(TRUE, \"\", 1))");
    model.set_a1("A3", "=MAX(LEFT(\"34\", 1), 2)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A2"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A3"), *"3");
}

#[test]
fn filter_with_nothing_found() {
    let mut model = new_empty_model();
    model.set_a1("A1", "10");
    model.set_a1("A2", "30");
    model.set_a1("A3", "20");
    model.set_a1("C1", "=MAX(FILTER(A1:A3, A1:A3>15, \"\"))");
    model.set_a1("C2", "=MAX(FILTER(A1:A3, A1:A3>100, \"\"))");
    model.set_a1("C3", "=IFERROR(MAX(FILTER(A1:A3, A1:A3>100, \"\")), \"\")");
    model.set_a1("C4", "=MIN(FILTER(A1:A3, A1:A3>100, \"\"))");
    model.evaluate();

    assert_eq!(model.get_text_a1("C1"), *"30");
    assert_eq!(model.get_text_a1("C2"), *"#VALUE!");
    assert_eq!(model.get_text_a1("C3"), *"");
    assert_eq!(model.get_text_a1("C4"), *"#VALUE!");
}

#[test]
fn errors() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "=1/0");
    model.set_a1("C1", "=MAX(A1:A2)");
    model.set_a1("C2", "=MIN(A1:A2)");
    model.set_a1("C3", "=MAX(A2)");
    model.set_a1("C4", "=MAX(1, 1/0)");
    model.set_a1("C5", "=MIN({1, #N/A})");
    model.evaluate();

    assert_eq!(model.get_text_a1("C1"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("C2"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("C3"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("C4"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("C5"), *"#N/A");
}

#[test]
fn implicit_intersection_is_a_reference() {
    let mut model = new_empty_model();
    model.set_a1("A1", "5");
    model.set_a1("A2", "text");
    model.set_a1("A3", "7");
    // `@` intersects by row: C1 sees A1 (5), C2 sees A2, whose text is ignored.
    model.set_a1("C1", "=MAX(@A1:A3)");
    model.set_a1("C2", "=MAX(@A1:A3)");
    model.evaluate();

    assert_eq!(model.get_text_a1("C1"), *"5");
    assert_eq!(model.get_text_a1("C2"), *"0");
}

#[test]
fn inside_a_lambda() {
    let mut model = new_empty_model();
    model.new_sheet();
    model.set_a1("Sheet2!A1", "h");
    model.set_a1("Sheet2!B1", "e");
    model.set_a1("Sheet2!D1", "10");
    model.set_a1("Sheet2!A2", "h");
    model.set_a1("Sheet2!B2", "e");
    model.set_a1("Sheet2!D2", "4");
    model
        .new_defined_name(
            "score",
            None,
            "=LAMBDA(left_char, right_char, IF((left_char = \"\") + (right_char = \"\"), \"\", IFERROR(MAX(FILTER(Sheet2!$D:$D, (left_char = Sheet2!$A:$A) * (right_char = Sheet2!$B:$B), \"\")), \"\")))",
        )
        .unwrap();
    model
        .new_defined_name("biggest", None, "=LAMBDA(x, MAX(x))")
        .unwrap();
    model.set_a1("A1", "h");
    model.set_a1("B1", "e");
    model.set_a1("C1", "l");
    model.set_a1("A3", "=score(A1, B1)");
    model.set_a1("B3", "=score(B1, C1)");
    model.set_a1("C3", "=score(C1, D1)");
    // The best score, and whether each pair has it
    model.set_a1("A4", "=MAX(A3:C3)");
    model.set_a1("A5", "=A4=A3");
    model.set_a1("B5", "=A4=B3");
    // A parameter that is a reference is still a reference, to a single
    // cell as well: the text in A1 does not count
    model.set_a1("A7", "=biggest(A1)");
    model.set_a1("A8", "=biggest(A1:C1)");
    model.set_a1("A9", "=biggest(\"h\")");
    model.evaluate();

    assert_eq!(model.get_text_a1("A3"), *"10");
    assert_eq!(model.get_text_a1("B3"), *"");
    assert_eq!(model.get_text_a1("C3"), *"");
    assert_eq!(model.get_text_a1("A4"), *"10");
    assert_eq!(model.get_text_a1("A5"), *"TRUE");
    assert_eq!(model.get_text_a1("B5"), *"FALSE");
    assert_eq!(model.get_text_a1("A7"), *"0");
    assert_eq!(model.get_text_a1("A8"), *"0");
    assert_eq!(model.get_text_a1("A9"), *"#VALUE!");
}

#[test]
fn an_argument_left_out_counts_as_zero() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=MAX(,-1)");
    model.set_a1("A2", "=MIN(5,)");
    model.set_a1("A3", "=MAX(,)");
    model.set_a1("A4", "=MAX(1,,2)");
    model.set_a1("A5", "=MIN(1,,2)");
    model.set_a1("A6", "=MIN(-1,)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"0");
    assert_eq!(model.get_text_a1("A2"), *"0");
    assert_eq!(model.get_text_a1("A3"), *"0");
    assert_eq!(model.get_text_a1("A4"), *"2");
    assert_eq!(model.get_text_a1("A5"), *"0");
    assert_eq!(model.get_text_a1("A6"), *"-1");
}

// A defined name that stands for a cell is a reference, like the cell itself:
// the text in it does not count. The same for a name local to a sheet.
#[test]
fn defined_names() {
    let mut model = new_empty_model();
    model.new_sheet();
    model.set_a1("K1", "text");
    model.set_a1("K2", "7");
    model.set_a1("L1", "10");
    model.set_a1("L2", "20");
    model
        .new_defined_name("TextCell", None, "Sheet1!$K$1")
        .unwrap();
    model
        .new_defined_name("NumCell", None, "Sheet1!$K$2")
        .unwrap();
    model
        .new_defined_name("NumCells", None, "Sheet1!$L$1:$L$2")
        .unwrap();
    model
        .new_defined_name("LocalText", Some(0), "Sheet1!$K$1")
        .unwrap();
    model.set_a1("A1", "=MAX(TextCell)");
    model.set_a1("A2", "=MIN(TextCell)");
    model.set_a1("A3", "=MAX(TextCell, -3)");
    model.set_a1("A4", "=MAX(NumCell)");
    model.set_a1("A5", "=MAX(NumCells)");
    model.set_a1("A6", "=MAX(K1)");
    model.set_a1("A7", "=MAX(\"text\")");
    model.set_a1("A8", "=MAX(LocalText)");
    model.set_a1("A9", "=MIN(LocalText, 3)");
    model.set_a1("A10", "=MIN(NumCells, NumCell, TextCell)");
    // From another sheet
    model.set_a1("Sheet2!A1", "=MAX(TextCell)");
    model.set_a1("Sheet2!A2", "=MAX(NumCells)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"0");
    assert_eq!(model.get_text_a1("A2"), *"0");
    assert_eq!(model.get_text_a1("A3"), *"-3");
    assert_eq!(model.get_text_a1("A4"), *"7");
    assert_eq!(model.get_text_a1("A5"), *"20");
    assert_eq!(model.get_text_a1("A6"), *"0");
    assert_eq!(model.get_text_a1("A7"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A8"), *"0");
    assert_eq!(model.get_text_a1("A9"), *"3");
    assert_eq!(model.get_text_a1("A10"), *"7");
    assert_eq!(model.get_text_a1("Sheet2!A1"), *"0");
    assert_eq!(model.get_text_a1("Sheet2!A2"), *"20");
}
