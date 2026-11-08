//! A written-empty branch that IF, CHOOSE, IFERROR or IFNA picks reaches a reader of a
//! written-empty argument as an omission: the `ToolPak` functions are `#N/A` and the newer
//! roundings take their default, while IFS, SWITCH, an operator, an array condition and a
//! blank-valued branch read 0 (T-4912; probes/t4901-mround-blank-let-lambda-roundings,
//! t5067-written-empty-branch-wrappers, t4912-written-empty-branch-readers, excel-graph).

use crate::test::util::new_empty_model;

fn check(cells: &[(&str, &str, &str)]) {
    let mut model = new_empty_model();
    for (cell, formula, _) in cells {
        model.set_a1(cell, formula);
    }
    model.evaluate();
    for (cell, formula, expected) in cells {
        assert_eq!(model.get_text_a1(cell), *expected, "{cell} {formula}");
    }
}

#[test]
fn a_picked_written_empty_branch_is_an_omission() {
    check(&[
        ("A1", "=MROUND(IF(TRUE,),1)", "#N/A"),
        ("A2", "=MROUND(IF(FALSE,1,),1)", "#N/A"),
        ("A3", "=MROUND(CHOOSE(1,,2),1)", "#N/A"),
        ("A4", "=QUOTIENT(IF(TRUE,),1)", "#N/A"),
        ("A5", "=QUOTIENT(7,IF(TRUE,))", "#N/A"),
        ("A6", "=MROUND(IFERROR(,1),1)", "#N/A"),
        ("A7", "=MROUND(IFNA(NA(),),1)", "#N/A"),
        ("A8", "=MROUND(IFERROR(1/0,),1)", "#N/A"),
        ("A9", "=MROUND((IF(TRUE,)),1)", "#N/A"),
        ("A10", "=MROUND(+IF(TRUE,),1)", "#N/A"),
        ("A11", "=MROUND(IF(FALSE,1,IF(TRUE,)),1)", "#N/A"),
        ("A12", "=MROUND(IFERROR(IF(TRUE,),1),1)", "#N/A"),
        ("A13", "=LET(x,IF(TRUE,),MROUND(x,1))", "#N/A"),
        ("A14", "=LET(x,IF(TRUE,),ISOMITTED(x))", "TRUE"),
        ("A15", "=LAMBDA(a,MROUND(a,1))(IF(TRUE,))", "#N/A"),
        ("A16", "=CEILING.MATH(2.675,IF(TRUE,))", "3"),
        ("A17", "=FLOOR.PRECISE(-2.5,IF(TRUE,))", "-3"),
        ("A18", "=SQRTPI(IF(TRUE,))", "#N/A"),
        ("A19", "=FACTDOUBLE(IF(TRUE,))", "#N/A"),
        ("A20", "=EDATE(IF(TRUE,),1)", "#N/A"),
        ("A21", "=SERIESSUM(2,1,1,IF(TRUE,))", "#N/A"),
        ("A22", "=DEC2BIN(5,IF(TRUE,))", "101"),
        ("A23", "=IMSUB(\"3+4i\",IF(TRUE,))", "#N/A"),
        ("A24", "=ISEVEN(IF(TRUE,))", "#N/A"),
        ("A25", "=WEEKNUM(45000,IF(TRUE,))", "11"),
        ("A26", "=MROUND(IFERROR(MROUND(IF(TRUE,),1),),1)", "#N/A"),
    ]);
}

#[test]
fn a_value_or_another_wrapper_reads_zero() {
    check(&[
        ("A1", "=MROUND(IFERROR(Z1,),1)", "0"),
        ("A2", "=MROUND(IF(TRUE,Z1,),1)", "0"),
        ("A3", "=MROUND(IFS(TRUE,),1)", "0"),
        ("A4", "=MROUND(SWITCH(1,1,),1)", "0"),
        ("A5", "=MROUND(IF(TRUE,)+0,1)", "0"),
        ("A6", "=INDEX(MROUND(IF({TRUE;FALSE},,1),1),1)", "0"),
        ("A7", "=MROUND(IFNA(1/0,),1)", "#DIV/0!"),
        ("A8", "=MROUND(IF(\"x\",,1),1)", "#VALUE!"),
        ("A9", "=MROUND(CHOOSE(3,1,),1)", "#VALUE!"),
        ("A10", "=LET(x,IF(FALSE,,1),MROUND(x,1))", "1"),
        ("A11", "=CEILING.MATH(IF(TRUE,),1)", "0"),
        ("A12", "=MROUND(IF(Z1,,1),1)", "1"),
    ]);
}

/// A LAMBDA parameter the body reads once is moved into that read: the probe reads the
/// condition first, so the parameter is copied before, and the call's own read still finds it.
#[test]
fn a_parameter_read_once_survives_the_probe() {
    check(&[
        ("A1", "=LAMBDA(c,LET(x,IF(c,,1),MROUND(x,1)))(TRUE)", "#N/A"),
        ("A2", "=LAMBDA(c,LET(x,IF(c,,1),MROUND(x,1)))(FALSE)", "1"),
        ("A3", "=LAMBDA(c,LET(x,IF(c,1,),MROUND(x,1)))(TRUE)", "1"),
    ]);
}

#[test]
fn a_wrapper_whose_pick_cannot_be_repeated_is_not_probed() {
    // A KNOWN GAP, not Excel's answer: Excel's one evaluation hands each of A1, A3 and A4 on as an
    // omission (`#N/A`), and T-5099 flips them. The probe would evaluate the condition a second
    // time, so a random draw or a call whose body it cannot see (a LET-bound or defined-name
    // LAMBDA) is not probed and the branch reads as 0 (numbers.md "Which wrappers hand a
    // written-empty branch on as an omission", Ours). A LAMBDA written in the formula is walked,
    // so A2 is probed and hands the omission on, as the same form without a call does (A5).
    check(&[
        ("A1", "=MROUND(CHOOSE(RANDBETWEEN(1,1),,5),1)", "0"),
        ("A2", "=MROUND(IF(LAMBDA(x,x)(TRUE),,1),1)", "#N/A"),
        ("A3", "=LET(f,LAMBDA(x,x),MROUND(IF(f(TRUE),,1),1))", "0"),
        ("A4", "=MROUND(IF(LAMBDA(x,RAND()<2)(1),,1),1)", "0"),
        ("A5", "=MROUND(CHOOSE(1,,5),1)", "#N/A"),
    ]);
}
