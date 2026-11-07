#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn arguments() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=CELL(\"address\",A1)");
    model.set_a1("A2", "=CELL()");

    model.set_a1("A3", "=INFO(\"recalc\")");
    model.set_a1("A4", "=INFO()");

    model.set_a1("A5", "=N(TRUE)");
    model.set_a1("A6", "=N()");
    model.set_a1("A7", "=N(1, 2)");

    model.set_a1("A8", "=SHEETS()");
    model.set_a1("A9", "=SHEETS(1)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"$A$1");
    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");

    assert_eq!(model.get_text_a1("A3"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A4"), *"#ERROR!");

    assert_eq!(model.get_text_a1("A5"), *"1");
    assert_eq!(model.get_text_a1("A6"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A7"), *"#ERROR!");

    assert_eq!(model.get_text_a1("A8"), *"1");
    assert_eq!(model.get_text_a1("A9"), *"#N/IMPL!");
}

/// INFO is `#VALUE!` for every `type_text` (probes/info-sheets-info-types Alpha!D25:D49, excel-graph).
#[test]
fn info_is_value_error_for_every_type_text() {
    let mut model = new_empty_model();
    for (i, text) in [
        "recalc", "SYSTEM", "release", "timezone", "numfile", "memavail", "", "x",
    ]
    .iter()
    .enumerate()
    {
        model.set_a1(&format!("A{}", i + 1), &format!("=INFO(\"{text}\")"));
    }
    model.evaluate();
    for i in 1..=8 {
        assert_eq!(model.get_text_a1(&format!("A{i}")), *"#VALUE!");
    }
}

/// `CELL("filename")` is `#VALUE!` in every form, named workbook or not (probes/
/// info-cell-address-cross-sheet Calc!B1:B4, excel-graph, T-2052).
#[test]
fn cell_filename_is_value_error() {
    let mut model = new_empty_model();
    model.add_sheet("Data").unwrap();
    model.set_a1("A1", "=CELL(\"filename\")");
    model.set_a1("A2", "=CELL(\"filename\",B2)");
    model.set_a1("A3", "=CELL(\"filename\",Data!B2)");
    model.evaluate();
    for cell in ["A1", "A2", "A3"] {
        assert_eq!(model.get_text_a1(cell), *"#VALUE!");
    }

    model.workbook.name = String::new();
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"#VALUE!");
}

/// `CELL("address")` of a reference on another sheet is prefixed `[<file>]<sheet>!`, the sheet
/// name never quoted, a range giving its top-left cell (probes/info-cell-address-cross-sheet,
/// excel-graph, T-2042).
#[test]
fn cell_address_on_another_sheet_names_the_file() {
    let mut model = new_empty_model();
    model.workbook.name = "book".to_string();
    model.add_sheet("My Sheet").unwrap();
    model.add_sheet("Bob's").unwrap();
    model.set_a1("A1", "=CELL(\"address\",'My Sheet'!B2)");
    model.set_a1("A2", "=CELL(\"address\",'Bob''s'!B2:C3)");
    model.set_a1("A3", "=CELL(\"address\",Sheet1!B2)");
    model.set_a1("A4", "=CELL(\"address\",('My Sheet'!B2,'My Sheet'!C3))");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"[book.xlsx]My Sheet!$B$2");
    assert_eq!(model.get_text_a1("A2"), *"[book.xlsx]Bob's!$B$2");
    assert_eq!(model.get_text_a1("A3"), *"$B$2");
    // A union on another sheet is unprobed.
    assert_eq!(model.get_text_a1("A4"), *"#N/IMPL!");

    // Excel's form for an unsaved workbook is unprobed.
    model.workbook.name = String::new();
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"#N/IMPL!");
    assert_eq!(model.get_text_a1("A3"), *"$B$2");
}

/// N lifts over an array, an error element kept (probes/n-array-lift; T-1616).
#[test]
fn n_lifts_over_an_array() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=SUM(N({1,\"a\",TRUE}))");
    model.set_a1("A2", "=N({1,#N/A})");
    model.set_a1("A3", "=SUMPRODUCT(INDEX(D1:D6,N(IF(1,{2;3}))))");
    for (row, value) in (1..=6).enumerate() {
        model.set_a1(&format!("D{}", row + 1), &value.to_string());
    }
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"2");
    assert_eq!(model.get_text_a1("A2"), *"1");
    assert_eq!(model.get_text_a1("B2"), *"#N/A");
    assert_eq!(model.get_text_a1("A3"), *"5");
}

/// T lifts over an array value, text and errors kept and anything else "" (probes/t-array-lift,
/// legacy-array-result S!G1:G2; T-1729); a multi-cell range with a number top-left cell stays one
/// "" (probes/text-argument-modes F9).
#[test]
fn t_lifts_over_an_array() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=T(IF(1,{\"a\";\"b\"}))");
    model.set_a1("C1", "=T(IF(1,{\"a\",1,#N/A,TRUE}))");
    model.set_a1("E3", "=T(G1:G2)");
    model.set_a1("G1", "5");
    model.set_a1("G2", "y");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"a");
    assert_eq!(model.get_text_a1("A2"), *"b");
    assert_eq!(model.get_text_a1("C1"), *"a");
    assert_eq!(model.get_text_a1("D1"), *"");
    assert_eq!(model.get_text_a1("E1"), *"#N/A");
    assert_eq!(model.get_text_a1("F1"), *"");
    assert_eq!(model.get_text_a1("E3"), *"");
    assert_eq!(model.get_text_a1("E4"), *"");
}

/// T reads a multi-cell range as its top-left cell, legacy or dynamic, with no implicit
/// intersection, and a legacy formula puts no `@` on the range, an OFFSET or a literal-condition
/// IF of it (probes/t-n-range-top-left F2, F10, F20, P20, Q20, R20; T-0985).
#[test]
fn t_reads_the_top_left_cell_of_a_range() {
    let mut model = new_empty_model();
    model.set_a1("A1", "x");
    model.set_a1("A2", "5");
    model.set_a1("B1", "5");
    model.set_a1("B2", "ab");
    model.set_a1("C2", "=T(A1:A2)");
    model.set_a1("D2", "=T(B1:B2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C2"), *"x");
    assert_eq!(model.get_text_a1("D2"), *"");
    for (formula, legacy) in [
        ("=T(A1:A3)", "=T(A1:A3)"),
        ("=T(OFFSET(A1:A3,0,0))", "=T(OFFSET(A1:A3,0,0))"),
        ("=T(IF(1,A1:A3))", "=T(IF(1,A1:A3))"),
        ("=T(A1:A3&\"\")", "=T(@A1:A3&\"\")"),
    ] {
        let text = model.legacy_formula_text(0, 10, 6, formula).unwrap();
        assert_eq!(text, legacy, "{formula}");
    }
}

