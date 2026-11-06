use crate::test::util::new_empty_model;

#[test]
fn arguments() {
    let mut model = new_empty_model();

    model._set("A1", "=DATEDIF()");
    model._set("A2", "=DATEDIF(\"1/1/2025\")");
    model._set("A3", "=DATEDIF(\"1/1/2025\", \"1/1/2026\")");
    model._set("A4", "=DATEDIF(\"1/1/2025\", \"1/1/2026\", \"D\")");
    model._set("A5", "=DATEDIF(\"1/1/2025\", \"1/1/2026\", \"D\", \"D\")");

    model._set("B1", "=YEARFRAC()");
    model._set("B2", "=YEARFRAC(\"1/1/2025\")");
    model._set("B3", "=YEARFRAC(\"1/1/2025\", \"1/1/2026\")");
    model._set("B4", "=YEARFRAC(\"1/1/2025\", \"1/1/2026\", 3)");
    model._set("B5", "=YEARFRAC(\"1/1/2025\", \"1/1/2026\", 3, 3)");

    model.evaluate();

    assert_eq!(model._get_text("A1"), *"#ERROR!");
    assert_eq!(model._get_text("A2"), *"#ERROR!");
    assert_eq!(model._get_text("A3"), *"#ERROR!");
    assert_eq!(model._get_text("A4"), *"365");
    assert_eq!(model._get_text("A5"), *"#ERROR!");

    assert_eq!(model._get_text("B1"), *"#ERROR!");
    assert_eq!(model._get_text("B2"), *"#ERROR!");
    assert_eq!(model._get_text("B3"), *"1");
    assert_eq!(model._get_text("B4"), *"1");
    assert_eq!(model._get_text("B5"), *"#ERROR!");
}

#[test]
fn yearfrac_out_of_calendar_start_beats_an_error_end() {
    // dates.md "YEARFRAC reads its start's range before the end's error" (T-2266): the start's
    // #NUM! comes before the end's error, but an error start keeps its own error.
    let mut model = new_empty_model();
    model._set("A1", "=YEARFRAC(-1,1/0)");
    model._set("A2", "=YEARFRAC(NA(),-1)");
    model._set("A3", "=YEARFRAC(123456790.12,1/0,0)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#NUM!");
    assert_eq!(model._get_text("A2"), *"#N/A");
    assert_eq!(model._get_text("A3"), *"#NUM!");
}

#[test]
fn datedif_calendar_units_carry_half_a_second_and_yearfrac_floors() {
    // dates.md "DATEDIF's calendar units carry half a second; YEARFRAC floors on every basis"
    // (probes/t4574-feb29-near-midnight-date-args, T-4574): just below midnight of the fictitious
    // 1900-02-29, "D" floors to 1900-02-28 while "M", "MD" and "YD" read 1900-02-29.
    let mut model = new_empty_model();
    model._set("A1", "=DATEDIF(60-0.4/86400,425,\"D\")");
    model._set("A2", "=DATEDIF(60-0.4/86400,425,\"M\")");
    model._set("A3", "=DATEDIF(60-0.4/86400,425,\"MD\")");
    model._set("A4", "=DATEDIF(60-0.4/86400,425,\"YD\")");
    model._set("A5", "=DATEDIF(31,61-0.4/86400,\"MD\")");
    // The cut is fl(N-0.5/86400): one ulp below it is still day N-1.
    model._set("A6", "=DATEDIF(59-0.5/86400,425,\"MD\")");
    model._set("A7", "=DATEDIF(59-0.5/86400-2^-47,425,\"MD\")");
    // The floors decide a reversed pair; the carried dates must lie in the calendar, for "D" too.
    model._set("A8", "=DATEDIF(60-0.4/86400,59.5,\"M\")");
    model._set("A9", "=DATEDIF(59.3,59-0.4/86400,\"M\")");
    model._set("A10", "=DATEDIF(1,2958466-0.4/86400,\"D\")");
    model._set("B1", "=YEARFRAC(59-0.4/86400,425,4)");
    model._set("B2", "=YEARFRAC(31,61-0.4/86400,4)");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"366");
    assert_eq!(model._get_text("A2"), *"11");
    assert_eq!(model._get_text("A3"), *"30");
    assert_eq!(model._get_text("A4"), *"365");
    assert_eq!(model._get_text("A5"), *"-1");
    assert_eq!(model._get_text("A6"), *"0");
    assert_eq!(model._get_text("A7"), *"1");
    assert_eq!(model._get_text("A8"), *"-1");
    assert_eq!(model._get_text("A9"), *"#NUM!");
    assert_eq!(model._get_text("A10"), *"#NUM!");
    model._set("C1", "=361/360");
    model._set("C2", "=29/360");
    model.evaluate();
    assert_eq!(model._get_text("B1"), model._get_text("C1"));
    assert_eq!(model._get_text("B2"), model._get_text("C2"));
}
