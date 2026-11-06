#![allow(clippy::unwrap_used)]

//! REDUCE keeps a reference its LAMBDA returns, as the next accumulator and as the result; SCAN
//! reads it. Excel: probes/t4478-reduce-scan-reference-accumulator (excel-graph, 2026-11-06;
//! docs/excel-semantics/arrays.md "REDUCE keeps a returned reference; SCAN reads it").

use crate::test::util::new_empty_model;

/// A1:A4 = 1, 2, 3, "x", Sheet2!A1 = 5 and Sheet3!A1 = 7 as in the probe, with `formulas` in
/// column E of the first sheet; the texts of E1, E2, ...
fn results(formulas: &[&str]) -> Vec<String> {
    let mut model = new_empty_model();
    model.add_sheet("Sheet2").unwrap();
    model.add_sheet("Sheet3").unwrap();
    for (cell, value) in [("A1", "1"), ("A2", "2"), ("A3", "3"), ("A4", "x")] {
        model._set(cell, value);
    }
    model._set("Sheet2!A1", "5");
    model._set("Sheet3!A1", "7");
    for (i, formula) in formulas.iter().enumerate() {
        model._set(&format!("E{}", i + 1), formula);
    }
    model.evaluate();
    (1..=formulas.len())
        .map(|i| model._get_text(&format!("E{i}")))
        .collect()
}

#[test]
fn reduce_keeps_a_returned_reference() {
    // Calc!H2, E3, E8, H21.
    let actual = results(&[
        r#"=REDUCE(A1,A2:A3,LAMBDA(a,v,IF(ISREF(a),v,"no")))"#,
        "=ISREF(REDUCE(A1,A2:A3,LAMBDA(a,v,v)))",
        "=ISREF(REDUCE(A1,{1,2},LAMBDA(a,v,OFFSET(a,1,0))))",
        "=SUM(REDUCE(A1,A2:A3,LAMBDA(p_a,p_v,p_a:p_v)))",
    ]);
    assert_eq!(actual, ["3", "TRUE", "TRUE", "6"]);
}

#[test]
fn scan_reads_a_returned_reference() {
    // Calc!H13: from the second step on SCAN's accumulator is a value.
    let actual = results(&[
        "=ARRAYTOTEXT(SCAN(A1,A2:A4,LAMBDA(a,v,IF(ISREF(a),v,0))),1)",
        "=ARRAYTOTEXT(SCAN(A1,A2:A3,LAMBDA(p_a,p_v,ROWS(p_a:p_v))),1)",
    ]);
    assert_eq!(actual, ["{2;0;0}", "{2;#VALUE!}"]);
}

#[test]
fn reduce_spills_a_returned_range_and_a_3d_initial_value_is_value_error() {
    // Calc!P1:P3 and Calc!H19.
    let mut model = new_empty_model();
    model.add_sheet("Sheet2").unwrap();
    model.add_sheet("Sheet3").unwrap();
    for (cell, value) in [("A1", "1"), ("A2", "2"), ("A3", "3")] {
        model._set(cell, value);
    }
    model._set("C1", "=REDUCE(A1,A2:A3,LAMBDA(p_a,p_v,p_a:p_v))");
    model._set("D1", "=REDUCE(Sheet2:Sheet3!A1,{1},LAMBDA(a,v,a))");
    model.evaluate();
    let spill: Vec<String> = (1..=4).map(|r| model._get_text(&format!("C{r}"))).collect();
    assert_eq!(spill, ["1", "2", "3", ""]);
    assert_eq!(model._get_text("D1"), "#VALUE!");
}

#[test]
fn bycol_reads_a_one_cell_reference_reduce_returns() {
    // gen-m7/611ce3bbfab27cef A1421: the REDUCE returns the reference, BYCOL reads its value.
    let actual = results(&["=BYCOL(A1:A2,LAMBDA(y,REDUCE(y,y,LAMBDA(acc,v,A3))))"]);
    assert_eq!(actual, ["3"]);
}

#[test]
fn parameter_ranges_and_unions_stay_references() {
    // Calc!H5, H9 (`a:v` is the range between the parameters, not columns A:V; step 2's `a` is
    // a value, so its `a:v` is `#VALUE!`), H7, H16, H22
    // (AREAS of a union REDUCE returns) and H17 (a union initial value is a reference).
    let actual = results(&[
        "=ARRAYTOTEXT(REDUCE(A1,A2:A3,LAMBDA(a,v,a:v)),1)",
        "=REDUCE(A1,A2:A3,LAMBDA(a,v,ROWS(a:v)))",
        "=AREAS(REDUCE(A1,A2:A3,LAMBDA(a,v,(a,v))))",
        "=AREAS(REDUCE((A1,A3),{1},LAMBDA(a,v,a)))",
        "=AREAS(REDUCE(A1,A2:A3,LAMBDA(p_a,p_v,(p_a,p_v))))",
        "=REDUCE((A1,A3),{1},LAMBDA(a,v,ISREF(a)))",
    ]);
    assert_eq!(actual, ["{1;2;3}", "#VALUE!", "3", "2", "3", "TRUE"]);
}

#[test]
fn a_range_between_parameters_keeps_its_text() {
    // The names are not read as column labels, so the text shows them as written, and a column
    // label next to one bound name stays a column range (not probed: this pins the current reading).
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("E1", "=REDUCE(A1,A2:A3,LAMBDA(a,v,ROWS(a:v)))");
    model._set("E2", "=LAMBDA(a,COLUMNS(a:C))(A1)");
    model.evaluate();
    assert_eq!(
        model._get_formula("E1"),
        "=REDUCE(A1,A2:A3,LAMBDA(a,v,ROWS(a:v)))"
    );
    assert_eq!(model._get_formula("E2"), "=LAMBDA(a,COLUMNS(A:C))(A1)");
    assert_eq!(model._get_text("E2"), "3");
}
