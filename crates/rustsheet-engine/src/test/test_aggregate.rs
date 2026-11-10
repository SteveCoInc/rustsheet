#![allow(clippy::unwrap_used)]

//! AGGREGATE options over hidden rows and nested SUBTOTAL cells, which the probe case format
//! cannot express (docs/excel-semantics/numbers.md "AGGREGATE", UNVERIFIED parts).
use crate::test::util::new_empty_model;

#[test]
fn hidden_rows_and_nested_subtotals() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("A3", "4");
    model.set_a1("A4", "=SUBTOTAL(9,A1:A2)");
    model.set_row_hidden(0, 2, true).unwrap();
    model.set_a1("B1", "=AGGREGATE(9,4,A1:A4)");
    model.set_a1("B2", "=AGGREGATE(9,5,A1:A4)");
    model.set_a1("B3", "=AGGREGATE(9,0,A1:A4)");
    model.set_a1("B4", "=AGGREGATE(9,1,A1:A4)");
    model.set_a1("B5", "=AGGREGATE(14,5,A1:A4,1)");
    model.evaluate();

    // Option 4 keeps everything; 5 drops hidden row 2; 0 drops the SUBTOTAL cell; 1 both.
    assert_eq!(model.get_text_a1("B1"), *"10");
    assert_eq!(model.get_text_a1("B2"), *"8");
    assert_eq!(model.get_text_a1("B3"), *"7");
    assert_eq!(model.get_text_a1("B4"), *"5");
    assert_eq!(model.get_text_a1("B5"), *"4");
}

