#![allow(clippy::unwrap_used)]

use crate::test::util::{new_empty_model, whole_column_arms};

#[test]
fn test_fn_choose_args_number() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=CHOOSE()");
    model.set_a1("A2", "=CHOOSE(1)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");
}

#[test]
fn test_fn_choose_incorrect_index() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=CHOOSE(-1, 42)");
    model.set_a1("A2", "=CHOOSE(0, 42)");
    model.set_a1("A3", "=CHOOSE(1, 42)");
    model.set_a1("A4", "=CHOOSE(2, 42)");
    model.set_a1("B1", "TEST");
    model.set_a1("A5", "=CHOOSE(B1, 42)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A2"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A3"), *"42");
    assert_eq!(model.get_text_a1("A4"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A5"), *"#VALUE!");
}

#[test]
fn test_fn_choose_basic_tests() {
    let mut model = new_empty_model();
    model.set_a1("B1", "1");
    model.set_a1("B2", "2");
    model.set_a1("B3", "3");
    model.set_a1("A1", "=CHOOSE(3.1, B1, B2, B3)");
    model.set_a1("A2", "=SUM(B1:CHOOSE(1, B1, B2, B3))");
    model.set_a1("A3", "=SUM(CHOOSE(3, B1:B1, B1:B2, B1:B3))");
    model.set_a1("A4", "=CHOOSE(3,\"Wide\",115,\"world\",8)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"3");
    assert_eq!(model.get_text_a1("A2"), *"1");
    assert_eq!(model.get_text_a1("A3"), *"6");
    assert_eq!(model.get_text_a1("A4"), *"world");
}

// CHOOSE broadcasts when the index argument is an array/range: each index element
// selects the matching value argument, and the result spills with the index shape.
#[test]
fn test_fn_choose_array_index() {
    let mut model = new_empty_model();
    model.set_a1("A1", "10");
    model.set_a1("A2", "20");
    model.set_a1("A3", "30");

    // Inline array index reorders the scalar values.
    model.set_a1("C1", "=CHOOSE({1;3;2}, 100, 200, 300)");
    // Array index selecting cell references (collapsed to scalars).
    model.set_a1("E1", "=CHOOSE({1;2;3}, A1, A2, A3)");
    // An out-of-range index errors only in its own cell.
    model.set_a1("G1", "=CHOOSE({1;5}, 7, 8)");

    model.evaluate();

    assert_eq!(model.get_text_a1("C1"), "100");
    assert_eq!(model.get_text_a1("C2"), "300");
    assert_eq!(model.get_text_a1("C3"), "200");
    assert_eq!(model.get_text_a1("C4"), "");

    assert_eq!(model.get_text_a1("E1"), "10");
    assert_eq!(model.get_text_a1("E2"), "20");
    assert_eq!(model.get_text_a1("E3"), "30");

    assert_eq!(model.get_text_a1("G1"), "7");
    assert_eq!(model.get_text_a1("G2"), "#VALUE!");
}

// A scalar index keeps the original behaviour, including selecting a range that spills.
#[test]
fn test_fn_choose_scalar_index_unaffected() {
    let mut model = new_empty_model();
    model.set_a1("A1", "10");
    model.set_a1("A2", "20");
    model.set_a1("A3", "30");
    model.set_a1("C1", "=CHOOSE(1, A1:A3, 999)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), "10");
    assert_eq!(model.get_text_a1("C2"), "20");
    assert_eq!(model.get_text_a1("C3"), "30");
}

// CHOOSE passes references through: a chosen range arm reaches a range consumer
// whole (Excel: SUM(CHOOSE(2, A1:A5, B1:B5)) sums the chosen range).
#[test]
fn test_fn_choose_range_arm_flows_to_range_consumer() {
    let mut model = new_empty_model();
    model.set_a1("A1", "10");
    model.set_a1("A2", "20");
    model.set_a1("A3", "30");
    model.set_a1("C1", "=SUM(CHOOSE(2, A1:A2, A1:A3))");
    // CHOOSE as a range endpoint: A1:CHOOSE(2, A2, A3) -> A1:A3.
    model.set_a1("C2", "=SUM(A1:CHOOSE(2, A2, A3))");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), "60");
    assert_eq!(model.get_text_a1("C2"), "60");
}

// The legacy-import shape: `@` on the CHOOSE call intersects its result at the
// consuming cell (Excel upgrades legacy `=CHOOSE(2,A1:A10,...)` to `=@CHOOSE(...)`).
#[test]
fn test_fn_choose_at_call_intersects_result() {
    let mut model = new_empty_model();
    model.set_a1("A1", "10");
    model.set_a1("A2", "20");
    model.set_a1("A3", "30");
    model.set_a1("C2", "=@CHOOSE(1, A1:A3, 999)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C2"), "20");
    // No spill: C3 stays empty.
    assert_eq!(model.get_text_a1("C3"), "");
}

// A CHOOSE arm intersected to a scalar (`@B1:B6*1`) read by INDEX, MODE and LOOKUP
// (probes/legacy-choose-arm-aggregate-and-array-constant-at H1, I1, I10, J1, excel-graph, T-1643).
#[test]
fn test_fn_choose_scalar_arm_under_index_mode_lookup() {
    let mut model = new_empty_model();
    for r in 1..=6 {
        model.set_a1(&format!("A{r}"), &format!("{r}"));
    }
    for row in [1, 10] {
        model.set_a1(&format!("B{row}"), "=MODE(CHOOSE(1,@A1:A6*1))");
        model.set_a1(&format!("C{row}"), "=LOOKUP(2,CHOOSE(1,@A1:A6*1))");
        model.set_a1(&format!("D{row}"), "=INDEX(CHOOSE(1,@A1:A6*1),3)");
    }
    model.evaluate();
    // On the range's rows the arm is the scalar 1; off them its `@` is #VALUE!.
    assert_eq!(model.get_text_a1("B1"), "#N/A");
    assert_eq!(model.get_text_a1("C1"), "1");
    assert_eq!(model.get_text_a1("D1"), "#REF!");
    assert_eq!(model.get_text_a1("B10"), "#VALUE!");
    assert_eq!(model.get_text_a1("C10"), "#VALUE!");
    assert_eq!(model.get_text_a1("D10"), "#VALUE!");
}

// A dynamic INDEX of a CHOOSE with arithmetic arms keeps spilling: only a legacy formula leaves
// the `@` off the call (T-1643). MODE propagates a direct error (`MODE(1/0,1,1)` is `#DIV/0!`,
// probes/mode-typed-arguments A30; T-1291), and LOOKUP reads a scalar as a one-element array
// (`LOOKUP(2,1)` is 1, probes/mode-direct-error-lookup-scalar-vector B1; T-1651).
#[test]
fn test_fn_choose_arithmetic_arm_under_dynamic_index_spills() {
    let mut model = new_empty_model();
    for r in 1..=3 {
        model.set_a1(&format!("A{r}"), &format!("{r}"));
    }
    model.set_a1("C1", "=INDEX(CHOOSE(1,A1:A3*1),0)");
    model.set_a1("D1", "=INDEX(CHOOSE({1,2},A1:A3*1,A1:A3*2),0,2)");
    model.set_a1("E1", "=MODE(1/0,1,1)");
    model.set_a1("E2", "=LOOKUP(2,1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C3"), "3");
    assert_eq!(model.get_text_a1("D3"), "6");
    assert_eq!(model.get_text_a1("E1"), "#DIV/0!");
    assert_eq!(model.get_text_a1("E2"), "1");
}

