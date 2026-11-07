// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::unreadable_literal)]
// LINT-DEBT-END
#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_confidence_norm_smoke() {
    let mut model = new_empty_model();

    model.set_a1("A1", "=CONFIDENCE.NORM(0.05, 2.5, 50)");

    // Some edge/error cases
    model.set_a1("A2", "=CONFIDENCE.NORM(0, 2.5, 50)"); // alpha <= 0  -> #NUM!
    model.set_a1("A3", "=CONFIDENCE.NORM(1, 2.5, 50)"); // alpha >= 1  -> #NUM!
    model.set_a1("A4", "=CONFIDENCE.NORM(0.05, -1, 50)"); // std_dev <=0 -> #NUM!
    model.set_a1("A5", "=CONFIDENCE.NORM(0.05, 2.5, 1)");
    model.set_a1("A6", "=CONFIDENCE.NORM(0.05, 2.5, 0.99)"); // size < 1 -> #NUM!

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"0.692951912");

    assert_eq!(model.get_text_a1("A2"), *"#NUM!");
    assert_eq!(model.get_text_a1("A3"), *"#NUM!");
    assert_eq!(model.get_text_a1("A4"), *"#NUM!");
    assert_eq!(model.get_text_a1("A5"), *"4.899909961");
    assert_eq!(model.get_text_a1("A6"), *"#NUM!");
}

#[test]
fn test_fn_confidence_t_smoke() {
    let mut model = new_empty_model();

    model.set_a1("A1", "=CONFIDENCE.T(0.05, 50000, 100)");

    // Some edge/error cases
    model.set_a1("A2", "=CONFIDENCE.T(0, 50000, 100)"); // alpha <= 0 -> #NUM!
    model.set_a1("A3", "=CONFIDENCE.T(1, 50000, 100)"); // alpha >= 1 -> #NUM!
    model.set_a1("A4", "=CONFIDENCE.T(0.05, -1, 100)");
    model.set_a1("A5", "=CONFIDENCE.T(0.05, 50000, 1)");
    model.set_a1("A6", "=CONFIDENCE.T(0.05, 50000, 1.7)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"9921.084758");

    assert_eq!(model.get_text_a1("A2"), *"#NUM!");
    assert_eq!(model.get_text_a1("A3"), *"#NUM!");
    assert_eq!(model.get_text_a1("A4"), *"#NUM!");
    assert_eq!(model.get_text_a1("A5"), *"#DIV/0!");
    assert_eq!(model.get_text_a1("A6"), *"#DIV/0!");
}

#[test]
fn test_fn_confidence_arguments() {
    let mut model = new_empty_model();

    model.set_a1("A1", "=CONFIDENCE.NORM()");
    model.set_a1("A2", "=CONFIDENCE.NORM(0.3)");
    model.set_a1("A3", "=CONFIDENCE.NORM(0.3, 0.5)");
    model.set_a1("A4", "=CONFIDENCE.NORM(0.3, 0.5, 2)");
    model.set_a1("A5", "=CONFIDENCE.NORM(0.3, 0.5, 2, 1)");

    model.set_a1("B1", "=CONFIDENCE.T()");
    model.set_a1("B2", "=CONFIDENCE.T(0.3)");
    model.set_a1("B3", "=CONFIDENCE.T(0.3, 0.5)");
    model.set_a1("B4", "=CONFIDENCE.T(0.3, 0.5, 2)");
    model.set_a1("B5", "=CONFIDENCE.T(0.3, 0.5, 2, 1)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A3"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A4"), *"0.366434539");
    assert_eq!(model.get_text_a1("A5"), *"#ERROR!");

    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B3"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B4"), *"0.693887599");
    assert_eq!(model.get_text_a1("B5"), *"#ERROR!");
}

// T-2858: the df 1 branch through the formula path (ironcalc CONFIDENCE.xlsx E22), and a margin past
// the double range is an error, never an infinity (TODO(T-3540): the error kind is unverified).
#[test]
fn test_fn_confidence_t_df_one() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=CONFIDENCE.T(0.05,1,2)");
    model.set_a1("A2", "=CONFIDENCE.T(1E-10,1E307,2)");
    model.set_a1("A3", "=CONFIDENCE.T(1E-300,1E300,2.9)");
    model.set_a1("A4", "=CONFIDENCE.T(1E-15,1E307,3)");
    model.evaluate();
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!A1"),
        Ok(crate::cell::CellValue::Number(8.984643532093761))
    );
    assert_eq!(model.get_text_a1("A2"), *"#NUM!");
    assert_eq!(model.get_text_a1("A3"), *"#NUM!");
    assert_eq!(model.get_text_a1("A4"), *"#NUM!");
}
