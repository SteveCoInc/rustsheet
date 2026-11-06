#![allow(clippy::unwrap_used)]

//! The ERROR.TYPE/CELL "mark" (statistics.md "ERROR.TYPE and CELL results are #NUM! in the
//! paired functions") kept through LET, IF, CHOOSE, IFERROR, a LAMBDA call and INDEX, and
//! dropped by everything else (probes/t2576-error-type-cell-paired, T-2766).

use crate::test::util::new_empty_model;

/// Sets each formula in column A (from A1), evaluates, and checks each cell's text.
fn assert_texts(cases: &[(&str, &str)]) {
    let mut model = new_empty_model();
    model._set("B1", "5");
    model._set("Z1", "=ERROR.TYPE(#REF!)");
    for (row, (formula, _)) in (1..).zip(cases) {
        model._set(&format!("A{row}"), formula);
    }
    model.evaluate();
    for (row, (formula, want)) in (1..).zip(cases) {
        assert_eq!(
            model._get_text(&format!("A{row}")),
            *want,
            "A{row}: {formula}"
        );
    }
}

#[test]
fn the_mark_passes_through_let_if_choose_iferror_lambda_and_index() {
    // probes/t2576-error-type-cell-paired Calc!D1, D2, D3, D5, D19, D20, D24
    assert_texts(&[
        ("=FTEST({1;2;3},LET(x,ERROR.TYPE(#REF!),x))", "#NUM!"),
        ("=FTEST({1;2;3},IF(TRUE,ERROR.TYPE(#REF!)))", "#NUM!"),
        ("=FTEST({1;2;3},CHOOSE(1,ERROR.TYPE(#REF!)))", "#NUM!"),
        ("=FTEST({1;2;3},IFERROR(ERROR.TYPE(#REF!),0))", "#NUM!"),
        ("=FTEST({1;2;3},LAMBDA(x,x)(ERROR.TYPE(#REF!)))", "#NUM!"),
        ("=FTEST({1;2;3},INDEX(ERROR.TYPE(#REF!),1))", "#NUM!"),
        ("=FTEST({1;2;3},ERROR.TYPE({#REF!}))", "#NUM!"),
        ("=FTEST({1;2;3},(LET(x,(CELL(\"col\",B1)),(x))))", "#NUM!"),
        (
            "=FTEST({1;2;3},IF(TRUE,LET(x,ERROR.TYPE(#REF!),y,x,y)))",
            "#NUM!",
        ),
    ]);
}

#[test]
fn anything_else_drops_the_mark() {
    // probes/t2576-error-type-cell-paired Calc!D9, D17, D18, D21, D25, D28; probes/t2436
    assert_texts(&[
        ("=FTEST({1;2;3},ERROR.TYPE(#REF!)*1)", "#DIV/0!"),
        ("=FTEST({1;2;3},-ERROR.TYPE(#REF!))", "#DIV/0!"),
        ("=FTEST({1;2;3},MAX(ERROR.TYPE(#REF!)))", "#DIV/0!"),
        ("=FTEST({1;2;3},@ERROR.TYPE(#REF!))", "#DIV/0!"),
        ("=FTEST({1;2;3},Z1)", "#DIV/0!"),
        ("=FTEST({1;2;3},LET(x,ERROR.TYPE(#REF!),x+0))", "#DIV/0!"),
        ("=FTEST({1;2;3},LET(x,ERROR.TYPE(#REF!),y,5,y))", "#DIV/0!"),
        ("=FTEST({1;2;3},IF(TRUE,ERROR.TYPE(#REF!)+0))", "#DIV/0!"),
        ("=FTEST({1;2;3},IF(FALSE,ERROR.TYPE(#REF!),4))", "#DIV/0!"),
        ("=FTEST({1;2;3},IFERROR(ERROR.TYPE(1),4))", "#DIV/0!"),
        ("=FTEST({1;2;3},SUM(LET(x,ERROR.TYPE(#REF!),x)))", "#DIV/0!"),
        (
            "=FTEST({1;2;3},LAMBDA(x,x+0)(ERROR.TYPE(#REF!)))",
            "#DIV/0!",
        ),
        ("=FTEST({1;2;3},INDEX({4,5},1))", "#DIV/0!"),
    ]);
}

