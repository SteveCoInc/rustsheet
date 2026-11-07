#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// ── SORT ──────────────────────────────────────────────────────────────────────

#[test]
fn sort_single_column_ascending() {
    let mut model = new_empty_model();
    model.set_a1("A1", "3");
    model.set_a1("A2", "1");
    model.set_a1("A3", "2");
    model.set_a1("B1", "=SORT(A1:A3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("B1"), "1");
    assert_eq!(model.get_text_a1("B2"), "2");
    assert_eq!(model.get_text_a1("B3"), "3");
}

#[test]
fn sort_omitted_index_and_order_use_defaults() {
    // probes/counta-empty-value D22-D24 (Excel): `SORT(arr,,)` is ascending, `SORT(arr,,-1)` descending.
    let mut model = new_empty_model();
    model.set_a1("A1", "=INDEX(SORT({3;1;2},,),1)");
    model.set_a1("A2", "=INDEX(SORT({3;1;2},,-1),1)");
    model.set_a1("A3", "=COUNTA(SORT({3;1;2},,))");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "1");
    assert_eq!(model.get_text_a1("A2"), "3");
    assert_eq!(model.get_text_a1("A3"), "3");
}

// T-1837: an array sort_index is a list of keys, ties broken by the next one; a sort_order list
// must be as long (probes/m7-argument-modes F10:H11).
#[test]
fn sort_index_list_breaks_ties_with_the_next_key() {
    let mut model = new_empty_model();
    // Rows (1,9), (1,2), (0,5): by column 1, then column 2.
    model.set_a1("A1", "=TEXTJOIN(\",\",,SORT({1,9;1,2;0,5},{1;2}))");
    model.set_a1("A2", "=TEXTJOIN(\",\",,SORT({1,9;1,2;0,5},{1,2},{1;-1}))");
    model.set_a1("A3", "=TEXTJOIN(\",\",,SORT({1,9;1,2;0,5},{1;2},-1))");
    // Columns by row 1, then row 2.
    model.set_a1("A4", "=TEXTJOIN(\",\",,SORT({1,1,0;9,2,5},{1;2},,TRUE))");
    model.set_a1("A5", "=SORT({1,9;1,2;0,5},1,{1;-1})");
    model.set_a1("A6", "=SORT({1,9;1,2;0,5},{1;2},{1;-1;1})");
    model.set_a1("A7", "=SORT({1,9;1,2;0,5},{1;3})");
    model.set_a1("A8", "=TEXTJOIN(\",\",,SORT({1,9;1,2;0,5},2,-1))");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "0,5,1,2,1,9");
    assert_eq!(model.get_text_a1("A2"), "0,5,1,9,1,2");
    assert_eq!(model.get_text_a1("A3"), "1,9,1,2,0,5");
    assert_eq!(model.get_text_a1("A4"), "0,1,1,5,2,9");
    assert_eq!(model.get_text_a1("A5"), "#VALUE!");
    assert_eq!(model.get_text_a1("A6"), "#VALUE!");
    assert_eq!(model.get_text_a1("A7"), "#VALUE!");
    assert_eq!(model.get_text_a1("A8"), "1,9,0,5,1,2");
}

