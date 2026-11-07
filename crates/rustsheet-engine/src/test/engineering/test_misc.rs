use crate::test::util::new_empty_model;

#[test]
fn fn_getstep() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=GESTEP(7, 4.6)");
    model.set_a1("A2", "=GESTEP(45, 45)");
    model.set_a1("A3", "=GESTEP(-7, -6)");
    model.set_a1("A4", "=GESTEP(0.1)");
    model.set_a1("A5", "=GESTEP(-0.1)");

    model.set_a1("B1", "=GESTEP()");
    model.set_a1("B2", "=GESTEP(1,2,3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"1");
    assert_eq!(model.get_text_a1("A2"), *"1");
    assert_eq!(model.get_text_a1("A3"), *"0");
    assert_eq!(model.get_text_a1("A4"), *"1");
    assert_eq!(model.get_text_a1("A5"), *"0");

    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
}

#[test]
fn fn_delta() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=DELTA(7, 7)");
    model.set_a1("A2", "=DELTA(-7, -7)");
    model.set_a1("A3", "=DELTA(-7, 7)");
    model.set_a1("A4", "=DELTA(5, 0.5)");
    model.set_a1("A5", "=DELTA(-0, 0)");

    model.set_a1("B1", "=DELTA()");
    model.set_a1("B2", "=DELTA(1,2,3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"1");
    assert_eq!(model.get_text_a1("A2"), *"1");
    assert_eq!(model.get_text_a1("A3"), *"0");
    assert_eq!(model.get_text_a1("A4"), *"0");
    assert_eq!(model.get_text_a1("A5"), *"1");

    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
}

#[test]
fn fn_delta_misc() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=3+1e-16");
    model.set_a1("A2", "=3");
    model.set_a1("A3", "=3+1e-15");
    model.set_a1("B1", "=DELTA(A1, A2)");
    model.set_a1("B2", "=DELTA(A1, A3)");

    model.set_a1("B1", "1");
    model.set_a1("B2", "0");

    model.evaluate();

    assert_eq!(model.get_text_a1("B1"), *"1");
}