#[test]
fn a_let_bound_array_of_references_keeps_its_mark() {
    // probes/t2577-indirect-ref-array Calc!A64; statistics.md "An array of references is #NUM!
    // in the paired functions"
    assert_texts(&[
        ("=F.TEST(LET(x,INDIRECT(\"B1\",{1}),x),{-1,0,1})", "#NUM!"),
        ("=F.TEST(INDEX(INDIRECT(\"B1\",{1}),1),{-1,0,1})", "#DIV/0!"),
    ]);
}

#[test]
fn sumxmy2_and_linest_reject_a_marked_first_argument() {
    // probes/t2576-error-type-cell-paired Calc!E16, E34; probes/t2577-indirect-ref-array A148
    assert_texts(&[
        ("=SUMXMY2(ERROR.TYPE(#REF!),5)", "#NUM!"),
        ("=SUMXMY2(INDIRECT(\"B1\",{1}),4)", "#NUM!"),
        ("=SUMXMY2(ERROR.TYPE(#REF!)+0,5)", "1"),
        ("=LINEST(ERROR.TYPE(#REF!))", "#VALUE!"),
        ("=LINEST(2)", "0"),
    ]);
}

#[test]
fn the_mark_in_the_positions_probed_by_t2766() {
    // probes/t2766-mark-positions A8:A12, A15:A19, A23:A25, A28
    assert_texts(&[
        (
            "=CORREL({1;2;3},ERROR.TYPE({#REF!;#DIV/0!;#VALUE!}))",
            "#DIV/0!",
        ),
        (
            "=SLOPE({1;2;3},ERROR.TYPE({#REF!;#DIV/0!;#VALUE!}))",
            "#DIV/0!",
        ),
        ("=CORREL({1;2;3},{4;2;3})", "-0.5"),
        ("=SUMX2MY2(ERROR.TYPE(#REF!),5)", "#NUM!"),
        ("=SUMXMY2(ERROR.TYPE(#REF!),{1;2})", "#N/A"),
        ("=SUMXMY2(ERROR.TYPE(#REF!),1/0)", "#DIV/0!"),
        ("=SUMXMY2(5,ERROR.TYPE(#REF!))", "#NUM!"),
        ("=FTEST({1;2;3},IFNA(ERROR.TYPE(#REF!),0))", "#NUM!"),
        ("=FTEST({1;2;3},IFS(TRUE,ERROR.TYPE(#REF!)))", "#DIV/0!"),
        (
            "=FTEST({1;2;3},LET(f,LAMBDA(x,x),f(ERROR.TYPE(#REF!))))",
            "#NUM!",
        ),
    ]);
}

#[test]
fn the_mark_passes_through_nested_wrappers_in_any_order() {
    // probes/t2766-mark-positions-2 A1:A9
    assert_texts(&[
        (
            "=FTEST({1;2;3},CHOOSE(1,LET(x,ERROR.TYPE(#REF!),x)))",
            "#NUM!",
        ),
        (
            "=FTEST({1;2;3},CHOOSE(1,LAMBDA(x,x)(ERROR.TYPE(#REF!))))",
            "#NUM!",
        ),
        (
            "=FTEST({1;2;3},LET(x,CHOOSE(1,ERROR.TYPE(#REF!)),x))",
            "#NUM!",
        ),
        (
            "=FTEST({1;2;3},INDEX(LET(x,ERROR.TYPE(#REF!),x),1))",
            "#NUM!",
        ),
        ("=IRR(CHOOSE(1,LET(x,CELL(\"row\",B1),x)))", "#VALUE!"),
        (
            "=FTEST({1;2;3},IF(TRUE,LET(x,ERROR.TYPE(#REF!),x)))",
            "#NUM!",
        ),
        (
            "=FTEST({1;2;3},LAMBDA(x,x)(CHOOSE(1,ERROR.TYPE(#REF!))))",
            "#NUM!",
        ),
        (
            "=FTEST({1;2;3},INDEX(CHOOSE(1,ERROR.TYPE(#REF!)),1))",
            "#NUM!",
        ),
        (
            "=FTEST({1;2;3},IFERROR(LET(x,ERROR.TYPE(#REF!),x),0))",
            "#NUM!",
        ),
        // Still dropped by anything that makes a new value on the way.
        (
            "=FTEST({1;2;3},CHOOSE(1,LET(x,ERROR.TYPE(#REF!),x+0)))",
            "#DIV/0!",
        ),
        (
            "=FTEST({1;2;3},LET(x,CHOOSE(1,ERROR.TYPE(#REF!)+0),x))",
            "#DIV/0!",
        ),
    ]);
}

