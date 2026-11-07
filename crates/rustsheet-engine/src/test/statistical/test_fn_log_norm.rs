#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_log_norm_dist_smoke() {
    let mut model = new_empty_model();

    // Valid: CDF and PDF
    model.set_a1("A1", "=LOGNORM.DIST(4, 3.5, 1.2, TRUE)");
    model.set_a1("A2", "=LOGNORM.DIST(4, 3.5, 1.2, FALSE)");

    // Wrong number of arguments -> #ERROR!
    model.set_a1("A3", "=LOGNORM.DIST(4, 3.5, 1.2)");
    model.set_a1("A4", "=LOGNORM.DIST(4, 3.5, 1.2, TRUE, FALSE)");

    // Domain errors:
    // x <= 0 -> #NUM!
    model.set_a1("A5", "=LOGNORM.DIST(0, 3.5, 1.2, TRUE)");
    // std_dev <= 0 -> #NUM!
    model.set_a1("A6", "=LOGNORM.DIST(4, 3.5, 0, TRUE)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"0.039083556");
    assert_eq!(model.get_text_a1("A2"), *"0.017617597");

    assert_eq!(model.get_text_a1("A3"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A4"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A5"), *"#NUM!");
    assert_eq!(model.get_text_a1("A6"), *"#NUM!");
}

#[test]
fn test_fn_log_norm_inv_smoke() {
    let mut model = new_empty_model();

    // Valid call
    model.set_a1("A1", "=LOGNORM.INV(0.5, 3.5, 1.2)");

    // Wrong number of arguments -> #ERROR!
    model.set_a1("A2", "=LOGNORM.INV(0.5, 3.5)");
    model.set_a1("A3", "=LOGNORM.INV(0.5, 3.5, 1.2, 0)");

    // Domain errors:
    // probability <= 0 or >= 1 -> #NUM!
    model.set_a1("A4", "=LOGNORM.INV(0, 3.5, 1.2)");
    model.set_a1("A5", "=LOGNORM.INV(1, 3.5, 1.2)");
    // std_dev <= 0 -> #NUM!
    model.set_a1("A6", "=LOGNORM.INV(0.5, 3.5, 0)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"33.11545196");

    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A3"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A4"), *"#NUM!");
    assert_eq!(model.get_text_a1("A5"), *"#NUM!");
    assert_eq!(model.get_text_a1("A6"), *"#NUM!");
}

#[test]
fn test_fn_log_norm_arguments() {
    let mut model = new_empty_model();

    model.set_a1("A1", "=LOGNORM.DIST()");
    model.set_a1("A2", "=LOGNORM.DIST(0.5)");
    model.set_a1("A3", "=LOGNORM.DIST(0.5, 3.5)");
    model.set_a1("A4", "=LOGNORM.DIST(0.5, 3.5, 1.2)");
    model.set_a1("A5", "=LOGNORM.DIST(0.5, 3.5, 1.2, 0)");
    model.set_a1("A6", "=LOGNORM.DIST(0.5, 3.5, 1.2, 0, 1)");

    model.set_a1("B1", "=LOGNORM.INV()");
    model.set_a1("B2", "=LOGNORM.INV(0.5)");
    model.set_a1("B3", "=LOGNORM.INV(0.5, 3.5)");
    model.set_a1("B4", "=LOGNORM.INV(0.5, 3.5, 1.2)");
    model.set_a1("B5", "=LOGNORM.INV(0.5, 3.5, 1.2, 0)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A3"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A4"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A5"), *"0.001483811");
    assert_eq!(model.get_text_a1("A6"), *"#ERROR!");

    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B3"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B4"), *"33.11545196");
    assert_eq!(model.get_text_a1("B5"), *"#ERROR!");
}

#[test]
fn log_norm_inv_tail_is_exp_of_the_ppnd16_quantile() {
    // Excel's `LOGNORM.INV(1E-20,0,1)` (probes-numeric/t1543-inverse-cdf-tiny-p A16): EXP of NORM.INV's
    // PPND16 tail quantile, where statrs' `LogNormal::inverse_cdf` was 9.493291347224338E-05 (T-0925).
    let mut model = new_empty_model();
    model.set_a1("A1", "=LOGNORM.INV(1E-20,0,1)=9.493291347224372E-05");
    model.set_a1("A2", "=LOGNORM.INV(1E-20,0,1)=EXP(NORM.INV(1E-20,0,1))");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"TRUE");
    assert_eq!(model.get_text_a1("A2"), *"TRUE");
}
