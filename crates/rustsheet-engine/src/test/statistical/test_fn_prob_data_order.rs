#![allow(clippy::unwrap_used)]
//! PROB's data-argument order and pairing (T-2468; statistics.md "PROB: an error data argument first").

use crate::test::util::new_empty_model;

/// Only the sum of the probabilities is checked, not each one: `PROB({1,2},{1.5,-0.5},1)` is 1.5
/// (probes/t4098-tests-prob-domains A32, A33; accuracy/PROB/s1 A113, A125).
#[test]
fn prob_checks_only_the_sum() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=PROB({1,2},{1.5,-0.5},1)");
    model.set_a1("A2", "=PROB({1,2,3},{1.2,-0.1,-0.1},1)");
    model.set_a1("A3", "=PROB({1,2},{1.5,-0.5},2)");
    model.set_a1("A4", "=PROB({1,2},{1.5,-0.4},2)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"1.5");
    assert_eq!(model.get_text_a1("A2"), *"1.2");
    assert_eq!(model.get_text_a1("A3"), *"-0.5");
    assert_eq!(model.get_text_a1("A4"), *"#NUM!");
}

/// The double sum passes when it is 1 at 15 significant digits: 4.88E-15 above and 4.44E-16 below
/// pass, 5.11E-15 above and 5.55E-16 below are `#NUM!` (probes/t4578-prob-sum-tolerance A5, A11,
/// A17, A22; t4098 A1:A5).
#[test]
fn prob_sum_is_one_at_fifteen_digits() {
    let mut model = new_empty_model();
    for (cell, formula, expected) in [
        ("A1", "=PROB({1,2},{0.5,0.500000000000005},1)", "0.5"),
        ("A2", "=PROB({1,2},{1,5E-15},1)", "#NUM!"),
        ("A3", "=PROB({1,2},{0.5,0.5}-{0,5E-16},1)", "0.5"),
        ("A4", "=PROB({1,2},{1,-5E-16},1)", "#NUM!"),
        ("A5", "=PROB({1,2},{0.5,0.500000000000001},1)", "0.5"),
        ("A6", "=PROB({1,2},{0.5,0.499999999999999},1)", "#NUM!"),
        ("A7", "=PROB({1,2},{0.5,0.50000000000001},1)", "#NUM!"),
        (
            "A8",
            "=PROB({1,2,3,4,5,6,7,8,9,10},{0.1,0.1,0.1,0.1,0.1,0.1,0.1,0.1,0.1,0.1},1)",
            "0.1",
        ),
    ] {
        model.set_a1(cell, formula);
        model.evaluate();
        assert_eq!(model.get_text_a1(cell), *expected, "{formula}");
    }
}

#[test]
fn prob_checks_data_arguments_before_limits() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("A3", "3");
    model.set_a1("B1", "0.5");
    model.set_a1("B2", "0.5");
    model.set_a1("D1", "1");
    model.set_a1("D2", "=NA()");
    model.set_a1("D3", "3");
    model.set_a1("F1", "0.2");
    model.set_a1("F2", "0.3");
    model.set_a1("F3", "0.5");
    model.set_a1("C1", "=PROB(1/0,{0.5;0.5},1)");
    model.set_a1("C2", "=PROB({1;2},NA(),\"a\")");
    model.set_a1("C5", "=PROB(A1:A3,B1:B2,1)");
    model.set_a1("C6", "=PROB(D1:D3,F1:F3,\"a\")");
    model.set_a1("C7", "=PROB(H1,B1,1)");
    model.evaluate();

    assert_eq!(model.get_text_a1("C1"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("C2"), *"#N/A");
    assert_eq!(model.get_text_a1("C5"), *"#N/A");
    assert_eq!(model.get_text_a1("C6"), *"#VALUE!");
    assert_eq!(model.get_text_a1("C7"), *"#VALUE!");
}

#[test]
fn prob_pairs_cells_by_position() {
    let mut model = new_empty_model();
    // x {0; blank; "b"; -0.5; blank; 1E15} against p {0.3; -1; 3.14159; 0.2; blank; 0.5}: only
    // the three positions with two numbers pair, so -1 and 3.14159 never reach the range check
    // (gen-m9/e6cf3b20a014e6a3 A229's PROB(Data!D3:D8,Data!C5:C10,..) is 0).
    for (cell, value) in [("A1", "0"), ("A3", "b"), ("A4", "-0.5"), ("A6", "1E+15")] {
        model.set_a1(cell, value);
    }
    for (cell, value) in [
        ("B1", "0.3"),
        ("B2", "-1"),
        ("B3", "3.14159"),
        ("B4", "0.2"),
        ("B6", "0.5"),
    ] {
        model.set_a1(cell, value);
    }
    model.set_a1("G1", "TRUE");
    model.set_a1("H1", "5");
    model.set_a1("C1", "=PROB(A1:A6,B1:B6,0.3,0.3)");
    model.set_a1("C2", "=PROB(A1:A6,B1:B6,-1,1)");
    // Counts are cells: one number and one blank against one number is unequal.
    model.set_a1("C3", "=PROB(H1:H2,0.5,1)");
    // A single value on each side with no pair of numbers is #DIV/0!; a pair is checked as usual.
    model.set_a1("C4", "=PROB(H1,G1,1)");
    // A bare typed boolean is inferred from a TRANSPOSE'd one (5cebac1f02bf30f2 A372); unprobed.
    model.set_a1("C5", "=PROB(-6,1=1,0,3)");
    model.set_a1("C6", "=PROB(3,0.1,-1,-1)");
    model.set_a1("C7", "=PROB(H1,1,5)");
    model.evaluate();

    assert_eq!(model.get_text_a1("C1"), *"0");
    assert_eq!(model.get_text_a1("C2"), *"0.5");
    assert_eq!(model.get_text_a1("C3"), *"#N/A");
    assert_eq!(model.get_text_a1("C4"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("C5"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("C6"), *"#NUM!");
    assert_eq!(model.get_text_a1("C7"), *"1");
}
