#![allow(clippy::unwrap_used)]
use crate::test::util::{new_empty_model, round_trip_value};
#[test]
fn test_fn_t_test_smoke() {
    let mut model = new_empty_model();
    model._set("A2", "3");
    model._set("A3", "4");
    model._set("A4", "5");
    model._set("A5", "6");
    model._set("A6", "10");
    model._set("A7", "3");
    model._set("A8", "2");
    model._set("A9", "4");
    model._set("A10", "7");

    model._set("B2", "6");
    model._set("B3", "19");
    model._set("B4", "3");
    model._set("B5", "2");
    model._set("B6", "13");
    model._set("B7", "4");
    model._set("B8", "5");
    model._set("B9", "17");
    model._set("B10", "3");

    model._set("C1", "=T.TEST(A2:A10, B2:B10, 1, 1)");
    model._set("C2", "=T.TEST(A2:A10, B2:B10, 1, 2)");
    model._set("C3", "=T.TEST(A2:A10, B2:B10, 1, 3)");
    model._set("C4", "=T.TEST(A2:A10, B2:B10, 2, 1)");
    model._set("C5", "=T.TEST(A2:A10, B2:B10, 2, 2)");
    model._set("C6", "=T.TEST(A2:A10, B2:B10, 2, 3)");

    model.evaluate();

    assert_eq!(model._get_text("C1"), *"0.103836888");
    assert_eq!(model._get_text("C2"), *"0.100244599");
    assert_eq!(model._get_text("C3"), *"0.105360319");
    assert_eq!(model._get_text("C4"), *"0.207673777");
    assert_eq!(model._get_text("C5"), *"0.200489197");
    assert_eq!(model._get_text("C6"), *"0.210720639");
}

#[test]
fn test_fn_t_test_error_sample_before_tails_and_type() {
    let mut model = new_empty_model();
    for (i, v) in ["3", "4", "5", "6"].iter().enumerate() {
        model._set(&format!("A{}", i + 1), v);
        model._set(&format!("B{}", i + 1), &(i * 2 + 1).to_string());
    }
    model._set("B2", "#N/A");
    model._set("D1", "=T.TEST(1/0,A1:A3,3,2)");
    model._set("D2", "=T.TEST(A1:A3,NA(),1,9)");
    model._set("D3", "=T.TEST(A1:A3,A1:A3,-2.5,\"a\")");
    // A hand expectation: Excel's evidence is the computed empty text (gen-m9 46ac3ef5e61583ca A344); T-2853 probes
    // the typed literal.
    model._set("D4", "=T.TEST(\"\",A1:A4,1,3)");
    // An error inside a range still loses to a bad tails (T-1813).
    model._set("D5", "=T.TEST(B1:B3,A1:A3,3,2)");

    model.evaluate();

    assert_eq!(model._get_text("D1"), *"#DIV/0!");
    assert_eq!(model._get_text("D2"), *"#N/A");
    assert_eq!(model._get_text("D3"), *"#VALUE!");
    assert_eq!(model._get_text("D4"), *"#DIV/0!");
    assert_eq!(model._get_text("D5"), *"#NUM!");
}

// T-3730: at df 13 to 16 (Welch's fractional df among them) and t <= 1, T.TEST's tail is the
// ported TOMS 708 kernel K(df/2, 1/2, y, 1 - y), whose prefactor carries Excel's GAMMALN-sum
// normalization on the centre before 1 minus it; the series scaled the tail after it and stood 5
// to 19 ulp off. The ranges of each gen-m9 cell are written out as arrays in range order (text and
// logicals dropped, as T.TEST drops them from a reference). Excel values (excel-graph):
// 928d08e3e1eecb5f A393, ce312817dc9c7f0b A228, 9bc247733a1ca9fa A254, f82473275380882e A308,
// c7664e494e418395 A219.
#[test]
fn test_fn_t_test_centre_tail_kernel() {
    let mut model = new_empty_model();
    model._set(
        "A1",
        "=T.TEST({2.5,0.3,-0.5,0.5,-1,0.3,0.5,0.5,0},{0.3,-0.5,1.005,0.5,0.5,1e-15,-1,1},2,2)",
    );
    model._set(
        "A2",
        "=TTEST({0.2,0},{0.2,100,1000000000000000,42,-0.5,123456789.123,2,7,1000000000000000,-1,1e-15,0.3,100,0.1,1.005},2,2)",
    );
    model._set(
        "A3",
        "=T.TEST({123456789.123,2,0.3,2,42},{123456789.123,0.2,0.2,123456789.123,2,0.3,2,0.1,42,0.2},2,2)",
    );
    model._set(
        "A4",
        "=TTEST({2.5,2.5},{2.5,1.005,0,-2.5,100,0,-0.0001,0,100,0.3,-0.5,0,0.3},2,2)",
    );
    model._set(
        "A5",
        "=TTEST({3.14159,-0.0001,0.5,123456789.123},{1000000000000000,1e-15,1000000000000000,0.3},1,3)",
    );
    model.evaluate();
    let number = |cell: &str| round_trip_value(&model, cell).parse::<f64>().unwrap();
    for (cell, excel) in [
        ("A1", 0.777_203_160_191_955_1_f64),
        ("A2", 0.609_938_634_913_881),
        ("A3", 0.999_999_877_397_965_9),
        ("A4", 0.643_679_992_884_589_9),
        ("A5", 0.090_845_066_732_484_53),
    ] {
        assert_eq!(number(cell).to_bits(), excel.to_bits(), "{cell}");
    }
}
