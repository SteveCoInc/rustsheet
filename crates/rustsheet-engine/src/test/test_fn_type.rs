#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn fn_type_array() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=TYPE()");
    model.set_a1("A2", "=TYPE(A1:C30)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A2"), *"64");
}

// Text over 255 UTF-16 units is 64 in TYPE and 3 in ERROR.TYPE, though ISERROR is FALSE for it
// (probes/type-long-text).
#[test]
fn fn_type_long_text() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=TYPE(REPT(\"a\",255))");
    model.set_a1("A2", "=TYPE(REPT(\"a\",256))");
    model.set_a1("A3", "=ERROR.TYPE(REPT(\"a\",255))");
    model.set_a1("A4", "=ERROR.TYPE(REPT(\"a\",256))");
    model.set_a1("A5", "=TYPE(REPT(UNICHAR(128512),127))");
    model.set_a1("A6", "=TYPE(REPT(UNICHAR(128512),128))");
    model.set_a1("A7", "=ISERROR(REPT(\"a\",256))");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"2");
    assert_eq!(model.get_text_a1("A2"), *"64");
    assert_eq!(model.get_text_a1("A3"), *"#N/A");
    assert_eq!(model.get_text_a1("A4"), *"3");
    assert_eq!(model.get_text_a1("A5"), *"2");
    assert_eq!(model.get_text_a1("A6"), *"64");
    assert_eq!(model.get_text_a1("A7"), *"FALSE");
}
