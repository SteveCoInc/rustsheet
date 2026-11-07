#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn arguments() {
    // Day 9 Dec 2025 = serial number 46000, Tuesday, week 50
    let mut model = new_empty_model();
    model.set_a1("A1", "=WORKDAY()");
    model.set_a1("A2", "=WORKDAY(46000)");
    model.set_a1("A3", "=WORKDAY(46000, 5)");
    model.set_a1("A4", "=WORKDAY(46000, 5, 46001)");
    model.set_a1("A5", "=WORKDAY(46000, 5, 46001, 2)");

    model.set_a1("B1", "=WORKDAY.INTL()");
    model.set_a1("B2", "=WORKDAY.INTL(46000)");
    model.set_a1("B3", "=WORKDAY.INTL(46000, 5)");
    model.set_a1("B4", "=WORKDAY.INTL(46000, 5, 11)");
    model.set_a1("B5", "=WORKDAY.INTL(46000, 5, 11, 46001)");
    model.set_a1("B6", "=WORKDAY.INTL(46000, 5, 11, 46001, 2)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A3"), *"46007");
    assert_eq!(model.get_text_a1("A4"), *"46008");
    assert_eq!(model.get_text_a1("A5"), *"#ERROR!");

    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B3"), *"46007");
    assert_eq!(model.get_text_a1("B4"), *"46006");
    assert_eq!(model.get_text_a1("B5"), *"46007");
    assert_eq!(model.get_text_a1("B6"), *"#ERROR!");
}

