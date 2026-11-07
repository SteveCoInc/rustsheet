#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn base_min_length_is_at_most_255() {
    // Excel (probes/base-min-length): min_length truncates and may be at most 255; beyond it BASE is
    // #NUM! without padding, so `BASE(245,30,1E+15)` must not allocate 10^15 characters (T-1243).
    let mut model = new_empty_model();
    model.set_a1("A1", "=LEN(BASE(1,2,255.9))");
    model.set_a1("A2", "=BASE(1,2,256)");
    model.set_a1("A3", "=BASE(245,30,1E+15)");
    model.set_a1("A4", "=LEN(BASE(1,2,254.5))");
    model.set_a1("A5", "=BASE(\"x\",2,1E+15)");
    model.set_a1("A6", "=BASE(1,2,\"300\")");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "255");
    assert_eq!(model.get_text_a1("A2"), "#NUM!");
    assert_eq!(model.get_text_a1("A3"), "#NUM!");
    assert_eq!(model.get_text_a1("A4"), "254");
    assert_eq!(model.get_text_a1("A5"), "#VALUE!");
    assert_eq!(model.get_text_a1("A6"), "#NUM!");
}
