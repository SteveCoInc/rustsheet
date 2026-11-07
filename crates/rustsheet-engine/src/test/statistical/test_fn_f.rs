#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_f_dist_sanity() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=F.DIST(15, 6, 4, TRUE)");
    model.set_a1("A2", "=F.DIST(15, 6, 4, FALSE)");
    model.set_a1("A3", "=F.DIST(15, 6, 4)");
    model.set_a1("A4", "=F.DIST(15, 6, 4, TRUE, FALSE)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"0.989741952");
    assert_eq!(model.get_text_a1("A2"), *"0.001271447");
    assert_eq!(model.get_text_a1("A3"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A4"), *"#ERROR!");
}

#[test]
fn test_fn_f_dist_density_at_zero() {
    // probes-numeric/t1049-fdist A27, A28 (T-2095).
    let mut model = new_empty_model();
    model.set_a1("A1", "=F.DIST(0,1,7,FALSE)");
    model.set_a1("A2", "=F.DIST(0,2,7,FALSE)");
    model.set_a1("A3", "=F.DIST(0,2.9,7,FALSE)");
    model.set_a1("A4", "=F.DIST(0,3,7,FALSE)");
    model.set_a1("A5", "=F.DIST(0,1,7,TRUE)");
    model.set_a1("A6", "=F.DIST.RT(0,1,7)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"#NUM!");
    assert_eq!(model.get_text_a1("A2"), *"1");
    assert_eq!(model.get_text_a1("A3"), *"1");
    assert_eq!(model.get_text_a1("A4"), *"0");
    assert_eq!(model.get_text_a1("A5"), *"0");
    assert_eq!(model.get_text_a1("A6"), *"1");
}

#[test]
fn test_fn_f_dist_rt_sanity() {
    let mut model = new_empty_model();

    // Valid call
    model.set_a1("A1", "=F.DIST.RT(15, 6, 4)");
    // Too few args
    model.set_a1("A2", "=F.DIST.RT(15, 6)");
    // Too many args
    model.set_a1("A3", "=F.DIST.RT(15, 6, 4, 1)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"0.010258048");
    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A3"), *"#ERROR!");
}

#[test]
fn test_fn_f_inv_sanity() {
    let mut model = new_empty_model();

    // Valid call: left-tail inverse
    model.set_a1("A1", "=F.INV(0.9897419523940, 6, 4)");

    // Too many args
    model.set_a1("A2", "=F.INV(0.5, 6, 4, 2)");

    // Too few args
    model.set_a1("A3", "=F.INV(0.5, 6)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"15");
    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A3"), *"#ERROR!");
}

#[test]
fn test_fn_f_inv_rt_sanity() {
    let mut model = new_empty_model();

    // Valid call: left-tail inverse
    model.set_a1("A1", "=F.INV.RT(0.0102580476059808, 6, 4)");

    // Too many args
    model.set_a1("A2", "=F.INV.RT(0.5, 6, 4, 2)");

    // Too few args
    model.set_a1("A3", "=F.INV.RT(0.5, 6)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"15");
    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A3"), *"#ERROR!");
}

#[test]
fn f_inv_rt_deep_upper_tail_is_solved_on_its_own_tail() {
    // statrs' estimate solves the rounded 1 - p, 7% off for F.INV.RT(1E-16,9,3); the refinement on
    // F.DIST.RT gives Excel's 60647458731.09812 and FINV(1E-16,1,7) = 183379.00311608665 (gen-m9
    // 0a9226016e8f5f33 A45, 146b46e74f2dac3b A117; T-0974).
    let mut model = new_empty_model();
    model.set_a1("A1", "=F.INV.RT(1E-16,9,3)");
    model.set_a1("A2", "=FINV(1E-16,1,7)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"60647458731");
    assert_eq!(model.get_text_a1("A2"), *"183379.0031");
}

#[test]
fn test_fn_f_dist_cdf_flushes_a_subnormal_result() {
    // probes/t4026-f-cdf-subnormal (T-4026): Excel's cdf is a true 0 below the smallest normal
    // double (B24, B57, B1 TRUE) and keeps 3.3E-308 just above it (B23, B56 FALSE).
    let mut model = new_empty_model();
    model.set_a1("A1", "=(F.DIST(3.68471993975E-5,316,1,TRUE))=0");
    model.set_a1("A2", "=(F.DIST(1.1432683661E-17,40,1,TRUE))=0");
    model.set_a1("A3", "=(F.DIST(3.66425083e-05,316,1,TRUE))=0");
    model.set_a1("A4", "=(F.DIST(3.69653490428E-5,316,1,TRUE))=0");
    model.set_a1("A5", "=(F.DIST(1.17221034258E-17,40,1,TRUE))=0");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"TRUE");
    assert_eq!(model.get_text_a1("A2"), *"TRUE");
    assert_eq!(model.get_text_a1("A3"), *"TRUE");
    assert_eq!(model.get_text_a1("A4"), *"FALSE");
    assert_eq!(model.get_text_a1("A5"), *"FALSE");
}
