#![allow(clippy::unwrap_used)]

//! Lotus-style dotted ranges: `A1.B2` in a formula is the range `A1:B2` (docs/excel-semantics/
//! references.md "R1C1-shaped sheet and defined names"; probes/lotus-dot-range*, excel-graph,
//! T-1659).

use crate::test::util::new_empty_model;
use crate::Model;

fn model_with_values() -> Model {
    let mut model = new_empty_model();
    model.add_sheet("Data").unwrap();
    for (cell, value) in [
        ("A1", "1"),
        ("B1", "4"),
        ("A2", "8"),
        ("B2", "2"),
        ("C3", "16"),
    ] {
        model.set_a1(cell, value);
    }
    model.set_a1("Data!A1", "10");
    model.set_a1("Data!B2", "20");
    model
}

#[test]
fn dotted_ranges_read_and_print_as_colon_ranges() {
    let mut model = model_with_values();
    let cases = [
        ("=SUM(A1.B2)", "=SUM(A1:B2)", "15"),
        ("=SUM(B2.A1)", "=SUM(A1:B2)", "15"),
        ("=SUM(a1..b2)", "=SUM(A1:B2)", "15"),
        ("=SUM(A1...B2)", "=SUM(A1:B2)", "15"),
        ("=SUM($A$1.B2)", "=SUM($A$1:B2)", "15"),
        ("=SUM(A$1..$B$2)", "=SUM(A$1:$B$2)", "15"),
        ("=SUM(A1.$B$2)", "=SUM(A1:$B$2)", "15"),
        ("=SUM(A1 . B2)", "=SUM(A1:B2)", "15"),
        // Whitespace on one side of the dots only (probes/t4271-lotus-dot-one-side-*, T-4271).
        ("=SUM(A1 .B2)", "=SUM(A1:B2)", "15"),
        ("=SUM(A1. B2)", "=SUM(A1:B2)", "15"),
        ("=SUM(A1 ..B2)", "=SUM(A1:B2)", "15"),
        ("=SUM(a1.. b2)", "=SUM(A1:B2)", "15"),
        ("=SUM(A1\n.B2)", "=SUM(A1:B2)", "15"),
        // Whitespace after the dots on a line feed or a `$` cell (probes/t4284-lotus-dot-ws-after-*, T-4284).
        ("=SUM(A1.\nB2)", "=SUM(A1:B2)", "15"),
        ("=SUM($A$1. B2)", "=SUM($A$1:B2)", "15"),
        ("=SUM(A$1.. B2)", "=SUM(A$1:B2)", "15"),
        ("=ROWS(A1. B2)", "=ROWS(A1:B2)", "2"),
        ("=ISREF(A1.. B2)", "=ISREF(A1:B2)", "TRUE"),
        ("=SUM(Data!A1.B2)", "=SUM(Data!A1:B2)", "30"),
        ("=SUM('Data'!B2..A1)", "=SUM(Data!A1:B2)", "30"),
        ("=ISREF(A1.B2)", "=ISREF(A1:B2)", "TRUE"),
        ("=SUM(A1.B2 B1:B3)", "=SUM(A1:B2 B1:B3)", "6"),
        ("=ROWS(A1.B2:C3)", "=ROWS(A1:B2:C3)", "3"),
    ];
    for (row, (formula, ..)) in cases.iter().enumerate() {
        model.set_a1(&format!("F{}", row + 1), formula);
    }
    model.evaluate();
    let mut reloaded = Model::from_bytes(&model.to_bytes(), "en").unwrap();
    reloaded.evaluate();
    for (row, (formula, text, value)) in cases.iter().enumerate() {
        let cell = format!("F{}", row + 1);
        for m in [&model, &reloaded] {
            assert_eq!(m.get_formula_a1(&cell), *text, "{formula}");
            assert_eq!(m.get_text_a1(&cell), *value, "{formula}");
        }
    }
}

