use crate::cell::CellValue;
use crate::test::util::new_empty_model;

#[test]
fn fn_besseli() {
    let mut model = new_empty_model();

    model.set_a1("B1", "=BESSELI()");
    model.set_a1("B2", "=BESSELI(1,2, 1)");

    model.evaluate();

    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
}

#[test]
fn fn_besselj() {
    let mut model = new_empty_model();

    model.set_a1("B1", "=BESSELJ()");
    model.set_a1("B2", "=BESSELJ(1,2, 1)");

    model.evaluate();

    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
}

#[test]
fn fn_besselk() {
    let mut model = new_empty_model();

    model.set_a1("B1", "=BESSELK()");
    model.set_a1("B2", "=BESSELK(1,2, 1)");

    model.evaluate();

    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
}

#[test]
fn fn_bessely() {
    let mut model = new_empty_model();

    model.set_a1("B1", "=BESSELY()");
    model.set_a1("B2", "=BESSELY(1,2, 1)");

    model.evaluate();

    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
}

/// Excel: any negative order is `#NUM!`, checked before truncation
/// (probes/eng-gen-sample-edges E1:E10).
#[test]
fn negative_fractional_order_is_num_error() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=BESSELK(5,-0.0001)");
    model.set_a1("A2", "=BESSELJ(5,-0.9999)");
    model.set_a1("A3", "=BESSELY(5,-1E-300)");
    model.set_a1("A4", "=BESSELI(5,-0.5)");
    model.set_a1("A5", "=BESSELK(5,0.9999)=BESSELK(5,0)");
    model.evaluate();
    for cell in ["A1", "A2", "A3", "A4"] {
        assert_eq!(model.get_text_a1(cell), *"#NUM!", "{cell}");
    }
    assert_eq!(model.get_text_a1("A5"), *"TRUE");
}

/// Excel: past the phase 134217727 the asymptotic form's sine and cosine do not answer, so
/// BESSELJ and BESSELY are `#NUM!` there and in every order recurring from it, while the order-1
/// phase, 1.57 lower, can still answer (probes/t2547-bessel-phase-cut A9:A12, A37:A41, A49:A50).
#[test]
fn asymptotic_phase_past_the_inner_trig_cut_is_num_error() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=BESSELJ(134217727.785398,0)");
    model.set_a1("A2", "=BESSELJ(134217727.786398,0)");
    model.set_a1("A3", "=BESSELY(134217727.786398,0)");
    model.set_a1("A4", "=BESSELJ(134217728.5,1)");
    model.set_a1("A5", "=BESSELJ(134217728.5,2)");
    model.set_a1("A6", "=BESSELY(134217728.5,4)");
    model.set_a1("A7", "=BESSELJ(-134217728,0)");
    model.set_a1("A8", "=BESSELJ(-134217728,1)");
    model.set_a1("A9", "=BESSELY(1E+15,1)");
    model.set_a1("A10", "=BESSELY(134217727.785398,0)");
    model.evaluate();
    for cell in ["A2", "A3", "A5", "A6", "A7", "A9"] {
        assert_eq!(model.get_text_a1(cell), *"#NUM!", "{cell}");
    }
    let bits = |c: &str| match model.get_cell_value_by_ref(&format!("Sheet1!{c}")) {
        Ok(CellValue::Number(n)) => n.to_bits(),
        other => panic!("expected number, got {other:?}"),
    };
    assert_eq!(bits("A4"), (-6.297_380_020_515_363e-5_f64).to_bits());
    assert_eq!(bits("A8"), 6.863_275_229_685_527e-5_f64.to_bits());
    assert_eq!(bits("A1"), (-2.020_590_456_739_121_3e-5_f64).to_bits());
    assert_eq!(bits("A10"), (-6.584_002_087_696_111e-5_f64).to_bits());
}

/// Excel drops a subnormal term `2n/x * K(n)` from BESSELK's upward recurrence, so near x = 700
/// higher orders repeat K0 and K1 (probes-numeric/m9-besselk-underflow).
#[test]
fn besselk_recurrence_flushes_a_subnormal_term() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=BESSELK(700,2)=BESSELK(700,0)");
    model.set_a1("A2", "=BESSELK(702,4)=BESSELK(702,0)");
    model.set_a1("A3", "=BESSELK(705,5)=BESSELK(705,1)");
    model.set_a1("A4", "=BESSELK(700,3)");
    model.set_a1("A5", "=BESSELK(702,4)");
    model.evaluate();
    for cell in ["A1", "A2", "A3"] {
        assert_eq!(model.get_text_a1(cell), *"TRUE", "{cell}");
    }
    let bits = |c: &str| match model.get_cell_value_by_ref(&format!("Sheet1!{c}")) {
        Ok(CellValue::Number(n)) => n.to_bits(),
        other => panic!("expected number, got {other:?}"),
    };
    assert_eq!(bits("A4"), 4.699_795_140_140_56e-306_f64.to_bits());
    assert_eq!(bits("A5"), 6.310_849_433_265_845e-307_f64.to_bits());
}

/// BESSELJ and BESSELI return 0 for a subnormal result; BESSELK answers at x = 2^-1021 exactly
/// and BESSELY answers down to 2^-1022 (probes/t4552-bessel-tiny-x-cut B1:B2, C6, C8, D2, D6, D8,
/// E8; engineering.md "BESSELK at a tiny x").
#[test]
fn bessel_results_at_the_bottom_of_the_normal_range() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=BESSELJ(2^-1022,1)");
    model.set_a1("A2", "=BESSELI(4.4501477170144E-308,1)");
    model.set_a1("A3", "=BESSELI(2^-1022,1)");
    model.set_a1("A4", "=BESSELI(2^-1021,1)=2^-1022");
    model.set_a1("A5", "=BESSELJ(2^-1021,1)>0");
    model.set_a1("A6", "=BESSELK(2^-1021,0)");
    model.set_a1("A7", "=BESSELY(2^-1022,0)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"0");
    assert_eq!(model.get_text_a1("A2"), *"0");
    assert_eq!(model.get_text_a1("A3"), *"0");
    assert_eq!(model.get_text_a1("A4"), *"TRUE");
    assert_eq!(model.get_text_a1("A5"), *"TRUE");
    assert_eq!(model.get_text_a1("A6"), *"707.8192029");
    assert_eq!(model.get_text_a1("A7"), *"-451.052972");
}
