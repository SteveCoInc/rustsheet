#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_expon_dist_smoke() {
    let mut model = new_empty_model();

    // λ = 1, x = 0.5
    // CDF = 1 - e^-0.5 ≈ 0.393469340
    // PDF = e^-0.5 ≈ 0.606530660
    model.set_a1("A1", "=EXPON.DIST(0.5, 1, TRUE)");
    model.set_a1("A2", "=EXPON.DIST(0.5, 1, FALSE)");

    // Wrong number of args
    model.set_a1("A3", "=EXPON.DIST()");
    model.set_a1("A4", "=EXPON.DIST(0.5)");
    model.set_a1("A5", "=EXPON.DIST(0.5, 1)");
    model.set_a1("A6", "=EXPON.DIST(0.5, 1, TRUE, FALSE)");

    // Domain errors
    model.set_a1("A7", "=EXPON.DIST(-1, 1, TRUE)"); // x < 0
    model.set_a1("A8", "=EXPON.DIST(0.5, 0, TRUE)"); // lambda <= 0

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"0.39346934");
    assert_eq!(model.get_text_a1("A2"), *"0.60653066");

    assert_eq!(model.get_text_a1("A3"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A4"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A5"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A6"), *"#ERROR!");

    assert_eq!(model.get_text_a1("A7"), *"#NUM!");
    assert_eq!(model.get_text_a1("A8"), *"#NUM!");
}

#[test]
fn test_fn_expon_dist_cdf_tiny_argument_keeps_precision() {
    // Excel (EXPONDIST.xlsx D16) gives 9.999999999500002e-11 for lambda*x = 1e-10.
    // The residual vs lambda*x is -5e-21 only if 1 - exp(-t) cancellation is avoided.
    let mut model = new_empty_model();
    model.set_a1("A1", "=EXPON.DIST(1E-10, 1, TRUE)-1E-10");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"-5E-21");
}
