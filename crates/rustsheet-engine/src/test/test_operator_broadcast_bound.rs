#![allow(clippy::unwrap_used, reason = "tests")]

//! A binary operator over two arrays checks the size of its broadcast result before building it
//! (T-1420, CLAUDE.md directive 5). DEVIATION (decision 0014): Excel tries to build the array; we
//! refuse one above a whole column (docs/excel-semantics/arrays.md "Array results above 1,048,576
//! cells").

use crate::test::util::new_empty_model;

#[test]
fn whole_column_against_whole_row_is_bounded() {
    let mut model = new_empty_model();
    model.add_sheet("Data").unwrap();
    // The formula_eval fuzz reproducer (nightly run 38109900859), verbatim: the space before `>`
    // and the row range `$4:44` are the fuzzer's. 2^20 x 2^14 cells.
    model.set_a1("C3", "=Data!$D:$D >Data!$4:44");
    // The smaller forms, out of column D and row 4 (where they would be circular).
    model.set_a1("G1", "=D:D+4:4");
    model.set_a1("G2", "=D:D>4:4");
    model.set_a1("G3", "=D1:D100000>4:4");
    model.set_a1("G5", "=SUM(D:D>4:4)");
    model.set_a1("G6", "=D:D&4:4");
    model.set_a1("G7", "=4:4^D:D");

    model.evaluate();

    // In a cell each broadcast runs past the grid, so its shape alone makes it #SPILL!, Excel's
    // answer, before anything is built (T-0480; arrays.md). In a value context the bound stays.
    for cell in ["C3", "G1", "G2", "G3", "G6", "G7"] {
        assert_eq!(model.get_text_a1(cell), *"#SPILL!", "{cell}");
    }
    assert_eq!(model.get_text_a1("G5"), *"#ERROR!");
}

#[test]
fn broadcast_under_the_limit_is_unchanged() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("B1", "10");
    model.set_a1("C1", "20");
    // A column of 2 against a row of 2 (and a column of 3 padded with #N/A).
    model.set_a1("E1", "=A1:A2+B1:C1");
    model.set_a1("E4", "=A1:A2&B1:C1");
    model.set_a1("E6", "=A1:A2<B1:C1");
    model.set_a1("E8", "=SUM(A1:A1048576*B1:C1)");
    model.set_a1("H1", "=A1:A3+{1,2}");

    model.evaluate();

    assert_eq!(model.get_text_a1("E1"), *"11");
    assert_eq!(model.get_text_a1("F1"), *"21");
    assert_eq!(model.get_text_a1("E2"), *"12");
    assert_eq!(model.get_text_a1("F2"), *"22");
    assert_eq!(model.get_text_a1("E4"), *"110");
    assert_eq!(model.get_text_a1("F5"), *"220");
    assert_eq!(model.get_text_a1("E6"), *"TRUE");
    // A whole column against two columns is 2^21 cells: above the limit.
    assert_eq!(model.get_text_a1("E8"), *"#ERROR!");
    assert_eq!(model.get_text_a1("H1"), *"2");
    assert_eq!(model.get_text_a1("I3"), *"2");
}

#[test]
fn if_iferror_ifna_whole_column_against_whole_row_is_bounded() {
    let mut model = new_empty_model();
    // Out of column D and row 4 (where they would be circular). T-1448.
    model.set_a1("G1", "=IF(D:D,4:4)");
    model.set_a1("G2", "=IF(D:D,1,4:4)");
    model.set_a1("G6", "=IFERROR(D:D,4:4)");
    model.set_a1("G5", "=IFNA(D:D,4:4)");

    model.evaluate();

    for cell in ["G1", "G2", "G6", "G5"] {
        assert_eq!(model.get_text_a1(cell), *"#ERROR!", "{cell}");
    }
}
