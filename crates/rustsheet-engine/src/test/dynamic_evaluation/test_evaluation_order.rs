#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

//         ║    B    |    C    |   D   |
// ════════╬═════════╪═════════╪═══════╪
//    1    ║ =C1:C3  | =D1:D3  |   5   |
// ────────╫─────────┼─────────┼───────┼
//    2    ║         |         |  10   |
// ────────╫─────────┼─────────┼───────┼
//    3    ║         |         |  15   |
// ────────╫─────────┼─────────┼───────┼
//
// C1=D1:D3 spills to C1=5, C2=10, C3=15.
// B1=C1:C3 spills to B1=5, B2=10, B3=15 (reads C's spill area).
// In natural order B1 is evaluated before C1 and reads zeros.
#[test]
fn spill_reads_other_spill_area() {
    let mut model = new_empty_model();

    model.set_a1("B1", "=C1:C3");
    model.set_a1("C1", "=D1:D3");
    model.set_a1("D1", "5");
    model.set_a1("D2", "10");
    model.set_a1("D3", "15");

    model.evaluate();

    assert_eq!(model.get_text_a1("C1"), "5");
    assert_eq!(model.get_text_a1("C2"), "10");
    assert_eq!(model.get_text_a1("C3"), "15");
    assert_eq!(model.get_text_a1("B1"), "5");
    assert_eq!(model.get_text_a1("B2"), "10");
    assert_eq!(model.get_text_a1("B3"), "15");
}

//         ║    C    |    D    |   E   |   F   |
// ════════╬═════════╪═════════╪═══════╪═══════╪
//    1    ║ =E1:E2  | =F1:F2  |   1   |   3   |
// ────────╫─────────┼─────────┼───────┼───────┼
//    2    ║         |         |   2   |   4   |
// ────────╫─────────┼─────────┼───────┼───────┼
//
// Two independent spill formulas with no cross-dependency.
// Both should evaluate correctly regardless of the initial order.
#[test]
fn independent_spills() {
    let mut model = new_empty_model();

    model.set_a1("C1", "=E1:E2");
    model.set_a1("D1", "=F1:F2");
    model.set_a1("E1", "1");
    model.set_a1("E2", "2");
    model.set_a1("F1", "3");
    model.set_a1("F2", "4");

    model.evaluate();

    assert_eq!(model.get_text_a1("C1"), "1");
    assert_eq!(model.get_text_a1("C2"), "2");
    assert_eq!(model.get_text_a1("D1"), "3");
    assert_eq!(model.get_text_a1("D2"), "4");
}

//         ║    C    |   E   |
// ════════╬═════════╪═══════╪
//    1    ║ =E1:E2  |   1   |
// ────────╫─────────┼───────┼
//    2    ║   99    |   2   |
// ────────╫─────────┼───────┼
//
// A value cell at C2 blocks the spill of C1.
// The ordering algorithm must not suppress the #SPILL! error.
#[test]
fn spill_blocked_by_value() {
    let mut model = new_empty_model();

    model.set_a1("C1", "=E1:E2");
    model.set_a1("C2", "99");
    model.set_a1("E1", "1");
    model.set_a1("E2", "2");

    model.evaluate();

    assert_eq!(model.get_text_a1("C1"), "#SPILL!");
    assert_eq!(model.get_text_a1("C2"), "99");
}

//         ║    A    |    C           |
// ════════╬═════════╪════════════════╪
//    1    ║ =C3*2   | =SEQUENCE(3)   |
// ────────╫─────────┼────────────────┼
//    2    ║         |                |
// ────────╫─────────┼────────────────┼
//    3    ║         |                |
// ────────╫─────────┼────────────────┼
//
// C1=SEQUENCE(3) spills to C1=1, C2=2, C3=3.
// A1=C3*2 reads C3 (a spill cell); after correct evaluation A1=6.
// In natural order A1 is evaluated before C1 and reads 0 from empty C3.
#[test]
fn non_spill_reads_spill_area() {
    let mut model = new_empty_model();

    model.set_a1("A1", "=C3*2");
    model.set_a1("C1", "=SEQUENCE(3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("C1"), "1");
    assert_eq!(model.get_text_a1("C2"), "2");
    assert_eq!(model.get_text_a1("C3"), "3");
    assert_eq!(model.get_text_a1("A1"), "6");
}
