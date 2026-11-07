#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn fn_if_arguments() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=IF()");
    model.set_a1("A2", "=IF(1, 2, 3, 4)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");
}

#[test]
fn fn_if_2_args() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=IF(2 > 3, TRUE)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"FALSE");
}

#[test]
fn fn_if_missing_args() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=IF(2 > 3, TRUE, )");
    model.set_a1("A2", "=IF(2 > 3, , 5)");
    model.set_a1("A3", "=IF(2 < 3, , 5)");

    model.evaluate();

    // assert_eq!(model.get_text_a1("A1"), *"0");
    assert_eq!(model.get_text_a1("A2"), *"5");
    assert_eq!(model.get_text_a1("A3"), *"0");
}

// Excel: IF and INDEX hand an aggregate a reference, so its text and booleans
// are skipped like a range cell's; elsewhere they are values (T-0216;
// docs/excel-semantics/references.md).
#[test]
fn test_if_and_index_pass_references_to_aggregates() {
    let mut model = new_empty_model();
    model.set_a1("A1", "abc");
    model.set_a1("A2", "'1");
    model.set_a1("A3", "TRUE");
    model.set_a1("A4", "5");
    model.set_a1("B1", "=SUM(IF(FALSE,-2.5,A1),A4)");
    model.set_a1("B2", "=COUNT(IF(TRUE,A2),A4)");
    model.set_a1("B3", "=SUM(A4,INDEX(A1:A2,1))");
    model.set_a1("B4", "=MAX(IF(TRUE,A1),INDEX(A1:A4,3),A4)");
    model.set_a1("B5", "=AVERAGE(IF(TRUE,A3),A4)");
    model.set_a1("B6", "=ISREF(INDEX(A1:A2,2))");
    model.set_a1("B7", "=IF(TRUE,A1)&\"x\"");
    model.set_a1("B8", "=SUM(IF(TRUE,\"abc\"))");
    model.set_a1("B9", "=MIN(IF(TRUE,A1),INDEX(A1:A4,3),A4)");
    model.set_a1("B10", "=ISREF(INDEX(A1,1,2,2))");
    model.evaluate();
    assert_eq!(model.get_text_a1("B1"), *"5");
    assert_eq!(model.get_text_a1("B2"), *"1");
    assert_eq!(model.get_text_a1("B3"), *"5");
    assert_eq!(model.get_text_a1("B4"), *"5");
    assert_eq!(model.get_text_a1("B5"), *"5");
    assert_eq!(model.get_text_a1("B6"), *"TRUE");
    assert_eq!(model.get_text_a1("B7"), *"abcx");
    assert_eq!(model.get_text_a1("B8"), *"#VALUE!");
    assert_eq!(model.get_text_a1("B9"), *"5");
    assert_eq!(model.get_text_a1("B10"), *"FALSE");
}

// An eta-reduced IF called with one argument is `#VALUE!` per element that is not an error
// (probes/eta-if-one-argument, T-1947).
#[test]
fn eta_if_with_one_argument() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("B1", "0");
    model.set_a1("C1", "=LET(f,_xleta.IF,f(TRUE))");
    model.set_a1("C2", "=LET(f,_xleta.IF,f(NA()))");
    model.set_a1("C3", "=LET(f,_xleta.IF,f({1,#DIV/0!}))");
    model.set_a1("C5", "=BYROW(A1:B1,_xleta.IF)");
    model.set_a1("C6", "=LET(f,_xleta.IF,f(FALSE,5))");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), "#VALUE!");
    assert_eq!(model.get_text_a1("C2"), "#N/A");
    assert_eq!(model.get_text_a1("C3"), "#VALUE!");
    assert_eq!(model.get_text_a1("D3"), "#DIV/0!");
    assert_eq!(model.get_text_a1("C5"), "#CALC!");
    assert_eq!(model.get_text_a1("C6"), "FALSE");
}

#[test]
fn typed_if_with_one_argument_is_refused() {
    // Excel for the web refuses typing IF with only a condition (Graph PATCH 400 InvalidArgument
    // for each of these, excel-graph 2026-10-28, T-2939) and takes IF with an empty or given second
    // argument (probes/t2939-typed-if-two-argument-controls).
    let mut model = new_empty_model();
    for formula in [
        "=IF(TRUE)",
        "=IF(FALSE)",
        "=IF(1)",
        "=IF(0)",
        "=IF({1,0})",
        "=IF(A1:B1)",
        "=IF(\"x\")",
        "=IF(NA())",
        "=ISLOGICAL(IF(TRUE))",
        "=IF(TRUE)+1",
    ] {
        assert!(
            model
                .set_typed_user_input(0, 1, 5, formula.to_string())
                .is_err(),
            "{formula}"
        );
    }
    model
        .set_typed_user_input(0, 1, 5, "=IF(TRUE,)".to_string())
        .unwrap();
    model
        .set_typed_user_input(0, 2, 5, "=IF(TRUE,1)".to_string())
        .unwrap();
    model
        .set_typed_user_input(0, 3, 5, "=IF(FALSE,1)".to_string())
        .unwrap();
    // Stored text is not refused: it takes the one-argument eta path (`#VALUE!`, T-1947).
    model.set_a1("A9", "=IF(TRUE)");
    model.evaluate();
    assert_eq!(model.get_text_a1("E1"), "0");
    assert_eq!(model.get_text_a1("E2"), "1");
    assert_eq!(model.get_text_a1("E3"), "FALSE");
    assert_eq!(model.get_text_a1("A9"), "#VALUE!");
}

#[test]
fn test_fn_if_array_branch_no_element_takes_still_sizes_the_result() {
    // A branch no condition element takes is only sized, not copied (T-1564); the result still
    // spans it.
    let mut model = new_empty_model();
    model.set_a1("C1", "5");
    model.set_a1("A1", "=ROWS(IF({TRUE},C1,D1:D4))");
    model.set_a1("A2", "=SUM(IF({TRUE},C1,D1:D4))");
    model.set_a1("A3", "=ROWS(IF({FALSE},C1:C3,D1))");
    model.set_a1("A4", "=SUM(IF({TRUE},C:C,D:D))");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "4");
    assert_eq!(model.get_text_a1("A2"), "20");
    assert_eq!(model.get_text_a1("A3"), "3");
    assert_eq!(model.get_text_a1("A4"), "5");
}
