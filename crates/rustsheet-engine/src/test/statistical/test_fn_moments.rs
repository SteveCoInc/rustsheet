#![allow(clippy::unwrap_used)]

//! SKEW, SKEW.P and KURT bit for bit: every operation of the standardized moment rounds twice and
//! the fourth power is (z·z)·(z·z) (docs/excel-semantics/statistics.md "Standardized moments use
//! stored x87 operations (T-4328)").

use crate::cell::CellValue;
use crate::test::util::new_empty_model;

fn number(formula: &str) -> f64 {
    let mut model = new_empty_model();
    model._set("A1", formula);
    model.evaluate();
    match model.get_cell_value_by_ref("Sheet1!A1").unwrap() {
        CellValue::Number(n) => n,
        other => panic!("{formula}: {other:?}"),
    }
}

#[test]
fn skew_takes_the_twice_rounded_root() {
    // gen-m9/d208bbafc3bc1997 A262, probes/t2475-stat-subexpressions P!A520 (excel-graph).
    assert_eq!(number("=SKEW({1E-15,1,2})"), 2.498_001_805_406_602_2E-15);
}

#[test]
fn kurt_squares_the_square() {
    // gen-m9/27601819397dd56e A200 (excel-graph).
    assert_eq!(
        number("=KURT({0.1,42,123456789.123,0.1})"),
        3.999_999_999_997_953_6
    );
}

#[test]
fn kurt_of_four_mixed_values() {
    // gen-m9/7bd219ec0c0c5e22 A279 `=KURT(Data!A1:A12)`, whose numbers these are (excel-graph).
    assert_eq!(
        number("=KURT({0.2,7,3.14159,-2.5})"),
        -0.762_827_238_580_870_3
    );
}

#[test]
fn kurt_of_nine_values() {
    // gen-m9/c2974c061384fdd9 A55 `=KURT(Data!A6:A6,Data!D1:D11)`, whose numbers these are (excel-graph).
    assert_eq!(
        number("=KURT(1,0.1,100,100,0.5,0.3,0.3,1E-15,0.5)"),
        0.734_384_233_823_282
    );
}

#[test]
fn kurt_of_one_to_ten() {
    // gen-m9/a013a45a8fbcab4f A2, a54c738c697617c3 A41 (excel-graph; T-4328 context).
    assert_eq!(
        number("=KURT({1,2,3,4,5,6,7,8,9,10})"),
        -1.200_000_000_000_000_2
    );
}

#[test]
fn zero_and_overflowing_variance_stay_div_zero() {
    let mut model = new_empty_model();
    model._set("A1", "=KURT({1,1,1,1})");
    model._set("A2", "=SKEW({1E+200,1E+200,-1E+200})");
    model._set("A3", "=SKEW.P({1E+200,1E+200,-1E+200})");
    model._set("A4", "=SKEW(1,2)");
    model.evaluate();
    for cell in ["A1", "A2", "A3", "A4"] {
        assert_eq!(model._get_text(cell), *"#DIV/0!", "{cell}");
    }
}
