//! SQRTPI and FACTDOUBLE of a LET value or LAMBDA parameter bound to nothing are the `ToolPak`
//! `#N/A`, while an operator over the omission reads it as 0 and a blank cell is 0 (T-4923;
//! probes/t4923-sqrtpi-factdouble-omitted, excel-graph, 2026-11-07).

use crate::test::util::new_empty_model;

#[test]
fn an_omission_is_na_but_an_operator_or_a_blank_reads_zero() {
    let mut model = new_empty_model();
    let cells = [
        ("A1", "=LET(e,,SQRTPI(e))", "#N/A"),
        ("A2", "=LAMBDA(a,b,SQRTPI(b))(1,)", "#N/A"),
        ("A3", "=LET(e,,FACTDOUBLE(e))", "#N/A"),
        ("A4", "=LAMBDA(a,b,FACTDOUBLE(b))(1,)", "#N/A"),
        ("A6", "=LET(e,,SQRTPI(e+0))", "0"),
        ("A7", "=LET(e,,FACTDOUBLE(e+0))", "1"),
        ("A12", "=SQRTPI(Z1)", "0"),
        ("A13", "=FACTDOUBLE(Z1)", "1"),
    ];
    for (cell, formula, _) in cells {
        model.set_a1(cell, formula);
    }
    model.evaluate();
    for (cell, formula, expected) in cells {
        assert_eq!(model.get_text_a1(cell), expected, "{cell} {formula}");
    }
}
