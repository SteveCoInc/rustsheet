#![allow(clippy::unwrap_used)]

use crate::cell::CellValue;
use crate::test::util::new_empty_model;

#[test]
fn arguments() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=MOD(5,2)");
    model.set_a1("A2", "=MOD()");
    model.set_a1("A3", "=MOD(5, 2, 1)");
    model.set_a1("A4", "=QUOTIENT(5, 2)");
    model.set_a1("A5", "=QUOTIENT()");
    model.set_a1("A6", "=QUOTIENT(5, 2, 1)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"1");
    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A3"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A4"), *"2");
    assert_eq!(model.get_text_a1("A5"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A6"), *"#ERROR!");
}

// Excel: MOD is #NUM! once |number/divisor| reaches 1.1259E+12 (probes/mod-quotient-boundary).
#[test]
fn huge_quotient_is_num() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=MOD(1125899999999,1)");
    model.set_a1("A2", "=MOD(1125900000000,1)");
    model.set_a1("A3", "=MOD(-1125900000000,-1)");
    model.set_a1("A4", "=MOD(2251799999999,2)");
    model.set_a1("A5", "=MOD(2251800000000,2)");
    model.set_a1("A6", "=MOD(562950000000.5,0.5)");
    model.set_a1("A7", "=MOD(1,1E-12)");
    model.set_a1("A8", "=MOD(1.2,1E-12)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"0");
    assert_eq!(model.get_text_a1("A2"), *"#NUM!");
    assert_eq!(model.get_text_a1("A3"), *"#NUM!");
    assert_eq!(model.get_text_a1("A4"), *"1");
    assert_eq!(model.get_text_a1("A5"), *"#NUM!");
    assert_eq!(model.get_text_a1("A6"), *"#NUM!");
    assert_ne!(model.get_text_a1("A7"), *"#NUM!");
    assert_eq!(model.get_text_a1("A8"), *"#NUM!");
}

// Excel: an opposite-sign shift that rounds back to the divisor is 0
// (probes/mod-huge-opposite-divisor A6-A7, A47-A48, A99-A100, A113).
#[test]
fn shift_rounding_to_divisor_is_zero() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=MOD(1,-(2^53))");
    model.set_a1("A2", "=MOD(1,-(2^54))");
    model.set_a1("A3", "=MOD(0.5,-(2^52))");
    model.set_a1("A4", "=MOD(0.5,-(2^53))");
    model.set_a1("A5", "=MOD(1,-1E+30)");
    model.set_a1("A6", "=MOD(-1,1E+30)");
    model.set_a1("A7", "=MOD(1.005,-1E+16)");
    model.set_a1("A8", "=MOD(1,2^60)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"-9.0072E+15");
    assert_eq!(model.get_text_a1("A2"), *"0");
    assert_eq!(model.get_text_a1("A3"), *"-4.5036E+15");
    assert_eq!(model.get_text_a1("A4"), *"0");
    assert_eq!(model.get_text_a1("A5"), *"0");
    assert_eq!(model.get_text_a1("A6"), *"0");
    assert_eq!(model.get_text_a1("A7"), *"-1E+16");
    assert_eq!(model.get_text_a1("A8"), *"1");
}

// Excel: MOD lifts element-wise over arrays (probes/geomean-overflow-cut-3 E4).
#[test]
fn mod_lifts_over_arrays() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=SUM(MOD(SEQUENCE(4),2))");
    model.set_a1("A2", "=SUM(MOD(-SEQUENCE(4),3))");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"2");
    // MOD takes the sign of the divisor: -1 -> 2, -2 -> 1, -3 -> 0, -4 -> 2
    assert_eq!(model.get_text_a1("A2"), *"5");
}

#[test]
fn mod_lifts_over_ranges_with_error_and_zero_elements() {
    let mut model = new_empty_model();
    model.set_a1("B1", "5");
    model.set_a1("B2", "=NA()");
    model.set_a1("B3", "7");
    model.set_a1("C1", "2");
    model.set_a1("C2", "2");
    model.set_a1("C3", "0");
    model.set_a1("A1", "=MOD(B1:B3,C1:C3)");
    // a scalar divisor broadcasts and a size-1 column repeats
    model.set_a1("D1", "=MOD(B1:B3,4)");
    // a shorter array leaves #N/A past its end
    model.set_a1("E1", "=MOD(B1:B3,C1:C2)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"1");
    assert_eq!(model.get_text_a1("A2"), *"#N/A");
    assert_eq!(model.get_text_a1("A3"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("D1"), *"1");
    assert_eq!(model.get_text_a1("D2"), *"#N/A");
    assert_eq!(model.get_text_a1("D3"), *"3");
    assert_eq!(model.get_text_a1("E3"), *"#N/A");
}

// Excel: a remainder that passes below the smallest normal double is #NUM!, except that with a
// divisor of exactly ±2^-1022 a subnormal remainder and shift give `d - fmod`
// (probes/t2714-mod-near-smallest-normal, -b, -c; numbers.md "MOD near the smallest normal").
#[test]
fn mod_near_the_smallest_normal() {
    let mut model = new_empty_model();
    // calipers formulas_math_basic I88 and H89
    model.set_a1("A1", "=MOD(10^(-307),-10^(-307))");
    model.set_a1("A2", "=MOD(-10^(-307),10^(-307))");
    // a subnormal remainder of a same-sign pair
    model.set_a1("A3", "=MOD(1E-300*(1+2^-52),1E-300)");
    // a normal remainder whose shift is subnormal
    model.set_a1("A4", "=MOD(1E-300,-1E-300*(1+2^-52))");
    // both subnormal, |d| = 2^-1022: d - fmod, which is 2^-1022 + 2^-1073
    model.set_a1("A5", "=MOD(-(2^-1021*(1+2^-52)),2^-1022)");
    // both subnormal, |d| above 2^-1022
    model.set_a1("A6", "=MOD(-(2^-1022*(1+4*2^-52)),2^-1022*(1+2^-52))");
    // a normal shift still works
    model.set_a1("A7", "=MOD(1E-290,-1E-290*(1+2^-52))<0");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#NUM!");
    assert_eq!(model.get_text_a1("A2"), *"#NUM!");
    assert_eq!(model.get_text_a1("A3"), *"#NUM!");
    assert_eq!(model.get_text_a1("A4"), *"#NUM!");
    assert_eq!(model.get_text_a1("A6"), *"#NUM!");
    assert_eq!(model.get_text_a1("A7"), *"TRUE");
    match model.get_cell_value_by_ref("Sheet1!A5") {
        Ok(CellValue::Number(n)) => assert_eq!(
            n.to_bits(),
            (f64::MIN_POSITIVE * (1.0 + f64::EPSILON * 2.0)).to_bits()
        ),
        other => panic!("expected number, got {other:?}"),
    }
}
