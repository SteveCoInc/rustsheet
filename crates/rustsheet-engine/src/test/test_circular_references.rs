#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;
use crate::types::{ArrayKind, Cell};

#[test]
fn test_simple_circ() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=A1+1");
    model.evaluate();
    assert!(model.is_circular_a1("A1"));
}

#[test]
fn test_simple_circ_propagate() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=B6");
    model.set_a1("A2", "=A1+1");
    model.set_a1("A3", "=A2+1");
    model.set_a1("A4", "=A3+5");
    model.set_a1("B6", "=A4*7");
    model.evaluate();
    assert!(model.is_circular_a1("A1"));
    assert!(model.is_circular_a1("A2"));
    assert!(model.is_circular_a1("A3"));
    assert!(model.is_circular_a1("A4"));
    assert!(model.is_circular_a1("B6"));
}

#[test]
fn every_reader_of_a_loop_is_zero_however_indirect() {
    // probes/circular-iteration-off and probes/t0445-circular-readers (excel-graph): with
    // iteration off, a loop and every cell that reads it, at any depth and whatever it
    // computes, are the number 0.
    let mut model = new_empty_model();
    model.set_a1("A1", "=A1+1");
    model.set_a1("B1", "=A1+10");
    model.set_a1("C1", "=B1+1");
    model.set_a1("D1", "=C1*2+3");
    model.set_a1("E1", "=IFERROR(A1,5)");
    model.set_a1("F1", "=ISERROR(A1)");
    model.set_a1("G1", r#"=IF(ISNUMBER(B1),"n","x")"#);
    model.set_a1("H1", "=SUM(A1:A3)+1");
    model.set_a1("A2", "=1+2");
    model.set_a1("D2", r#"=LEN("abc")+IF(FALSE,A1,1)"#);
    model.evaluate();
    for cell in ["A1", "B1", "C1", "D1", "E1", "F1", "G1", "H1"] {
        assert!(model.is_circular_a1(cell), "{cell}");
    }
    assert_eq!(model.get_text_a1("A2"), "3");
    // A reference IF does not evaluate is no read.
    assert_eq!(model.get_text_a1("D2"), "4");
    // The verdict does not depend on which cell the evaluation starts from.
    model.set_a1("A3", "=C1");
    model.evaluate();
    assert!(model.is_circular_a1("A3"));
    assert!(model.is_circular_a1("H1"));
}

#[test]
fn a_loop_and_its_readers_keep_their_values_from_before_the_recalculation() {
    // fixtures/circular-keeps-cached-values (excel-graph): with iteration off, a cell on a loop,
    // or one whose evaluation reads a loop, keeps the value it had before the recalculation (its
    // cached value in a loaded workbook); a formula never evaluated before is 0. A cell that does
    // not read the loop recalculates.
    let mut model = new_empty_model();
    model.set_a1("A1", "=B1+1");
    model.set_a1("B1", "=5");
    model.set_a1("C1", "=A1*2");
    model.set_a1("D1", "=C1+F1");
    model.set_a1("E1", "=IF(F1>100,A1,F1*10)");
    model.set_a1("F1", "=1+1");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "6");
    assert_eq!(model.get_text_a1("C1"), "12");
    assert_eq!(model.get_text_a1("D1"), "14");
    // B1 now closes the loop A1 -> B1 -> A1, and F1 changes.
    model.set_a1("B1", "=A1");
    model.set_a1("F1", "=1+2");
    model.evaluate();
    for cell in ["A1", "B1", "C1", "D1"] {
        let at = model.parse_a1(cell);
        let key = (at.sheet, at.row, at.column);
        assert!(model.evaluation.circular.contains(&key), "{cell}");
    }
    assert_eq!(model.get_text_a1("A1"), "6");
    // B1's formula was never evaluated: 0.
    assert_eq!(model.get_text_a1("B1"), "0");
    assert_eq!(model.get_text_a1("C1"), "12");
    // D1 reads the loop through C1, so it keeps 14 although F1 changed.
    assert_eq!(model.get_text_a1("D1"), "14");
    // E1 never reads A1: it recalculates.
    assert_eq!(model.get_text_a1("E1"), "30");
    assert_eq!(model.get_text_a1("F1"), "3");
}

#[test]
fn a_bare_reference_on_a_loop_keeps_its_value() {
    // calipers pv23_022 D88 `=D37`: a loop member that is only a reference keeps its value too.
    let mut model = new_empty_model();
    model.set_a1("A1", "=B1+1");
    model.set_a1("B1", "=C1");
    model.set_a1("C1", "=5");
    model.evaluate();
    assert_eq!(model.get_text_a1("B1"), "5");
    model.set_a1("C1", "=A1*1");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "6");
    assert_eq!(model.get_text_a1("B1"), "5");
}

#[test]
fn a_one_cell_dynamic_array_on_a_loop_keeps_its_value() {
    // calipers pv23_022 D88: a loop member saved as a one-cell dynamic array (`cm="1"`).
    let mut model = new_empty_model();
    model.set_a1("A1", "=B1+1");
    model.set_a1("B1", "=C1");
    model.set_a1("C1", "=5");
    model.evaluate();
    // B1 as the xlsx import loads a one-cell dynamic array with its cached value.
    let Cell::CellFormula { f, s, v } = model.get_cell_a1("B1").clone() else {
        panic!("B1 is a formula");
    };
    let anchor = Cell::ArrayFormula {
        f,
        s,
        r: (1, 1),
        kind: ArrayKind::Dynamic,
        v,
    };
    model
        .workbook
        .worksheet_mut(0)
        .unwrap()
        .sheet_data
        .set_cell(1, 2, anchor);
    model.set_a1("C1", "=A1*1");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "6");
    assert_eq!(model.get_text_a1("B1"), "5");
}
