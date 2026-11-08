//! The element-wise text functions check the size of their broadcast result before building it,
//! as `&` and ROUND do (T-1771, CLAUDE.md directive 5). DEVIATION (decision 0014): Excel
//! (expected, not measured) tries to build the array; we refuse one above a whole column
//! (docs/excel-semantics/arrays.md "Array results above 1,048,576 cells").

use crate::test::util::new_empty_model;

#[test]
fn column_against_row_above_the_limit_is_bounded() {
    let mut model = new_empty_model();
    // A filled column, so the result does not depend on how much of the sheet is used.
    for row in 1..=2000 {
        model.set_a1(&format!("A{row}"), "abc");
    }
    // 2,000 x 1,000 = 2,000,000 cells. LEFT's text fits the 64 MiB text budget and FIND's
    // numbers do not count against it, so only the array limit stops them.
    // The cells are below row 2000 so they sit outside both referenced ranges (no circularity).
    model.set_a1("C3001", "=ROWS(LEFT(A1:A2000,COLUMN(A1:ALL1)))");
    model.set_a1("C3002", "=ROWS(FIND(\"a\",A1:A2000,COLUMN(A1:ALL1)))");
    model.set_a1("C3003", "=ROWS(MID(A1:A2000,COLUMN(A1:ALL1),1))");
    model.set_a1("C3004", "=ROWS(CONCATENATE(A1:A2000,COLUMN(A1:ALL1)))");
    model.set_a1("C3005", "=ROWS(REPT(A1:A2000,COLUMN(A1:ALL1)))");

    model.evaluate();

    for cell in ["C3001", "C3002", "C3003", "C3004", "C3005"] {
        assert_eq!(model.get_text_a1(cell), *"#ERROR!", "{cell}");
    }
}

#[test]
fn whole_column_against_whole_row_is_bounded() {
    let mut model = new_empty_model();
    // 2^20 x 2^14 = 2^34 cells; out of column A and row 1 (where they would be circular).
    model.set_a1("C3", "=ROWS(LEFT(A:A,1:1))");
    model.set_a1("C4", "=ROWS(FIND(\"a\",A:A,1:1))");
    // In a cell the shape runs past the grid: #SPILL! from the shapes alone (T-0483).
    model.set_a1("C5", "=LEFT(A:A,1:1)");

    model.evaluate();

    for cell in ["C3", "C4"] {
        assert_eq!(model.get_text_a1(cell), *"#ERROR!", "{cell}");
    }
    assert_eq!(model.get_text_a1("C5"), *"#SPILL!");
}

#[test]
fn broadcast_under_the_limit_is_unchanged() {
    let mut model = new_empty_model();
    model.set_a1("A1", "abc");
    model.set_a1("A2", "xyz");
    // A column of 2 against a row of 3 (and a whole column against one column, 2^20 cells).
    model.set_a1("E1", "=LEFT(A1:A2,COLUMN(A1:C1))");
    model.set_a1("E4", "=ROWS(LEN(A:A))");

    model.evaluate();

    assert_eq!(model.get_text_a1("E1"), *"a");
    assert_eq!(model.get_text_a1("G1"), *"abc");
    assert_eq!(model.get_text_a1("F2"), *"xy");
    assert_eq!(model.get_text_a1("E4"), *"1048576");
}

#[test]
fn text_budget_below_the_limit_stays_value() {
    let mut model = new_empty_model();
    // 3,000 cells is under the array limit, but 3,000 x 32,767 bytes is over the 64 MiB text
    // budget (T-0232), so the budget's own `#VALUE!` still applies: the size check only adds the
    // `#ERROR!` above 1,048,576 cells.
    model.set_a1("C1", "=REPT(\"x\",32767+0*SEQUENCE(3000))");

    model.evaluate();

    assert_eq!(model.get_text_a1("C1"), *"#VALUE!");
}

#[test]
fn unary_text_transforms_share_the_text_budget() {
    // Audit 2 item 1 (directive 5): the unary transforms copied every element without the
    // broadcast text budget, so 4096 shared 32767-character texts built ~128 MiB. They now stop
    // at the cap with LEFT's `#VALUE!` (DEVIATION, decision 0014).
    let mut model = new_empty_model();
    let source = r#"IF(SEQUENCE(4096),REPT("a",32767))"#;
    let calls = [
        "LEFT(#,32767)",
        "UPPER(#)",
        "LOWER(#)",
        "PROPER(#)",
        "TRIM(#)",
        "CLEAN(#)",
        "ASC(#)",
        "DBCS(#)",
    ];
    for (row, call) in calls.iter().enumerate() {
        let call = call.replace('#', source);
        model.set_a1(&format!("A{}", row + 1), &format!("=ROWS({call})"));
    }
    // Under the budget nothing changes.
    model.set_a1("B1", r#"=ROWS(UPPER(IF(SEQUENCE(16),REPT("a",32767))))"#);
    model.set_a1("B2", r#"=INDEX(UPPER({"ab","cd"}),1,2)"#);

    model.evaluate();

    for row in 1..=calls.len() {
        assert_eq!(model.get_text_a1(&format!("A{row}")), *"#VALUE!", "A{row}");
    }
    assert_eq!(model.get_text_a1("B1"), *"16");
    assert_eq!(model.get_text_a1("B2"), *"CD");
}

#[test]
fn arraytotext_is_built_under_the_text_budget() {
    // Audit 2 item 1: 2048 x 32767 bytes of text passes the 64 MiB budget; the bounded builder
    // charges each piece first and returns the budget's `#VALUE!` instead of aborting.
    let mut model = new_empty_model();
    model.set_a1(
        "A1",
        r#"=LEN(ARRAYTOTEXT(IF(SEQUENCE(2048),REPT("a",32767)),1))"#,
    );
    model.set_a1(
        "A2",
        r#"=LEN(ARRAYTOTEXT(IF(SEQUENCE(2048),REPT("a",32767)),0))"#,
    );
    // Separators, quotes and doubled quotes as before.
    model.set_a1("C1", "1");
    model.set_a1("D1", r#"a"b"#);
    model.set_a1("C2", "TRUE");
    model.set_a1("A3", "=ARRAYTOTEXT(C1:E2,1)");
    model.set_a1("A4", "=ARRAYTOTEXT(C1:E2,0)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A2"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A3"), *r#"{1,"a""b",;TRUE,,}"#);
    assert_eq!(model.get_text_a1("A4"), *r#"1, a"b, , TRUE, , "#);
}
