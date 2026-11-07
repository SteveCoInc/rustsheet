#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// ── INDEX with inline arrays ──────────────────────────────────────────────────

#[test]
fn test_index_array_row_vector() {
    let mut model = new_empty_model();
    // A 1×4 row vector: the single index is a column index -> scalar.
    model.set_a1("A1", "=INDEX({1,2,3,4},2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "2");
}

#[test]
fn test_index_array_column_vector() {
    let mut model = new_empty_model();
    // A 4×1 column vector: the single index is a row index -> scalar.
    model.set_a1("A1", "=INDEX({1;2;3;4},3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "3");
}

#[test]
fn test_index_array_whole_row() {
    let mut model = new_empty_model();
    // A 2×2 array: a single index selects an entire row -> array (spills).
    model.set_a1("A1", "=INDEX({1,2;3,4},1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "1");
    assert_eq!(model.get_text_a1("B1"), "2");
}

#[test]
fn test_index_array_whole_row_second() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=INDEX({1,2;3,4},2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "3");
    assert_eq!(model.get_text_a1("B1"), "4");
}

#[test]
fn test_index_array_row_and_col() {
    let mut model = new_empty_model();
    // Both row_num and col_num given -> single cell.
    model.set_a1("A1", "=INDEX({1,2;3,4},1,1)");
    model.set_a1("A2", "=INDEX({1,2;3,4},2,1)");
    model.set_a1("A3", "=INDEX({1,2;3,4},2,2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "1");
    assert_eq!(model.get_text_a1("A2"), "3");
    assert_eq!(model.get_text_a1("A3"), "4");
}

#[test]
fn test_index_array_out_of_range() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=INDEX({1,2,3,4},9)");
    model.set_a1("A2", "=INDEX({1,2;3,4},3,1)");
    model.set_a1("A3", "=INDEX({1,2;3,4},1,5)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#REF!");
    assert_eq!(model.get_text_a1("A2"), "#REF!");
    assert_eq!(model.get_text_a1("A3"), "#REF!");
}

#[test]
fn test_index_array_strings_and_booleans() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=INDEX({\"a\",\"b\";TRUE,FALSE},2,1)");
    model.set_a1("A2", "=INDEX({\"a\",\"b\";TRUE,FALSE},1,2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "TRUE");
    assert_eq!(model.get_text_a1("A2"), "b");
}

// ── Zero / missing indices select whole rows, columns or the entire array ─────

#[test]
fn test_index_missing_both_returns_whole_array() {
    let mut model = new_empty_model();
    // Missing row and column -> the whole array spills.
    model.set_a1("A1", "=INDEX({1,2,3,4},,)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "1");
    assert_eq!(model.get_text_a1("B1"), "2");
    assert_eq!(model.get_text_a1("C1"), "3");
    assert_eq!(model.get_text_a1("D1"), "4");
}

#[test]
fn test_index_missing_row_returns_whole_column() {
    let mut model = new_empty_model();
    // Missing row, column 1 -> the whole first column spills down.
    model.set_a1("A1", "=INDEX({1,2;3,4},,1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "1");
    assert_eq!(model.get_text_a1("A2"), "3");
}

#[test]
fn test_index_zero_row_returns_whole_column() {
    let mut model = new_empty_model();
    // A row index of 0 behaves like a missing row.
    model.set_a1("A1", "=INDEX({1,2,3;4,5,6},0,1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "1");
    assert_eq!(model.get_text_a1("A2"), "4");
}

#[test]
fn test_index_zero_column_returns_whole_row() {
    let mut model = new_empty_model();
    // A column index of 0 (with the area argument) selects the whole row.
    model.set_a1("A1", "=INDEX({1,2,3},1,0,1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "1");
    assert_eq!(model.get_text_a1("B1"), "2");
    assert_eq!(model.get_text_a1("C1"), "3");
}

#[test]
fn test_index_negative_row_is_value_error() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=INDEX({1,2,3;4,5,6},-1,1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#VALUE!");
}

#[test]
fn test_index_area_out_of_range_is_ref_error() {
    let mut model = new_empty_model();
    // Only one area exists, so area_num = 2 is #REF!.
    model.set_a1("A1", "=INDEX({1,2,3},1,0,2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#REF!");
}

#[test]
fn test_index_range_missing_row_spills_column() {
    let mut model = new_empty_model();
    model.set_a1("A1", "10");
    model.set_a1("A2", "20");
    model.set_a1("B1", "30");
    model.set_a1("B2", "40");
    // Missing row, column 2 of A1:B2 -> the whole second column spills.
    model.set_a1("D1", "=INDEX(A1:B2,,2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("D1"), "30");
    assert_eq!(model.get_text_a1("D2"), "40");
}

// One index into a 2-D range is #REF!, also for 0 and fractions; vectors keep
// selecting along the vector (T-0213; docs/excel-semantics/lookup.md).
#[test]
fn test_index_single_index_into_2d_range_is_ref_error() {
    let mut model = new_empty_model();
    for (i, cell) in ["A1", "B1", "A2", "B2"].iter().enumerate() {
        model.set_a1(cell, &(i + 1).to_string());
    }
    model.set_a1("D1", "=INDEX(A1:B2,1)");
    model.set_a1("D2", "=INDEX(A1:B2,0)");
    model.set_a1("D3", "=INDEX(A1:B2,1E-15)");
    model.set_a1("D4", "=INDEX(A1:B2,1/0)");
    model.set_a1("D5", "=INDEX(A1:B2,1,2)");
    model.set_a1("D6", "=INDEX(A1:A2,2)");
    model.set_a1("D7", "=INDEX(A1:B1,2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("D1"), *"#REF!");
    assert_eq!(model.get_text_a1("D2"), *"#REF!");
    assert_eq!(model.get_text_a1("D3"), *"#REF!");
    assert_eq!(model.get_text_a1("D4"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("D5"), *"2");
    assert_eq!(model.get_text_a1("D6"), *"3");
    assert_eq!(model.get_text_a1("D7"), *"2");
}

// ── INDEX lifted over array indices (T-1260, probes/index-array-indices) ──────

fn index_column_model() -> crate::Model {
    let mut model = new_empty_model();
    for (i, v) in [10, 20, 30, 40, 50].iter().enumerate() {
        model.set_a1(&format!("A{}", i + 1), &v.to_string());
        model.set_a1(&format!("B{}", i + 1), &(i + 1).to_string());
    }
    model
}

#[test]
fn index_lifts_over_an_array_or_range_index() {
    let mut model = index_column_model();
    model.set_a1("D1", "=INDEX(A1:A5,{1,3})");
    model.set_a1("D3", "=SUM(INDEX(A1:A5,{1,3}))");
    model.set_a1("D4", "=ROWS(INDEX(A1:A5,{1,3}))");
    model.set_a1("D5", "=INDEX(A1:A5,SEQUENCE(2))");
    model.set_a1("J1", "1");
    model.set_a1("J2", "3");
    model.set_a1("F1", "=INDEX(A1:A5,J1:J2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("D1"), "10");
    assert_eq!(model.get_text_a1("E1"), "30");
    assert_eq!(model.get_text_a1("D3"), "40");
    assert_eq!(model.get_text_a1("D4"), "1");
    assert_eq!(model.get_text_a1("D5"), "10");
    assert_eq!(model.get_text_a1("D6"), "20");
    assert_eq!(model.get_text_a1("F1"), "10");
    assert_eq!(model.get_text_a1("F2"), "30");
}

#[test]
fn index_indices_broadcast_and_whole_selections_keep_their_top_left() {
    let mut model = index_column_model();
    // A column of rows against a row of columns gives every pair.
    model.set_a1("D1", "=INDEX(A1:B5,{1;2;3},{1,2})");
    // Row 0 selects the whole column; a lifted element keeps its first cell.
    model.set_a1("D5", "=INDEX(A1:A5,{0,2})");
    model.set_a1("D6", "=SUM(INDEX(A1:B5,0,{1,2}))");
    // One index into an array literal picks the row, whose first value is the element.
    model.set_a1("D7", "=INDEX({1,2;3,4},{1,2})");
    model.evaluate();
    assert_eq!(model.get_text_a1("D1"), "10");
    assert_eq!(model.get_text_a1("E1"), "1");
    assert_eq!(model.get_text_a1("D3"), "30");
    assert_eq!(model.get_text_a1("E3"), "3");
    assert_eq!(model.get_text_a1("D5"), "10");
    assert_eq!(model.get_text_a1("E5"), "20");
    assert_eq!(model.get_text_a1("D6"), "11");
    assert_eq!(model.get_text_a1("D7"), "1");
    assert_eq!(model.get_text_a1("E7"), "3");
}

#[test]
fn lifted_index_elements_fail_on_their_own() {
    let mut model = index_column_model();
    model.set_a1("D1", "=INDEX(A1:A5,{1,#N/A})");
    model.set_a1("D2", "=INDEX(A1:A5,{1,9})");
    model.set_a1("D3", "=INDEX(A1:A5,{\"2\",\"x\"})");
    // One index into a 2-D range is #REF!, element by element.
    model.set_a1("D4", "=INDEX(A1:B5,{1,2})");
    model.set_a1("D5", "=INDEX(A1:A5,{1.9,2.5})");
    // An area_num array lifts too.
    model.set_a1("D6", "=INDEX((A1:A5,B1:B5),{1,2},1,{1,2})");
    model.evaluate();
    assert_eq!(model.get_text_a1("D1"), "10");
    assert_eq!(model.get_text_a1("E1"), "#N/A");
    assert_eq!(model.get_text_a1("E2"), "#REF!");
    assert_eq!(model.get_text_a1("D3"), "20");
    assert_eq!(model.get_text_a1("E3"), "#VALUE!");
    assert_eq!(model.get_text_a1("D4"), "#REF!");
    assert_eq!(model.get_text_a1("E4"), "#REF!");
    assert_eq!(model.get_text_a1("D5"), "10");
    assert_eq!(model.get_text_a1("E5"), "20");
    assert_eq!(model.get_text_a1("D6"), "10");
    assert_eq!(model.get_text_a1("E6"), "2");
}

#[test]
fn lifted_index_over_a_large_array_reads_one_value_per_element() {
    let mut model = new_empty_model();
    // Every element selects the whole 100000-row array and keeps its first value. Copying the
    // selection per element would be 1e10 node copies; reading the top-left one is linear.
    model.set_a1("A1", "=SUM(INDEX(SEQUENCE(100000),SEQUENCE(100000)*0))");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "100000");
}

#[test]
fn lifted_index_broadcast_is_bounded() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    // 1048576 rows against 16384 columns would be 1.7e10 elements: past the shared array bound.
    model.set_a1(
        "B1",
        "=INDEX(A1:A5,SEQUENCE(1048576)*0+1,SEQUENCE(1,16384)*0+1)",
    );
    model.evaluate();
    assert_eq!(model.get_text_a1("B1"), "#ERROR!");
}

// probes/index-tiny-negative and probes/index-snap-tolerance (T-1407): a negative index within
// 2^-23 + 2^-34 of 0 is 0 (the whole row), and an index within 2^-22 + 2^-33 below an integer
// is that integer; gen-m9/1466c66c61189a83 reads `INDEX(Data!A1:C3,3,-1e-15)` as row A3:C3.
#[test]
fn index_snaps_near_integer_indexes() {
    let mut model = new_empty_model();
    for (cell, value) in [
        ("C1", "1"),
        ("D1", "2"),
        ("E1", "3"),
        ("C3", "0.5"),
        ("D3", "8"),
        ("E3", "9"),
    ] {
        model.set_a1(cell, value);
    }
    model.set_a1("A1", "=SUM(INDEX(C1:E3,3,-1e-15))");
    model.set_a1("A2", "=COLUMNS(INDEX(C1:E3,3,-1.19E-7))");
    model.set_a1("A3", "=COLUMNS(INDEX(C1:E3,3,-1.193E-7))");
    model.set_a1("A4", "=COLUMNS(INDEX(C1:E3,3,1-2.3853E-7))");
    model.set_a1("A5", "=COLUMNS(INDEX(C1:E3,3,1-2.3854E-7))");
    model.set_a1("A6", "=ROWS(INDEX(C1:E3,-1E-7,1))");
    model.set_a1("A7", "=INDEX(C1:E3,3,-0.5)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "17.5");
    assert_eq!(model.get_text_a1("A2"), "3");
    assert_eq!(model.get_text_a1("A3"), "#VALUE!");
    assert_eq!(model.get_text_a1("A4"), "1");
    assert_eq!(model.get_text_a1("A5"), "3");
    assert_eq!(model.get_text_a1("A6"), "3");
    assert_eq!(model.get_text_a1("A7"), "#VALUE!");
}

// An error column index beats a row index below 1 (gen-m5 372a676d47aa1427 Calc!A371, T-2375).
#[test]
fn test_index_error_column_beats_negative_row() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=INDEX(C1:E3,-0.5,NA())");
    model.set_a1("A2", "=INDEX({1,2;3,4},-1,NA())");
    model.set_a1("A3", "=INDEX(C1:E3,-0.5,1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#N/A");
    assert_eq!(model.get_text_a1("A2"), "#N/A");
    assert_eq!(model.get_text_a1("A3"), "#VALUE!");
}
