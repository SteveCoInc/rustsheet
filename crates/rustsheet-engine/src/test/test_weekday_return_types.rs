use crate::test::util::new_empty_model;

#[test]
fn test_weekday_return_types_11_to_17() {
    let mut model = new_empty_model();

    // Test date: 44561 corresponds to a Friday (2021-12-31). We verify the
    // numeric result for each custom week start defined by return_type 11-17.
    model.set_a1("A1", "=WEEKDAY(44561,11)"); // Monday start
    model.set_a1("A2", "=WEEKDAY(44561,12)"); // Tuesday start
    model.set_a1("A3", "=WEEKDAY(44561,13)"); // Wednesday start
    model.set_a1("A4", "=WEEKDAY(44561,14)"); // Thursday start
    model.set_a1("A5", "=WEEKDAY(44561,15)"); // Friday start
    model.set_a1("A6", "=WEEKDAY(44561,16)"); // Saturday start
    model.set_a1("A7", "=WEEKDAY(44561,17)"); // Sunday start

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"5"); // Mon=1 .. Sun=7 ⇒ Fri=5
    assert_eq!(model.get_text_a1("A2"), *"4"); // Tue start ⇒ Fri=4
    assert_eq!(model.get_text_a1("A3"), *"3"); // Wed start ⇒ Fri=3
    assert_eq!(model.get_text_a1("A4"), *"2"); // Thu start ⇒ Fri=2
    assert_eq!(model.get_text_a1("A5"), *"1"); // Fri start ⇒ Fri=1
    assert_eq!(model.get_text_a1("A6"), *"7"); // Sat start ⇒ Fri=7
    assert_eq!(model.get_text_a1("A7"), *"6"); // Sun start ⇒ Fri=6
}

/// Excel (probes/weekday-serial-zero): a serial in [0, 1) is Saturday 0 January 1900, and a
/// `return_type` of 0, -1 or -0.0001 is `#NUM!`.
#[test]
fn test_weekday_serial_zero_and_return_type_zero() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=WEEKDAY(0)");
    model.set_a1("A2", "=WEEKDAY(0.5,2)");
    model.set_a1("A3", "=WEEKDAY(0.5,3)");
    model.set_a1("A4", "=WEEKDAY(31330,-0.0001)");
    model.set_a1("A5", "=WEEKDAY(31330,0)");
    model.set_a1("A6", "=WEEKDAY(31330,-1)");
    model.set_a1("A7", "=WEEKDAY(-0.5)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"7");
    assert_eq!(model.get_text_a1("A2"), *"6");
    assert_eq!(model.get_text_a1("A3"), *"5");
    assert_eq!(model.get_text_a1("A4"), *"#NUM!");
    assert_eq!(model.get_text_a1("A5"), *"#NUM!");
    assert_eq!(model.get_text_a1("A6"), *"#NUM!");
    assert_eq!(model.get_text_a1("A7"), *"#NUM!");
}
