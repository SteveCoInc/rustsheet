#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn fn_encodeurl_args_number() {
    let mut model = new_empty_model();

    model.set_a1("A1", "=ENCODEURL()");
    model.set_a1("A2", r#"=ENCODEURL("a", "b")"#);

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");
}

// docs/excel-semantics/web.md: `#VALUE!` for every argument, lifted over an array
// (probes/info-logic-argument-modes-2 A1:A6, B5; gen-m9/fcf11f2ec6f7a59d A7).
#[test]
fn fn_encodeurl_is_value_error_on_the_web() {
    let mut model = new_empty_model();

    model.set_a1("A1", r#"=ENCODEURL("a b&c")"#);
    model.set_a1("A2", r#"=ENCODEURL("abc")"#);
    model.set_a1("A3", "=ENCODEURL(1)");
    model.set_a1("A4", "=ENCODEURL(TRUE)");
    model.set_a1("D1", "a b");
    model.set_a1("D2", "c");
    model.set_a1("C5", "=ENCODEURL(D1:D2)");
    model.set_a1("A6", "=ENCODEURL(1/0)");
    model.set_a1("G1", "x");
    model.set_a1("H1", "=1/0");
    model.set_a1("E5", "=ENCODEURL(G1:H1)");
    model.set_a1("A7", r#"=CODE(ENCODEURL("a"))"#);

    model.evaluate();

    for cell in ["A1", "A2", "A3", "A4", "C5", "C6", "A7"] {
        assert_eq!(model.get_text_a1(cell), *"#VALUE!", "{cell}");
    }
    assert_eq!(model.get_text_a1("A6"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("E5"), *"#VALUE!");
    assert_eq!(model.get_text_a1("F5"), *"#DIV/0!");
}
