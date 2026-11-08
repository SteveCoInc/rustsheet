#![allow(clippy::unwrap_used)]

//! IRR, NPV and XIRR over a sheet-scale range that is almost all blank (T-4976, T-4975 audit 4):
//! the flows are read without walking or allocating per blank cell, so the call ends at once
//! instead of aborting on allocation (CLAUDE.md directive 5).

use crate::test::util::new_empty_model;

#[test]
fn irr_and_npv_over_a_huge_blank_range_read_only_the_stored_cells() {
    let mut model = new_empty_model();
    model.set_a1("B1", "-100");
    model.set_a1("C1", "110");
    // B1:XFC1000000 is 1.6E+10 cells, not a whole column or row, so no used-area trim applies.
    model.set_a1("A2", "=IRR(B1:XFC1000000)");
    model.set_a1("A3", "=NPV(0.1,B1:XFC1000000)");
    model.set_a1("A4", "=IRR(B5:XFC1000000)");
    model.set_a1("A5", "=NPV(0.1,B1:C1)-NPV(0.1,B1:XFC1000000)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A2"), "10%");
    assert!(
        !model.get_text_a1("A3").starts_with('#'),
        "{}",
        model.get_text_a1("A3")
    );
    assert_eq!(model.get_text_a1("A5"), "$0.00");
    // No flows at all: no sign change.
    assert_eq!(model.get_text_a1("A4"), "#NUM!");
}

#[test]
fn xirr_over_a_huge_blank_range_is_refused_before_reading_it() {
    let mut model = new_empty_model();
    model.set_a1("B1", "-100");
    model.set_a1("C1", "110");
    model.set_a1("B2", "45000");
    model.set_a1("C2", "45365");
    // XIRR reads a blank value as a 0 flow, so the whole area would be read: the call is charged
    // the area before the walk and refused (a DEVIATION: Excel's result here is unprobed).
    model.set_a1("A3", "=XIRR(B1:XFC1000000,B2:XFC1000001)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A3"), "#ERROR!");
}
