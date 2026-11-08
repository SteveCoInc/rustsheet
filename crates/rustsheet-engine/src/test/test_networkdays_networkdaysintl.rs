#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn arguments() {
    let mut model = new_empty_model();
    // 01/01/2025 = serial number 45658
    // 01/01/2026 = serial number 46023
    model.set_a1("A1", "=NETWORKDAYS()");
    model.set_a1("A2", "=NETWORKDAYS(45658)");
    model.set_a1("A3", "=NETWORKDAYS(45658, 46023)");
    model.set_a1("A4", "=NETWORKDAYS(45658, 46023, 46000)");
    model.set_a1("A5", "=NETWORKDAYS(45658, 46023, 5, 6)");

    model.set_a1("B1", "=NETWORKDAYS.INTL()");
    model.set_a1("B2", "=NETWORKDAYS.INTL(45658)");
    model.set_a1("B3", "=NETWORKDAYS.INTL(45658, 46023)");
    model.set_a1("B4", "=NETWORKDAYS.INTL(45658, 46023, 5)");
    model.set_a1("B5", "=NETWORKDAYS.INTL(45658, 46023, 5, 46000)");
    model.set_a1("B6", "=NETWORKDAYS.INTL(45658, 46023, 5, 46000, 5)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A3"), *"262");
    assert_eq!(model.get_text_a1("A4"), *"261");
    assert_eq!(model.get_text_a1("A5"), *"#ERROR!");

    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B3"), *"262");
    assert_eq!(model.get_text_a1("B4"), *"260");
    assert_eq!(model.get_text_a1("B5"), *"259");
    assert_eq!(model.get_text_a1("B6"), *"#ERROR!");
}

#[test]
fn text_literal_holiday_is_a_scalar_holiday() {
    // probes/t4397-holiday-text-forms (excel-graph; dates.md "WORKDAY and NETWORKDAYS holidays"):
    // date text given as a literal is a holiday, and NETWORKDAYS.INTL drops a scalar one on a
    // Saturday whatever the weekend (10), while the same text in a cell is subtracted (9).
    let mut model = new_empty_model();
    model.set_a1("C1", "3/18/2023");
    model.set_a1("A1", "=NETWORKDAYS(45000,45010,\"3/16/2023\")");
    model.set_a1("A2", "=NETWORKDAYS.INTL(45000,45010,13,\"3/18/2023\")");
    model.set_a1("A3", "=NETWORKDAYS.INTL(45000,45010,13,C1)");
    model.set_a1("A4", "=WORKDAY(45000,5,\"$45,001\")");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"7");
    assert_eq!(model.get_text_a1("A2"), *"10");
    assert_eq!(model.get_text_a1("A3"), *"9");
    assert_eq!(model.get_text_a1("A4"), *"45008");
}
