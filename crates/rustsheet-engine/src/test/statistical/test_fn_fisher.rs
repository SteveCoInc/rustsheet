// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::float_cmp)]
// LINT-DEBT-END
#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;
#[test]
fn test_fn_fisher_smoke() {
    let mut model = new_empty_model();

    // Valid inputs
    model.set_a1("A1", "=FISHER(0.1)");
    model.set_a1("A2", "=FISHER(-0.5)");
    model.set_a1("A3", "=FISHER(0.8)");

    // Domain errors: x <= -1 or x >= 1 -> #NUM!
    model.set_a1("A4", "=FISHER(1)");
    model.set_a1("A5", "=FISHER(-1)");
    model.set_a1("A6", "=FISHER(2)");

    // Wrong number of arguments -> #ERROR!
    model.set_a1("A7", "=FISHER()");
    model.set_a1("A8", "=FISHER(0.1, 2)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"0.100335348");
    assert_eq!(model.get_text_a1("A2"), *"-0.549306144");
    assert_eq!(model.get_text_a1("A3"), *"1.098612289");

    assert_eq!(model.get_text_a1("A4"), *"#NUM!");
    assert_eq!(model.get_text_a1("A5"), *"#NUM!");
    assert_eq!(model.get_text_a1("A6"), *"#NUM!");

    assert_eq!(model.get_text_a1("A7"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A8"), *"#ERROR!");
}

#[test]
fn test_fn_fisher_inv_smoke() {
    let mut model = new_empty_model();

    // Valid inputs
    model.set_a1("A1", "=FISHERINV(-1.5)");
    model.set_a1("A2", "=FISHERINV(0.5)");
    model.set_a1("A3", "=FISHERINV(2)");

    // Wrong number of arguments -> #ERROR!
    model.set_a1("A4", "=FISHERINV()");
    model.set_a1("A5", "=FISHERINV(0.5, 1)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"-0.905148254");
    assert_eq!(model.get_text_a1("A2"), *"0.462117157");
    assert_eq!(model.get_text_a1("A3"), *"0.96402758");

    assert_eq!(model.get_text_a1("A4"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A5"), *"#ERROR!");
}

#[test]
fn test_fn_fisherinv_bit_exact() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=FISHERINV(1E-7)");
    model.set_a1("A2", "=FISHERINV(1000)");
    model.set_a1("A3", "=FISHERINV(-1000)");
    model.set_a1("A4", "=FISHERINV(0)");
    model.evaluate();

    let value = |cell: &str| match model.get_cell_value_by_ref(cell).unwrap() {
        crate::cell::CellValue::Number(n) => n,
        other => panic!("expected number, got {other:?}"),
    };
    assert_eq!(value("Sheet1!A1"), 1.000_000_000_503_929_2e-7);
    assert_eq!(value("Sheet1!A2"), 1.0);
    assert_eq!(value("Sheet1!A3"), -1.0);
    assert_eq!(value("Sheet1!A4"), 0.0);
}

#[test]
fn f_distributions_read_a_true_as_one() {
    let mut model = new_empty_model();
    // Excel values: probes-numeric/t1049-fdistrt A19, A26 and m9-legacy-fdist A90 (T-1849).
    model.set_a1("A1", "=ROUND(F.DIST.RT(TRUE,3,7),12)");
    model.set_a1("A2", "=ROUND(F.DIST.RT(1,TRUE,TRUE),12)");
    model.set_a1("A3", "=ROUND(FDIST(TRUE,2,5),12)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "0.447079613");
    assert_eq!(model.get_text_a1("A2"), "0.5");
    assert_eq!(model.get_text_a1("A3"), "0.43120115");
}
