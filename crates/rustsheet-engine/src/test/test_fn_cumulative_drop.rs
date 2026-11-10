#![allow(
    clippy::panic,
    reason = "a test fails by panicking on a non-number result"
)]

//! CUMIPMT and CUMPRINC from the grouped balance drop (T-4975 item 13, T-3731;
//! docs/excel-semantics/financial.md "CUMIPMT and CUMPRINC over many periods"). Each value is
//! Excel's, to the bit.

use crate::cell::CellValue;
use crate::test::util::new_empty_model;

fn assert_bits(cases: &[(&str, f64)]) {
    let mut model = new_empty_model();
    for (i, (formula, _)) in cases.iter().enumerate() {
        model.set_a1(&format!("A{}", i + 1), formula);
    }
    model.evaluate();
    for (i, (formula, expected)) in cases.iter().enumerate() {
        match model.get_cell_value_by_ref(&format!("Sheet1!A{}", i + 1)) {
            Ok(CellValue::Number(n)) => {
                assert_eq!(
                    n.to_bits(),
                    expected.to_bits(),
                    "{formula}: {n:e} vs {expected:e}"
                );
            }
            other => panic!("{formula}: expected a number, got {other:?}"),
        }
    }
}

#[test]
fn cumprinc_is_the_grouped_balance_drop() {
    // ironcalc-calc FINANCIAL/CUMPRINC_CUMIPMT.xlsx Sheet1!H2, H3, H4, H5, H7, H12, H16, H17,
    // H29, H31 (Excel for Mac's cached values); probes/fin-cumprinc-form-boundaries F2 (twice-rounded
    // products: an ulp off with single roundings)
    assert_bits(&[
        ("=CUMPRINC(0.01,101,100000,1,12,1)", -8_240.638_550_875_348),
        ("=CUMPRINC(0.0083,36,8000,1,10,1)", -2_038.518_055_313_885),
        ("=CUMPRINC(0.1,10,345,2,2,1)", -21.647_161_234_466_502),
        ("=CUMPRINC(0.3,36,8000,3,30,1)", -1_274.208_907_309_859),
        (
            "=CUMPRINC(0.7,36,8000,2,10,1)",
            -0.004_756_542_342_248_707_6,
        ),
        (
            "=CUMPRINC(0.8,36,18000,2.7,9.667,1)",
            -0.001_260_346_687_754_882_4,
        ),
        ("=CUMPRINC(0.1,30,8000,1.9,10,1)", -660.424_090_214_400_5),
        ("=CUMPRINC(0.1,30.34,8000,3,10,1)", -591.136_668_916_254),
        ("=CUMPRINC(0.8,12,8000,5,5,1)", -32.292_599_595_232_54),
        ("=CUMPRINC(0.0083,36,8000,3,10,0)", -1_604.273_009_770_884),
        ("=CUMPRINC(0.0083,36,8000,3,10,1)", -1_591.067_152_405_915),
    ]);
}

#[test]
fn cumipmt_is_the_payments_less_the_grouped_drop() {
    // CUMPRINC_CUMIPMT.xlsx Sheet1!G2, G3, G29 (G29 was visibly off), G31; and
    // probes/fin-cumipmt-tiny-rate-structure A10, H2, O10 (T-3731's visibly-off cells)
    assert_bits(&[
        ("=CUMIPMT(0.0083,36,8000,1,10,1)", -520.118_119_102_138_5),
        ("=CUMIPMT(0.1,10,345,2,2,1)", -29.395_712_615_048_502),
        ("=CUMIPMT(0.0083,36,8000,3,10,0)", -459.625_273_960_056_8),
        ("=CUMIPMT(0.0083,36,8000,3,10,1)", -455.841_787_126_903_7),
        (
            "=CUMIPMT(2E-5,100,100000,12,12,1)",
            -1.780_160_143_925_854_8,
        ),
        ("=CUMIPMT(1E-4,60,100000,4,4,0)", -9.501_423_646_180_91),
        ("=CUMIPMT(0.0083,36,8000,1,11,1)", -569.598_419_243_033_5),
    ]);
}
