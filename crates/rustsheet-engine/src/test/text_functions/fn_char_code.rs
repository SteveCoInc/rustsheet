#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// ── CHAR ──────────────────────────────────────────────────────────────────────

#[test]
fn test_char_basic() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=CHAR(65)");
    model.set_a1("A2", "=CHAR(97)");
    model.set_a1("A3", "=CHAR(32)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "A");
    assert_eq!(model.get_text_a1("A2"), "a");
    assert_eq!(model.get_text_a1("A3"), " ");
}

#[test]
fn test_char_win1252() {
    let mut model = new_empty_model();
    // Code point 128 in Windows-1252 is €
    model.set_a1("A1", "=CHAR(128)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "€");
}

#[test]
fn test_char_error() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=CHAR(0)");
    model.set_a1("A2", "=CHAR(300)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#VALUE!");
    assert_eq!(model.get_text_a1("A2"), "#VALUE!");
}

// ── CODE ──────────────────────────────────────────────────────────────────────

#[test]
fn test_code_basic() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=CODE(\"A\")");
    model.set_a1("A2", "=CODE(\"ABC\")");
    model.set_a1("A3", "=CODE(\" \")");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "65");
    assert_eq!(model.get_text_a1("A2"), "65"); // only first char
    assert_eq!(model.get_text_a1("A3"), "32");
}

#[test]
fn test_code_empty() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=CODE(\"\")");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#VALUE!");
}

// ── UNICHAR ───────────────────────────────────────────────────────────────────

#[test]
fn test_unichar_basic() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=UNICHAR(65)");
    model.set_a1("A2", "=UNICHAR(9786)"); // ☺ U+263A
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "A");
    assert_eq!(model.get_text_a1("A2"), "☺");
}

#[test]
fn test_unichar_error() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=UNICHAR(0)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#VALUE!");
}