// T-4271 fallback: no second cell after the dots restores the lexer position. Excel refuses `A1. 5` at load
// (`FileCorruptTryRepair`, so no oracle case); ours keeps the pre-T-4271 reading.
#[test]
fn dots_and_whitespace_without_a_second_cell_keep_the_old_reading() {
    let mut model = model_with_values();
    for (cell, formula) in [
        ("F1", "=SUM(A1. 5)"),
        ("F2", "=SUM(A1. )"),
        ("F3", "=A1. +1"),
    ] {
        model.set_a1(cell, formula);
    }
    model.evaluate();
    assert_eq!(model.get_text_a1("F1"), "#ERROR!");
    // `A1.` stays a name here, so the trailing-dot forms are #NAME? (the task text expected #ERROR!).
    assert_eq!(model.get_text_a1("F2"), "#NAME?");
    assert_eq!(model.get_text_a1("F3"), "#NAME?");
    assert_eq!(model.get_formula_a1("F3"), "=A1. +1");
}

// probes/lotus-dot-range-{column,column-dotdot,number,offgrid,trailing-letter,name-second}:
// no range, a name Excel cannot resolve, kept as typed. `A1.B2.C3` makes Excel reject the file.
#[test]
fn other_dotted_names_stay_names() {
    let mut model = model_with_values();
    let formulas = [
        "=SUM(A.B)",
        "=SUM(A..B)",
        "=SUM(A1.5)",
        "=SUM(A1.XFE1)",
        "=SUM(A1.B2D)",
        "=SUM(A1.foo)",
        "=SUM(A1.B2.C3)",
    ];
    for (row, formula) in formulas.iter().enumerate() {
        model.set_a1(&format!("F{}", row + 1), formula);
    }
    model.evaluate();
    for (row, formula) in formulas.iter().enumerate() {
        let cell = format!("F{}", row + 1);
        assert_eq!(model.get_text_a1(&cell), "#NAME?", "{formula}");
        assert_eq!(model.get_formula_a1(&cell), *formula);
    }
}

// probes/r1c1-name-r1-dot-c1, -c1-c1-dot: the range wins over a defined name spelled so.
#[test]
fn a_dotted_range_shadows_a_defined_name() {
    let mut model = new_empty_model();
    model.set_a1("B1", "7");
    model.define_name("R1.C1", None, "=Sheet1!$B$1").unwrap();
    model.define_name("C1.C1", None, "=Sheet1!$B$1").unwrap();
    model.set_a1("A1", "=R1.C1*1");
    model.set_a1("A3", "=C1.C1*1");
    model.set_a1("A4", "=ISREF(R1.C1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#SPILL!");
    assert_eq!(model.get_text_a1("A3"), "0");
    assert_eq!(model.get_text_a1("A4"), "TRUE");
}

// probes/r1c1-let-name-r1-c1, lotus-dot-name-*: a LET or LAMBDA binding the name makes it the variable within the
// call; outside it the name is the range again (printed with `:`; the formulas below are written
// as Excel prints them).
#[test]
fn let_and_lambda_names_spelled_like_dotted_ranges_are_variables_in_scope() {
    let mut model = model_with_values();
    let cases = [
        ("=LET(R1.C1,5,R1.C1*2)", "10"),
        // A use prints in the spelling of its declaration (probes/lotus-dot-name-let-lower, T-1720).
        ("=LET(a1.b2,5,a1.b2*2)", "10"),
        ("=LET(x,1,SUM(A1:B2))", "15"),
        ("=LET(A1.B2,5,A1.B2)+SUM(A1:B2)", "20"),
        // Unprobed as typed (the probe file prefixed the first use; references.md).
        ("=LET(x,SUM(A1:B2),A1.B2,x,A1.B2+1)", "16"),
        ("=LAMBDA(A1.B2,A1.B2*2)(4)", "8"),
        ("=LAMBDA([A1.B2],ISOMITTED(A1.B2))()", "TRUE"),
        ("=LAMBDA(x,LAMBDA(A1.B2,A1.B2+x)(1)+SUM(A1:B2))(2)", "18"),
    ];
    for (row, (formula, _)) in cases.iter().enumerate() {
        model.set_a1(&format!("F{}", row + 1), formula);
    }
    model.evaluate();
    let mut reloaded = Model::from_bytes(&model.to_bytes(), "en").unwrap();
    reloaded.evaluate();
    for (row, (formula, value)) in cases.iter().enumerate() {
        let cell = format!("F{}", row + 1);
        for m in [&model, &reloaded] {
            assert_eq!(m.get_text_a1(&cell), *value, "{formula}");
            assert_eq!(m.get_formula_a1(&cell), *formula);
        }
    }
}
