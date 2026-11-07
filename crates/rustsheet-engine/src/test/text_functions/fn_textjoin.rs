#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn simple_cases() {
    let mut model = new_empty_model();
    model.set_a1("A1", "Monday");
    model.set_a1("A2", "Tuesday");
    model.set_a1("A3", "Wednesday");

    model.set_a1("B1", "=TEXTJOIN(\", \", TRUE, A1:A3)");
    model.set_a1("B2", "=TEXTJOIN(\" and \", TRUE, A1:A3)");
    // This formula might have the _xlfn. prefix
    model.set_a1("B3", "=_xlfn.TEXTJOIN(\" or \", , A1:A3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("B1"), *"Monday, Tuesday, Wednesday");
    assert_eq!(model.get_text_a1("B2"), *"Monday and Tuesday and Wednesday");
    assert_eq!(model.get_text_a1("B3"), *"Monday or Tuesday or Wednesday");
    // Our text version removes the prefix, and keeps the whitespace Excel keeps: the run in the
    // missing argument's place (`=IF(1, ,2)`, probes/t2786-whitespace-structured-refs-runs I2, T-2944).
    assert_eq!(model.get_formula_a1("B3"), *"=TEXTJOIN(\" or \", , A1:A3)");
}

#[test]
fn wrong_number_of_arguments() {
    let mut model = new_empty_model();
    model.set_a1("A1", "Monday");
    model.set_a1("A2", "Tuesday");
    model.set_a1("A3", "Wednesday");

    model.set_a1("B1", "=TEXTJOIN(\", \", TRUE)");
    model.set_a1("B2", "=TEXTJOIN(\" and \", A1:A3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
}

// ── whole-column range and a spill beyond the used area ─────────────────────

// A whole-column range is clipped to the sheet's used area before it is
// walked. Inside an anchor, the column may be spilled on demand during the
// walk, past the bound the walk was clipped to. The skipped part is on
// record, so the spill restarts the pass and the second run sees it all.
#[test]
fn whole_column_sees_a_spill_beyond_the_used_area() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=SEQUENCE(1,1,LEN(TEXTJOIN(\",\",TRUE,D:D)))");
    model.set_a1("C1", "=SEQUENCE(3,1,10)");
    model.set_a1("D1", "=SEQUENCE(5)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "9");
    assert_eq!(model.evaluation.restarts_in_last_evaluation, 1);

    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "9");
    assert_eq!(model.evaluation.restarts_in_last_evaluation, 0);
}

// Excel values: probes/textjoin-whole-rows-self-reference-full-width B5, B8, D3. With ignore_empty FALSE
// the padding passes the limit in row 2, so no later cell is read: a range holding the TEXTJOIN cell is
// #CALC! even when the used area spans every column; with ignore_empty TRUE it is circular.
#[test]
fn full_width_rows_stop_reading_at_the_limit() {
    let mut model = new_empty_model();
    model.set_a1("A1", "x");
    model.set_a1("XFD1", "z");
    model.set_a1("A2", "y");
    model.set_a1("XFD2", "w");
    model.set_a1("B5", "=TEXTJOIN(\",\",FALSE,1:10)");
    model.set_a1("B8", "=TEXTJOIN(\",\",FALSE,A1:XFD10)");
    model.set_a1("D3", "=TEXTJOIN(\",\",TRUE,1:10)");
    model.evaluate();
    assert_eq!(model.get_text_a1("B5"), "#CALC!");
    assert_eq!(model.get_text_a1("B8"), "#CALC!");
    assert_eq!(model.get_text_a1("D3"), "0");
}

// Excel values: probes/t3293-textjoin-own-row-limit ZZ1, B3, ZZ7. The limit passed partway along the
// formula's own row stops the read before the formula cell: #CALC!, not circular; before the limit
// (B3) or with ignore_empty TRUE (ZZ7) the self-reference is circular (0).
#[test]
fn limit_passed_inside_the_formulas_own_row_stops_the_read() {
    let mut model = new_empty_model();
    let long = "=TEXTJOIN(\"\",TRUE,REPT(\"x\",200)&SEQUENCE(160,1,0,0))";
    model.set_a1("A1", long);
    model.set_a1("ZZ1", "=TEXTJOIN(\",\",FALSE,1:1)");
    model.set_a1("A3", long);
    model.set_a1("B3", "=TEXTJOIN(\",\",FALSE,3:3)");
    model.set_a1("A7", long);
    model.set_a1("ZZ7", "=TEXTJOIN(\",\",TRUE,7:7)");
    // A cell value (not padding) takes the join over the limit; the formula cell comes next, after a
    // blank, or with ignore_empty TRUE (C11, C12, D13, D14, E15).
    let long2 = "=TEXTJOIN(\"\",TRUE,REPT(\"x\",200)&SEQUENCE(162,1,0,0))";
    for (row, column, ignore_empty) in [
        (11, "C", "FALSE"),
        (12, "C", "TRUE"),
        (13, "D", "FALSE"),
        (14, "D", "TRUE"),
    ] {
        model.set_a1(&format!("A{row}"), long2);
        model.set_a1(&format!("B{row}"), "=REPT(\"y\",250)");
        model.set_a1(
            &format!("{column}{row}"),
            &format!("=TEXTJOIN(\",\",{ignore_empty},{row}:{row})"),
        );
    }
    model.evaluate();
    for cell in ["C11", "C12", "D13", "D14"] {
        assert_eq!(model.get_text_a1(cell), "#CALC!", "{cell}");
    }
    assert_eq!(model.get_text_a1("ZZ1"), "#CALC!");
    assert_eq!(model.get_text_a1("B3"), "0");
    assert_eq!(model.get_text_a1("ZZ7"), "0");
}

// Excel values: probes/textjoin-array-arguments.
#[test]
fn array_literals_and_function_results() {
    let mut model = new_empty_model();
    model.set_a1("B1", "pear");
    model.set_a1("B3", "apple");
    model.set_a1("A1", "=TEXTJOIN(\",\",FALSE,{1,2;3,4})");
    model.set_a1("A2", "=TEXTJOIN(\",\",TRUE,{\"a\",\"\",\"c\"})");
    model.set_a1("A3", "=TEXTJOIN(\",\",FALSE,{\"a\",\"\",\"c\"})");
    model.set_a1("A4", "=TEXTJOIN(\",\",FALSE,{1,#N/A,3})");
    model.set_a1("A5", "=TEXTJOIN(\",\",FALSE,{#DIV/0!,#N/A})");
    model.set_a1("A6", "=TEXTJOIN(\",\",FALSE,SORT(B1:B3))");
    model.set_a1("A7", "=TEXTJOIN(\",\",TRUE,SORT(B1:B3))");
    model.set_a1("A8", "=TEXTJOIN(\",\",FALSE,SEQUENCE(2,2))");
    model.set_a1("A9", "=TEXTJOIN(\",\",FALSE,{1,2},5,\"x\")");
    model.set_a1("A10", "=TEXTJOIN(\",\",FALSE,{TRUE,FALSE,1.5})");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"1,2,3,4");
    assert_eq!(model.get_text_a1("A2"), *"a,c");
    assert_eq!(model.get_text_a1("A3"), *"a,,c");
    assert_eq!(model.get_text_a1("A4"), *"#N/A");
    assert_eq!(model.get_text_a1("A5"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("A6"), *"apple,pear,");
    assert_eq!(model.get_text_a1("A7"), *"apple,pear");
    assert_eq!(model.get_text_a1("A8"), *"1,2,3,4");
    assert_eq!(model.get_text_a1("A9"), *"1,2,5,x");
    assert_eq!(model.get_text_a1("A10"), *"TRUE,FALSE,1.5");
}

// Excel values: probes/textjoin-array-limit-empty-strings A7-A9.
#[test]
fn empty_strings_are_skipped_with_ignore_empty() {
    let mut model = new_empty_model();
    model.set_a1("B1", "a");
    model.set_a1("B2", "=\"\"");
    model.set_a1("B3", "c");
    model.set_a1("A1", "=TEXTJOIN(\",\",TRUE,\"a\",\"\",\"c\")");
    model.set_a1("A2", "=TEXTJOIN(\",\",TRUE,B1:B3)");
    model.set_a1("A3", "=TEXTJOIN(\",\",FALSE,B1:B3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"a,c");
    assert_eq!(model.get_text_a1("A2"), *"a,c");
    assert_eq!(model.get_text_a1("A3"), *"a,,c");
}

#[test]
fn textjoin_array_delimiter_cycles_and_counts_toward_the_limit() {
    // probes/textjoin-delimiter-array A1-A6 (excel-graph, T-1510). The limit with a cycled
    // delimiter is unprobed: the engine counts each delimiter's own size.
    let mut model = new_empty_model();
    model.set_a1("A1", "=TEXTJOIN({\",\",\";\"},FALSE,{1,2,3,4,5})");
    model.set_a1(
        "A2",
        "=TEXTJOIN({\"\",\"ab\"},FALSE,REPT(\"a\",32766),\"b\",\"c\")",
    );
    model.set_a1(
        "A3",
        "=TEXTJOIN({\"\",\"ab\"},FALSE,REPT(\"a\",32763),\"b\",\"c\")",
    );
    model.set_a1("A4", "=TEXTJOIN({\"x\",#N/A},FALSE,1,2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "1,2;3,4;5");
    assert_eq!(model.get_text_a1("A2"), "#CALC!");
    assert_eq!(model.get_text_a1("A3").len(), 32_767);
    assert_eq!(model.get_text_a1("A4"), "#N/A");
}

#[test]
fn array_or_range_ignore_empty_lifts_one_join_per_element() {
    // probes/textjoin-ignore-empty-lift rows 3, 7, 8, 13, 14 and 30 (excel-graph, T-2318).
    let mut model = new_empty_model();
    model.set_a1("C1", "TRUE");
    model.set_a1("C2", "FALSE");
    model.set_a1("A1", "=TEXTJOIN(\",\",{TRUE;FALSE},\"a\",\"\",\"b\")");
    model.set_a1("D1", "=INDEX(TEXTJOIN(\",\",C1:C2,\"a\",\"\",\"b\"),2,1)");
    model.set_a1(
        "D2",
        "=INDEX(TEXTJOIN(\",\",{TRUE;#N/A},\"a\",\"\",\"b\"),2,1)",
    );
    model.set_a1(
        "D3",
        "=INDEX(TEXTJOIN(\",\",{TRUE;\"x\"},\"a\",\"\",\"b\"),2,1)",
    );
    model.set_a1("D4", "=ROWS(TEXTJOIN(#N/A,{TRUE;FALSE},\"a\"))");
    model.set_a1("D5", "=INDEX(TEXTJOIN(#N/A,{TRUE;FALSE},\"a\"),2,1)");
    model.set_a1("D6", "=TEXTJOIN(\",\",TRUE,\"a\",\"\",\"b\")");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "a,b");
    assert_eq!(model.get_text_a1("A2"), "a,,b");
    assert_eq!(model.get_text_a1("D1"), "a,,b");
    assert_eq!(model.get_text_a1("D2"), "#N/A");
    assert_eq!(model.get_text_a1("D3"), "#VALUE!");
    assert_eq!(model.get_text_a1("D4"), "2");
    assert_eq!(model.get_text_a1("D5"), "#N/A");
    assert_eq!(model.get_text_a1("D6"), "a,b");
}

/// A bad `ignore_empty` wins over the error of a delimiter given as a reference, never over an error
/// value (probes/t3885-textjoin-error-delimiter-text-ignore-empty; T-3885).
#[test]
fn error_delimiter_with_text_ignore_empty() {
    let mut model = new_empty_model();
    model.set_a1("B1", "=1/0");
    model.set_a1("B2", "=NA()");
    model.set_a1("A1", "=TEXTJOIN(B1,\"x\",\"a\")");
    model.set_a1("A2", "=TEXTJOIN(B1,\"\",\"a\")");
    model.set_a1("A3", "=TEXTJOIN(IF(TRUE,B1),\"12\",\"a\")");
    model.set_a1("A4", "=TEXTJOIN(B1:B2,\"x\",\"a\")");
    model.set_a1("A5", "=TEXTJOIN(B1,B2,\"a\")");
    model.set_a1("A6", "=TEXTJOIN(B1,TRUE,\"a\")");
    model.set_a1("A7", "=TEXTJOIN(B1,1,\"a\")");
    model.set_a1("A8", "=TEXTJOIN(B1,,\"a\")");
    model.set_a1("A9", "=TEXTJOIN(B1:B2,TRUE,\"a\")");
    model.set_a1("A10", "=TEXTJOIN(1/0,\"x\",\"a\")");
    model.set_a1("A11", "=TEXTJOIN(NA(),B2,\"a\")");
    model.set_a1("A12", "=TEXTJOIN(#DIV/0!,\"x\",\"a\")");
    model.evaluate();
    for cell in ["A1", "A2", "A3", "A4"] {
        assert_eq!(model.get_text_a1(cell), *"#VALUE!", "{cell}");
    }
    assert_eq!(model.get_text_a1("A5"), *"#N/A");
    for cell in ["A6", "A7", "A8", "A9", "A10", "A12"] {
        assert_eq!(model.get_text_a1(cell), *"#DIV/0!", "{cell}");
    }
    assert_eq!(model.get_text_a1("A11"), *"#N/A");
}

/// A nested error delimiter is evaluated once per level: 30 levels of
/// `TEXTJOIN(TEXTJOIN(...(1/0,1,"a")...),1,"a")` keep the innermost `#DIV/0!` within the evaluation
/// budget (T-3885 review).
#[test]
fn nested_error_delimiters_are_evaluated_once() {
    let mut model = new_empty_model();
    let mut formula = "1/0".to_string();
    for _ in 0..30 {
        formula = format!("TEXTJOIN({formula},1,\"a\")");
    }
    model.set_a1("A1", &format!("={formula}"));
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"#DIV/0!");
}
