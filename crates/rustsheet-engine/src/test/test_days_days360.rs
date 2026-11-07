use crate::test::util::new_empty_model;

#[test]
fn test_days_days360_arguments() {
    let mut model = new_empty_model();

    model.set_a1("A1", "=DAYS()");
    model.set_a1("A2", "=DAYS(DATE(2026,8,6))");
    model.set_a1("A3", "=DAYS(DATE(2026,8,8), DATE(2026,8,6))");
    model.set_a1(
        "A4",
        "=DAYS(DATE(2026,8,8), DATE(2026,8,6), DATE(2026,8,4))",
    );

    model.set_a1("B1", "=DAYS360()");
    model.set_a1("B2", "=DAYS360(DATE(2026,8,6))");
    model.set_a1("B3", "=DAYS360(DATE(2026,8,8), DATE(2026,8,6))");
    model.set_a1("B4", "=DAYS360(DATE(2026,8,8), DATE(2026,8,6), TRUE)");
    model.set_a1(
        "B5",
        "=DAYS360(DATE(2026,8,8), DATE(2026,8,6), TRUE, FALSE)",
    );

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A3"), *"2");
    assert_eq!(model.get_text_a1("A4"), *"#ERROR!");

    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B3"), *"-2");
    assert_eq!(model.get_text_a1("B4"), *"-2");
    assert_eq!(model.get_text_a1("B5"), *"#ERROR!");
}

#[test]
fn days_with_two_errors_returns_the_leftmost() {
    // probes/t4397-days-two-errors A1:A16 (excel-graph; dates.md "DAYS error order").
    let mut model = new_empty_model();
    model.set_a1("A1", "=DAYS(NA(),1/0)");
    model.set_a1("A2", "=DAYS(1/0,NA())");
    model.set_a1("A3", "=DAYS(\"abc\",NA())");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#N/A");
    assert_eq!(model.get_text_a1("A2"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("A3"), *"#N/A");
}
