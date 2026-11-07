#![allow(clippy::unwrap_used)]

//! IFS and SWITCH lift element by element over array and range conditions and SWITCH
//! expressions (probes/ifs-switch-array-values, excel-graph; docs/excel-semantics/logic.md).

use crate::test::util::new_empty_model;

// A1:B2 = 1,2;3,4
fn model() -> crate::model::Model {
    let mut model = new_empty_model();
    for (cell, value) in [("A1", "1"), ("B1", "2"), ("A2", "3"), ("B2", "4")] {
        model.set_a1(cell, value);
    }
    model
}

#[test]
fn ifs_lifts_over_array_and_range_conditions() {
    let mut model = model();
    model.set_a1("D1", "=IFS({TRUE,FALSE},1,TRUE,2)");
    model.set_a1("D3", "=IFS(A1:B2>1,5)");
    // an earlier FALSE scalar condition, then an array one
    model.set_a1("D6", "=IFS(FALSE,9,{FALSE;TRUE},7,TRUE,8)");
    // an error element is that element's result
    model.set_a1("F1", "=IFS({TRUE,#DIV/0!},1)");
    // a value argument broadcasts against the condition (probes/ifs-switch-lift-edges)
    model.set_a1("H1", "=IFS({TRUE,FALSE},{10,20},TRUE,{30,40})");
    model.evaluate();
    assert_eq!(model.get_text_a1("D1"), "1");
    assert_eq!(model.get_text_a1("E1"), "2");
    assert_eq!(model.get_text_a1("D3"), "#N/A");
    assert_eq!(model.get_text_a1("E3"), "5");
    assert_eq!(model.get_text_a1("D4"), "5");
    assert_eq!(model.get_text_a1("E4"), "5");
    assert_eq!(model.get_text_a1("D6"), "8");
    assert_eq!(model.get_text_a1("D7"), "7");
    assert_eq!(model.get_text_a1("F1"), "1");
    assert_eq!(model.get_text_a1("G1"), "#DIV/0!");
    assert_eq!(model.get_text_a1("H1"), "10");
    assert_eq!(model.get_text_a1("I1"), "40");
}

#[test]
fn ifs_with_scalar_conditions_keeps_a_range_value() {
    let mut model = model();
    model.set_a1("D1", "=SUM(IFS(TRUE,A1:B2))");
    model.set_a1("D2", "=IFS(FALSE,1,TRUE,\"x\")");
    model.set_a1("D3", "=IFS(FALSE,1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("D1"), "10");
    assert_eq!(model.get_text_a1("D2"), "x");
    assert_eq!(model.get_text_a1("D3"), "#N/A");
}

#[test]
fn switch_lifts_over_an_array_or_range_expression() {
    let mut model = model();
    model.set_a1("D1", "=SWITCH({1,2},1,\"a\",2,\"b\")");
    model.set_a1("D3", "=SWITCH(A1:B2,1,\"a\",\"z\")");
    // no default: #N/A where nothing matches
    model.set_a1("D6", "=SWITCH({1,5},1,\"a\")");
    // an error element is that element's result
    model.set_a1("D8", "=SWITCH({1,#N/A},1,\"a\",\"z\")");
    // a value argument broadcasts against the expression (probes/ifs-switch-lift-edges)
    model.set_a1("D10", "=SWITCH({1,2},1,{10,20},{30,40})");
    model.evaluate();
    assert_eq!(model.get_text_a1("D1"), "a");
    assert_eq!(model.get_text_a1("E1"), "b");
    assert_eq!(model.get_text_a1("D3"), "a");
    assert_eq!(model.get_text_a1("E3"), "z");
    assert_eq!(model.get_text_a1("D4"), "z");
    assert_eq!(model.get_text_a1("E4"), "z");
    assert_eq!(model.get_text_a1("D6"), "a");
    assert_eq!(model.get_text_a1("E6"), "#N/A");
    assert_eq!(model.get_text_a1("D8"), "a");
    assert_eq!(model.get_text_a1("E8"), "#N/A");
    assert_eq!(model.get_text_a1("D10"), "10");
    assert_eq!(model.get_text_a1("E10"), "40");
}

#[test]
fn switch_with_a_scalar_expression_keeps_a_range_value() {
    let mut model = model();
    model.set_a1("D1", "=SUM(SWITCH(1,1,A1:B2))");
    model.set_a1("D2", "=SWITCH(9,1,5)");
    model.evaluate();
    assert_eq!(model.get_text_a1("D1"), "10");
    assert_eq!(model.get_text_a1("D2"), "#N/A");
}

/// An array case value lifts SWITCH over a scalar expression (probes/subtotal-take-array-selector-b
/// A10, T-2012), and a nest of them reads each case once: 2^depth reads would not finish.
#[test]
fn switch_lifts_over_an_array_case_value_and_nests_linearly() {
    let mut model = model();
    model.set_a1("D1", "=SWITCH(1,{1,2},10,20,30,40)");
    let mut nested = "SWITCH(1,{1,2},10,20)".to_string();
    for _ in 0..40 {
        nested = format!("SWITCH(1,{nested},10,20)");
    }
    model.set_a1("D3", &format!("={nested}"));
    model.evaluate();
    assert_eq!(model.get_text_a1("D1"), "10");
    assert_eq!(model.get_text_a1("E1"), "40");
    // {10,20} at the innermost level; every outer level misses its cases and takes the default.
    assert_eq!(model.get_text_a1("D3"), "20");
}

/// A one-cell range condition (`INDIRECT("K1")`) picks a branch like a cell does, and a missing
/// `value_if_false` gives FALSE (probes/subtotal-take-array-selector-b A130, T-2012).
#[test]
fn if_reads_a_one_cell_range_condition_as_a_scalar() {
    let mut model = model();
    model.set_a1("K1", "=FALSE");
    model.set_a1("K2", "=TRUE");
    model.set_a1("D1", "=IF(INDIRECT(\"K1\"),1,2)");
    model.set_a1("D2", "=IF(INDIRECT(\"K1\"),1)");
    model.set_a1("D3", "=IF(INDIRECT(\"K2\"),1,2)");
    model.set_a1("D4", "=IF(INDIRECT(\"K1\"))");
    model.evaluate();
    assert_eq!(model.get_text_a1("D1"), "2");
    assert_eq!(model.get_text_a1("D2"), "FALSE");
    assert_eq!(model.get_text_a1("D3"), "1");
    assert_eq!(model.get_text_a1("D4"), "#VALUE!");
}
