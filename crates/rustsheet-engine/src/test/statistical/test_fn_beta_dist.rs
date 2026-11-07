#![allow(clippy::unwrap_used)]

use crate::test::util::{new_empty_model, round_trip_value, within_fifteen_digits};

#[test]
fn empty_argument_cumulative() {
    let mut model = new_empty_model();

    model._set("A1", "=BETA.DIST(0.234, 2, 2.5, , 0.15, 1.2)");
    model._set("A2", "=BETA.DIST(0.234, 2, 2.5, C15 , 0.15, 1.2)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"0.588288667");
    assert_eq!(model._get_text("A2"), *"0.588288667");
}

#[test]
fn arguments() {
    let mut model = new_empty_model();

    model._set("A1", "=BETA.DIST()");
    model._set("A2", "=BETA.DIST(3)");
    model._set("A3", "=BETA.DIST(3, 2)");
    model._set("A4", "=BETA.DIST(3, 2, 3)");
    model._set("A5", "=BETA.DIST(3, 2, 3, 1)");
    model._set("A6", "=BETA.DIST(3, 2, 3, 1, 1)");
    model._set("A7", "=BETA.DIST(3, 2, 3, 1, 1, 10)");
    model._set("A8", "=BETA.DIST(3, 2, 3, 1, 1, 10, 1)");

    model._set("A9", "=BETA.DIST(0.23, 2, 3, 1)"); // Missing interval parameters
    model._set("A10", "=BETA.DIST(0.23, 2, 3, 1, 0)"); // Missing interval end
    model._set("A11", "=BETA.DIST(0.23, 2, 3, 1, , 1)"); // Empty interval start

    model._set("B1", "=BETA.INV()");
    model._set("B2", "=BETA.INV(0.5)");
    model._set("B3", "=BETA.INV(0.5, 2)");
    model._set("B4", "=BETA.INV(0.5, 2, 2)"); // Missing interval parameters
    model._set("B5", "=BETA.INV(0.5, 2, 2, 0)"); // Missing interval end
    model._set("B6", "=BETA.INV(0.5, 2, 2, 0, 1)");
    model._set("B7", "=BETA.INV(0.5, 2, 2, 0, 1, 10)");

    model._set("B8", "=BETA.INV(0.5, 2, 2, , 1)"); // Empty interval start

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"#ERROR!");
    assert_eq!(model._get_text("A5"), *"#NUM!");
    assert_eq!(model._get_text("A6"), *"#NUM!");
    assert_eq!(model._get_text("A7"), *"0.215820759");
    assert_eq!(model._get_text("A8"), *"#ERROR!");

    assert_eq!(model._get_text("A9"), *"0.22845923");
    assert_eq!(model._get_text("A10"), *"0.22845923");
    assert_eq!(model._get_text("A11"), *"0.22845923");

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"#ERROR!");
    assert_eq!(model._get_text("B4"), *"0.5");
    assert_eq!(model._get_text("B5"), *"0.5");
    assert_eq!(model._get_text("B6"), *"0.5");
    assert_eq!(model._get_text("B7"), *"#ERROR!");
}

// T-1543: `statrs`' Beta inverse_cdf panics from p about 1E-200 and does not finish for
// BETA.INV(1E-100,1000,1); the quantile is found by bounded bisection on the cdf.
#[test]
fn test_fn_beta_inv_tiny_probability_is_bounded() {
    let mut model = new_empty_model();
    model._set("A1", "=BETA.INV(1E-300,5,2)");
    model._set("A2", "=BETA.INV(1E-200,1,1)");
    model._set("A3", "=BETA.INV(1E-100,1000,1)");
    model._set("A4", "=BETA.INV(1E-300,1E5,1E5)");
    model._set("A5", "=BETA.INV(1E-100,0.5,100)");
    model.evaluate();
    // Excel (probes-numeric t1543-inverse-cdf-tiny-p) agrees to 6 digits at A2-A5; at A1 it gives
    // 6.988271195E-61 (an Excel quirk, see statistics.md).
    assert_eq!(model._get_text("A1"), "6.98827E-61");
    assert_eq!(model._get_text("A2"), "1E-200");
    assert_eq!(model._get_text("A3"), "0.794328235");
    assert_eq!(model._get_text("A4"), "0.458650995");
    assert_eq!(model._get_text("A5"), "7.8736E-203");
}

// T-3664: at a shape of exactly 1 and p of 1E-15 or more the quantile is the closed form, not the
// crossing of BETA.DIST's cdf, which is flat over tens of doubles there (T-3007). Excel values from
// probes-numeric/m9-beta-inv A69 and A149 (the same points as t1745-beta-inv-roundtrip J67, J144);
// below 1E-15 the search stays (t1543-inverse-cdf-tiny-p A73 `BETA.INV(1E-300,1,1)`, Excel
// 1.0000000000000237E-300, not p).
#[test]
fn test_fn_beta_inv_shape_one_closed_form() {
    let mut model = new_empty_model();
    model._set("A1", "=BETA.INV(0.999,1,100)");
    model._set("A2", "=BETA.INV(0.001,100,1)");
    model._set("A3", "=BETA.INV(0.25,1,1)");
    model._set("A4", "=BETA.INV(1E-300,1,1)");
    model.evaluate();
    let number = |cell: &str| round_trip_value(&model, cell).parse::<f64>().unwrap();
    for (cell, excel) in [
        ("A1", 0.066_745_699_203_008_99),
        ("A2", 0.933_254_300_796_991),
    ] {
        let value = number(cell);
        assert!(
            within_fifteen_digits(excel, value),
            "{cell}: {value} against Excel {excel}"
        );
    }
    assert_eq!(number("A3").to_bits(), 0.25_f64.to_bits());
    assert_ne!(number("A4").to_bits(), 1e-300_f64.to_bits());
}

