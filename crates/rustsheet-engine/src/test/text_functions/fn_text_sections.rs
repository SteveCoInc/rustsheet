#![allow(clippy::unwrap_used, reason = "tests")]

//! TEXT applies the format's text section to text and booleans.

use crate::test::util::new_empty_model;

#[test]
fn test_text_applies_text_section_to_text_and_booleans() {
    // poi TextFormatTests A3:A10, A12:A29; GeneralFormatTests A72, A73 (Excel-cached).
    let mut model = new_empty_model();
    model.set_a1("A1", r#"=TEXT("jello",";;;-@-@-")"#);
    model.set_a1("A2", r#"=TEXT("jello",";;;""hi""")"#);
    model.set_a1("A3", r#"=TEXT("jello",";;;\@@\@")"#);
    model.set_a1("A4", r#"=TEXT(TRUE,";;;-@-")"#);
    model.set_a1("A5", r#"=TEXT(FALSE,";;;-@-@-")"#);
    model.set_a1("A6", r#"=TEXT(TRUE,"General")"#);
    model.set_a1("A7", r#"=TEXT(FALSE,"General")"#);
    model.set_a1("B1", "jello");
    model.set_a1("B2", "TRUE");
    model.set_a1("C1", r#"=TEXT(B1:B1,";;;-@-")"#);
    model.set_a1("D1", r#"=TEXT({"a";TRUE},";;;<@>")"#);
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "-jello-jello-");
    assert_eq!(model.get_text_a1("A2"), "hi");
    assert_eq!(model.get_text_a1("A3"), "@jello@");
    assert_eq!(model.get_text_a1("A4"), "-TRUE-");
    assert_eq!(model.get_text_a1("A5"), "-FALSE-FALSE-");
    assert_eq!(model.get_text_a1("A6"), "TRUE");
    assert_eq!(model.get_text_a1("A7"), "FALSE");
    assert_eq!(model.get_text_a1("C1"), "-jello-");
    assert_eq!(model.get_text_a1("D1"), "<a>");
    assert_eq!(model.get_text_a1("D2"), "<TRUE>");
}

#[test]
fn test_text_section_doubled_quotes_and_single_at_section() {
    // A doubled quote ends one literal and starts the next; a lone section with `@` is the text
    // section (probes/format-doubled-quotes C8, C11; poi TextFormatTests A4).
    let mut model = new_empty_model();
    model.set_a1("A1", r#"=TEXT("jello",";;;""h""""I""")"#);
    model.set_a1("A2", r#"=TEXT("x","@"" """"q""""""")"#);
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "hI");
    assert_eq!(model.get_text_a1("A2"), "x q");
}

#[test]
fn test_text_on_text_over_255_characters_is_value_error() {
    // probes/text-format-text-limit A10, A11, A20, A24, A25.
    let mut model = new_empty_model();
    model.set_a1("A1", r#"=LEN(TEXT(REPT("a",255),";;;@"))"#);
    model.set_a1("A2", r#"=TEXT(REPT("a",256),";;;@")"#);
    model.set_a1("A3", r#"=TEXT(REPT("a",256),"General")"#);
    model.set_a1("A4", r#"=TEXT(REPT("a",128),";;;@@")"#);
    model.set_a1("A5", r#"=LEN(TEXT(REPT("a",127),";;;@@""x"""))"#);
    model.set_a1("A6", r#"=TEXT(REPT("a",32767),REPT("@",1000))"#);
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "255");
    assert_eq!(model.get_text_a1("A2"), "#VALUE!");
    assert_eq!(model.get_text_a1("A3"), "#VALUE!");
    assert_eq!(model.get_text_a1("A4"), "#VALUE!");
    assert_eq!(model.get_text_a1("A5"), "255");
    assert_eq!(model.get_text_a1("A6"), "#VALUE!");
}

#[test]
fn test_text_reads_number_like_text_and_prints_control_characters() {
    // gen-m5/901e91c09b347914 Calc!A210, eb942f4a59b1c004 Calc!A54, d586a1e0c9896942 Calc!A38,
    // c0651f0e0fb544d0 Calc!A336 (excel-graph, T-2158).
    let mut model = new_empty_model();
    model.set_a1("A1", r#"=TEXT("1","0,000.0")"#);
    model.set_a1("A2", r#"=TEXT(DOLLAR(3.14159),"h:mm:ss.00")"#);
    model.set_a1("A3", "=TEXT(100,CHAR(10))");
    model.set_a1("A4", "=TEXT(-2.5,CHAR(1))");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "0,001.0");
    assert_eq!(model.get_text_a1("A2"), "3:21:36.00");
    assert_eq!(model.get_text_a1("A3"), "\n");
    assert_eq!(model.get_text_a1("A4"), "-\u{1}");
}

#[test]
fn test_text_reads_number_like_text_whatever_the_sections() {
    // probes/text-number-like-text-and-control-chars (excel-graph, T-2332): probe A1, A2, A9, A10, A12 and A15 are
    // this test's A1 to A6.
    let mut model = new_empty_model();
    model.set_a1("A1", r#"=TEXT("001","@")"#);
    model.set_a1("A2", r#"=TEXT("1","0;0;0;@")"#);
    model.set_a1("A3", r#"=TEXT("1","""@""0")"#);
    model.set_a1("A4", r#"=TEXT("1","\@")"#);
    model.set_a1("A5", r#"=TEXT("abc","0;0;0;""x""@")"#);
    model.set_a1("A6", r#"=TEXT("1","0;0;0;;")"#);
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "1");
    assert_eq!(model.get_text_a1("A2"), "1");
    assert_eq!(model.get_text_a1("A3"), "@1");
    assert_eq!(model.get_text_a1("A4"), "@");
    assert_eq!(model.get_text_a1("A5"), "xabc");
    assert_eq!(model.get_text_a1("A6"), "#VALUE!");
}