#[test]
fn test_fn_choose_index_array_unpicked_arms_are_not_copied() {
    // 120 whole-column arms would be 120 x 1,048,576 cells copied, past the formula's work budget
    // (`#VALUE!` before T-1564); an arm no index element picks is only sized, so only C:C is.
    let mut model = new_empty_model();
    model.set_a1("C1", "5");
    model.set_a1("C2", "7");
    model.set_a1("D1", "100");
    model.set_a1(
        "A1",
        &format!("=SUM(CHOOSE({{1}},{}))", whole_column_arms(120)),
    );
    // Three picked whole columns side by side are past the array-cell limit: refused from the
    // arms' extents before any is copied.
    model.set_a1(
        "A2",
        &format!("=ROWS(CHOOSE({{1,2,3}},{}))", whole_column_arms(120)),
    );
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "12");
    assert_eq!(model.get_text_a1("A2"), "#ERROR!");
}

#[test]
fn test_fn_choose_index_array_unpicked_arm_still_sizes_the_result() {
    // lookup.md "CHOOSE over an index array broadcasts its value arguments": the result spans
    // every arm, picked or not.
    let mut model = new_empty_model();
    model.set_a1("C1", "5");
    model.set_a1("A1", "=ROWS(CHOOSE({1},C1,D1:D5))");
    model.set_a1("A2", "=SUM(CHOOSE({1},C1,D1:D5))");
    model.set_a1("A3", "=COLUMNS(CHOOSE({1},C1,D1:G1))");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "5");
    assert_eq!(model.get_text_a1("A2"), "25");
    assert_eq!(model.get_text_a1("A3"), "4");
}

#[test]
fn test_fn_choose_index_array_unpicked_arm_reads_its_cells() {
    // An unpicked range arm is still read (not kept), so a cycle through it is a cycle, as
    // through a picked one: both formulas read their own cell.
    let mut model = new_empty_model();
    model.set_a1("A1", "=ROWS(CHOOSE({1},5,A1:A2))");
    model.set_a1("B1", "=ROWS(CHOOSE({2},5,B1:B2))");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "0");
    assert_eq!(model.get_text_a1("B1"), "0");
}

#[test]
fn test_fn_choose_index_array_cycle_wins_over_an_oversized_result() {
    // Every range arm is read before the result's size is checked, so a formula that reads its
    // own cell through an arm is circular (0), not the cell-count `#ERROR!` (T-1564).
    for formula in [
        "=ROWS(CHOOSE({1,2},A:A,1:1))",
        "=ROWS(CHOOSE({1},B:B,1:1))",
        "=ROWS(CHOOSE({1,2},A:A,1/0))",
        "=ROWS(IF({TRUE,FALSE},A:A,1:1))",
        "=ROWS(IF({TRUE},B:B,1:1))",
        "=ROWS(IF({TRUE;FALSE},A1:A2,B1:XFD1048576))",
    ] {
        let mut model = new_empty_model();
        model.set_a1("A1", formula);
        model.evaluate();
        assert_eq!(model.get_text_a1("A1"), "0", "{formula}");
    }
}