#[test]
fn array_function_num_spills() {
    // probes/aggregate-args Misc!A37:B37: one AGGREGATE per element (SUM, then MAX).
    let mut model = new_empty_model();
    for (i, v) in [3, 1, 4, 1, 5, 9].iter().enumerate() {
        model.set_a1(&format!("E{}", i + 1), &v.to_string());
    }
    model.set_a1("A1", "=AGGREGATE({9,4},0,E1:E6)");
    model.set_a1("A3", "=AGGREGATE(9,0,E1:E6)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"23");
    assert_eq!(model.get_text_a1("B1"), *"9");
    assert_eq!(model.get_text_a1("A3"), *"23");
}

#[test]
fn lifted_calls_over_large_arrays_hit_the_visit_cap() {
    // 2^20 elements, each reading a 2^20-element computed array: far above the 2^27 cap
    // (directive 5), so the call is #ERROR! instead of running ~10^12 steps.
    let mut model = new_empty_model();
    model.set_a1("A1", "=AGGREGATE(14,6,SEQUENCE(1048576),SEQUENCE(1048576))");
    model.set_a1("A2", "=AGGREGATE(14,6,SEQUENCE(1000),{1,2,3})");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A2"), *"1000");
    assert_eq!(model.get_text_a1("C2"), *"998");
}

#[test]
fn range_function_num_grid_spills_and_off_grid_names() {
    // The formula_eval fuzz reproducer (T-0296): static analysis took a 2x2 range function_num
    // for a single cell, so the 2x2 result reached a scalar cell (a debug assertion), and
    // `Data!H0` lexed as a row-0 reference. Excel: probes/aggregate-range-function-num-grid.
    let mut model = new_empty_model();
    model.add_sheet("Data").unwrap();
    let data = [
        ("E1", "3"),
        ("E2", "1"),
        ("E3", "4"),
        ("E4", "1"),
        ("E5", "5"),
        ("E6", "9"),
        ("G1", "9"),
        ("H1", "4"),
        ("G2", "1"),
        ("H2", "14"),
    ];
    for (cell, value) in data {
        model.set_a1(&format!("Data!{cell}"), value);
    }
    model.set_a1("C3", "=AGGREGATE(Data!H0:I1,0,Data!E1:E6)");
    model.set_a1("A5", "=AGGREGATE(Data!G1:H2,0,Data!E1:E6)");
    model.evaluate();

    assert_eq!(model.get_text_a1("C3"), *"#NAME?");
    assert_eq!(model.get_text_a1("C4"), *"");
    assert_eq!(model.get_text_a1("A5"), *"23");
    assert_eq!(model.get_text_a1("B5"), *"9");
    assert_eq!(model.get_text_a1("A6"), *"3.833333333");
    assert_eq!(model.get_text_a1("B6"), *"#VALUE!");
}

#[test]
fn choose_of_a_range_function_num_spills_like_the_range() {
    // CHOOSE's static shape is a range's extents, not its counts, so a 2x2 CHOOSE arm spills
    // 2x2 like the range itself and a 1x1 one stays in its cell (T-1265).
    let mut model = new_empty_model();
    for (cell, value) in [
        ("E1", "3"),
        ("E2", "1"),
        ("E3", "4"),
        ("E4", "1"),
        ("E5", "5"),
        ("E6", "9"),
        ("G1", "9"),
        ("H1", "4"),
        ("G2", "1"),
        ("H2", "14"),
    ] {
        model.set_a1(cell, value);
    }
    model.set_a1("A1", "=AGGREGATE(CHOOSE(1,G1:H2),0,E1:E6)");
    model.set_a1("A4", "=AGGREGATE(CHOOSE(1,G1),0,E1:E6)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"23");
    assert_eq!(model.get_text_a1("B1"), *"9");
    assert_eq!(model.get_text_a1("A2"), *"3.833333333");
    assert_eq!(model.get_text_a1("B2"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A4"), *"23");
    assert_eq!(model.get_text_a1("B4"), *"");
}

/// SUBTOTAL's PRODUCT over no numbers is 0, as `PRODUCT` is, not the empty product 1
/// (docs/excel-semantics/numbers.md "SUBTOTAL 6 over no numbers is 0", gen-m6/216e3c05f407ca79).
#[test]
fn subtotal_product_over_no_numbers_is_zero() {
    let mut model = new_empty_model();
    model.set_a1("A1", "abc");
    model.set_a1("A2", "?b");
    model.set_a1("B1", "=SUBTOTAL(6,A1:A2)");
    // 106 is a hand expectation (same code path as 6), not oracle-verified.
    model.set_a1("B2", "=SUBTOTAL(106,C1:C3)");
    model.set_a1("B3", "=PRODUCT(A1:A2)");
    model.set_a1("D1", "3");
    model.set_a1("D2", "4");
    model.set_a1("B5", "=SUBTOTAL(6,D1:D2)");
    model.evaluate();
    assert_eq!(model.get_text_a1("B1"), "0");
    assert_eq!(model.get_text_a1("B2"), "0");
    assert_eq!(model.get_text_a1("B3"), "0");
    assert_eq!(model.get_text_a1("B5"), "12");
}

/// The SUBTOTAL/AGGREGATE flag is stored on the parsed formula (computed once, T-1195): it is the
/// formula's syntax, so an untaken IF branch counts and a plain SUM does not.
#[test]
fn parsed_formula_flags_nested_subtotal_calls() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=SUBTOTAL(9,B1:B2)");
    model.set_a1("A2", "=IF(FALSE,AGGREGATE(9,4,B1:B2),1)");
    model.set_a1("A3", "=SUM(B1:B2)");
    model.evaluate();
    let flags: Vec<bool> = model.parsed_formulas[0]
        .iter()
        .map(|f| f.calls_subtotal)
        .collect();
    assert_eq!(flags, [true, true, false]);
}

#[test]
fn computed_range_fourth_argument_spills() {
    // The formula_eval fuzz reproducer (T-5106, issue #4105): static analysis read a fourth
    // argument with a range shape as a reference, but `-range` and `range%` evaluate to arrays
    // that lift, so a 3x6 result reached a scalar cell (a debug assertion). Excel:
    // probes/aggregate-computed-range-fourth.
    let mut model = new_empty_model();
    model.add_sheet("Data").unwrap();
    for (cell, value) in [("C1", "1"), ("D1", "200"), ("E1", "3"), ("E2", "9")] {
        model.set_a1(&format!("Data!{cell}"), value);
    }
    model.set_a1("C3", "=AGGREGATE(7,2,Data!B3:B9,Data!C6:E11%");
    model.set_a1("A1", "=AGGREGATE(14,0,Data!E1:E2,Data!C1:D1%)");
    model.set_a1("A20", "=AGGREGATE(9,0,Data!E1:E2,-Data!C1:D1)");
    model.evaluate();

    assert_eq!(model.get_text_a1("C3"), *"#VALUE!");
    assert_eq!(model.get_text_a1("E8"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A1"), *"#NUM!");
    assert_eq!(model.get_text_a1("B1"), *"3");
    assert_eq!(model.get_text_a1("A20"), *"#VALUE!");
    assert_eq!(model.get_text_a1("B20"), *"#VALUE!");
}
