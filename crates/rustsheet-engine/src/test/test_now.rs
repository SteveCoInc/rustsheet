// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::unreadable_literal)]
// LINT-DEBT-END
#![allow(clippy::unwrap_used)]

use crate::test::util::{fixed_clock, new_empty_model};

// 14:44 20 Mar 2023 Berlin
const TIMESTAMP_2023: i64 = 1679319865208;

#[test]
fn arguments() {
    let mut model = new_empty_model();
    model.set_clock(fixed_clock(TIMESTAMP_2023));

    model.set_a1("A1", "=NOW(1, 1)");
    model.set_a1("A2", "=NOW(\"Europe/Berlin\")");
    model.set_a1("A3", "=NOW(\"faketimezone\")");
    model.evaluate();

    assert_eq!(
        model.get_text_a1("A1"),
        "#ERROR!",
        "Wrong number of arguments"
    );
    assert_eq!(model.get_text_a1("A2"), *"3/20/2023, 2:44 PM");
    assert_eq!(
        model.get_text_a1("A3"),
        "#VALUE!",
        "Invalid timezone: faketimezone"
    );
}

#[test]
fn returns_date_time() {
    let mut model = new_empty_model();
    model.set_clock(fixed_clock(TIMESTAMP_2023));
    model.set_a1("A1", "=NOW()");
    model.evaluate();
    let text = model.get_text_a1("A1");
    assert_eq!(text, *"3/20/2023, 1:44 PM");
}

#[test]
fn keeps_sub_second_precision() {
    // 13:44:25.208 UTC: Excel's NOW() is not truncated to whole seconds.
    let mut model = new_empty_model();
    model.set_clock(fixed_clock(TIMESTAMP_2023));
    model.set_a1("A1", "=ROUND((NOW()-INT(NOW()))*86400, 3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"49465.208");
}
