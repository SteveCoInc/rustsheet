#![allow(clippy::unwrap_used)]

use crate::cell::CellValue;
use crate::test::util::new_empty_model;

#[test]
fn arguments() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=FACT()");
    model.set_a1("A2", "=FACTDOUBLE()");
    model.set_a1("A3", "=FACT(3)");
    model.set_a1("A4", "=FACTDOUBLE(3)");
    model.set_a1("A5", "=FACT(1, 2)");
    model.set_a1("A6", "=FACTDOUBLE(1, 2)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A3"), *"6");
    assert_eq!(model.get_text_a1("A4"), *"3");
    assert_eq!(model.get_text_a1("A5"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A6"), *"#ERROR!");
}

// Excel: numbers.md "FACT and FACTDOUBLE of a huge argument" (probes/fact-large-arguments).
#[test]
fn huge_arguments_are_num_without_looping() {
    let mut model = new_empty_model();
    for (i, f) in [
        "FACT(171)",
        "FACT(1E+15)",
        "FACT(1E+300)",
        "FACTDOUBLE(301)",
        "FACTDOUBLE(1E+15)",
        "FACTDOUBLE(1E+300)",
    ]
    .iter()
    .enumerate()
    {
        model.set_a1(&format!("A{}", i + 1), &format!("={f}"));
    }
    model.set_a1("B1", "=FACT(170.9)");
    model.set_a1("B2", "=FACTDOUBLE(300)");
    model.evaluate();
    for i in 1..=6 {
        assert_eq!(model.get_text_a1(&format!("A{i}")), *"#NUM!");
    }
    assert_eq!(model.get_text_a1("B1"), *"7.2574E+306");
    assert_eq!(model.get_text_a1("B2"), *"8.1544E+307");
}

#[test]
fn large_factorials_round_like_excel() {
    // Excel multiplies from n down: probes/fact-product-order-sweep A43, A171, B246, B247, B249
    // (the ascending product is off by up to ten ulps; T-1249).
    let mut model = new_empty_model();
    model.set_a1("A1", "=FACT(42)");
    model.set_a1("A2", "=FACT(170)");
    model.set_a1("A3", "=FACTDOUBLE(297)");
    model.set_a1("A4", "=FACTDOUBLE(298)");
    model.set_a1("A5", "=FACTDOUBLE(300)");
    model.evaluate();
    let bits = |c: &str| match model.get_cell_value_by_ref(&format!("Sheet1!{c}")) {
        Ok(CellValue::Number(n)) => n.to_bits(),
        other => panic!("expected number, got {other:?}"),
    };
    assert_eq!(bits("A1"), 1.405_006_117_752_880_1e51_f64.to_bits());
    assert_eq!(bits("A2"), 7.257_415_615_308_004e306_f64.to_bits());
    assert_eq!(bits("A3"), 1.255_275_622_599_305_7e304_f64.to_bits());
    assert_eq!(bits("A4"), 2.718_138_023_126_864e305_f64.to_bits());
    assert_eq!(bits("A5"), 8.154_414_069_380_598e307_f64.to_bits());
}
