//! GROUPBY and PIVOTBY `function` arguments T-1325 left unprobed: a ragged array of functions,
//! several error elements and a LAMBDA without parameters. Excel:
//! probes/t4449-groupby-function-array-edges (excel-graph, 2026-11-06;
//! docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY").

use super::test_fn_pivotby::records;

/// Evaluates each formula of `formulas` in column H of the records and returns the texts.
fn results(formulas: &[&str]) -> Vec<String> {
    let mut model = records();
    for (i, formula) in formulas.iter().enumerate() {
        model._set(&format!("H{}", i + 1), formula);
    }
    model.evaluate();
    (1..=formulas.len())
        .map(|i| model._get_text(&format!("H{i}")))
        .collect()
}

#[test]
fn ragged_function_array_is_value_error_not_its_padding() {
    // G!C1, G!C2, G!C25: HSTACK pads the shorter part with #N/A, but the 2-D array of
    // functions is #VALUE!, not the padding's #N/A.
    let actual = results(&[
        "=GROUPBY(A2:A11,D2:D11,HSTACK(VSTACK(_xleta.SUM,_xleta.MAX),_xleta.MIN))",
        "=GROUPBY(A2:A11,D2:D11,VSTACK(HSTACK(_xleta.SUM,_xleta.MAX),_xleta.MIN))",
        "=PIVOTBY(A2:A11,C2:C11,D2:D11,HSTACK(VSTACK(_xleta.SUM,_xleta.MAX),_xleta.MIN))",
    ]);
    assert_eq!(actual, ["#VALUE!"; 3]);
}

#[test]
fn first_error_element_in_reading_order_is_the_result() {
    // G!C3:C8, G!C42: the first error wins; an error beats a value before it.
    let actual = results(&[
        "=GROUPBY(A2:A11,D2:D11,HSTACK(NA(),1/0))",
        "=GROUPBY(A2:A11,D2:D11,HSTACK(1/0,NA()))",
        "=GROUPBY(A2:A11,D2:D11,HSTACK(_xleta.SUM,#REF!,NA()))",
        "=GROUPBY(A2:A11,D2:D11,HSTACK(5,NA()))",
        "=PIVOTBY(A2:A11,C2:C11,D2:E11,HSTACK(_xleta.SUM,#REF!,NA()))",
    ]);
    assert_eq!(actual, ["#N/A", "#DIV/0!", "#REF!", "#N/A", "#REF!"]);
}

#[test]
fn lambda_without_parameters_is_value_error() {
    // G!C9:C12, G!C33:C36: alone, in an array beside an eta function and through LET.
    let actual = results(&[
        "=GROUPBY(A2:A11,D2:D11,LAMBDA(1))",
        "=GROUPBY(A2:A11,D2:E11,HSTACK(_xleta.SUM,LAMBDA(1)))",
        "=GROUPBY(A2:A11,D2:D11,LET(f,LAMBDA(1),f))",
        "=PIVOTBY(A2:A11,C2:C11,D2:D11,LAMBDA(SUM(D2:D3)))",
    ]);
    assert_eq!(actual, ["#VALUE!"; 4]);
}
