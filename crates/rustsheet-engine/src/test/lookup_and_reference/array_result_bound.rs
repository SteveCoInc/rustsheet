//! EXPAND, WRAPCOLS, WRAPROWS and MMULT check the size of their result before building it, as
//! HSTACK and VSTACK do (T-1786, CLAUDE.md directive 5). DEVIATION (decision 0014): Excel
//! (expected, not measured) builds an in-grid array above a whole column, which we refuse with
//! `#ERROR!`; rows past the grid are `#NUM!` and columns past it are allowed, as for VSTACK and HSTACK
//! (probes/expand-wrap-past-grid; docs/excel-semantics/arrays.md "Stacked results past the grid").

use crate::test::util::new_empty_model;

#[test]
fn expand_rows_past_the_grid_are_num_columns_are_allowed_and_a_huge_wrap_count_clamps() {
    let mut model = new_empty_model();
    for row in 1..=10 {
        model.set_a1(&format!("A{row}"), &row.to_string());
    }
    // 10^18 cells: rows past the grid.
    model.set_a1("C1", "=EXPAND(1,1E9,1E9)");
    model.set_a1("C8", "=COLUMNS(EXPAND(1,1,16385))");
    // A wrap_count above the vector's length is the length, as in Excel (arrays.md "WRAPROWS
    // and WRAPCOLS"), so a billion pads nothing: a 10x1 column and a 1x10 row.
    model.set_a1("C2", "=ROWS(WRAPCOLS(A1:A10,1E9))");
    model.set_a1("C3", "=COLUMNS(WRAPROWS(A1:A10,1E9))");
    // Inside the grid but above 2^20 cells: 2,000 x 1,000.
    model.set_a1("C4", "=ROWS(EXPAND(1,2000,1000))");
    // Valid sizes keep their values.
    model.set_a1("C5", "=ROWS(EXPAND(A1:A10,12,2))");
    model.set_a1("C6", "=COLUMNS(WRAPROWS(A1:A10,4))");
    model.set_a1("C7", "=ROWS(WRAPCOLS(A1:A10,4))");

    model.evaluate();

    assert_eq!(model.get_text_a1("C1"), *"#NUM!");
    assert_eq!(model.get_text_a1("C8"), *"16385");
    assert_eq!(model.get_text_a1("C2"), *"10");
    assert_eq!(model.get_text_a1("C3"), *"10");
    assert_eq!(model.get_text_a1("C4"), *"#ERROR!");
    assert_eq!(model.get_text_a1("C5"), *"12");
    assert_eq!(model.get_text_a1("C6"), *"4");
    assert_eq!(model.get_text_a1("C7"), *"4");
}

#[test]
fn expand_columns_are_num_from_one_past_two_to_the_twentieth() {
    // Excel builds 1,048,576 columns in one row and refuses 1,048,577 with #NUM!; the cap counts
    // columns, so two rows of 524,289 are built, which is above our cell bound (#ERROR!, DEVIATION,
    // decision 0113). probes/t2452-expand-cols-*, -2x524289; arrays.md "Stacked results past the
    // grid" (T-2452).
    let mut model = new_empty_model();
    model.set_a1("A1", "=COLUMNS(EXPAND(1,1,1048576))");
    model.set_a1("A2", "=COLUMNS(EXPAND(1,1,1048577))");
    model.set_a1("A3", "=COLUMNS(EXPAND(1,2,524289))");
    model.set_a1("A4", "=COLUMNS(EXPAND(1,4,1048577))");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"1048576");
    assert_eq!(model.get_text_a1("A2"), *"#NUM!");
    assert_eq!(model.get_text_a1("A3"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A4"), *"#NUM!");
}

#[test]
fn mmult_product_above_the_limit_is_bounded() {
    let mut model = new_empty_model();
    // A filled 2,000-row column and a filled 1,000-column row, so the product is numeric and
    // 2,000 x 1,000 = 2,000,000 cells. The formulas sit outside both ranges.
    for row in 1..=2000 {
        model.set_a1(&format!("A{row}"), "1");
    }
    model.set_a1("B1", "=SEQUENCE(1,1000)");
    model.set_a1("C3001", "=ROWS(MMULT(A1:A2000,B1#))");
    // A blank cell in an input is `#VALUE!` first (arrays.md, MMULT reads no blanks).
    model.set_a1("C3002", "=ROWS(MMULT(A1:A1048576,A1:B1))");
    model.set_a1("C3003", "=SUM(MMULT(A1:A2,TRANSPOSE(A1:A2)))");

    model.evaluate();

    assert_eq!(model.get_text_a1("C3001"), *"#ERROR!");
    assert_eq!(model.get_text_a1("C3002"), *"#VALUE!");
    assert_eq!(model.get_text_a1("C3003"), *"4");
}