#[test]
fn a_marked_array_passes_and_an_array_condition_drops_it() {
    // probes/t2766-mark-positions-2 A10:A16
    assert_texts(&[
        ("=FTEST({1;2;3},IF(TRUE,ERROR.TYPE({#REF!;#N/A})))", "#NUM!"),
        (
            "=FTEST({1;2;3},CHOOSE(1,ERROR.TYPE({#REF!;#N/A})))",
            "#NUM!",
        ),
        ("=FTEST({1;2;3},LET(x,ERROR.TYPE({#REF!;#N/A}),x))", "#NUM!"),
        (
            "=FTEST({1;2;3},LAMBDA(x,x)(ERROR.TYPE({#REF!;#N/A})))",
            "#NUM!",
        ),
        (
            "=FTEST({1;2;3},INDEX(ERROR.TYPE({#REF!;#N/A}),0,1))",
            "#NUM!",
        ),
        (
            "=FTEST({1;2;3},IF({TRUE;TRUE},ERROR.TYPE(#REF!)))",
            "#DIV/0!",
        ),
    ]);
}

#[test]
fn second_arguments_and_fallbacks_probed_by_the_review() {
    // probes/t2766-mark-positions-2 A17:A28
    assert_texts(&[
        ("=CORREL({1;2},INDIRECT(\"B1\",{1;1}))", "#DIV/0!"),
        ("=INTERCEPT(5,INDIRECT(\"B1\",{1}))", "#DIV/0!"),
        ("=SUMXMY2(4,INDIRECT(\"B1\",{1}))", "#NUM!"),
        ("=LINEST(ERROR.TYPE(#REF!),1/0)", "#DIV/0!"),
        ("=SUMXMY2(ERROR.TYPE(#REF!),{#N/A})", "#NUM!"),
        ("=CORREL({1;2;3},ERROR.TYPE({#REF!;5;#VALUE!}))", "#N/A"),
        ("=FTEST({1;2;3},IFERROR(1/0,ERROR.TYPE(#REF!)))", "#NUM!"),
        ("=FTEST({1;2;3},IF(FALSE,4,ERROR.TYPE(#REF!)))", "#NUM!"),
        (
            "=CHITEST({1;2;3},ERROR.TYPE({#REF!;#DIV/0!;#VALUE!}))",
            "#NUM!",
        ),
        ("=CHITEST({1;2},INDIRECT(\"B1\",{1;1}))", "#NUM!"),
    ]);
}

#[test]
fn a_recursive_lambda_is_marked_only_when_its_outer_call_returns_the_mark() {
    // probes/t2766-mark-positions-2 A29, A30: only the inner call returns ERROR.TYPE. The outer
    // call evaluates the same `f(f,n-1)` node again inside itself, so the inner evaluation's
    // mark must not be read as the outer one's.
    assert_texts(&[
        (
            "=FTEST({1;2;3},LET(f,LAMBDA(f,n,IF(n=0,ERROR.TYPE(#REF!),f(f,n-1)+0)),f(f,1)))",
            "#DIV/0!",
        ),
        (
            "=FTEST({1;2;3},LET(f,LAMBDA(f,n,IF(n=0,ERROR.TYPE(#REF!),f(f,n-1))),f(f,1)))",
            "#NUM!",
        ),
        (
            "=FTEST({1;2;3},LET(f,LAMBDA(f,n,IF(n=0,ERROR.TYPE(#REF!),IF(n=1,f(f,n-1)+0,f(f,n-1)))),f(f,2)))",
            "#DIV/0!",
        ),
        // The reader's own argument node re-entered: at n=2 LINEST reads `f(f,n-1)`, whose
        // evaluation runs LINEST of the same node at n=1, marked there (`#VALUE!`, so 7). The
        // outer read is the unmarked 7, and `LINEST(7)` is 0 (the rules of
        // t2576-error-type-cell-paired E34 and t2766-mark-positions A28 combined).
        (
            "=LET(f,LAMBDA(f,n,IF(n=0,ERROR.TYPE(#REF!),IFERROR(LINEST(f(f,n-1)),7))),f(f,2))",
            "0",
        ),
    ]);
}