// T-3764: at b = 1 and x > 0.9 (bratio's a0 = 1 with y0 > 0.9) Excel's w = 1 - x^a is minus its
// expm1 below |a·ln x| = 1 and 1 - EXP(a·ln x) from there on; Kahan's expm1 at |z| >= 1 put these
// one 2^-53 step of w off. Excel values (excel-graph) from probes-numeric t3568-r5-bup-tune B856,
// t3568-r5-bup-hold B816 and B1610, and t3568-r5-bup-fresh B744 (x^a below 2^-53).
#[test]
fn test_fn_beta_dist_shape_one_complement_past_unit_exponent() {
    let mut model = new_empty_model();
    model._set("A1", "=BETA.DIST(0.9094554,241.0,1.0,TRUE)");
    model._set("A2", "=BETA.DIST(0.912187876,136.0,1.0,TRUE)");
    model._set("A3", "=BETA.DIST(0.9965,1414.0,1.0,TRUE)");
    model._set("A4", "=BETA.DIST(0.95591070645675,826.0,1.0,TRUE)");
    model.evaluate();
    let number = |cell: &str| round_trip_value(&model, cell).parse::<f64>().unwrap();
    for (cell, excel) in [
        ("A1", 1.164_981_444_645_718_5e-10_f64),
        ("A2", 3.727_899_646_532_861_8e-6),
        ("A3", 0.007_029_209_602_111_175),
        ("A4", 1.110_223_024_625_156_5e-16),
    ] {
        assert_eq!(number(cell).to_bits(), excel.to_bits(), "{cell}");
    }
}

// T-3764: BETA.DIST's cdf is Excel's exact 0 where L140 bup+bgrat's prefactor underflows (T-4045's
// rule, F's until now), though the true tail is a normal double. Excel values (excel-graph) from
// probes-numeric t3568-r5-bup-fresh A291 and t3568-r5-bup-hold A20 and A686; BETADIST shares it.
#[test]
fn test_fn_beta_dist_l140_prefactor_underflow_is_zero() {
    let mut model = new_empty_model();
    model._set("A1", "=BETA.DIST(0.7115070068952,2160.0,30.5,TRUE)");
    model._set("A2", "=BETA.DIST(0.749267,2496.51,16.8,TRUE)");
    model._set("A3", "=BETA.DIST(0.74342,2420.0,33.6362,TRUE)");
    model._set("A4", "=BETADIST(0.74342,2420.0,33.6362)");
    model.evaluate();
    for cell in ["A1", "A2", "A3", "A4"] {
        assert_eq!(round_trip_value(&model, cell), "0.0", "{cell}");
    }
}

// T-3351: BETA.INV at p above 1/2 whose quantile rounds to 1 is AS 109's flip 1 - y, y the lower
// quantile of Beta(beta, alpha) at 1 - p: exactly 1 (or 1 - 2^-53 where y is above 2^-54) for a
// normal y, #NUM! where y underflows. A quantile that rounds to 1 at p of 1/2 or less, or underflows
// at p above 1/2, stays #NUM!, which Excel is at some such points (accuracy/BETA.INV/s1 A89, A94).
// Excel values (excel-graph) from probes/t3351-beta-inv-p-near-1 and accuracy/BETA.INV/s1.
#[test]
fn test_fn_beta_inv_upper_quantile_rounding_to_one() {
    let cases = [
        ("=BETA.INV(1-1E-8,2,0.1)", "1"),    // A102, y = 3.9E-81
        ("=BETA.INV(1-1E-6,0.5,0.3)", "1"),  // A46
        ("=BETA.INV(1-1E-16,2,1)", "1"),     // A126, y = 5E-17
        ("=BETAINV(1-1E-12,2,0.1)", "1"),    // A195
        ("=BETA.INV(1-7E-10,10,0.03)", "1"), // C198, y = 4.1E-307
        // Hand-derived from A157 (Excel 1) mapped onto [3, 5]; no oracle cell yet, needs oracle
        // confirmation (T-4865).
        ("=BETA.INV(1-1E-9,10,0.03,3,5)", "5"),
        ("=BETA.INV(1-1E-15,10,1)", "0.9999999999999999"), // A188, y = 1E-16
        (
            "=BETA.INV(0.999999999999999,6.87566761020923,0.991674132119787)",
            "0.9999999999999999",
        ), // s1 A123
        ("=BETA.INV(1-1E-8,2,0.01)", "#NUM!"),             // A84, y = 3.7E-801
        ("=BETA.INV(1-1E-10,2,0.03)", "#NUM!"),            // A95, y = 1.7E-334
        ("=BETA.INV(0.51,2,1E-3)", "#NUM!"),               // D203, y = 1E-310
        (
            "=BETA.INV(0.120463618823915,4.85116218981846,3.81506961561941E-15)",
            "#NUM!",
        ), // s1 A89
        (
            "=BETA.INV(0.540825862774324,2.11840621275937E-15,0.719272803854574)",
            "#NUM!",
        ), // s1 A94
        ("=BETA.INV(1-1E-8,0.5,1)", "0.99999998"),         // A57: away from 1, unchanged
    ];
    let mut model = new_empty_model();
    for (row, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, excel)) in cases.iter().enumerate() {
        let cell = format!("A{}", row + 1);
        let value = round_trip_value(&model, &cell);
        match excel.parse::<f64>() {
            Ok(x) => assert_eq!(value.parse::<f64>().ok(), Some(x), "{formula}: {value}"),
            Err(_) => assert_eq!(model._get_text(&cell), *excel, "{formula}"),
        }
    }
}