#[test]
fn whole_week_skipping() {
    // 46000 is a Tuesday; 1000 workdays at 5 per week is exactly 200 weeks.
    let mut model = new_empty_model();
    model.set_a1("A1", "=WORKDAY(46000, 1000)");
    model.set_a1("A2", "=WORKDAY(46000, 1000, 46100)");
    model.set_a1("A3", "=WORKDAY(46000, -1000)");
    model.set_a1("A4", "=WORKDAY(46000, 1000, 46103)");
    model.set_a1("A5", "=WORKDAY.INTL(46000, 2000000, 11)");
    model.set_a1("A6", "=WORKDAY.INTL(46000, 4000000, 11)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"47400");
    // 46100 is a Thursday inside the span: one extra workday.
    assert_eq!(model.get_text_a1("A2"), *"47401");
    assert_eq!(model.get_text_a1("A3"), *"44600");
    // 46103 is a Sunday: no effect.
    assert_eq!(model.get_text_a1("A4"), *"47400");
    assert_eq!(model.get_text_a1("A5"), *"2379333");
    assert_eq!(model.get_text_a1("A6"), *"#NUM!");
}

#[test]
fn whole_week_skipping_dense_holidays() {
    // Every workday of a 2000-row holiday column is a holiday; the answer lies past the column.
    let mut model = new_empty_model();
    for row in 1..=2000 {
        model.set_a1(&format!("H{row}"), &format!("={}", 46000 + row));
    }
    model.set_a1("A1", "=WORKDAY.INTL(46000, 7, 1, H1:H2000)");
    model.set_a1("A2", "=WORKDAY.INTL(46000, -7, 1, H1:H2000)");
    model.set_a1("A3", "=WORKDAY.INTL(46000, 7, 11, H1:H2000)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"48009");
    assert_eq!(model.get_text_a1("A2"), *"45989");
    assert_eq!(model.get_text_a1("A3"), *"48008");
}

#[test]
fn whole_week_skipping_lifted_weekend() {
    // Every lifted weekend code steps 2M workdays without walking the days one by one.
    let mut model = new_empty_model();
    model.set_a1("A1", "=WORKDAY.INTL(46000, 1000000, SEQUENCE(7))");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"1446000");
}

// probes/t2238-holidays-if-3d-branch (T-2238): only the branch IF takes is read as holidays, so a
// 3-D branch not taken is no error; a taken one is #VALUE!, per element when lifted. A day count
// of 0 never reads the holidays, while NETWORKDAYS over one day still does.
#[test]
fn holidays_follow_the_branch_if_takes() {
    let mut model = new_empty_model();
    model.add_sheet("Sheet2").unwrap();
    model.set_a1("A1", "45005");
    model.set_a1("A9", "FALSE");
    model.set_a1("B1", "=WORKDAY(45000,5,IF(FALSE,Sheet1:Sheet2!A1,A1))");
    model.set_a1("B2", "=NETWORKDAYS(45000,45030,IF(A9,Sheet1:Sheet2!A1,A1))");
    model.set_a1(
        "B3",
        "=WORKDAY.INTL(45000,5,1,IF(TRUE,Sheet1:Sheet2!A1,A1))",
    );
    model.set_a1(
        "B4",
        "=ROWS(WORKDAY({45000;45001},5,IF(TRUE,Sheet1:Sheet2!A1,A1)))",
    );
    model.set_a1("B5", "=WORKDAY(45000,0,Sheet1:Sheet2!A1)");
    model.set_a1("B6", "=NETWORKDAYS(45000,45000,Sheet1:Sheet2!A1)");
    model.set_a1("B7", "=WORKDAY(45000,5,IF(FALSE,Sheet1:Sheet2!A1))");
    model.set_a1("B8", "=WORKDAY(45000,5,CHOOSE(2,Sheet1:Sheet2!A1,A1))");
    // An array condition: one element taking the 3-D branch is one #VALUE! (M1-M3).
    model.set_a1(
        "B9",
        "=ROWS(WORKDAY(45000,5,IF({TRUE;FALSE},Sheet1:Sheet2!A1,A1)))",
    );
    model.set_a1(
        "B10",
        "=WORKDAY(45000,5,IF({FALSE;FALSE},Sheet1:Sheet2!A1,A1))",
    );
    model.evaluate();

    assert_eq!(model.get_text_a1("B1"), *"45008");
    assert_eq!(model.get_text_a1("B2"), *"22");
    assert_eq!(model.get_text_a1("B3"), *"#VALUE!");
    assert_eq!(model.get_text_a1("B4"), *"2");
    assert_eq!(model.get_text_a1("B5"), *"45000");
    assert_eq!(model.get_text_a1("B6"), *"#VALUE!");
    assert_eq!(model.get_text_a1("B7"), *"#VALUE!");
    assert_eq!(model.get_text_a1("B8"), *"45008");
    assert_eq!(model.get_text_a1("B9"), *"#VALUE!");
    assert_eq!(model.get_text_a1("B10"), *"45008");
}

/// Excel's error order between the holidays, the dates, start + days and the weekend
/// (probes/workday-holidays-error-order; dates.md "WORKDAY and NETWORKDAYS holidays: blank and
/// number-text cells, and the error order").
#[test]
fn holidays_error_order() {
    let mut model = new_empty_model();
    model.set_a1("H1", "a");
    model.set_a1("H2", "-1");
    model.set_a1("H3", "a");
    let cases = [
        // start + days off the calendar beats the holidays and the weekend
        ("=WORKDAY(1,-18,H1)", "#NUM!"),
        ("=WORKDAY.INTL(1,-18,\"abc\",NA())", "#NUM!"),
        ("=WORKDAY.INTL(5,-5,\"1111111\")", "#VALUE!"),
        ("=WORKDAY.INTL(5,-6,\"1111111\")", "#NUM!"),
        // a walk off the calendar comes after the holidays
        ("=WORKDAY(10,-8,H1)", "#VALUE!"),
        ("=WORKDAY(10,-8)", "#NUM!"),
        // a single non-number holiday beats the weekend, an array or a range does not
        ("=WORKDAY.INTL(45000,5,99,\"a\")", "#VALUE!"),
        ("=WORKDAY.INTL(45000,5,99,H1:H1)", "#VALUE!"),
        ("=WORKDAY.INTL(45000,5,99,{\"a\"})", "#NUM!"),
        ("=WORKDAY.INTL(45000,5,99,H1:H2)", "#NUM!"),
        ("=WORKDAY.INTL(45000,5,\"1111111\",-1)", "#VALUE!"),
        // the first bad holiday decides
        ("=WORKDAY(45000,5,H2:H3)", "#NUM!"),
        ("=WORKDAY(45000,5,{\"a\",-1})", "#VALUE!"),
        // each date's range comes before the next argument
        ("=NETWORKDAYS(-1,\"x\")", "#NUM!"),
        ("=NETWORKDAYS(45000,-1,H1)", "#NUM!"),
        ("=WORKDAY(-1,TRUE)", "#NUM!"),
        // a fractional weekend truncates
        ("=WORKDAY.INTL(11303,-4,1.25,H1:H2)", "#VALUE!"),
    ];
    for (row, (formula, _)) in cases.iter().enumerate() {
        model.set_a1(&format!("A{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model.get_text_a1(&format!("A{}", row + 1)),
            *expected,
            "{formula}"
        );
    }
}
