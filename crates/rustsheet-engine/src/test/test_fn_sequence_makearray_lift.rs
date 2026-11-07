//! SEQUENCE and MAKEARRAY lift their scalar arguments over a range or an array, one call per
//! element whose value is that call's top-left (SPEC-0011 "Array-valued functions", T-1834;
//! probes/m7-argument-modes, excel-graph, 2026-10-09).

use crate::test::util::new_empty_model;

fn model_with_a1_a3() -> crate::model::Model {
    let mut model = new_empty_model();
    for (cell, value) in [("A1", "1"), ("A2", "2"), ("A3", "3")] {
        model.set_a1(cell, value);
    }
    model
}

#[test]
fn sequence_lifts_over_a_range_and_an_array() {
    let mut model = model_with_a1_a3();
    // F1:H1, F2:H2, F3:H3, K9 of probes/m7-argument-modes.
    let cells = [
        ("F1", "=ROWS(SEQUENCE(A1:A3))", "3"),
        ("G1", "=COLUMNS(SEQUENCE(A1:A3))", "1"),
        ("H1", "=INDEX(SEQUENCE(A1:A3),1,1)", "1"),
        ("F2", "=ROWS(SEQUENCE({1;2;3}))", "3"),
        ("G2", "=COLUMNS(SEQUENCE({1;2;3}))", "1"),
        ("H2", "=INDEX(SEQUENCE({1;2;3}),1,1)", "1"),
        ("F3", "=ROWS(SEQUENCE(2,2,{1;10}))", "2"),
        ("G3", "=COLUMNS(SEQUENCE(2,2,{1;10}))", "1"),
        ("H3", "=INDEX(SEQUENCE(2,2,{1;10}),1,1)", "1"),
        ("K9", "=ROWS(INDEX(SEQUENCE({1;2;3}),3,1))", "1"),
        ("K11", "=ROWS(SEQUENCE({3;4}))", "2"),
    ];
    for (cell, formula, _) in cells {
        model.set_a1(cell, formula);
    }
    model.evaluate();
    for (cell, formula, expected) in cells {
        assert_eq!(model.get_text_a1(cell), expected, "{cell} {formula}");
    }
}

#[test]
fn makearray_lifts_over_a_range() {
    let mut model = model_with_a1_a3();
    // F23:H23 and K10 of probes/m7-argument-modes; F48 and H48 keep passing.
    let cells = [
        ("F23", "=ROWS(MAKEARRAY(A1:A2,2,LAMBDA(r,c,r+c)))", "2"),
        ("G23", "=COLUMNS(MAKEARRAY(A1:A2,2,LAMBDA(r,c,r+c)))", "1"),
        ("H23", "=INDEX(MAKEARRAY(A1:A2,2,LAMBDA(r,c,r+c)),1,1)", "2"),
        (
            "K10",
            "=ROWS(INDEX(MAKEARRAY(A1:A2,2,LAMBDA(r,c,r+c)),2,1))",
            "1",
        ),
    ];
    for (cell, formula, _) in cells {
        model.set_a1(cell, formula);
    }
    model.evaluate();
    for (cell, formula, expected) in cells {
        assert_eq!(model.get_text_a1(cell), expected, "{cell} {formula}");
    }
}

#[test]
fn unlifted_calls_keep_their_values() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=SEQUENCE(2,2,10,5)");
    model.set_a1("D1", "=MAKEARRAY(2,2,LAMBDA(r,c,r*10+c))");
    model.set_a1("G1", "=SEQUENCE(0)");
    model.set_a1("H1", "=SEQUENCE(-1)");
    model.evaluate();
    for (cell, expected) in [("A1", "10"), ("B1", "15"), ("A2", "20"), ("B2", "25")] {
        assert_eq!(model.get_text_a1(cell), expected, "{cell}");
    }
    for (cell, expected) in [("D1", "11"), ("E1", "12"), ("D2", "21"), ("E2", "22")] {
        assert_eq!(model.get_text_a1(cell), expected, "{cell}");
    }
    assert_eq!(model.get_text_a1("G1"), "#CALC!");
    assert_eq!(model.get_text_a1("H1"), "#VALUE!");
}

#[test]
fn an_element_error_stays_in_its_position() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=SEQUENCE({1;0})");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "1");
    assert_eq!(model.get_text_a1("A2"), "#CALC!");
}

/// Each lifted element is charged the cells it builds (`KernelWork::Output`), so a lifted call
/// past `limits::MAX_LIFTED_CELL_VISITS` is `#ERROR!` (DEVIATION, docs/excel-semantics/arrays.md
/// "Array results above 1,048,576 cells").
#[test]
fn lifted_output_is_charged_against_the_visit_cap() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=ROWS(SEQUENCE(SEQUENCE(1024),1024))");
    model.set_a1("B1", "=ROWS(MAKEARRAY(SEQUENCE(2048),1024,LAMBDA(r,c,r)))");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#ERROR!");
    assert_eq!(model.get_text_a1("B1"), "#ERROR!");
}

/// A 1x1 array lifts to one element, but a one-cell reference, and arithmetic over one, is its
/// cell's value and runs one unlifted call that keeps its whole array: with A3 = 3,
/// `SEQUENCE({3})` and `SEQUENCE(A3+{0})` are the 1x1 1, while `SEQUENCE(A3*1)`,
/// `SEQUENCE(INDIRECT("A3")*1)` and `SEQUENCE(A3:A3)` spill three rows
/// (probes/m7-lift-take-drop-sortby Sheet1 rows 12, 14 and 38-43; T-2298). Any operator reads
/// a one-cell range as its value: `--`, `&` and `=` over INDIRECT keep three rows
/// (probes/m7-one-cell-operand-lift F2, F4, F8).
#[test]
fn a_one_cell_array_lifts_but_a_one_cell_reference_does_not() {
    let mut model = new_empty_model();
    model.set_a1("C3", "3");
    model.set_a1("A1", "=ROWS(SEQUENCE({3}))");
    model.set_a1("B1", "=ROWS(SEQUENCE(C3+{0}))");
    model.set_a1("C1", "=COLUMNS(SEQUENCE({3},2))");
    model.set_a1("D1", "=ROWS(SEQUENCE(INDIRECT(\"C3\")*1))");
    model.set_a1("E1", "=ROWS(SEQUENCE(C3:C3))");
    model.set_a1("F1", "=SEQUENCE(C3*1)");
    model.set_a1("G1", "=SEQUENCE({3})");
    model.set_a1("H1", "=ROWS(SEQUENCE(--INDIRECT(\"C3\")))");
    model.set_a1("I1", "=ROWS(SEQUENCE(INDIRECT(\"C3\")&\"\"))");
    model.set_a1("J1", "=ROWS(SEQUENCE(IF(INDIRECT(\"C3\")=3,3)))");
    model.set_a1("K1", "=ROWS(SEQUENCE(--{3}))");
    model.evaluate();
    for (cell, want) in [
        ("A1", "1"),
        ("B1", "1"),
        ("C1", "1"),
        ("D1", "3"),
        ("E1", "3"),
        ("F3", "3"),
        ("G1", "1"),
        ("G2", ""),
        ("H1", "3"),
        ("I1", "3"),
        ("J1", "3"),
        ("K1", "1"),
    ] {
        assert_eq!(model.get_text_a1(cell), want, "{cell}");
    }
}
