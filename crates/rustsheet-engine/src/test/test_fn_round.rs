#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn fn_round_approximation() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=ROUND(1.05*(0.0284+0.0046)-0.0284,4)");
    model.set_a1("A2", "=ROUNDDOWN(1.05*(0.0284+0.0046)-0.0284,5)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"0.0063");
    assert_eq!(model.get_text_a1("A2"), *"0.00625");
}

#[test]
fn fn_round_to_more_digits_than_a_double_holds() {
    // gen-m2/f7e6d351c8538ef5 A35, a339ea84ef64382e A138, 1b6ceed7489e0800 A337 (excel-graph).
    let mut model = new_empty_model();
    model.set_a1("A1", "=ROUNDDOWN(0.5,123456789.123)");
    model.set_a1("A2", "=ROUNDDOWN(1234.5,100000)");
    model.set_a1("A3", "=ROUNDUP(42,100000)");
    model.set_a1("A4", "=ROUND(1E-15,123456789)");
    model.set_a1("A5", "=ROUNDDOWN(TRUE,-123456789.323)");
    model.set_a1("A6", "=ROUND(12345,-400)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"0.5");
    assert_eq!(model.get_text_a1("A2"), *"1234.5");
    assert_eq!(model.get_text_a1("A3"), *"42");
    assert_eq!(model.get_text_a1("A4"), *"1E-15");
    assert_eq!(model.get_text_a1("A5"), *"0");
    assert_eq!(model.get_text_a1("A6"), *"0");
}

#[test]
fn round_family_lifts_over_array_arguments() {
    // gen-m2/893302d6c5b99e2e Calc!A313: the num_digits error (#DIV/0!) beats the array.
    let mut model = new_empty_model();
    model.set_a1("B1", "=SUM(ROUNDDOWN({1.26,2.58},1))");
    model.set_a1("B2", "=SUM(ROUND({1.26,2.58},{0;1}))");
    model.set_a1("B3", "=SUM(TRUNC({1.26,2.58}))");
    model.set_a1("B4", "=ROUNDDOWN({1.26,2.58},1/0)");
    model.evaluate();
    assert_eq!(model.get_text_a1("B1"), *"3.7");
    assert_eq!(model.get_text_a1("B2"), *"7.9");
    assert_eq!(model.get_text_a1("B3"), *"3");
    assert_eq!(model.get_text_a1("B4"), *"#DIV/0!");
}

#[test]
fn round_of_a_column_against_a_row_is_bounded() {
    // Resource limit (DEVIATION, arrays.md): the lifted result is capped, not allocated. In a
    // value context that is #ERROR!; in a cell the 2^34-cell shape runs past the grid, so it is
    // #SPILL! from the shapes alone (T-0483).
    let mut model = new_empty_model();
    model.set_a1("B2", "=ROUND(A1:A1048576,C1:XFD1)");
    model.set_a1("B3", "=ROWS(ROUND(A1:A1048576,C1:XFD1))");
    model.evaluate();
    assert_eq!(model.get_text_a1("B2"), *"#SPILL!");
    assert_eq!(model.get_text_a1("B3"), *"#ERROR!");
}

// A LAMBDA read as a number by a math function on the lifting driver is `#VALUE!`, as it was
// before they moved onto it (T-1773), and as MEDIAN and AND read one
// (probes/aggregate-picked-direct-a-variants D7-D12, probes/and-or-xor-picked-text).
#[test]
fn a_lambda_number_argument_is_value_error() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=ABS(LAMBDA(x,x))");
    model.set_a1("A2", "=ROUND(LAMBDA(x,x),1)");
    model.set_a1("A3", "=ROUND(1,LAMBDA(x,x))");
    model.set_a1("A4", "=CEILING.MATH(LAMBDA(x,x))");
    model.set_a1("A5", "=MOD(7,LAMBDA(x,x))");
    model.evaluate();
    for cell in ["A1", "A2", "A3", "A4", "A5"] {
        assert_eq!(model.get_text_a1(cell), *"#VALUE!", "{cell}");
    }
}
