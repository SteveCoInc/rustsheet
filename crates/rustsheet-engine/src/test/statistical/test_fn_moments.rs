#![allow(clippy::unwrap_used)]

//! SKEW, SKEW.P and KURT bit for bit: every operation of the standardized moment rounds twice and
//! the fourth power is (z·z)·(z·z) (docs/excel-semantics/statistics.md "Standardized moments use
//! stored x87 operations (T-4328)").

use crate::cell::CellValue;
use crate::test::util::new_empty_model;

/// The value of `formula` as bits: the tests compare Excel's double exactly.
fn bits(formula: &str) -> u64 {
    number(formula).to_bits()
}

fn number(formula: &str) -> f64 {
    let mut model = new_empty_model();
    model.set_a1("A1", formula);
    model.evaluate();
    match model.get_cell_value_by_ref("Sheet1!A1").unwrap() {
        CellValue::Number(n) => n,
        other => panic!("{formula}: {other:?}"),
    }
}

#[test]
fn skew_takes_the_twice_rounded_root() {
    // gen-m9/d208bbafc3bc1997 A262, probes/t2475-stat-subexpressions P!A520 (excel-graph).
    assert_eq!(
        bits("=SKEW({1E-15,1,2})"),
        f64::to_bits(2.498_001_805_406_602_2E-15)
    );
}

#[test]
fn kurt_squares_the_square() {
    // gen-m9/27601819397dd56e A200 (excel-graph).
    assert_eq!(
        bits("=KURT({0.1,42,123456789.123,0.1})"),
        f64::to_bits(3.999_999_999_997_953_6)
    );
}

#[test]
fn kurt_of_four_mixed_values() {
    // gen-m9/7bd219ec0c0c5e22 A279 `=KURT(Data!A1:A12)`, whose numbers these are (excel-graph).
    assert_eq!(
        bits("=KURT({0.2,7,3.14159,-2.5})"),
        f64::to_bits(-0.762_827_238_580_870_3)
    );
}

#[test]
fn kurt_of_nine_values() {
    // gen-m9/c2974c061384fdd9 A55 `=KURT(Data!A6:A6,Data!D1:D11)`, whose numbers these are (excel-graph).
    assert_eq!(
        bits("=KURT(1,0.1,100,100,0.5,0.3,0.3,1E-15,0.5)"),
        f64::to_bits(0.734_384_233_823_282)
    );
}

#[test]
fn kurt_of_one_to_ten() {
    // gen-m9/a013a45a8fbcab4f A2, a54c738c697617c3 A41 (excel-graph; T-4328 context).
    assert_eq!(
        bits("=KURT({1,2,3,4,5,6,7,8,9,10})"),
        f64::to_bits(-1.200_000_000_000_000_2)
    );
}

#[test]
fn kurt_of_five_values_and_of_a_boolean_argument() {
    // gen-m9/27601819397dd56e A108 `=KURT(Data!A11:A12,Data!E2:E5)`, whose numbers these are, and
    // probes/stats-evidence-floor-t1623 (excel-graph).
    assert_eq!(
        bits("=KURT(-0.0001,100,2,0.1,42)"),
        f64::to_bits(1.514_976_771_454_35)
    );
    assert_eq!(
        bits("=KURT(3,5,2,8,9,4,7,1,TRUE)"),
        f64::to_bits(-1.433_183_431_952_663_5)
    );
}

#[test]
fn zero_and_overflowing_variance_stay_div_zero() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=KURT({1,1,1,1})");
    model.set_a1("A2", "=SKEW({1E+200,1E+200,-1E+200})");
    model.set_a1("A3", "=SKEW.P({1E+200,1E+200,-1E+200})");
    model.set_a1("A4", "=SKEW(1,2)");
    model.evaluate();
    for cell in ["A1", "A2", "A3", "A4"] {
        assert_eq!(model.get_text_a1(cell), *"#DIV/0!", "{cell}");
    }
}
