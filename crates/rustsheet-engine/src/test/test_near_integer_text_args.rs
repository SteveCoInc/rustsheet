#![allow(clippy::unwrap_used)]

// T-4453: which M5 text arguments count a value just below an integer as that integer
// (probes/t4453-near-integer-text-args; docs/excel-semantics/text.md "LEFT, RIGHT and REPLACE
// round counts up near an integer"). `1-2^-23` lies inside the 2^-22 + 2^-33 bound and
// `1-2^-22-9*2^-36` just outside it.

use crate::model::Model;
use crate::test::util::new_empty_model;

fn eval(cells: &[(&str, &str)]) -> Model {
    let mut model = new_empty_model();
    for (cell, formula) in cells {
        model._set(cell, formula);
    }
    model.evaluate();
    model
}

#[test]
fn snapping_arguments() {
    let model = eval(&[
        ("A1", "=FIND(\"a\",REPT(\"a\",30),1-2^-23)"),
        ("A2", "=FIND(\"a\",REPT(\"a\",30),1-(2^-22+9*2^-36))"),
        ("B1", "=SEARCH(\"a\",REPT(\"a\",30),3-2^-23)"),
        ("B2", "=SEARCH(\"a\",REPT(\"a\",30),3-(2^-22+9*2^-36))"),
        ("C1", "=CODE(CHAR(66-2^-23))"),
        ("C2", "=CODE(CHAR(66-(2^-22+9*2^-36)))"),
        (
            "D1",
            "=FIND(\"b\",SUBSTITUTE(REPT(\"a\",30),\"a\",\"b\",2-2^-23))",
        ),
        (
            "D2",
            "=FIND(\"b\",SUBSTITUTE(REPT(\"a\",30),\"a\",\"b\",2-(2^-22+9*2^-36)))",
        ),
        ("E1", "=FIXED(1/3,2-2^-23)"),
        ("E2", "=FIXED(1/3,2-(2^-22+9*2^-36))"),
        ("F1", "=DOLLAR(1234.5678,-1+2^-23)"),
        ("F2", "=DOLLAR(1234.5678,-1+2^-22+9*2^-36)"),
        ("F3", "=FIXED(1234.5678,-1E-6)"),
    ]);
    assert_eq!(model._get_text("A1"), *"1");
    assert_eq!(model._get_text("A2"), *"#VALUE!");
    assert_eq!(model._get_text("B1"), *"3");
    assert_eq!(model._get_text("B2"), *"2");
    assert_eq!(model._get_text("C1"), *"66");
    assert_eq!(model._get_text("C2"), *"65");
    assert_eq!(model._get_text("D1"), *"2");
    assert_eq!(model._get_text("D2"), *"1");
    assert_eq!(model._get_text("E1"), *"0.33");
    assert_eq!(model._get_text("E2"), *"0.3");
    assert_eq!(model._get_text("F1"), *"$1,230");
    assert_eq!(model._get_text("F2"), *"$1,235");
    assert_eq!(model._get_text("F3"), *"1,235");
}

#[test]
fn exactly_truncating_arguments() {
    let model = eval(&[
        ("A1", "=LEN(REPT(\"a\",1-1E-15))"),
        ("A2", "=LEN(REPT(\"a\",1000-2^-23))"),
        ("B1", "=UNICODE(UNICHAR(66-2^-23))"),
        ("B2", "=UNICODE(UNICHAR(1000-1E-10))"),
    ]);
    assert_eq!(model._get_text("A1"), *"0");
    assert_eq!(model._get_text("A2"), *"999");
    assert_eq!(model._get_text("B1"), *"65");
    assert_eq!(model._get_text("B2"), *"999");
}
