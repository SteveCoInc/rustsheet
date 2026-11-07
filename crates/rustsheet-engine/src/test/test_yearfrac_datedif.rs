use crate::test::util::new_empty_model;

#[test]
fn arguments() {
    let mut model = new_empty_model();

    model.set_a1("A1", "=DATEDIF()");
    model.set_a1("A2", "=DATEDIF(\"1/1/2025\")");
    model.set_a1("A3", "=DATEDIF(\"1/1/2025\", \"1/1/2026\")");
    model.set_a1("A4", "=DATEDIF(\"1/1/2025\", \"1/1/2026\", \"D\")");
    model.set_a1("A5", "=DATEDIF(\"1/1/2025\", \"1/1/2026\", \"D\", \"D\")");

    model.set_a1("B1", "=YEARFRAC()");
    model.set_a1("B2", "=YEARFRAC(\"1/1/2025\")");
    model.set_a1("B3", "=YEARFRAC(\"1/1/2025\", \"1/1/2026\")");
    model.set_a1("B4", "=YEARFRAC(\"1/1/2025\", \"1/1/2026\", 3)");
    model.set_a1("B5", "=YEARFRAC(\"1/1/2025\", \"1/1/2026\", 3, 3)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A3"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A4"), *"365");
    assert_eq!(model.get_text_a1("A5"), *"#ERROR!");

    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B3"), *"1");
    assert_eq!(model.get_text_a1("B4"), *"1");
    assert_eq!(model.get_text_a1("B5"), *"#ERROR!");
}

#[test]
fn yearfrac_out_of_calendar_start_beats_an_error_end() {
    // dates.md "YEARFRAC reads its start's range before the end's error" (T-2266): the start's
    // #NUM! comes before the end's error, but an error start keeps its own error.
    let mut model = new_empty_model();
    model.set_a1("A1", "=YEARFRAC(-1,1/0)");
    model.set_a1("A2", "=YEARFRAC(NA(),-1)");
    model.set_a1("A3", "=YEARFRAC(123456790.12,1/0,0)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"#NUM!");
    assert_eq!(model.get_text_a1("A2"), *"#N/A");
    assert_eq!(model.get_text_a1("A3"), *"#NUM!");
}

#[test]
fn datedif_calendar_units_carry_half_a_second_and_yearfrac_floors() {
    // dates.md "DATEDIF's calendar units carry half a second; YEARFRAC floors on every basis"
    // (probes/t4574-feb29-near-midnight-date-args, T-4574): just below midnight of the fictitious
    // 1900-02-29, "D" floors to 1900-02-28 while "M", "MD" and "YD" read 1900-02-29.
    let mut model = new_empty_model();
    model.set_a1("A1", "=DATEDIF(60-0.4/86400,425,\"D\")");
    model.set_a1("A2", "=DATEDIF(60-0.4/86400,425,\"M\")");
    model.set_a1("A3", "=DATEDIF(60-0.4/86400,425,\"MD\")");
    model.set_a1("A4", "=DATEDIF(60-0.4/86400,425,\"YD\")");
    model.set_a1("A5", "=DATEDIF(31,61-0.4/86400,\"MD\")");
    // The cut is fl(N-0.5/86400): one ulp below it is still day N-1.
    model.set_a1("A6", "=DATEDIF(59-0.5/86400,425,\"MD\")");
    model.set_a1("A7", "=DATEDIF(59-0.5/86400-2^-47,425,\"MD\")");
    // The floors decide a reversed pair; the carried dates must lie in the calendar, for "D" too.
    model.set_a1("A8", "=DATEDIF(60-0.4/86400,59.5,\"M\")");
    model.set_a1("A9", "=DATEDIF(59.3,59-0.4/86400,\"M\")");
    model.set_a1("A10", "=DATEDIF(1,2958466-0.4/86400,\"D\")");
    model.set_a1("B1", "=YEARFRAC(59-0.4/86400,425,4)");
    model.set_a1("B2", "=YEARFRAC(31,61-0.4/86400,4)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"366");
    assert_eq!(model.get_text_a1("A2"), *"11");
    assert_eq!(model.get_text_a1("A3"), *"30");
    assert_eq!(model.get_text_a1("A4"), *"365");
    assert_eq!(model.get_text_a1("A5"), *"-1");
    assert_eq!(model.get_text_a1("A6"), *"0");
    assert_eq!(model.get_text_a1("A7"), *"1");
    assert_eq!(model.get_text_a1("A8"), *"-1");
    assert_eq!(model.get_text_a1("A9"), *"#NUM!");
    assert_eq!(model.get_text_a1("A10"), *"#NUM!");
    model.set_a1("C1", "=361/360");
    model.set_a1("C2", "=29/360");
    model.evaluate();
    assert_eq!(model.get_text_a1("B1"), model.get_text_a1("C1"));
    assert_eq!(model.get_text_a1("B2"), model.get_text_a1("C2"));
}
