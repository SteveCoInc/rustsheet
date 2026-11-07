#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// PERCENTOF ranks errors across both arguments like SUM, and an error inside a reference or an
// array is #NUM! (numbers.md "PERCENTOF: an error inside a reference or array is #NUM!";
// probes/t3653-percentof-argument-kinds, excel-graph, T-3653).
#[test]
fn errors_inside_references_and_arrays_are_num() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=NA()");
    model.set_a1("A2", "=1/0");
    model.set_a1("A3", "1");
    model.set_a1("A4", "3");
    model.set_a1("C1", "=PERCENTOF(A1,A2)");
    model.set_a1("C2", "=PERCENTOF(A3,A1:A4)");
    model.set_a1("C3", "=PERCENTOF({1,#N/A},4)");
    model.set_a1("C4", "=PERCENTOF({1,2}+NA(),1)");
    model.set_a1("C5", "=PERCENTOF(IF(TRUE,A1),1)");
    model.set_a1("C6", "=PERCENTOF(A1,0)");
    model.evaluate();
    for cell in ["C1", "C2", "C3", "C4", "C5", "C6"] {
        assert_eq!(model.get_text_a1(cell), *"#NUM!", "{cell}");
    }
}

#[test]
fn direct_errors_win_in_argument_order() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=NA()");
    model.set_a1("C1", "=PERCENTOF(A1,1/0)");
    model.set_a1("C2", "=PERCENTOF(A1,\"x\")");
    model.set_a1("C3", "=PERCENTOF(NA(),1/0)");
    model.set_a1("C4", "=PERCENTOF(1/0,NA())");
    model.set_a1("C5", "=PERCENTOF(A1+0,1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("C2"), *"#VALUE!");
    assert_eq!(model.get_text_a1("C3"), *"#N/A");
    assert_eq!(model.get_text_a1("C4"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("C5"), *"#N/A");
}
