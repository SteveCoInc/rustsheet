#![allow(clippy::unwrap_used)]

//! Lifting a range into an array is bounded before any cell is read (T-0047, CLAUDE.md
//! directive 5). Excel values: probes/lifted-range-size, probes/lifted-range-rows.

use crate::test::util::new_empty_model;

#[test]
fn whole_row_range_lifted_by_date_is_bounded() {
    let mut model = new_empty_model();
    // The formula_eval fuzz reproducer: 1,889 x 16,384 cells (31M).
    // DEVIATION (decision 0014): Excel builds it (ROWS = 1889, lifted-range-rows C1); we refuse
    // arrays above one whole column, so a value context is #ERROR!.
    model.set_a1("C2", "=ROWS(DATE(11:1899,12,1))");

    model.evaluate();

    assert_eq!(model.get_text_a1("C2"), *"#ERROR!");
}

#[test]
fn lift_above_the_bound_that_cannot_spill_is_spill() {
    let mut model = new_empty_model();
    // In a cell Excel answers #SPILL! for the 1,889 x 16,384 lift (probes/lifted-range-rows C3):
    // from column C it runs past the grid. The shape of the lifted range decides it before
    // anything is built (T-0483), as for an operator broadcast (T-0480).
    model.set_a1("C1", "=DATE(11:1899,12,1)");
    // A unary operator or `%` over a 1,048,576 x 3 range from row 3, a lift inside a lift or an
    // operator, and a lifted range beside a scalar argument (the same shape rule, unprobed).
    model.set_a1("C3", "=-H:J");
    model.set_a1("C4", "=H:J%");
    model.set_a1("C5", "=ABS(-H:J)");
    model.set_a1("C6", "=1+DATE(11:1899,12,1)");
    model.set_a1("C7", "=(DATE(2000,H:J,1))");
    // Value contexts keep the decision-0014 deviation, and so does a shape that fits the grid
    // from its anchor (row 1) and a call that does not lift its range argument (SUM).
    model.set_a1("D1", "=ROWS(-H:J)");
    model.set_a1("K1", "=-H:J");
    model.set_a1("L3", "=SUM(H:J)");

    model.evaluate();

    for cell in ["C1", "C3", "C4", "C5", "C6", "C7"] {
        assert_eq!(model.get_text_a1(cell), *"#SPILL!", "{cell}");
    }
    assert_eq!(model.get_text_a1("D1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("K1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("L3"), *"0");
}

#[test]
fn whole_column_range_lifts() {
    let mut model = new_empty_model();
    model.set_a1("A1", "-2");
    model.set_a1("A2", "3");
    model.set_a1("C4", "=ROWS(ABS(A:A))");
    model.set_a1("C5", "=SUM(ABS(A:A))");
    model.set_a1("C6", "=INDEX(ABS(A:A),2)");
    model.set_a1("C7", "=ABS(A:A)");
    model.set_a1("C8", "=SUM(--ISNUMBER(A:A))");
    model.set_a1("C10", "=ROWS(A:A+0)");

    model.evaluate();

    // lifted-range-size C4-C8, C10 (excel-graph, 2026-10-06).
    assert_eq!(model.get_text_a1("C4"), *"1048576");
    assert_eq!(model.get_text_a1("C5"), *"5");
    assert_eq!(model.get_text_a1("C6"), *"3");
    assert_eq!(model.get_text_a1("C7"), *"#SPILL!");
    assert_eq!(model.get_text_a1("C8"), *"2");
    assert_eq!(model.get_text_a1("C10"), *"1048576");
}

#[test]
fn bare_range_spilled_from_a_cell_is_bounded() {
    let mut model = new_empty_model();
    // Fits the grid from A1, so only the allocation bound stops it (DEVIATION, decision 0014).
    model.set_a1("A1", "=B:C");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
}

#[test]
fn broadcast_above_the_bound_that_cannot_spill_is_spill() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    // corpus-bench/55049 book1!B3 (excel-graph): a scalar times a whole-column 3-column range is
    // 1,048,576 x 3 cells; from row 3 it runs past the grid, so Excel answers #SPILL!. The shape
    // decides it before anything is built (T-0480).
    model.set_a1("C3", "=SUMPRODUCT(A:A=A1)*H:J");
    model.set_a1("C4", "=(1+H:J*2)");
    model.set_a1("C5", "=H:J&\"x\"");
    model.set_a1("C6", "=1/0*H:J");
    model.set_a1("C7", "=A:A*H1:J1+1");
    // Value contexts keep the decision-0014 deviation.
    model.set_a1("D1", "=ROWS(SUMPRODUCT(A:A=A1)*H:J)");
    model.set_a1("D2", "=SUM(H:J*2)");
    // From row 1 the shape fits the grid: only the allocation bound stops it (DEVIATION).
    model.set_a1("K1", "=1*H:J");

    model.evaluate();

    for cell in ["C3", "C4", "C5", "C6", "C7"] {
        assert_eq!(model.get_text_a1(cell), *"#SPILL!", "{cell}");
    }
    assert_eq!(model.get_text_a1("D1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("D2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("K1"), *"#ERROR!");
}

#[test]
fn whole_column_lift_blocked_by_a_neighbour_is_spill() {
    let mut model = new_empty_model();
    model.set_a1("A1", "-2");
    model.set_a1("C1", "=ABS(A:A)");
    model.set_a1("C9", "x");

    model.evaluate();

    // Exactly the bound: built, then refused by the occupied C9 (arrays.md).
    assert_eq!(model.get_text_a1("C1"), *"#SPILL!");
}

#[test]
fn sumif_criteria_range_is_bounded() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("D1", "=SUMIF(A1:A2,B:C,A1:A2)");

    model.evaluate();

    // DEVIATION (decision 0014): a 2-column criteria grid is above the shared bound.
    assert_eq!(model.get_text_a1("D1"), *"#ERROR!");
}

#[test]
fn long_percent_chain_on_a_whole_column_lifts_once() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "30");
    model.set_a1("B1", "5");
    // The formula_eval fuzz reproducer (T-1078): 95 postfix `%` on a whole column timed out
    // (> 30 s) because every operator lifted all 1,048,576 rows again.
    let percents = "%".repeat(95);
    model.set_a1("C1", &format!("=LOOKUP(20,A:A,B:B{percents})=5/10^190"));

    model.evaluate();

    assert_eq!(model.get_text_a1("C1"), *"TRUE");
}

#[test]
fn chained_unary_operators_keep_their_values() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("B1", "5");
    model.set_a1("B2", "-7");
    model.set_a1("B3", "TRUE");
    model.set_a1("C2", "=SUM(B1:B3%%)");
    model.set_a1("C3", "=SUM(-----B1:B3)");
    model.set_a1("C4", "=INDEX(-B1:B3%,2)");
    model.set_a1("C5", "=ROWS(--A:A%)");
    model.set_a1("C6", "=-B2%%");
    model.set_a1("C8", "=--\"x\"%");

    model.evaluate();

    assert_eq!(model.get_text_a1("C2"), *"-0.0001");
    assert_eq!(model.get_text_a1("C3"), *"1");
    assert_eq!(model.get_text_a1("C4"), *"0.07");
    assert_eq!(model.get_text_a1("C5"), *"1048576");
    assert_eq!(model.get_text_a1("C6"), *"0.0007");
    assert_eq!(model.get_text_a1("C8"), *"#VALUE!");
}
