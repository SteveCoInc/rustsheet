#![allow(
    clippy::panic,
    reason = "a test fails by panicking on a non-number result"
)]

//! PV, FV and NPER as fitted against the excel-graph sweeps (T-0901, fit lab `annuity-*`,
//! docs/excel-semantics/financial.md "Annuity evaluation order"). Each value below is the oracle's,
//! and each one the earlier `powf` formulations missed by an ulp or more.

use crate::cell::CellValue;
use crate::test::util::new_empty_model;

fn results(formulas: &[&str]) -> Vec<u64> {
    let mut model = new_empty_model();
    for (i, formula) in formulas.iter().enumerate() {
        model._set(&format!("A{}", i + 1), formula);
    }
    model.evaluate();
    (1..=formulas.len())
        .map(
            |row| match model.get_cell_value_by_ref(&format!("Sheet1!A{row}")) {
                Ok(CellValue::Number(n)) => n.to_bits(),
                other => panic!("A{row}: expected a number, got {other:?}"),
            },
        )
        .collect()
}

#[test]
fn pv_and_fv_use_the_fitted_annuity_factor() {
    // probes-numeric/m9-fin-pv-fv A1, A10, A100, A103
    let got = results(&[
        "=PV(0.2,1,-100)",
        "=PV(0.2,3,-100,1000,1)",
        "=FV(0.1,36,-100,-1000,1)",
        "=FV(0.1,60,-100)",
    ]);
    let expected = [
        83.333_333_333_333_31_f64,
        -325.925_925_925_926,
        63_816.629_119_028_57,
        303_481.639_541_419_34,
    ];
    assert_eq!(got, expected.map(f64::to_bits));
}

#[test]
fn nper_divides_plain_logarithms_with_excels_division() {
    // probes-numeric/m9-fin-nper-npv A9: the final `/` rounds twice
    let got = results(&["=NPER(0.2,-1000,-1000,20000)"]);
    assert_eq!(got, [7.827_469_119_589_407_5_f64.to_bits()]);
}

#[test]
fn fv_and_pv_round_each_operation_twice() {
    // probes/t3424-rate-noise-trace A132 and probes/t4146-annuity-twice-rounded A1 to A4: a
    // cancelling sum where plain doubles miss Excel by ulps (financial.md "FV and PV round each
    // operation twice", T-4146)
    let got = results(&[
        "=FV((2.88607473601363e-05+1.01643953670516e-20),321,-2890484105.96678,934408648396.347,1)",
        "=FV(5.76548179199485e-07,298,6359423665.24359,-1894944918694.86,0)",
        "=PV(-3.30989944207756e-05,302,9.02116389318385,-2707.16979461553,1)",
        "=FV(-1.86397153259747e-05,13,225.530751441347,-2932.5765394558,1)",
        "=PV(3.3027497091378e-06,47,4639.07463263248,-218078.350264974,0)",
    ]);
    let expected = [
        -10_935_378_180.042_114_f64,
        3_928.559_082_031_25,
        -3.641_724_285_483_088_7,
        0.348_759_002_021_324_7,
        25.275_031_138_019_653,
    ];
    assert_eq!(got, expected.map(f64::to_bits));
}

#[test]
fn nper_below_a_rate_of_1e_14_is_the_math() {
    // probes-numeric/t3367-nper-tiny-rate A22, A39, A47, A62, A97: Excel is 46%, 35%, 37%, 16% and
    // 0.9% off there, and the engine returns the 50-digit math (decision 0188 R3, financial.md
    // "NPER at a tiny positive rate", MATH-OVER-EXCEL).
    let got = results(&[
        "=NPER(1.2E-16,-100,10000,0,0)",
        "=NPER(3E-16,370.793811118894,-6303.49478902119,0,0)",
        "=NPER(3.04353262019695E-16,-1,0,1000,1)",
        "=NPER(7.7E-16,0,100,-200,0)",
        "=NPER(9.9E-15,0,100,-200,0)",
    ]);
    let math = [
        100.000_000_000_000_61_f64,
        17.000_000_000_000_025,
        999.999_999_999_847_7,
        900_191_143_584_344.9,
        70_014_866_723_227.14,
    ];
    for (bits, want) in got.into_iter().zip(math) {
        let value = f64::from_bits(bits);
        assert!(((value - want) / want).abs() < 1e-14, "{value} vs {want}");
    }
}

#[test]
fn nper_from_a_rate_of_1e_14_keeps_excels_value() {
    // probes-numeric/t3367-nper-tiny-rate A99 and A120: outside the R3 region the engine is Excel's
    // value, though Excel is 8.9E-4 and 8.0E-4 off the math there (an R3 candidate, not ruled).
    let got = results(&["=NPER(1E-14,-100,10000,0,0)", "=NPER(1E-13,-100,10000,0,0)"]);
    assert_eq!(
        got,
        [100.088_888_888_839_35_f64, 100.079_999_999_504_6].map(f64::to_bits)
    );
}
