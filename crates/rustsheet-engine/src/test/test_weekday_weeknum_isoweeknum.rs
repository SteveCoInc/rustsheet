#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn arguments() {
    let mut model = new_empty_model();
    // Day 9 Dec 2025 = serial number 46000, Tuesday, week 50
    model.set_a1("A1", "=WEEKDAY()");
    model.set_a1("A2", "=WEEKDAY(46000)");
    model.set_a1("A3", "=WEEKDAY(46000, 11)");
    model.set_a1("A4", "=WEEKDAY(46000, 11, 2)");

    model.set_a1("B1", "=WEEKNUM()");
    model.set_a1("B2", "=WEEKNUM(46000)");
    model.set_a1("B3", "=WEEKNUM(46000, 11)");
    model.set_a1("B4", "=WEEKNUM(46000, 11, 2)");

    model.set_a1("C1", "=ISOWEEKNUM()");
    model.set_a1("C2", "=ISOWEEKNUM(46000)");
    model.set_a1("C3", "=ISOWEEKNUM(46000, 11)");
    model.set_a1("C4", "=ISOWEEKNUM(46000, 11, 2)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A2"), *"3");
    assert_eq!(model.get_text_a1("A3"), *"2");
    assert_eq!(model.get_text_a1("A4"), *"#ERROR!");

    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"50");
    assert_eq!(model.get_text_a1("B3"), *"50");
    assert_eq!(model.get_text_a1("B4"), *"#ERROR!");

    assert_eq!(model.get_text_a1("C1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("C2"), *"50");
    assert_eq!(model.get_text_a1("C3"), *"#ERROR!");
    assert_eq!(model.get_text_a1("C4"), *"#ERROR!");
}

// WEEKDAY and ISOWEEKNUM broadcast element-wise over a range/array, spilling the
// result. A single-cell/scalar argument keeps returning a scalar.
#[test]
fn weekday_isoweeknum_broadcast_over_range() {
    let mut model = new_empty_model();
    // 2024-01-01 (Mon, serial 45292), 2024-01-02 (Tue), 2024-01-07 (Sun)
    model.set_a1("A1", "45292");
    model.set_a1("A2", "45293");
    model.set_a1("A3", "45298");

    // WEEKDAY, default return_type (Sunday = 1)
    model.set_a1("C1", "=WEEKDAY(A1:A3)");
    // WEEKDAY with return_type 2 (Monday = 1)
    model.set_a1("D1", "=WEEKDAY(A1:A3, 2)");
    // ISOWEEKNUM
    model.set_a1("E1", "=ISOWEEKNUM(A1:A3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("C1"), "2"); // Monday
    assert_eq!(model.get_text_a1("C2"), "3"); // Tuesday
    assert_eq!(model.get_text_a1("C3"), "1"); // Sunday

    assert_eq!(model.get_text_a1("D1"), "1");
    assert_eq!(model.get_text_a1("D2"), "2");
    assert_eq!(model.get_text_a1("D3"), "7");

    assert_eq!(model.get_text_a1("E1"), "1");
    assert_eq!(model.get_text_a1("E2"), "1");
    assert_eq!(model.get_text_a1("E3"), "1");

    // Scalars still return a single value (no spill).
    model.set_a1("G1", "=WEEKDAY(45292)");
    model.set_a1("G2", "=ISOWEEKNUM(45292)");
    model.evaluate();
    assert_eq!(model.get_text_a1("G1"), "2");
    assert_eq!(model.get_text_a1("G2"), "1");
}
