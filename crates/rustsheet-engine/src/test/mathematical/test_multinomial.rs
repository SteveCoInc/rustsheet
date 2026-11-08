// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::unreadable_literal)]
// LINT-DEBT-END
#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_multinomial_basic() {
    let mut model = new_empty_model();
    // MULTINOMIAL(2,3,4) = 9! / (2! * 3! * 4!) = 362880 / (2 * 6 * 24) = 362880 / 288 = 1260
    model.set_a1("A1", "=MULTINOMIAL(2,3,4)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "1260");
}

#[test]
fn test_multinomial_single() {
    let mut model = new_empty_model();
    // MULTINOMIAL(5) = 5!/5! = 1
    model.set_a1("A1", "=MULTINOMIAL(5)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "1");
}

#[test]
fn test_multinomial_zeros() {
    let mut model = new_empty_model();
    // MULTINOMIAL(0,0,1) = 1!/1! = 1
    model.set_a1("A1", "=MULTINOMIAL(0,0,1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "1");
}

#[test]
fn test_multinomial_range() {
    let mut model = new_empty_model();
    model.set_a1("A1", "2");
    model.set_a1("A2", "3");
    model.set_a1("A3", "4");
    model.set_a1("B1", "=MULTINOMIAL(A1:A3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("B1"), "1260");
}

#[test]
fn test_multinomial_negative_error() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=MULTINOMIAL(2,-1)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#NUM!");
}

#[test]
fn test_multinomial_two_two() {
    let mut model = new_empty_model();
    // MULTINOMIAL(2,2) = 4! / (2! * 2!) = 24 / 4 = 6
    model.set_a1("A1", "=MULTINOMIAL(2,2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "6");
}

#[test]
fn test_multinomial_overflow_num_error() {
    // Inputs large enough that (sum)! overflows f64 → must return #NUM!, not Inf.
    let mut model = new_empty_model();
    model.set_a1("A1", "=MULTINOMIAL(100000,100000)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "#NUM!");
}

#[test]
fn multinomial_past_170_is_num_at_once() {
    // probes/multinomial-large-arguments (excel-graph, 2026-10-09, T-1197): `#NUM!` once the
    // arguments add up to more than 170. Huge arguments used to loop once per unit (directive 5),
    // and `2^64` and above saturate instead of overflowing the sum.
    let mut model = new_empty_model();
    let cases = [
        ("A1", "=MULTINOMIAL(171)"),
        ("A2", "=MULTINOMIAL(170,1)"),
        ("A3", "=MULTINOMIAL(86,86)"),
        ("A4", "=MULTINOMIAL(1E+15)"),
        ("A5", "=MULTINOMIAL(1E+300)"),
        ("A6", "=MULTINOMIAL(2^64,2^64)"),
        ("A7", "=MULTINOMIAL(3,1E+15)"),
    ];
    for (cell, formula) in cases {
        model.set_a1(cell, formula);
    }
    model.set_a1("B1", "=MULTINOMIAL(85,85)");
    model.set_a1("B2", "=MULTINOMIAL(169,1)");
    let start = std::time::Instant::now();
    model.evaluate();
    assert!(start.elapsed() < std::time::Duration::from_secs(5));
    for (cell, formula) in cases {
        assert_eq!(model.get_text_a1(cell), "#NUM!", "{formula}");
    }
    assert_eq!(model.get_text_a1("B2"), "170");
    assert!(
        model.get_text_a1("B1").starts_with("9.1448"),
        "{}",
        model.get_text_a1("B1")
    );
}

// gen-m6/2c1a6e93a1974f48 Calc!A35: -2.5 first, a boolean later in the range is still `#VALUE!`.
#[test]
fn range_boolean_is_value_even_after_a_negative() {
    let mut model = new_empty_model();
    model.set_a1("A1", "-2.5");
    model.set_a1("A2", "FALSE");
    model.set_a1("B1", "-2.5");
    model.set_a1("B2", "3");
    model.set_a1("C1", "=MULTINOMIAL(A1:A2)");
    model.set_a1("C2", "=MULTINOMIAL(B1:B2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("C1"), *"#VALUE!");
    assert_eq!(model.get_text_a1("C2"), *"#NUM!");
}

#[test]
fn multinomial_has_excels_rounding_and_reads_numeric_text() {
    // T-1190: Excel's inexact values, argument order included (probes/multinomial-algorithm-sweep,
    // multinomial-large-arguments, math-argument-modes; excel-graph), and numeric text typed or in
    // a range cell as its number (math-argument-modes C9, C47, C52).
    use crate::cell::CellValue;
    let mut model = new_empty_model();
    let cases = [
        ("A1", "=MULTINOMIAL(2,3)", 9.999999999999998),
        ("A2", "=MULTINOMIAL(170)", 1.0000000000001974),
        ("A3", "=MULTINOMIAL(20,150)", 5.221123511164219e25),
        ("A4", "=MULTINOMIAL(150,20)", 5.221123511166111e25),
        ("A5", "=MULTINOMIAL(10,20,30)", 3.553261127085186e24),
        ("A6", "=MULTINOMIAL(30,20,10)", 3.5532611270850594e24),
        ("A7", "=MULTINOMIAL(1,2)", 3.0000000000000004),
        ("A8", "=MULTINOMIAL(\"2\",3)", 9.999999999999998),
        ("A9", "=MULTINOMIAL(\"4\",6)", 209.99999999999977),
        ("A10", "=MULTINOMIAL(D1:D3)", 13859.99999999998),
        // T-1600: a lone argument below 1 is `1 / exp(-y)`, not the nearer `exp(y)`
        // (probes/multinomial-single-arguments A5, A30, A140).
        ("A11", "=MULTINOMIAL(5)", 0.9999999999999996),
        ("A12", "=MULTINOMIAL(30)", 0.9999999999999953),
        ("A13", "=MULTINOMIAL(140)", 0.9999999999999674),
    ];
    model.set_a1("D1", "4");
    model.set_a1("D2", "'2");
    model.set_a1("D3", "6");
    for (cell, formula, _) in cases {
        model.set_a1(cell, formula);
    }
    model.set_a1("B1", "=MULTINOMIAL(\"x\",3)");
    model.set_a1("B2", "=MULTINOMIAL(TRUE,3)");
    model.evaluate();
    for (row, (cell, formula, expected)) in (1..).zip(cases) {
        assert_eq!(
            model.get_cell_value_by_index(0, row, 1).unwrap(),
            CellValue::Number(expected),
            "{cell}: {formula}"
        );
    }
    assert_eq!(model.get_text_a1("B1"), "#VALUE!");
    assert_eq!(model.get_text_a1("B2"), "#VALUE!");
}

// T-2294: every argument a lone blank cell is #VALUE!; a blank beside anything else, or in a
// multi-cell range, reads as 0 (probes/t2275-address-bounds column J).
#[test]
fn test_gcd_lcm_multinomial_lone_blank_is_value_error() {
    let mut model = new_empty_model();
    model.set_a1("B1", "=MULTINOMIAL(Z200)");
    model.set_a1("B2", "=MULTINOMIAL(Z200:Z200)");
    model.set_a1("B3", "=MULTINOMIAL(Z200,Z201)");
    model.set_a1("B4", "=GCD(Z200)");
    model.set_a1("B5", "=GCD(Z200:Z200)");
    model.set_a1("B6", "=LCM(Z200)");
    model.set_a1("B7", "=LCM(Z200:Z200)");
    model.set_a1("B8", "=MULTINOMIAL(IF(TRUE,Z200))");
    model.evaluate();
    for row in 1..=8 {
        assert_eq!(model.get_text_a1(&format!("B{row}")), "#VALUE!", "B{row}");
    }
}

#[test]
fn test_gcd_lcm_multinomial_blank_beside_others_is_zero() {
    let mut model = new_empty_model();
    model.set_a1("Z210", "0");
    model.set_a1("B1", "=MULTINOMIAL(Z200,2)");
    model.set_a1("B2", "=MULTINOMIAL(2,Z200)");
    model.set_a1("B3", "=MULTINOMIAL(Z200:Z201)");
    model.set_a1("B4", "=MULTINOMIAL(Z200:Z201,2)");
    model.set_a1("B5", "=GCD(Z200,4)");
    model.set_a1("B6", "=LCM(Z200,4)");
    model.set_a1("B7", "=MULTINOMIAL(Z210)");
    model.evaluate();
    let expected = ["1", "1", "1", "1", "4", "4", "1"];
    for (i, want) in expected.iter().enumerate() {
        assert_eq!(
            model.get_text_a1(&format!("B{}", i + 1)),
            *want,
            "B{}",
            i + 1
        );
    }
}
