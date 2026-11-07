use crate::test::util::new_empty_model;

#[test]
fn formulas_near_the_limits_do_not_panic() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=T.INV.2T(1.9999999999999998,1)");
    model.set_a1("A2", "=T.INV(1E-300,99999)");
    model.set_a1("A3", "=CONFIDENCE.T(1E-300,1,100000)");
    model.evaluate();
    // Excel gives -63712901546918.43; the last digits are T-1542's.
    let a1: f64 = model.get_text_a1("A1").parse().unwrap_or(f64::NAN);
    assert!((a1 / -63_712_901_546_918.43 - 1.0).abs() < 1e-6, "{a1}");
    assert_ne!(model.get_text_a1("A2"), *"#NUM!");
    assert_ne!(model.get_text_a1("A3"), *"#NUM!");
}

/// A two-tailed p whose half is below the smallest normal double is p = 0: T.INV.2T and TINV
/// are `#DIV/0!` at df 1 and `#NUM!` from df 2, CONFIDENCE.T `#NUM!` at every df; just above
/// `2.0 * f64::MIN_POSITIVE`, and T.INV at the same p, are numbers (probes/t4541-t-inv-2t-tiny-p rows 1-5;
/// statistics.md "T.DIST's density is #NUM! at a huge x, and T.INV.2T and TINV at p = 2.2E-308").
#[test]
fn two_tailed_p_with_a_subnormal_half_is_zero() {
    let mut model = new_empty_model();
    let below = "4.4501477170144E-308";
    let above = "4.45014771701441E-308";
    model.set_a1("A1", &format!("=T.INV.2T({below},1)"));
    model.set_a1("A2", &format!("=T.INV.2T({below},2)"));
    model.set_a1("A3", &format!("=TINV({below},30)"));
    model.set_a1("A4", &format!("=CONFIDENCE.T({below},1,2)"));
    model.set_a1("A5", &format!("=CONFIDENCE.T({below},1,31)"));
    model.set_a1("A6", "=T.INV.2T(2.22507385850721E-308,9)");
    model.set_a1("B1", &format!("=T.INV.2T({above},1)"));
    model.set_a1("B2", &format!("=T.INV.2T({above},2)"));
    model.set_a1("B3", &format!("=TINV({above},30)"));
    model.set_a1("B4", &format!("=CONFIDENCE.T({above},1,2)"));
    model.set_a1("B5", &format!("=CONFIDENCE.T({above},1,31)"));
    model.set_a1("B6", "=T.INV(2.22507385850721E-308,9)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"#DIV/0!");
    for cell in ["A2", "A3", "A4", "A5", "A6"] {
        assert_eq!(model.get_text_a1(cell), *"#NUM!", "{cell}");
    }
    for cell in ["B1", "B2", "B3", "B4", "B5", "B6"] {
        let value: f64 = model.get_text_a1(cell).parse().unwrap_or(f64::NAN);
        assert!(value.is_finite(), "{cell}: {}", model.get_text_a1(cell));
    }
}
