// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::unreadable_literal)]
// LINT-DEBT-END
#![allow(clippy::unwrap_used)]

use crate::model::Model;
use crate::test::util::{fixed_clock, new_empty_model};

// 14:44 20 Mar 2023 Berlin
const TIMESTAMP_2023: i64 = 1679319865208;

#[test]
fn today_basic() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=TODAY()");
    model.set_a1("A2", "=TEXT(A1, \"yyyy/m/d\")");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"11/8/2022");
    assert_eq!(model.get_text_a1("A2"), *"2022/11/8");
}

#[test]
fn today_now_wrong_arguments() {
    let mut model = new_empty_model();
    // 1 is an invalid timezone, it returns #VALUE!
    model.set_a1("A1", "=TODAY(1)");
    model.set_a1("A2", "=NOW(1)");

    model.set_a1("A3", "=TODAY(1, 2)");
    model.set_a1("A4", "=NOW(1, 2)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A2"), *"#VALUE!");

    assert_eq!(model.get_text_a1("A3"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A4"), *"#ERROR!");
}

#[test]
fn today_with_wrong_tz() {
    let model = Model::new_empty("model", "en", "Wrong Timezone", "en");
    assert!(model.is_err());
}

#[test]
fn now_basic_utc() {
    let mut model = Model::new_empty("model", "en", "UTC", "en").unwrap();
    model.set_clock(fixed_clock(TIMESTAMP_2023));
    model.set_a1("A1", "=TODAY()");
    model.set_a1("A2", "=NOW()");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"3/20/2023");
    // 45005.572511574
    assert_eq!(model.get_text_a1("A2"), *"3/20/2023, 1:44 PM");
}

#[test]
fn now_basic_europe_berlin() {
    let mut model = Model::new_empty("model", "en", "Europe/Berlin", "en").unwrap();
    model.set_clock(fixed_clock(TIMESTAMP_2023));
    model.set_a1("A1", "=TODAY()");
    model.set_a1("A2", "=NOW()");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"3/20/2023");
    // This is UTC + 1 hour: 45005.572511574 + 1/24
    assert_eq!(model.get_text_a1("A2"), *"3/20/2023, 2:44 PM");
}

#[test]
fn today_with_timezone() {
    let mut model = Model::new_empty("model", "en", "UTC", "en").unwrap();
    model.set_clock(fixed_clock(TIMESTAMP_2023));
    model.set_a1("A1", "=TODAY(\"Pacific/Kiritimati\")");
    model.set_a1("A2", "=NOW(\"Europe/Berlin\")");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"3/21/2023");
    // 45005.572511574
    assert_eq!(model.get_text_a1("A2"), *"3/20/2023, 2:44 PM");
    model.evaluate();
}

#[test]
fn today_with_invalid_timezone() {
    let mut model = Model::new_empty("model", "en", "UTC", "en").unwrap();
    model.set_clock(fixed_clock(TIMESTAMP_2023));
    model.set_a1("A1", "=TODAY(\"Invalid/Timezone\")");
    model.set_a1("A2", "=NOW(\"Invalid/Timezone\")");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A2"), *"#VALUE!");
}