/// A multi-cell range `info_type` lifts like an array one (gen-m5/c0651f0e0fb544d0 Calc!A293).
#[test]
fn cell_lifts_over_a_range_info_type() {
    let mut model = new_empty_model();
    model.set_a1("D1", "row");
    model.set_a1("D2", "col");
    model.set_a1("F1", "=CELL(D1:D2,B3)");
    model.set_a1("H2", "x");
    model.set_a1("H1", "=CELL(D1:D2,B3)");
    model.evaluate();

    assert_eq!(model.get_text_a1("F1"), *"3");
    assert_eq!(model.get_text_a1("F2"), *"2");
    assert_eq!(model.get_text_a1("H1"), *"#SPILL!");
}

/// A stack argument written as `CELL("contents", ref)` is `#VALUE!` when the target is blank and
/// stacks the target's value otherwise, for VSTACK and HSTACK alike; CELL alone and `VSTACK(0,..)`
/// are fine (probes/t2414-gen-m7-unexplained, probes/t2466-cell-contents-stack A1, A31, A41, A51,
/// T-3030).
#[test]
fn cell_contents_as_a_stack_argument() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=VSTACK(CELL(\"contents\",C11),1)");
    model.set_a1("B2", "=VSTACK(0,1)");
    model.set_a1("C3", "=CELL(\"contents\",C11)");
    model.set_a1("D1", "=HSTACK(CELL(\"contents\",C11),1)");
    model.set_a1("E20", "7");
    model.set_a1("F1", "=VSTACK(CELL(\"contents\",E20),1)");
    model.set_a1("G1", "=HSTACK(CELL(\"contents\",E20),1)");
    model.set_a1("J1", "=VSTACK(1/0,CELL(\"contents\",C11))");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#VALUE!");
    assert_eq!(model.get_text_a1("B2"), *"0");
    assert_eq!(model.get_text_a1("C3"), *"0");
    assert_eq!(model.get_text_a1("D1"), *"#VALUE!");
    assert_eq!(model.get_text_a1("F1"), *"7");
    assert_eq!(model.get_text_a1("F2"), *"1");
    assert_eq!(model.get_text_a1("G1"), *"7");
    assert_eq!(model.get_text_a1("H1"), *"1");
    assert_eq!(model.get_text_a1("J1"), *"#DIV/0!");
}

/// `CELL("contents"|"type", ref)` reads the sheet the reference names, nested or not
/// (probes/t3030-cell-contents-nested-sheet A1, A11, A41, A81, A321; probes/t3030-cell-first-sheet;
/// excel-graph 2026-10-28, T-3030).
#[test]
fn cell_contents_reads_the_named_sheet() {
    let mut model = new_empty_model();
    model.new_sheet();
    model.set_a1("Sheet2!B1", "7");
    model.set_a1("Sheet2!B3", "abc");
    model.set_a1("B1", "99");
    model.set_a1("A1", "=CELL(\"contents\",Sheet2!B1)");
    model.set_a1("A2", "=CELL(\"contents\",Sheet2!B1)+0");
    model.set_a1("A3", "=LET(x,CELL(\"contents\",Sheet2!B1),x)");
    model.set_a1("A4", "=CELL(\"type\",Sheet2!B3)");
    model.set_a1("A5", "=CELL(\"contents\",B1)");
    model.set_a1("Sheet2!A1", "=CELL(\"contents\",Sheet1!B1)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"7");
    assert_eq!(model.get_text_a1("A2"), *"7");
    assert_eq!(model.get_text_a1("A3"), *"7");
    assert_eq!(model.get_text_a1("A4"), *"l");
    assert_eq!(model.get_text_a1("A5"), *"99");
    assert_eq!(model.get_text_a1("Sheet2!A1"), *"99");
}
