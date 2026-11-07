//! A written-empty required argument of the ToolPak date functions is `#N/A`, and a written-empty
//! optional one is omitted (T-4870; probes/t4870-empty-date-args and t4870-r2-empty-optional,
//! excel-graph, 2026-11-07).

use crate::test::util::new_empty_model;

#[test]
fn empty_required_is_na_and_empty_optional_is_omitted() {
    let mut model = new_empty_model();
    let cells = [
        ("A1", "=EDATE(,1)", "#N/A"),
        ("A2", "=EOMONTH(40000,)", "#N/A"),
        ("A3", "=WORKDAY(,1)", "#N/A"),
        ("A4", "=NETWORKDAYS(1,)", "#N/A"),
        ("A5", "=WEEKNUM(,1)", "#N/A"),
        ("A6", "=YEARFRAC(1,)", "#N/A"),
        ("A7", "=WORKDAY.INTL(1,,1)", "#N/A"),
        ("A8", "=LET(e,,EDATE(1,e))", "#N/A"),
        ("A9", "=LAMBDA(a,b,EDATE(a,b))(1,)", "#N/A"),
        ("B1", "=WORKDAY(45000,10,)", "45014"),
        ("B2", "=NETWORKDAYS(45000,45030,)", "23"),
        ("B3", "=WEEKNUM(45010,)", "12"),
        ("B4", "=WORKDAY.INTL(45000,10,11,)", "45012"),
        ("B5", "=NETWORKDAYS.INTL(45000,45030,11,)", "27"),
        ("B6", "=LET(h,,WORKDAY(45000,10,h))", "45014"),
        ("B7", "=LAMBDA(a,r,WEEKNUM(a,r))(45010,)", "12"),
        ("C1", "=DAYS(10,)", "10"),
    ];
    for (cell, formula, _) in cells {
        model._set(cell, formula);
    }
    model.evaluate();
    for (cell, formula, expected) in cells {
        assert_eq!(model._get_text(cell), expected, "{cell} {formula}");
    }
}
