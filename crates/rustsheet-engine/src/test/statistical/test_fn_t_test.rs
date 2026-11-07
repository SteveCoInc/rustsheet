#![allow(clippy::unwrap_used)]
use crate::test::util::new_empty_model;
#[test]
fn test_fn_t_test_smoke() {
    let mut model = new_empty_model();
    model.set_a1("A2", "3");
    model.set_a1("A3", "4");
    model.set_a1("A4", "5");
    model.set_a1("A5", "6");
    model.set_a1("A6", "10");
    model.set_a1("A7", "3");
    model.set_a1("A8", "2");
    model.set_a1("A9", "4");
    model.set_a1("A10", "7");

    model.set_a1("B2", "6");
    model.set_a1("B3", "19");
    model.set_a1("B4", "3");
    model.set_a1("B5", "2");
    model.set_a1("B6", "13");
    model.set_a1("B7", "4");
    model.set_a1("B8", "5");
    model.set_a1("B9", "17");
    model.set_a1("B10", "3");

    model.set_a1("C1", "=T.TEST(A2:A10, B2:B10, 1, 1)");
    model.set_a1("C2", "=T.TEST(A2:A10, B2:B10, 1, 2)");
    model.set_a1("C3", "=T.TEST(A2:A10, B2:B10, 1, 3)");
    model.set_a1("C4", "=T.TEST(A2:A10, B2:B10, 2, 1)");
    model.set_a1("C5", "=T.TEST(A2:A10, B2:B10, 2, 2)");
    model.set_a1("C6", "=T.TEST(A2:A10, B2:B10, 2, 3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("C1"), *"0.103836888");
    assert_eq!(model.get_text_a1("C2"), *"0.100244599");
    assert_eq!(model.get_text_a1("C3"), *"0.105360319");
    assert_eq!(model.get_text_a1("C4"), *"0.207673777");
    assert_eq!(model.get_text_a1("C5"), *"0.200489197");
    assert_eq!(model.get_text_a1("C6"), *"0.210720639");
}

#[test]
fn test_fn_t_test_error_sample_before_tails_and_type() {
    let mut model = new_empty_model();
    for (i, v) in ["3", "4", "5", "6"].iter().enumerate() {
        model.set_a1(&format!("A{}", i + 1), v);
        model.set_a1(&format!("B{}", i + 1), &(i * 2 + 1).to_string());
    }
    model.set_a1("B2", "#N/A");
    model.set_a1("D1", "=T.TEST(1/0,A1:A3,3,2)");
    model.set_a1("D2", "=T.TEST(A1:A3,NA(),1,9)");
    model.set_a1("D3", "=T.TEST(A1:A3,A1:A3,-2.5,\"a\")");
    // A hand expectation: Excel's evidence is the computed empty text (gen-m9 46ac3ef5e61583ca A344); T-2853 probes
    // the typed literal.
    model.set_a1("D4", "=T.TEST(\"\",A1:A4,1,3)");
    // An error inside a range still loses to a bad tails (T-1813).
    model.set_a1("D5", "=T.TEST(B1:B3,A1:A3,3,2)");

    model.evaluate();

    assert_eq!(model.get_text_a1("D1"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("D2"), *"#N/A");
    assert_eq!(model.get_text_a1("D3"), *"#VALUE!");
    assert_eq!(model.get_text_a1("D4"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("D5"), *"#NUM!");
}
