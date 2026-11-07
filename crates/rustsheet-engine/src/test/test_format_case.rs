#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// Excel date-format tokens are case-insensitive: M/D/YYYY behaves like
// m/d/yyyy, including the month-vs-minute disambiguation.
#[test]
fn test_text_uppercase_date_tokens() {
    let mut model = new_empty_model();
    model.set_a1("A1", r#"=TEXT(45000,"m/d/yyyy")"#);
    model.set_a1("A2", r#"=TEXT(45000,"M/D/YYYY")"#);
    model.set_a1("A3", r#"=TEXT(45000,"MM/DD/YYYY")"#);
    model.set_a1("A4", r#"=TEXT(45000,"YYYY")"#);
    model.set_a1("A5", r#"=TEXT(45000,"MMM")"#);
    model.set_a1("A6", r#"=TEXT(45000,"D")"#);
    model.set_a1("A7", r#"=TEXT(45000,"MMM-YY")"#);
    model.set_a1("B3", r#"=TEXT(45000,"mm/dd/yyyy")"#);
    model.set_a1("B4", r#"=TEXT(45000,"yyyy")"#);
    model.set_a1("B5", r#"=TEXT(45000,"mmm")"#);
    model.set_a1("B6", r#"=TEXT(45000,"d")"#);
    model.set_a1("B7", r#"=TEXT(45000,"mmm-yy")"#);
    model.evaluate();
    let lower = model.get_text_a1("A1");
    assert_eq!(model.get_text_a1("A2"), lower);
    assert!(!model.get_text_a1("A3").starts_with('#'));
    assert_eq!(model.get_text_a1("A4"), "2023");
    assert_eq!(model.get_text_a1("A5"), "Mar");
    assert_eq!(model.get_text_a1("A6"), "15");
    assert_eq!(model.get_text_a1("A7"), "Mar-23");
    // Each upper-case format must match its lower-case counterpart.
    assert_eq!(model.get_text_a1("B3"), model.get_text_a1("A3"));
    assert_eq!(model.get_text_a1("B4"), model.get_text_a1("A4"));
    assert_eq!(model.get_text_a1("B5"), model.get_text_a1("A5"));
    assert_eq!(model.get_text_a1("B6"), model.get_text_a1("A6"));
    assert_eq!(model.get_text_a1("B7"), model.get_text_a1("A7"));
}

#[test]
fn test_text_uppercase_minutes_disambiguation() {
    let mut model = new_empty_model();
    model.set_a1("A1", r#"=TEXT(45000.526,"hh:mm:ss")"#);
    model.set_a1("A2", r#"=TEXT(45000.526,"hh:MM:SS")"#);
    model.set_a1("A3", r#"=TEXT(45000.526,"MM/DD/YYYY hh:MM:SS")"#);
    model.evaluate();
    let lower = model.get_text_a1("A1");
    assert_eq!(model.get_text_a1("A2"), lower);
    let combined = model.get_text_a1("A3");
    assert!(combined.ends_with(&lower), "got {combined}");
}
