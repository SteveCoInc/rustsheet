#![allow(
    clippy::panic,
    reason = "a test fails by panicking on a non-number result"
)]

//! RATE's restart and NPER's tiny-rate form, from the audit of
//! docs/research/excel-parity-findings-2026-11-07.md item 6 (T-4975 audit 6).

use crate::cell::CellValue;
use crate::test::util::new_empty_model;

fn number(formula: &str) -> f64 {
    let mut model = new_empty_model();
    model.set_a1("A1", formula);
    model.evaluate();
    match model.get_cell_value_by_ref("Sheet1!A1") {
        Ok(CellValue::Number(n)) => n,
        other => panic!("{formula}: expected a number, got {other:?}"),
    }
}

#[test]
fn a_restarted_rate_run_without_a_root_falls_back_to_the_hold() {
    // probes/t3906-rate-restart-oos Sheet1!A194: Excel 0.012686683557386938; the restarted run
    // met a second flat secant and was #NUM!. The hold is Excel's type, 6E-10 relative off.
    let excel = 0.012_686_683_557_386_938_f64;
    let ours = number(
        "=RATE(1,-57090142708.1999,57155400514.2199,-66085711.1548055,1,-0.0413434246245947)",
    );
    assert!(((ours - excel) / excel).abs() < 1e-9, "{ours:e}");
}

#[test]
fn nper_at_a_tiny_rate_does_not_overflow_fv_plus_pv() {
    // fv + pv is 1.8E+308, past the double range, while every product with the rate is finite:
    // the period count is log1p(18/91)/log1p(1E-15), the excess -(fv+pv)·rate/(pmt + pv·rate)
    // being 1.8E+293/9.1E+293.
    let ours = number("=NPER(1E-15,-1E+294,9E+307,9E+307)");
    let math = (18.0_f64 / 91.0).ln_1p() / 1e-15_f64.ln_1p();
    assert!(((ours - math) / math).abs() < 1e-13, "{ours:e} vs {math:e}");
}

#[test]
fn a_restart_far_from_its_hold_stays_an_error() {
    // probes/t3906-rate-restart-oos Sheet1!A26: the flat secant comes at step 3 on iterates 0.078
    // apart, and Excel is #NUM!: there the hold is no converged point to fall back to.
    let mut model = new_empty_model();
    model.set_a1(
        "A1",
        "=RATE(1,65619121.9595683,-65619121.9595674,0,1,0.0414411258870795)",
    );
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#NUM!");
}
