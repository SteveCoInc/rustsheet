#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// A written-empty argument of QUOTIENT or of SERIESSUM's x, n, m or coefficients is #N/A, judged left
// to right as MROUND's is (probes/t4873-empty-arg-toolpak-roundings A1:A23).
#[test]
fn an_empty_quotient_argument_is_na() {
    let mut model = new_empty_model();
    model._set("A1", "=QUOTIENT(,5)");
    model._set("A2", "=QUOTIENT(7,)");
    model._set("A3", "=QUOTIENT(,)");
    model._set("A4", "=QUOTIENT(-7,)");
    model._set("A5", "=QUOTIENT(2.5,)");
    model._set("A6", "=QUOTIENT(,-2.5)");
    model._set("A7", "=QUOTIENT(1/0,)");
    model._set("A8", "=QUOTIENT(,1/0)");
    model._set("A9", "=QUOTIENT(TRUE,)");
    model._set("A10", "=QUOTIENT(,TRUE)");
    model._set("A11", "=QUOTIENT(\"x\",)");
    model._set("A12", "=LET(e,,QUOTIENT(7,e))");
    model._set("A13", "=QUOTIENT({7,8},)");
    model._set("A14", "=SERIESSUM(,1,1,{1,2})");
    model._set("A15", "=SERIESSUM(2,,1,{1,2})");
    model._set("A16", "=SERIESSUM(2,1,,{1,2})");
    model._set("A17", "=SERIESSUM(2,1,1,)");
    model._set("A18", "=SERIESSUM(,,,)");
    model._set("A19", "=SERIESSUM(-2,,1,{1,2})");
    model._set("A20", "=SERIESSUM(0.5,1,,{1,2})");
    model._set("A21", "=SERIESSUM(1/0,,1,{1,2})");
    model._set("A22", "=SERIESSUM(2,1,1/0,)");
    model._set("A23", "=SERIESSUM(TRUE,,1,{1,2})");
    model.evaluate();
    for (cell, expected) in [
        ("A1", "#N/A"),
        ("A2", "#N/A"),
        ("A3", "#N/A"),
        ("A4", "#N/A"),
        ("A5", "#N/A"),
        ("A6", "#N/A"),
        ("A7", "#DIV/0!"),
        ("A8", "#N/A"),
        ("A9", "#VALUE!"),
        ("A10", "#N/A"),
        ("A11", "#VALUE!"),
        ("A12", "#N/A"),
        ("A13", "#N/A"),
        ("A14", "#N/A"),
        ("A15", "#N/A"),
        ("A16", "#N/A"),
        ("A17", "#N/A"),
        ("A18", "#N/A"),
        ("A19", "#N/A"),
        ("A20", "#N/A"),
        ("A21", "#DIV/0!"),
        ("A22", "#DIV/0!"),
        ("A23", "#VALUE!"),
        ("B13", "#N/A"),
    ] {
        assert_eq!(model._get_text(cell), expected, "{cell}");
    }
}

// A written-empty significance or mode of CEILING.MATH, FLOOR.MATH, CEILING.PRECISE, FLOOR.PRECISE
// and ISO.CEILING takes its default; CEILING and FLOOR still read it as 0
// (probes/t4873-empty-arg-toolpak-roundings A24:A55).
#[test]
fn an_empty_significance_of_the_newer_roundings_is_omitted() {
    let mut model = new_empty_model();
    model._set("A24", "=CEILING.MATH(2.675,)");
    model._set("A25", "=CEILING.MATH(-2.675,)");
    model._set("A26", "=CEILING.MATH(5,)");
    model._set("A27", "=CEILING.MATH(-5,)");
    model._set("A28", "=CEILING.MATH(0.3,)");
    model._set("A29", "=CEILING.MATH(2.5,,)");
    model._set("A30", "=CEILING.MATH(-2.5,,)");
    model._set("A31", "=CEILING.MATH(-2.5,2,)");
    model._set("A32", "=CEILING.MATH(-2.5,,1)");
    model._set("A33", "=CEILING.MATH(2.675,,0.1)");
    model._set("A34", "=CEILING.MATH(,2)");
    model._set("A35", "=FLOOR.MATH(2.675,)");
    model._set("A36", "=FLOOR.MATH(-2.675,)");
    model._set("A37", "=FLOOR.MATH(5,)");
    model._set("A38", "=FLOOR.MATH(-5,)");
    model._set("A39", "=FLOOR.MATH(0.3,)");
    model._set("A40", "=FLOOR.MATH(2.5,,)");
    model._set("A41", "=FLOOR.MATH(-2.5,,)");
    model._set("A42", "=FLOOR.MATH(-2.5,2,)");
    model._set("A43", "=FLOOR.MATH(-2.5,,1)");
    model._set("A44", "=FLOOR.MATH(-5,,{0,1})");
    model._set("A45", "=CEILING.PRECISE(2,)");
    model._set("A46", "=CEILING.PRECISE(-2.5,)");
    model._set("A47", "=CEILING.PRECISE(0.3,)");
    model._set("A48", "=FLOOR.PRECISE(2.5,)");
    model._set("A49", "=FLOOR.PRECISE(-2.5,)");
    model._set("A50", "=FLOOR.PRECISE(0.3,)");
    model._set("A51", "=ISO.CEILING(1.005,)");
    model._set("A52", "=ISO.CEILING(-2.5,)");
    model._set("A53", "=ISO.CEILING(0.3,)");
    model._set("A54", "=CEILING(2.5,)");
    model._set("A55", "=FLOOR(2.5,)");
    model.evaluate();
    for (cell, expected) in [
        ("A24", "3"),
        ("A25", "-2"),
        ("A26", "5"),
        ("A27", "-5"),
        ("A28", "1"),
        ("A29", "3"),
        ("A30", "-2"),
        ("A31", "-2"),
        ("A32", "-3"),
        ("A33", "3"),
        ("A34", "0"),
        ("A35", "2"),
        ("A36", "-3"),
        ("A37", "5"),
        ("A38", "-5"),
        ("A39", "0"),
        ("A40", "2"),
        ("A41", "-3"),
        ("A42", "-4"),
        ("A43", "-2"),
        ("A44", "-5"),
        ("A45", "2"),
        ("A46", "-2"),
        ("A47", "1"),
        ("A48", "2"),
        ("A49", "-3"),
        ("A50", "0"),
        ("A51", "2"),
        ("A52", "-2"),
        ("A53", "1"),
        ("A54", "0"),
        ("A55", "#DIV/0!"),
        ("B44", "-5"),
    ] {
        assert_eq!(model._get_text(cell), expected, "{cell}");
    }
}
