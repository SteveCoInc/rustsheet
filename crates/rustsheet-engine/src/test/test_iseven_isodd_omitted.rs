//! ISEVEN and ISODD of a LET value or LAMBDA parameter bound to nothing are the `ToolPak` `#N/A`,
//! while an operator over the omission and a blank cell read 0 (T-4928;
//! probes/t4928-iseven-isodd-omitted, excel-graph, 2026-11-07).

use crate::test::util::new_empty_model;

#[test]
fn an_omission_is_na_but_an_operator_or_a_blank_reads_zero() {
    let mut model = new_empty_model();
    let cells = [
        ("A1", "=LET(e,,ISEVEN(e))", "#N/A"),
        ("A2", "=LET(e,,ISODD(e))", "#N/A"),
        ("A3", "=LAMBDA(a,b,ISEVEN(b))(1,)", "#N/A"),
        ("A4", "=LAMBDA(a,b,ISODD(b))(1,)", "#N/A"),
        ("A5", "=LET(e,,ISEVEN(e+0))", "TRUE"),
        ("A8", "=LET(e,,ISNUMBER(e))", "FALSE"),
        ("A11", "=ISEVEN(Z1)", "TRUE"),
        ("A12", "=ISODD(Z1)", "FALSE"),
    ];
    for (cell, formula, _) in cells {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, formula, expected) in cells {
        assert_eq!(model._get_text(cell), expected, "{cell} {formula}");
    }
}
