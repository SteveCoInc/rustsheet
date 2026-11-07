#![allow(clippy::unwrap_used)]

//! The lookup and reference scalar arguments that lift over a range or an array, and the index
//! lists CHOOSECOLS and CHOOSEROWS read whole (T-1241; probes/lookup-argument-modes,
//! docs/excel-semantics/lookup.md "Lookup and reference function argument modes").

use crate::test::util::new_empty_model;

#[test]
fn address_lifts_over_a_range_and_an_array() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("A3", "3");
    model.set_a1("C1", "=ADDRESS(A1:A3,1)");
    model.set_a1("D1", "=ADDRESS(1,{1,2},{1;4})");
    model.set_a1("F1", "=ADDRESS({1;0},1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), *"$A$1");
    assert_eq!(model.get_text_a1("C3"), *"$A$3");
    assert_eq!(model.get_text_a1("D1"), *"$A$1");
    assert_eq!(model.get_text_a1("E1"), *"$B$1");
    assert_eq!(model.get_text_a1("E2"), *"B1");
    // Each element is its own scalar call: a bad row is `#VALUE!` in its slot only.
    assert_eq!(model.get_text_a1("F1"), *"$A$1");
    assert_eq!(model.get_text_a1("F2"), *"#VALUE!");
}

#[test]
fn hyperlink_lifts_its_location_and_friendly_name() {
    let mut model = new_empty_model();
    model.set_a1("A1", "x");
    model.set_a1("A2", "y");
    model.set_a1("C1", "=HYPERLINK({\"a\";\"b\"})");
    model.set_a1("D1", "=HYPERLINK(\"#A1\",A1:A2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), *"a");
    assert_eq!(model.get_text_a1("C2"), *"b");
    assert_eq!(model.get_text_a1("D1"), *"x");
    assert_eq!(model.get_text_a1("D2"), *"y");
}

#[test]
fn lookup_and_xmatch_lift_their_lookup_value() {
    let mut model = new_empty_model();
    for (cell, value) in [
        ("A1", "1"),
        ("A2", "2"),
        ("A3", "3"),
        ("B1", "10"),
        ("B2", "20"),
        ("B3", "30"),
    ] {
        model.set_a1(cell, value);
    }
    model.set_a1("D1", "=LOOKUP(A1:A3,A1:A3,B1:B3)");
    model.set_a1("E1", "=LOOKUP({2.5;0;9},A1:A3,B1:B3)");
    model.set_a1("F1", "=XMATCH({3;1},A1:A3)");
    model.set_a1("G1", "=XMATCH(2.5,A1:A3,{0;1;-1})");
    model.set_a1("H1", "=LOOKUP({1;2},{1,2,3},{4,5,6})");
    model.evaluate();
    assert_eq!(model.get_text_a1("D1"), *"10");
    assert_eq!(model.get_text_a1("D3"), *"30");
    // Below the first value is `#N/A` in that element only.
    assert_eq!(model.get_text_a1("E1"), *"20");
    assert_eq!(model.get_text_a1("E2"), *"#N/A");
    assert_eq!(model.get_text_a1("E3"), *"30");
    assert_eq!(model.get_text_a1("F1"), *"3");
    assert_eq!(model.get_text_a1("F2"), *"1");
    // match_mode lifts: exact, next larger, next smaller.
    assert_eq!(model.get_text_a1("G1"), *"#N/A");
    assert_eq!(model.get_text_a1("G2"), *"3");
    assert_eq!(model.get_text_a1("G3"), *"2");
    assert_eq!(model.get_text_a1("H1"), *"4");
    assert_eq!(model.get_text_a1("H2"), *"5");
}

#[test]
fn xmatch_lifts_its_search_mode() {
    // probes/reference-lift-elements F13:H14 (excel-graph) back the second elements and the TRUE
    // range element; the first elements are hand-written, pending an oracle run.
    let mut model = new_empty_model();
    for (cell, value) in [("A1", "1"), ("A2", "2"), ("A3", "2"), ("D1", "TRUE")] {
        model.set_a1(cell, value);
    }
    model.set_a1("F1", "=XMATCH(2,A1:A3,0,{1;-1})");
    model.set_a1("G1", "=XMATCH(2,A1:A3,0,{1;3})");
    model.set_a1("H1", "=XMATCH(2,A1:A3,0,{2;-2})");
    model.set_a1("I1", "=XMATCH(2,A1:A3,0,D1:D2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("F1"), *"2");
    assert_eq!(model.get_text_a1("F2"), *"3");
    assert_eq!(model.get_text_a1("G1"), *"2");
    assert_eq!(model.get_text_a1("G2"), *"#VALUE!");
    assert_eq!(model.get_text_a1("H2"), *"3");
    assert_eq!(model.get_text_a1("I1"), *"2");
}

#[test]
fn choosecols_and_chooserows_read_index_lists_whole() {
    let mut model = new_empty_model();
    for (cell, value) in [
        ("A1", "1"),
        ("B1", "2"),
        ("A2", "3"),
        ("B2", "4"),
        ("A3", "5"),
        ("B3", "6"),
        ("D1", "2"),
        ("D2", "-1"),
    ] {
        model.set_a1(cell, value);
    }
    model.set_a1("F1", "=CHOOSECOLS(A1:B3,D1:D2)");
    model.set_a1("I1", "=CHOOSEROWS(A1:B3,{3,1},2)");
    model.set_a1("L1", "=CHOOSEROWS(A1:B3,{1;4})");
    model.evaluate();
    // A column of indexes picks columns: 3x2, B then B (2 and -1).
    assert_eq!(model.get_text_a1("F1"), *"2");
    assert_eq!(model.get_text_a1("G1"), *"2");
    assert_eq!(model.get_text_a1("G3"), *"6");
    assert_eq!(model.get_text_a1("F4"), *"");
    // A row of indexes and a scalar add up to rows 3, 1, 2.
    assert_eq!(model.get_text_a1("I1"), *"5");
    assert_eq!(model.get_text_a1("I2"), *"1");
    assert_eq!(model.get_text_a1("J3"), *"4");
    // One index out of range fails the whole call, as a scalar one does.
    assert_eq!(model.get_text_a1("L1"), *"#VALUE!");
}

#[test]
fn chosen_index_lists_are_bounded_before_the_result_is_built() {
    let mut model = new_empty_model();
    // 2048 rows by 1024 copies of column 1 is past the 2^20-cell array bound.
    model.set_a1("C1", "=ROWS(CHOOSECOLS(A1:A2048,SEQUENCE(1,1024,1,0)))");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), *"#ERROR!");
}

#[test]
fn formulatext_gives_one_result_per_cell() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "=1+1");
    model.set_a1("C1", "=FORMULATEXT(A1:A3)");
    model.set_a1("D1", "=ROWS(FORMULATEXT(A1:B3))");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), *"#N/A");
    assert_eq!(model.get_text_a1("C2"), *"=1+1");
    assert_eq!(model.get_text_a1("C3"), *"#N/A");
    assert_eq!(model.get_text_a1("D1"), *"3");
}
