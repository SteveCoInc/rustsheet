use crate::test::util::new_empty_model;

#[test]
fn fn_bitand() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=BITAND(1,5)");
    model.set_a1("A2", "=BITAND(13, 25");

    model.set_a1("B1", "=BITAND(1)");
    model.set_a1("B2", "=BITAND(1, 2, 3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), "1");
    assert_eq!(model.get_text_a1("A2"), "9");
    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
}

#[test]
fn fn_bitor() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=BITOR(1, 5)");
    model.set_a1("A2", "=BITOR(13, 10");

    model.set_a1("B1", "=BITOR(1)");
    model.set_a1("B2", "=BITOR(1, 2, 3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), "5");
    assert_eq!(model.get_text_a1("A2"), "15");
    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
}

#[test]
fn fn_bitxor() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=BITXOR(1, 5)");
    model.set_a1("A2", "=BITXOR(13, 25");

    model.set_a1("B1", "=BITXOR(1)");
    model.set_a1("B2", "=BITXOR(1, 2, 3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), "4");
    assert_eq!(model.get_text_a1("A2"), "20");
    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
}

#[test]
fn fn_bitlshift() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=BITLSHIFT(4, 2)");
    model.set_a1("A2", "=BITLSHIFT(13, 7");

    model.set_a1("B1", "=BITLSHIFT(1)");
    model.set_a1("B2", "=BITLSHIFT(1, 2, 3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), "16");
    assert_eq!(model.get_text_a1("A2"), "1664");
    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
}

#[test]
fn fn_bitrshift() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=BITRSHIFT(4, 2)");
    model.set_a1("A2", "=BITRSHIFT(13, 7");
    model.set_a1("A3", "=BITRSHIFT(145, -3");

    model.set_a1("B1", "=BITRSHIFT(1)");
    model.set_a1("B2", "=BITRSHIFT(1, 2, 3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), "1");
    assert_eq!(model.get_text_a1("A2"), "0");
    assert_eq!(model.get_text_a1("A3"), "1160");
    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
}

// A left shift of 49 to 53 places is 0 in Excel, 48 and 54 are #NUM!
// (probes/eng-besselk-underflow-bit-shift).
#[test]
fn fn_bitshift_overflow() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=BITRSHIFT(12, -53)");
    model.set_a1("A2", "=BITLSHIFT(12, 53)");
    model.set_a1("A3", "=BITLSHIFT(12, 54)");
    model.set_a1("A4", "=BITLSHIFT(1, 48)");
    model.set_a1("A5", "=BITLSHIFT(1, 53.5)");
    model.set_a1("A6", "=BITLSHIFT(0, 100)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"0");
    assert_eq!(model.get_text_a1("A2"), *"0");
    assert_eq!(model.get_text_a1("A3"), *"#NUM!");
    assert_eq!(model.get_text_a1("A4"), *"#NUM!");
    assert_eq!(model.get_text_a1("A5"), *"0");
    assert_eq!(model.get_text_a1("A6"), *"0");
}