#[test]
fn sort_keys_may_repeat_up_to_the_array_width() {
    // probes/sort-index-key-ties A6, A15-A19: a repeated key is allowed, more keys than columns
    // is `#VALUE!` (so a long list costs nothing); a 2-D list and an error element are `#VALUE!`
    // after a later argument's error (A8, A11, A20).
    let mut model = new_empty_model();
    model.set_a1("A1", "=TEXTJOIN(\",\",,SORT({1,9;1,2;0,5},{1;1}))");
    model.set_a1("A2", "=SORT({1,9;1,2;0,5},SEQUENCE(100000,1,1,0))");
    model.set_a1("A3", "=SORT({1,9;1,2;0,5},{1,2;2,1})");
    model.set_a1("A4", "=SORT({1,9;1,2;0,5},IF({TRUE;FALSE},1,NA()))");
    model.set_a1("A5", "=SORT({1,9;1,2;0,5},IF({TRUE;FALSE},1,NA()),1/0)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "0,5,1,9,1,2");
    assert_eq!(model.get_text_a1("A2"), "#VALUE!");
    assert_eq!(model.get_text_a1("A3"), "#VALUE!");
    assert_eq!(model.get_text_a1("A4"), "#VALUE!");
    assert_eq!(model.get_text_a1("A5"), "#DIV/0!");
}

#[test]
fn sort_single_column_descending() {
    let mut model = new_empty_model();
    model.set_a1("A1", "3");
    model.set_a1("A2", "1");
    model.set_a1("A3", "2");
    model.set_a1("B1", "=SORT(A1:A3,1,-1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("B1"), "3");
    assert_eq!(model.get_text_a1("B2"), "2");
    assert_eq!(model.get_text_a1("B3"), "1");
}

#[test]
fn sort_multi_column_by_second_col() {
    let mut model = new_empty_model();
    // Col A: names, Col B: scores
    model.set_a1("A1", "Charlie");
    model.set_a1("B1", "3");
    model.set_a1("A2", "Alice");
    model.set_a1("B2", "1");
    model.set_a1("A3", "Bob");
    model.set_a1("B3", "2");
    model.set_a1("C1", "=SORT(A1:B3,2,1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), "Alice");
    assert_eq!(model.get_text_a1("D1"), "1");
    assert_eq!(model.get_text_a1("C2"), "Bob");
    assert_eq!(model.get_text_a1("D2"), "2");
    assert_eq!(model.get_text_a1("C3"), "Charlie");
    assert_eq!(model.get_text_a1("D3"), "3");
}

#[test]
fn sort_strings_ascending() {
    let mut model = new_empty_model();
    model.set_a1("A1", "banana");
    model.set_a1("A2", "apple");
    model.set_a1("A3", "cherry");
    model.set_a1("B1", "=SORT(A1:A3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("B1"), "apple");
    assert_eq!(model.get_text_a1("B2"), "banana");
    assert_eq!(model.get_text_a1("B3"), "cherry");
}

#[test]
fn sort_by_col_true() {
    let mut model = new_empty_model();
    // A row of numbers to sort horizontally
    model.set_a1("A1", "3");
    model.set_a1("B1", "1");
    model.set_a1("C1", "2");
    model.set_a1("A3", "=SORT(A1:C1,1,1,TRUE)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A3"), "1");
    assert_eq!(model.get_text_a1("B3"), "2");
    assert_eq!(model.get_text_a1("C3"), "3");
}

#[test]
fn sort_wrong_arg_count() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=SORT()");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#ERROR!");
}

#[test]
fn sort_invalid_sort_order() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("B1", "=SORT(A1:A2,1,2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("B1"), "#VALUE!");
}

// ── SORTBY ────────────────────────────────────────────────────────────────────

#[test]
fn sortby_basic() {
    let mut model = new_empty_model();
    model.set_a1("A1", "Charlie");
    model.set_a1("A2", "Alice");
    model.set_a1("A3", "Bob");
    model.set_a1("B1", "3");
    model.set_a1("B2", "1");
    model.set_a1("B3", "2");
    model.set_a1("C1", "=SORTBY(A1:A3,B1:B3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), "Alice");
    assert_eq!(model.get_text_a1("C2"), "Bob");
    assert_eq!(model.get_text_a1("C3"), "Charlie");
}

#[test]
fn sortby_descending() {
    let mut model = new_empty_model();
    model.set_a1("A1", "Charlie");
    model.set_a1("A2", "Alice");
    model.set_a1("A3", "Bob");
    model.set_a1("B1", "3");
    model.set_a1("B2", "1");
    model.set_a1("B3", "2");
    model.set_a1("C1", "=SORTBY(A1:A3,B1:B3,-1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), "Charlie");
    assert_eq!(model.get_text_a1("C2"), "Bob");
    assert_eq!(model.get_text_a1("C3"), "Alice");
}

#[test]
fn sortby_two_keys() {
    let mut model = new_empty_model();
    // Rows: (Dept, Score) — sort by Dept asc, then Score desc
    model.set_a1("A1", "B");
    model.set_a1("B1", "2");
    model.set_a1("A2", "A");
    model.set_a1("B2", "1");
    model.set_a1("A3", "A");
    model.set_a1("B3", "3");
    model.set_a1("C1", "=SORTBY(A1:B3,A1:A3,1,B1:B3,-1)");
    model.evaluate();
    // First: Dept A with score 3
    assert_eq!(model.get_text_a1("C1"), "A");
    assert_eq!(model.get_text_a1("D1"), "3");
    // Second: Dept A with score 1
    assert_eq!(model.get_text_a1("C2"), "A");
    assert_eq!(model.get_text_a1("D2"), "1");
    // Third: Dept B with score 2
    assert_eq!(model.get_text_a1("C3"), "B");
    assert_eq!(model.get_text_a1("D3"), "2");
}

#[test]
fn sortby_wrong_arg_count() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=SORTBY(A2:A3,B2:B3,1,C2:C3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#ERROR!");
}

/// A `by_array` sorts rows when it is a column as tall as the array and columns when it is a row as
/// wide as it, so one value leaves a one-column array as it is and is `#VALUE!` on a wider one
/// (probes/t2310-sortby-single-by-array H1, H13, H69, H77, H89).
#[test]
fn sortby_by_array_direction() {
    let mut model = new_empty_model();
    model.set_a1("A1", "3");
    model.set_a1("A2", "1");
    model.set_a1("A3", "2");
    model.set_a1("B1", "30");
    model.set_a1("B2", "10");
    model.set_a1("B3", "20");
    model.set_a1("D1", "=SORTBY(A1:A3,5)");
    model.set_a1("E1", "=SORTBY(A1:B3,5)");
    model.set_a1("F1", "=SORTBY(A1:B3,{2,1})");
    model.set_a1("H1", "=SORTBY(A1:A3,{3,1,2})");
    model.set_a1("I1", "=SORTBY(A1:A3,5,1,A1:A3,-1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("D1"), "3");
    assert_eq!(model.get_text_a1("D2"), "1");
    assert_eq!(model.get_text_a1("D3"), "2");
    assert_eq!(model.get_text_a1("E1"), "#VALUE!");
    assert_eq!(model.get_text_a1("F1"), "30");
    assert_eq!(model.get_text_a1("G1"), "3");
    assert_eq!(model.get_text_a1("F3"), "20");
    assert_eq!(model.get_text_a1("H1"), "#VALUE!");
    assert_eq!(model.get_text_a1("I1"), "#VALUE!");
}

/// A fractional `sort_order` reads as its floor, snapped up near an integer
/// (probes/t2310-sortby-single-by-array H97-H165).
#[test]
fn sortby_fractional_sort_order() {
    let mut model = new_empty_model();
    model.set_a1("A1", "3");
    model.set_a1("A2", "1");
    model.set_a1("A3", "2");
    model.set_a1("C1", "=SORTBY(A1:A3,A1:A3,-0.0001)");
    model.set_a1("D1", "=SORTBY(A1:A3,A1:A3,1.5)");
    model.set_a1("E1", "=SORTBY(A1:A3,A1:A3,0.5)");
    model.set_a1("F1", "=SORTBY(A1:A3,A1:A3,-1.5)");
    model.set_a1("G1", "=SORTBY(A1:A3,A1:A3,-1E-300)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), "3");
    assert_eq!(model.get_text_a1("C3"), "1");
    assert_eq!(model.get_text_a1("D1"), "1");
    assert_eq!(model.get_text_a1("D3"), "3");
    assert_eq!(model.get_text_a1("E1"), "#VALUE!");
    assert_eq!(model.get_text_a1("F1"), "#VALUE!");
    assert_eq!(model.get_text_a1("G1"), "#VALUE!");
}

// ── UNIQUE ────────────────────────────────────────────────────────────────────

#[test]
fn unique_basic() {
    let mut model = new_empty_model();
    model.set_a1("A1", "apple");
    model.set_a1("A2", "banana");
    model.set_a1("A3", "apple");
    model.set_a1("A4", "cherry");
    model.set_a1("B1", "=UNIQUE(A1:A4)");
    model.evaluate();
    assert_eq!(model.get_text_a1("B1"), "apple");
    assert_eq!(model.get_text_a1("B2"), "banana");
    assert_eq!(model.get_text_a1("B3"), "cherry");
}

#[test]
fn unique_exactly_once() {
    let mut model = new_empty_model();
    model.set_a1("A1", "apple");
    model.set_a1("A2", "banana");
    model.set_a1("A3", "apple");
    model.set_a1("A4", "cherry");
    model.set_a1("B1", "=UNIQUE(A1:A4,FALSE,TRUE)");
    model.evaluate();
    // Only banana and cherry appear once
    assert_eq!(model.get_text_a1("B1"), "banana");
    assert_eq!(model.get_text_a1("B2"), "cherry");
}

#[test]
fn unique_multi_column_rows() {
    let mut model = new_empty_model();
    model.set_a1("A1", "Alice");
    model.set_a1("B1", "1");
    model.set_a1("A2", "Bob");
    model.set_a1("B2", "2");
    model.set_a1("A3", "Alice");
    model.set_a1("B3", "1");
    model.set_a1("C1", "=UNIQUE(A1:B3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), "Alice");
    assert_eq!(model.get_text_a1("D1"), "1");
    assert_eq!(model.get_text_a1("C2"), "Bob");
    assert_eq!(model.get_text_a1("D2"), "2");
}

#[test]
fn unique_numbers() {
    let mut model = new_empty_model();
    model.set_a1("A1", "5");
    model.set_a1("A2", "3");
    model.set_a1("A3", "5");
    model.set_a1("A4", "7");
    model.set_a1("A5", "3");
    model.set_a1("B1", "=UNIQUE(A1:A5)");
    model.evaluate();
    assert_eq!(model.get_text_a1("B1"), "5");
    assert_eq!(model.get_text_a1("B2"), "3");
    assert_eq!(model.get_text_a1("B3"), "7");
}

#[test]
fn unique_by_col() {
    let mut model = new_empty_model();
    // A row where some columns are duplicates
    model.set_a1("A1", "1");
    model.set_a1("B1", "2");
    model.set_a1("C1", "1");
    model.set_a1("D1", "3");
    model.set_a1("A3", "=UNIQUE(A1:D1,TRUE)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A3"), "1");
    assert_eq!(model.get_text_a1("B3"), "2");
    assert_eq!(model.get_text_a1("C3"), "3");
}

// ── FILTER ────────────────────────────────────────────────────────────────────

#[test]
fn filter_basic() {
    let mut model = new_empty_model();
    model.set_a1("A1", "Alice");
    model.set_a1("B1", "TRUE");
    model.set_a1("A2", "Bob");
    model.set_a1("B2", "FALSE");
    model.set_a1("A3", "Charlie");
    model.set_a1("B3", "TRUE");
    model.set_a1("C1", "=FILTER(A1:A3,B1:B3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), "Alice");
    assert_eq!(model.get_text_a1("C2"), "Charlie");
}

#[test]
fn filter_with_numbers() {
    let mut model = new_empty_model();
    model.set_a1("A1", "10");
    model.set_a1("B1", "1");
    model.set_a1("A2", "20");
    model.set_a1("B2", "0");
    model.set_a1("A3", "30");
    model.set_a1("B3", "1");
    model.set_a1("C1", "=FILTER(A1:A3,B1:B3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), "10");
    assert_eq!(model.get_text_a1("C2"), "30");
}

#[test]
fn filter_no_match_returns_calc_error() {
    let mut model = new_empty_model();
    model.set_a1("A1", "x");
    model.set_a1("B1", "FALSE");
    model.set_a1("C1", "=FILTER(A1:A1,B1:B1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), "#CALC!");
}

#[test]
fn filter_no_match_with_if_empty() {
    let mut model = new_empty_model();
    model.set_a1("A1", "x");
    model.set_a1("B1", "FALSE");
    model.set_a1("C1", "=FILTER(A1:A1,B1:B1,\"No results\")");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), "No results");
}

// Excel: probes/t3654-filter-result-types (docs/excel-semantics/arrays.md "FILTER: the type of
// each include element and of the result").
#[test]
fn filter_include_text_lone_blank_and_blank_if_empty() {
    let mut model = new_empty_model();
    model.set_a1("A1", "10");
    model.set_a1("A2", "20");
    model.set_a1("A3", "30");
    model.set_a1("D1", "=ARRAYTOTEXT(FILTER(A1:A3,{\"true\";0;\"FALSE\"}))");
    model.set_a1("D2", "=FILTER(A1:A3,{\" TRUE\";1;0})");
    model.set_a1("D3", "=ISBLANK(FILTER(Z1,TRUE))");
    model.set_a1("D4", "=FILTER(Z1,TRUE)&\"|\"");
    model.set_a1("D5", "=FILTER(A1:A3,{0;0;0},Z1)&\"|\"");
    model.set_a1("D6", "=ROWS(FILTER(A1:A3,{0;0;0},Z1))");
    model.set_a1("D7", "=FILTER(A1:A3,{0;#N/A;0},\"none\")");
    model.evaluate();
    assert_eq!(model.get_text_a1("D1"), "10");
    assert_eq!(model.get_text_a1("D2"), "#VALUE!");
    assert_eq!(model.get_text_a1("D3"), "TRUE");
    assert_eq!(model.get_text_a1("D4"), "|");
    assert_eq!(model.get_text_a1("D5"), "0|");
    assert_eq!(model.get_text_a1("D6"), "1");
    assert_eq!(model.get_text_a1("D7"), "#N/A");
}

// Excel: upstream MAX_MIN.xlsx H39/H40/H43; probes/min-max-reference-forms E1-E3.
#[test]
fn filter_if_empty_is_the_value_itself() {
    // It is not an array holding the value: a text reaches the outer function
    // as a text, which MAX does not accept, and an array spills.
    let mut model = new_empty_model();
    model.set_a1("A1", "x");
    model.set_a1("B1", "FALSE");
    model.set_a1("C1", "=MAX(FILTER(A1:A1,B1:B1,\"\"))");
    model.set_a1("C2", "=FILTER(A1:A1,B1:B1,7)+1");
    model.set_a1("C3", "=FILTER(A1:A1,B1:B1,{1,2,3})");
    model.set_a1("C4", "=FILTER(A1:A1,B1:B1,1/0)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), "#VALUE!");
    assert_eq!(model.get_text_a1("C2"), "8");
    assert_eq!(model.get_text_a1("C3"), "1");
    assert_eq!(model.get_text_a1("D3"), "2");
    assert_eq!(model.get_text_a1("E3"), "3");
    assert_eq!(model.get_text_a1("C4"), "#DIV/0!");
}

// Excel: probes/filter-if-empty-array (docs/excel-semantics/arrays.md).
#[test]
fn filter_no_match_with_array_if_empty_spills_it() {
    let mut model = new_empty_model();
    model.set_a1("A1", "x");
    model.set_a1("B1", "FALSE");
    model.set_a1("C1", "=FILTER(A1:A1,B1:B1,{1,2;3,4})");
    model.set_a1("C5", "=FILTER(A1:A1,B1:B1,{1,2,3})");
    model.set_a1("D5", "blocker");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), "1");
    assert_eq!(model.get_text_a1("D1"), "2");
    assert_eq!(model.get_text_a1("C2"), "3");
    assert_eq!(model.get_text_a1("D2"), "4");
    assert_eq!(model.get_text_a1("C5"), "#SPILL!");
}

#[test]
fn filter_multi_column() {
    let mut model = new_empty_model();
    model.set_a1("A1", "Alice");
    model.set_a1("B1", "100");
    model.set_a1("C1", "TRUE");
    model.set_a1("A2", "Bob");
    model.set_a1("B2", "200");
    model.set_a1("C2", "FALSE");
    model.set_a1("A3", "Charlie");
    model.set_a1("B3", "300");
    model.set_a1("C3", "TRUE");
    model.set_a1("D1", "=FILTER(A1:B3,C1:C3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("D1"), "Alice");
    assert_eq!(model.get_text_a1("E1"), "100");
    assert_eq!(model.get_text_a1("D2"), "Charlie");
    assert_eq!(model.get_text_a1("E2"), "300");
}

#[test]
fn filter_wrong_arg_count() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=FILTER(A2:A3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#ERROR!");
}

#[test]
fn filter_string_include_returns_value_error() {
    // A string in the include argument is not a valid boolean/numeric mask → #VALUE!
    let mut model = new_empty_model();
    model.set_a1("A1", "3");
    model.set_a1("A2", "5");
    model.set_a1("A3", "7");
    model.set_a1("B1", "=FILTER(A1:A3,\"=5\")");
    model.evaluate();
    assert_eq!(model.get_text_a1("B1"), "#VALUE!");
}

// T-3506: a one-cell reference to an error cell is a 1x1 array holding the error, as array or as
// include, so FILTER checks the shape and the include first (gen-m7 arg-pairs, excel-graph;
// docs/excel-semantics/arrays.md "A one-cell error reference is a 1x1 array in TOCOL, TOROW and
// FILTER").
#[test]
fn filter_one_cell_error_reference_is_a_one_by_one_array() {
    let mut model = new_empty_model();
    model.set_a1("G9", "=1/0");
    for row in 1..=3 {
        model.set_a1(&format!("A{row}"), &row.to_string());
    }
    // An include of 0 leaves nothing (gen-m7/68e7f10771a7118a A10798).
    model.set_a1("C1", "=FILTER(G9,0)");
    // A 1x1 array against a 3-row include (gen-m7/ee81c689b5fde629 A1160).
    model.set_a1("C2", "=FILTER(G9,A1:A3>0,-2.5)");
    // A 1x1 include against 3 rows and 2 columns (gen-m7/d7a127bd9334c2b4 A2380).
    model.set_a1("C3", "=FILTER(A1:B3,G9,-2.5)");
    // Kept, the error is the one element; as a fitting include it is the error
    // (probes/t4477-filter-include-error-shapes J19, J71).
    model.set_a1("C4", "=FILTER(G9,TRUE)");
    model.set_a1("C5", "=FILTER(A1,G9)");
    // A literal error is still the error (gen-m7/ee81c689b5fde629 A977).
    model.set_a1("C6", "=FILTER(#DIV/0!,A1:A3>0)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), "#CALC!");
    assert_eq!(model.get_text_a1("C2"), "#VALUE!");
    assert_eq!(model.get_text_a1("C3"), "#VALUE!");
    assert_eq!(model.get_text_a1("C4"), "#DIV/0!");
    assert_eq!(model.get_text_a1("C5"), "#DIV/0!");
    assert_eq!(model.get_text_a1("C6"), "#DIV/0!");
}

// T-4477: an include's shape is checked before its elements, so a wrongly shaped include holding
// an error is `#VALUE!` (probes/t4477-filter-include-error-shapes J27:L39); a right-shaped one is
// its error (J43:L47).
#[test]
fn filter_wrongly_shaped_range_include_with_an_error_is_value() {
    let mut model = new_empty_model();
    for row in 1..=3 {
        model.set_a1(&format!("A{row}"), &row.to_string());
        model.set_a1(&format!("B{row}"), &row.to_string());
    }
    model.set_a1("C1", "=1/0");
    model.set_a1("C2", "TRUE");
    model.set_a1("C3", "TRUE");
    model.set_a1("D1", "=FILTER(A1:A3,C1:C2)");
    model.set_a1("D2", "=FILTER(A1:A3,(C1:C2))");
    model.set_a1("D3", "=FILTER(A1:A3,{1;#N/A})");
    model.set_a1("D4", "=FILTER(A1:A3,{1;#N/A;1})");
    model.set_a1("D5", "=FILTER(A1:B3,C1:C3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("D1"), "#VALUE!");
    assert_eq!(model.get_text_a1("D2"), "#VALUE!");
    assert_eq!(model.get_text_a1("D3"), "#VALUE!");
    assert_eq!(model.get_text_a1("D4"), "#N/A");
    assert_eq!(model.get_text_a1("D5"), "#DIV/0!");
}

// T-4477: a one-cell error reached through LET, INDEX or IF is a 1x1 array, as a written one is
// (probes/t4477-filter-include-error-shapes J51:L59, J67:L71).
#[test]
fn filter_indirect_one_cell_error_reference_is_a_one_by_one_array() {
    let mut model = new_empty_model();
    model.set_a1("G9", "=1/0");
    model.set_a1("G2", "0");
    for row in 1..=3 {
        model.set_a1(&format!("A{row}"), &row.to_string());
        model.set_a1(&format!("B{row}"), &row.to_string());
    }
    model.set_a1("D1", "=FILTER(A1:B3,LET(x,G9,x))");
    model.set_a1("D2", "=FILTER(A1:B3,INDEX(G1:G9,9))");
    model.set_a1("D3", "=FILTER(A1:B3,IF(TRUE,G9))");
    model.set_a1("D4", "=FILTER(LET(x,G9,x),G2)");
    model.set_a1("D5", "=ROWS(FILTER(INDEX(G1:G9,9),TRUE))");
    model.set_a1("D6", "=FILTER(INDEX(G1:G9,9),TRUE)");
    model.evaluate();
    assert_eq!(model.get_text_a1("D1"), "#VALUE!");
    assert_eq!(model.get_text_a1("D2"), "#VALUE!");
    assert_eq!(model.get_text_a1("D3"), "#VALUE!");
    assert_eq!(model.get_text_a1("D4"), "#CALC!");
    assert_eq!(model.get_text_a1("D5"), "1");
    assert_eq!(model.get_text_a1("D6"), "#DIV/0!");
}

#[test]
fn filter_string_include_in_range_returns_value_error() {
    // A range whose cells contain strings is also invalid as an include mask.
    let mut model = new_empty_model();
    model.set_a1("A1", "3");
    model.set_a1("A2", "5");
    model.set_a1("A3", "7");
    model.set_a1("B1", "yes");
    model.set_a1("B2", "no");
    model.set_a1("B3", "yes");
    model.set_a1("C1", "=FILTER(A1:A3,B1:B3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), "#VALUE!");
}

#[test]
fn filter_scalar_false_include_returns_calc_not_crash() {
    // =FILTER(B1:B3, 1=0): include evaluates to a scalar FALSE whose 1x1 size
    // coincides with the array's single column.  Previously this triggered
    // column-filter mode, produced a Nx0 array, and caused a panic in the
    // spill/evaluate path.  It should return #CALC! instead.
    let mut model = new_empty_model();
    model.set_a1("B1", "apple");
    model.set_a1("B2", "banana");
    model.set_a1("B3", "cherry");
    model.set_a1("A1", "=FILTER(B1:B3,1=0)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#CALC!");
}

#[test]
fn filter_scalar_false_include_with_if_empty() {
    // Same scenario as above, but with an if_empty fallback — must return the
    // fallback value rather than crashing.
    let mut model = new_empty_model();
    model.set_a1("B1", "apple");
    model.set_a1("B2", "banana");
    model.set_a1("B3", "cherry");
    model.set_a1("A1", "=FILTER(B1:B3,1=0,\"none\")");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "none");
}

#[test]
fn filter_all_columns_excluded_returns_calc() {
    // FILTER on a multi-column array where the column-include mask is all-false.
    // The result is a Nx0 array which must be treated as an empty result (#CALC!).
    let mut model = new_empty_model();
    model.set_a1("A1", "10");
    model.set_a1("B1", "20");
    model.set_a1("A2", "30");
    model.set_a1("B2", "40");
    // Include mask is a row vector [FALSE, FALSE] — filters columns
    model.set_a1("C1", "=FILTER(A1:B2,{FALSE,FALSE})");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), "#CALC!");
}

// ── Combination tests ─────────────────────────────────────────────────────────

#[test]
fn sort_after_filter() {
    let mut model = new_empty_model();
    model.set_a1("A1", "Charlie");
    model.set_a1("B1", "TRUE");
    model.set_a1("A2", "Alice");
    model.set_a1("B2", "TRUE");
    model.set_a1("A3", "Bob");
    model.set_a1("B3", "FALSE");
    // FILTER first, then SORT the result using a helper: place FILTER in a helper col
    // (direct nesting not yet tested — use separate cells)
    model.set_a1("C1", "=FILTER(A1:A3,B1:B3)");
    model.evaluate();
    // C1=Charlie, C2=Alice (filtered)
    model.set_a1("D1", "=SORT(C1:C2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("D1"), "Alice");
    assert_eq!(model.get_text_a1("D2"), "Charlie");
}

#[test]
fn anchorarray_spells_the_spill_operator() {
    // probes/at-operand-formulatext J1, K1, K30 (T-1109): `ANCHORARRAY(x)` is `x#`, and a
    // reference-returning call is a valid anchor.
    let mut model = new_empty_model();
    model.set_a1("D1", "=SEQUENCE(3)");
    model.set_a1("J30", "=ANCHORARRAY(D1)");
    model.set_a1("J1", "=FORMULATEXT(J30)");
    model.set_a1("K30", "=ANCHORARRAY(INDEX(D1:D2,1))");
    model.set_a1("K1", "=FORMULATEXT(K30)");
    model.evaluate();
    assert_eq!(model.get_text_a1("J1"), "=D1#");
    assert_eq!(model.get_text_a1("K1"), "=INDEX(D1:D2,1)#");
    assert_eq!(model.get_text_a1("J32"), "3");
    assert_eq!(model.get_text_a1("K32"), "3");
}

#[test]
fn anchorarray_of_an_at_with_no_cell_is_value() {
    // probes/anchorarray-at-operand D20 (T-1115): D20 is outside rows 1-2, so `@` finds no cell.
    let mut model = new_empty_model();
    model.set_a1("D20", "=ANCHORARRAY((@A1:B2))");
    model.evaluate();
    assert_eq!(model.get_text_a1("D20"), "#VALUE!");
}

#[test]
fn range_operator_passes_a_right_operand_error_on() {
    // probes/range-right-operand-errors D11, D14 (T-1115).
    let mut model = new_empty_model();
    model.set_a1("D11", "=A1:INDEX(A1:A3,5)");
    model.set_a1("D14", "=A1:(A1 B2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("D11"), "#REF!");
    assert_eq!(model.get_text_a1("D14"), "#NULL!");
}

#[test]
fn anchorarray_of_at_negation_is_value_and_of_at_index_spills() {
    // probes/anchorarray-at-operand-kinds H101, H81 (T-1240, T-1247): `@-1` is a value, so
    // `#VALUE!`, not a parse failure; `@` of a reference-returning call keeps the reference.
    let mut model = new_empty_model();
    model.set_a1("D1", "=SEQUENCE(3)");
    model.set_a1("H81", "=ANCHORARRAY(@INDEX(D1:D2,1))");
    model.set_a1("H101", "=ANCHORARRAY(@-1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("H101"), "#VALUE!");
    assert_eq!(model.get_text_a1("H81"), "1");
    assert_eq!(model.get_text_a1("H82"), "2");
    assert_eq!(model.get_text_a1("H83"), "3");
}

// The `@` of an IF that picks one cell keeps the reference, so ANCHORARRAY spills it
// (probes/at-signed-operand H71:H73, T-1321).
#[test]
fn anchorarray_of_at_if_spills_the_picked_cell() {
    let mut model = new_empty_model();
    model.set_a1("D1", "=SEQUENCE(3)");
    model.set_a1("H1", "=ANCHORARRAY(@IF(TRUE,D1))");
    model.evaluate();

    assert_eq!(model.get_text_a1("H1"), "1");
    assert_eq!(model.get_text_a1("H2"), "2");
    assert_eq!(model.get_text_a1("H3"), "3");
}

// An IF that picks a value is not a spill anchor: `@IF(FALSE,D1,5)` is `#VALUE!` like
// `ANCHORARRAY(@1)` (probes/colon-hash-value-operands D3, T-1226; T-1321).
#[test]
fn anchorarray_of_at_if_picking_a_value_is_value_error() {
    let mut model = new_empty_model();
    model.set_a1("D1", "=SEQUENCE(3)");
    model.set_a1("H1", "=ANCHORARRAY(@IF(FALSE,D1,5))");
    model.evaluate();

    assert_eq!(model.get_text_a1("H1"), "#VALUE!");
}

// The `@` operand of a spill anchor is evaluated once, so nesting ANCHORARRAY(@IF(...)) and
// ANCHORARRAY(@INDEX(...,0)) whose inner pick is a multi-cell range stays linear instead of
// doubling per level (T-1321).
#[test]
fn nested_anchorarray_of_at_if_and_at_index_is_linear() {
    let mut model = new_empty_model();
    model.set_a1("D1", "=SEQUENCE(3)");
    let mut if_formula = "D1".to_string();
    let mut index_formula = "D1".to_string();
    for _ in 0..40 {
        if_formula = format!("ANCHORARRAY(@IF(TRUE,{if_formula}))");
        index_formula = format!("ANCHORARRAY(@INDEX({index_formula},0))");
    }
    model.set_a1("H1", &format!("={if_formula}"));
    model.set_a1("J1", &format!("={index_formula}"));
    model.evaluate();

    assert_eq!(model.get_text_a1("H1"), "1");
    assert_eq!(model.get_text_a1("H3"), "3");
    assert_eq!(model.get_text_a1("J1"), "1");
    assert_eq!(model.get_text_a1("J3"), "3");
}

#[test]
fn sort_error_argument_wins_over_invalid_sort_index_and_order() {
    // gen-m7/c5d278a8662ef4b2 Calc!A39 (Excel): by_col `#N/A` beats sort_index 0.
    let mut model = new_empty_model();
    model.set_a1("A1", "=SORT({1;2},0,1,NA())");
    model.set_a1("A2", "=SORT({1;2},0,1)");
    model.set_a1("A3", "=SORT({1;2},1,0)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#N/A");
    assert_eq!(model.get_text_a1("A2"), "#VALUE!");
    assert_eq!(model.get_text_a1("A3"), "#VALUE!");
}

// T-3667: CHOOSEROWS, CHOOSECOLS, VSTACK and HSTACK read a one-cell error reference as a 1x1 array
// (probes/one-cell-error-ref-array-functions, excel-graph; arrays.md "A one-cell error reference
// is a 1x1 array ..."). A literal error is still the error.
#[test]
fn choose_and_stack_read_a_one_cell_error_reference_as_a_one_by_one_array() {
    let mut model = new_empty_model();
    model.set_a1("G9", "=1/0");
    model.set_a1("D1", "1");
    model.set_a1("E1", "4");
    model.set_a1("A1", "=CHOOSEROWS(G9,2)");
    model.set_a1("A2", "=CHOOSECOLS(G9,0)");
    model.set_a1("A3", "=CHOOSEROWS(#DIV/0!,2)");
    model.set_a1("A4", "=CHOOSEROWS(G9,1)");
    model.set_a1("A5", "=VSTACK(D1:E1,G9)");
    model.set_a1("A8", "=HSTACK(G9,D1:E1)");
    model.set_a1("A10", "=VSTACK(D1:E1,#DIV/0!)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#VALUE!");
    assert_eq!(model.get_text_a1("A2"), "#VALUE!");
    assert_eq!(model.get_text_a1("A3"), "#DIV/0!");
    assert_eq!(model.get_text_a1("A4"), "#DIV/0!");
    assert_eq!(model.get_text_a1("A5"), "1");
    assert_eq!(model.get_text_a1("B5"), "4");
    assert_eq!(model.get_text_a1("A6"), "#DIV/0!");
    assert_eq!(model.get_text_a1("B6"), "#N/A");
    assert_eq!(model.get_text_a1("A8"), "#DIV/0!");
    assert_eq!(model.get_text_a1("B8"), "1");
    assert_eq!(model.get_text_a1("C8"), "4");
    assert_eq!(model.get_text_a1("A10"), "#DIV/0!");
}

// T-4447: SORT floors a fractional sort_order and sort_index, snapped up just below an integer,
// as SORTBY's sort_order (probes/t4447-sort-fractional-keys E2, I2 and E17; B3 is the
// one-column analogue of I10's `3-2^-23` over two columns).
#[test]
fn sort_floors_fractional_keys_with_the_near_integer_snap() {
    let mut model = new_empty_model();
    model.set_a1("A1", "3");
    model.set_a1("A2", "1");
    model.set_a1("A3", "2");
    model.set_a1("B1", "=TEXTJOIN(\",\",,SORT(A1:A3,1,-0.5))");
    model.set_a1("B2", "=TEXTJOIN(\",\",,SORT(A1:A3,1-2^-23,-1))");
    model.set_a1("B3", "=SORT(A1:A3,2-2^-23)");
    model.set_a1("B4", "=SORT(A1:A3,1,2-2^-22)");
    model.evaluate();
    assert_eq!(model.get_text_a1("B1"), "3,2,1");
    assert_eq!(model.get_text_a1("B2"), "3,2,1");
    assert_eq!(model.get_text_a1("B3"), "#VALUE!");
    assert_eq!(model.get_text_a1("B4"), "#VALUE!");
}

// T-4479: SORT and SORTBY read a text sort_order apart, and Excel does too
// (probes/t4479-sort-bad-argument-order; docs/excel-semantics/arrays.md "SORT and SORTBY: which
// of several bad arguments wins"). SORT's text order fails only after by_col is read, while a
// computed error order is itself at once; SORTBY's text order is `#VALUE!` at once, before a
// later pair's error, while its blank or FALSE order fails only in validation.
#[test]
fn sort_and_sortby_read_a_text_sort_order_apart() {
    let mut model = new_empty_model();
    model.set_a1("A1", "3");
    model.set_a1("A2", "1");
    model.set_a1("A3", "2");
    model.set_a1("C1", "c");
    model.set_a1("C2", "a");
    model.set_a1("C3", "b");
    model.set_a1("E1", "=SORT(A1:A3,1,\"a\",NA())");
    model.set_a1("E2", "=SORT(A1:A3,1,\"a\"+1,NA())");
    model.set_a1("E3", "=SORT(A1:A3,\"x\",Z1,SQRT(-1))");
    model.set_a1("E4", "=SORTBY(A1:A3,C1:C3,\"a\",C1:C3,NA())");
    model.set_a1("E5", "=SORTBY(A1:A3,C1:C3,\"a\",NA(),1)");
    model.set_a1("E6", "=SORTBY(A1:A3,C1:C3,Z1,C1:C3,NA())");
    model.set_a1("E7", "=SORTBY(A1:A3,C1:C3,TRUE,C1:C3,NA())");
    model.set_a1("E8", "=SORTBY(A1:A3,C1:C3,0,C1:C2,1/0)");
    model.evaluate();
    assert_eq!(model.get_text_a1("E1"), "#N/A");
    assert_eq!(model.get_text_a1("E2"), "#VALUE!");
    assert_eq!(model.get_text_a1("E3"), "#VALUE!");
    assert_eq!(model.get_text_a1("E4"), "#VALUE!");
    assert_eq!(model.get_text_a1("E5"), "#VALUE!");
    assert_eq!(model.get_text_a1("E6"), "#N/A");
    assert_eq!(model.get_text_a1("E7"), "#N/A");
    assert_eq!(model.get_text_a1("E8"), "#DIV/0!");
}
