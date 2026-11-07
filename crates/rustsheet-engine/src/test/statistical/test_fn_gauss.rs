#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_gauss_smoke() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=GAUSS(-3)");
    model.set_a1("A2", "=GAUSS(-2.3)");
    model.set_a1("A3", "=GAUSS(-1.7)");
    model.set_a1("A4", "=GAUSS(0)");
    model.set_a1("A5", "=GAUSS(0.5)");
    model.set_a1("A6", "=GAUSS(1)");
    model.set_a1("A7", "=GAUSS(1.3)");
    model.set_a1("A8", "=GAUSS(3)");
    model.set_a1("A9", "=GAUSS(4)");

    model.set_a1("G6", "=GAUSS()");
    model.set_a1("G7", "=GAUSS(1, 1)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"-0.498650102");
    assert_eq!(model.get_text_a1("A2"), *"-0.48927589");
    assert_eq!(model.get_text_a1("A3"), *"-0.455434537");
    assert_eq!(model.get_text_a1("A4"), *"0");
    assert_eq!(model.get_text_a1("A5"), *"0.191462461");
    assert_eq!(model.get_text_a1("A6"), *"0.341344746");
    assert_eq!(model.get_text_a1("A7"), *"0.403199515");
    assert_eq!(model.get_text_a1("A8"), *"0.498650102");
    assert_eq!(model.get_text_a1("A9"), *"0.499968329");

    assert_eq!(model.get_text_a1("G6"), *"#ERROR!");
    assert_eq!(model.get_text_a1("G7"), *"#ERROR!");
}

#[test]
fn test_fn_gauss_booleans() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=GAUSS(TRUE)");
    model.set_a1("A2", "=GAUSS(FALSE)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"0.341344746");
    assert_eq!(model.get_text_a1("A2"), *"0");
}

#[test]
fn test_fn_gauss_tiny_argument_is_z_times_phi_zero() {
    // Excel (gen-m9 oracle, T-2434): the leading term below ~2E-15, the cancelled Phi(z) - 0.5 above.
    let mut model = new_empty_model();
    model.set_a1("A1", "=GAUSS(1E-300)");
    model.set_a1("A2", "=GAUSS(-1E-300)");
    model.set_a1("A3", "=GAUSS(1E-10)");
    model.set_a1("A4", "=GAUSS(4E-15)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"3.9894E-301");
    assert_eq!(model.get_text_a1("A2"), *"-3.9894E-301");
    assert_eq!(model.get_text_a1("A3"), *"3.98943E-11");
    assert_eq!(model.get_text_a1("A4"), *"1.55431E-15");
}
