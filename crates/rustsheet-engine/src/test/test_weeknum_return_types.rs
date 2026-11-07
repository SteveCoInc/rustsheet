use crate::test::util::new_empty_model;

#[test]
fn test_weeknum_return_types_11_to_17_and_21() {
    let mut model = new_empty_model();

    // Date 44561 -> 2021-12-31 (Friday). Previously verified as week 53 (Sunday/Monday start).
    // We verify that custom week-start codes 11-17 all map to week 53 and ISO variant (21) maps to 52.
    let formulas = [
        ("A1", "=WEEKNUM(44561,11)"),
        ("A2", "=WEEKNUM(44561,12)"),
        ("A3", "=WEEKNUM(44561,13)"),
        ("A4", "=WEEKNUM(44561,14)"),
        ("A5", "=WEEKNUM(44561,15)"),
        ("A6", "=WEEKNUM(44561,16)"),
        ("A7", "=WEEKNUM(44561,17)"),
        ("A8", "=WEEKNUM(44561,21)"), // ISO week numbering
    ];
    for (cell, formula) in formulas {
        model.set_a1(cell, formula);
    }

    model.evaluate();

    // All 11-17 variations should yield 53
    for cell in ["A1", "A2", "A3", "A4", "A5", "A6", "A7"] {
        assert_eq!(model.get_text_a1(cell), *"53", "{cell} should be 53");
    }
    // ISO week (return_type 21)
    assert_eq!(model.get_text_a1("A8"), *"52");
}

/// Excel numbers 1900's weeks as if serial 1 were a Sunday
/// (probes/dates-early-1900-edges, probes/weeknum-1900-later; T-0995).
#[test]
fn test_weeknum_1900_anchors_on_a_sunday_serial_1() {
    let mut model = new_empty_model();
    let cases = [
        ("=WEEKNUM(0)", "0"),
        ("=WEEKNUM(0,2)", "1"),
        ("=WEEKNUM(0,12)", "1"),
        ("=WEEKNUM(0,17)", "0"),
        ("=WEEKNUM(2,2)", "2"),
        ("=WEEKNUM(61,2)", "10"),
        ("=WEEKNUM(61)", "9"),
        ("=WEEKNUM(100,2)", "16"),
        ("=WEEKNUM(100)", "15"),
        ("=WEEKNUM(366,2)", "54"),
        ("=WEEKNUM(366)", "53"),
        ("=WEEKNUM(366,16)", "53"),
        // 1901 uses the real calendar again.
        ("=WEEKNUM(367,2)", "1"),
    ];
    for (i, (formula, _)) in cases.iter().enumerate() {
        model.set_a1(&format!("A{}", i + 1), formula);
    }
    model.evaluate();
    for (i, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model.get_text_a1(&format!("A{}", i + 1)),
            *expected,
            "{formula}"
        );
    }
}

#[test]
fn weeknum_return_type_snaps_near_an_integer() {
    // probes/t4454-near-midnight-date-args AA1:AB4 (T-4454): 2^-22 + 2^-33 below an integer is
    // that type, 2^-22 + 9*2^-36 below truncates.
    let mut model = new_empty_model();
    model.set_a1("A1", "=WEEKNUM(45019,2-(2^-22+2^-33))");
    model.set_a1("A2", "=WEEKNUM(45019,2-(2^-22+9*2^-36))");
    model.set_a1("A3", "=WEEKNUM(45017,17-2^-23)");
    model.set_a1("A4", "=WEEKNUM(45017,17-1E-6)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), *"15");
    assert_eq!(model.get_text_a1("A2"), *"14");
    assert_eq!(model.get_text_a1("A3"), *"13");
    assert_eq!(model.get_text_a1("A4"), *"14");
}

#[test]
fn weeknum_negative_return_type_snaps_within_half_the_positive_window() {
    // probes/t4619-weeknum-negative-snap-window (T-4619): A3 sets Monday, A2 is the edge type, A1
    // reads the state A2 leaves. A snap to 0 is #NUM! and empties it; a floor to -1 reads it (16).
    for (edge, snaps) in [
        ("-(2^-22+2^-33)", false),
        ("-2^-23", true),
        ("-(2^-23+2^-34)", true),
        ("-(2^-22+9*2^-36)", false),
    ] {
        let mut model = new_empty_model();
        model.set_a1("A1", "=WEEKNUM(100,99)");
        model.set_a1("A2", &format!("=WEEKNUM(100,{edge})"));
        model.set_a1("A3", "=WEEKNUM(100,2)");
        model.evaluate();
        let expected = if snaps { "#NUM!" } else { "16" };
        assert_eq!(model.get_text_a1("A2"), *expected, "{edge}");
        assert_eq!(model.get_text_a1("A1"), *expected, "{edge}");
    }
}
