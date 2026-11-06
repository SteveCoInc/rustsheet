use crate::cell::CellValue;
use crate::test::util::new_empty_model;

#[test]
fn fn_convert() {
    let mut model = new_empty_model();
    model._set("A1", r#"=CONVERT(1, "lbm", "kg")"#);
    model._set("A2", r#"=CONVERT(68, "F", "C")"#);
    model._set("A3", r#"=CONVERT(2.5, "ft", "sec")"#);
    model._set("A4", r#"=CONVERT(CONVERT(100,"ft","m"),"ft","m""#);

    model._set("B1", "6");

    model._set("A5", r#"=CONVERT(B1,"C","F")"#);
    model._set("A6", r#"=CONVERT(B1,"tsp","tbs")"#);
    model._set("A7", r#"=CONVERT(B1,"gal","l")"#);
    model._set("A8", r#"=CONVERT(B1,"mi","km")"#);
    model._set("A9", r#"=CONVERT(B1,"km","mi")"#);
    model._set("A10", r#"=CONVERT(B1,"in","ft")"#);
    model._set("A11", r#"=CONVERT(B1,"cm","in")"#);

    model.evaluate();

    assert_eq!(model._get_text("A1"), "0.45359237");
    assert_eq!(model._get_text("A2"), "20");
    assert_eq!(model._get_text("A3"), "#N/A");
    assert_eq!(model._get_text("A4"), "9.290304");

    assert_eq!(model._get_text("A5"), "42.8");
    assert_eq!(model._get_text("A6"), "2");
    assert_eq!(model._get_text("A7"), "22.7124707"); //22.71741274");
    assert_eq!(model._get_text("A8"), "9.656064");
    assert_eq!(model._get_text("A9"), "3.728227153");
    assert_eq!(model._get_text("A10"), "0.5");
    assert_eq!(model._get_text("A11"), "2.362204724");
}

#[test]
fn convert_matches_excel_bits() {
    // Excel-cached values: ironcalc-calc ENGINEERING/CONVERT.xlsx (docs/excel-semantics/
    // engineering.md "CONVERT constants and evaluation order").
    let mut model = new_empty_model();
    model._set("A1", r#"=CONVERT(1,"sg","g")"#); // Weight!C3
    model._set("A2", r#"=CONVERT(0.05,"Mu","g")"#); // Weight!G5
    model._set("A3", r#"=CONVERT(4,"parsec","m")"#); // distance!G11
    model._set("A4", r#"=CONVERT(1,"pica","m")"#); // distance!C15
    model._set("A5", r#"=CONVERT(0.0002,"Mang3","m3")"#); // Volume!J17
    model._set("A6", r#"=CONVERT(0.0002,"Muk_pt","m3")"#); // Volume!J9
    model._set("A7", r#"=CONVERT(5,"cal","J")"#); // Energy!G5
    model._set("A8", r#"=CONVERT(5,"eV","J")"#); // Energy!G6
    model._set("A9", r#"=CONVERT(0.05,"Mdyn","N")"#); // Force!J3
    model._set("A10", r#"=CONVERT(1,"Picapt2","m2")"#); // Area!C21
    model._set("A11", r#"=CONVERT(1,"F","K")"#); // Temperature!C4
    model._set("A12", r#"=CONVERT(0.004,"Mm/h","m/s")"#); // Speed!J4
    model.evaluate();
    let bits = |c: &str| match model.get_cell_value_by_ref(&format!("Sheet1!{c}")) {
        Ok(CellValue::Number(n)) => n.to_bits(),
        other => panic!("expected number, got {other:?}"),
    };
    assert_eq!(bits("A1"), 14_593.902_937_206_363_f64.to_bits());
    assert_eq!(bits("A2"), 8.302_693_91e-20_f64.to_bits());
    assert_eq!(bits("A3"), 1.234_271_032_512_621_3e17_f64.to_bits());
    assert_eq!(bits("A4"), 0.004_233_333_333_333_334_f64.to_bits());
    assert_eq!(bits("A5"), 2.000_000_000_000_000_2e-16_f64.to_bits());
    assert_eq!(bits("A6"), 0.113_652_250_000_000_02_f64.to_bits());
    assert_eq!(bits("A7"), 20.934_f64.to_bits());
    assert_eq!(bits("A8"), 8.010_882_434_999_999e-19_f64.to_bits());
    assert_eq!(bits("A9"), 0.5_f64.to_bits());
    assert_eq!(bits("A10"), 1.244_521_604_938_271_5e-7_f64.to_bits());
    assert_eq!(bits("A11"), 255.927_777_777_777_75_f64.to_bits());
    assert_eq!(bits("A12"), 1.111_111_111_111_111_f64.to_bits());
}

/// Excel: the two prefixes are one factor, the nearest double to their combined power of ten,
/// not a product or quotient of the prefixes (probes/convert-unit-names, probes/convert-temperature-
/// ov00 and ov20, probes-numeric/t3861-convert-prefix-sweep; excel-graph; T-3861).
#[test]
fn prefixes_are_one_power_of_ten() {
    let mut model = new_empty_model();
    model._set("A1", r#"=CONVERT(1,"m","nm")"#); // convert-unit-names, 1/1E-9 is 999999999.9999999
    model._set("A2", r#"=CONVERT(1,"dm2","m2")"#); // 0.1 x 0.1 is 0.010000000000000002
    model._set("A3", r#"=CONVERT(1,"cm3","l")"#); // 0.01^3 x 1000 is 0.0010000000000000002
    model._set("A4", r#"=CONVERT(1,"Pm3","l")"#); // 1E15^3 x 1000 is 1E48
    model._set("A5", r#"=CONVERT(0.0002,"m3","Mm3")"#); // x 1E-18, not / 1E18
    model._set("A6", r#"=CONVERT(1E290,"Ym","Ym")"#); // ov00: 1E24 x 1E-24 is 1 - 2^-53
    model._set("A7", r#"=CONVERT(1E300,"Ym","Zm")"#); // ov20
    model._set("A8", r#"=CONVERT(1E300,"Ym","m")"#); // ov02: overflows
    model.evaluate();
    let bits = |c: &str| match model.get_cell_value_by_ref(&format!("Sheet1!{c}")) {
        Ok(CellValue::Number(n)) => n.to_bits(),
        other => panic!("expected number, got {other:?}"),
    };
    assert_eq!(bits("A1"), 1e9_f64.to_bits());
    assert_eq!(bits("A2"), 0.01_f64.to_bits());
    assert_eq!(bits("A3"), 0.001_f64.to_bits());
    assert_eq!(bits("A4"), 9.999_999_999_999_999e47_f64.to_bits());
    assert_eq!(bits("A5"), 2.000_000_000_000_000_3e-22_f64.to_bits());
    assert_eq!(bits("A6"), 1e290_f64.to_bits());
    assert_eq!(bits("A7"), 1e303_f64.to_bits());
    assert_eq!(model._get_text("A8"), "#NUM!");
}

/// Excel: a boolean number is `#VALUE!`, before an unknown unit's `#N/A`
/// (probes/eng-gen-sample-edges A8, A11:A14).
#[test]
fn boolean_number_is_value_error() {
    let mut model = new_empty_model();
    model._set("B1", "TRUE");
    model._set("A1", r#"=CONVERT(TRUE,"m","ft")"#);
    model._set("A2", r#"=CONVERT(B1,"xyz","m")"#);
    model._set("A3", r#"=INDEX(CONVERT({TRUE;1},"m","m"),1)"#);
    model._set("A4", r#"=CONVERT("2.5","ft","xyz")"#);
    model.evaluate();
    assert_eq!(model._get_text("A1"), *"#VALUE!");
    assert_eq!(model._get_text("A2"), *"#VALUE!");
    assert_eq!(model._get_text("A3"), *"#VALUE!");
    assert_eq!(model._get_text("A4"), *"#N/A");
}

/// Excel: a boolean unit read from a cell is `#VALUE!`, not an unknown unit (T-2793;
/// gen-m9 290180acf017709c Calc!A148, A158, f81bd3b6dc373e5c Calc!A291).
#[test]
fn boolean_unit_cell_is_value_error() {
    let mut model = new_empty_model();
    model._set("B1", "TRUE");
    model._set("B2", "FALSE");
    model._set("A1", r#"=CONVERT(1,B1,"m")"#);
    model._set("A2", r#"=CONVERT(1,B2,"mm")"#);
    model._set("A3", r#"=CONVERT(1,"C",B1)"#);
    model._set("A4", r#"=CONVERT(1,"C",B2)"#);
    model.evaluate();
    model._set("A5", r#"=CONVERT(1,NA(),B1)"#);
    model.evaluate();
    assert_eq!(model._get_text("A5"), *"#N/A");
    for cell in ["A1", "A2", "A3", "A4"] {
        assert_eq!(model._get_text(cell), *"#VALUE!");
    }
}

#[test]
fn unit_names_follow_excel() {
    // Excel (excel-graph): probes/convert-unit-names (docs/excel-semantics/engineering.md "CONVERT
    // unit names"). Each pair: formula, Excel's value.
    let cases = [
        // Seconds take a decimal prefix.
        (r#"=CONVERT(1,"ms","sec")"#, "0.001"),
        (r#"=CONVERT(1,"sec","ksec")"#, "0.001"),
        // Binary prefixes only on bit and byte.
        (r#"=CONVERT(1,"kibyte","bit")"#, "8192"),
        (r#"=CONVERT(1,"kim","m")"#, "#N/A"),
        (r#"=CONVERT(1,"Mig","g")"#, "#N/A"),
        (r#"=CONVERT(1,"kiK","K")"#, "#N/A"),
        // "pc" is the parsec against a distance and pico-calories against an energy.
        (r#"=CONVERT(1,"pc","parsec")"#, "1"),
        (r#"=CONVERT(1,"pc","c")"#, "1E-12"),
        (r#"=CONVERT(1,"kpc","c")"#, "#N/A"),
        // Units outside the prefixable list, case and surrounding spaces.
        (r#"=CONVERT(1,"kft","m")"#, "#N/A"),
        (r#"=CONVERT(1,"KM","m")"#, "#N/A"),
        (r#"=CONVERT(1," m","m")"#, "#N/A"),
        (r#"=CONVERT(1,"daar","m2")"#, "1000"),
    ];
    let mut model = new_empty_model();
    for (row, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("A{}", row + 1)),
            *expected,
            "{formula}"
        );
    }
}

#[test]
fn convert_mixed_binary_decimal_prefixes() {
    // Excel (excel-graph): probes/convert-mixed-prefix and probes-numeric/t3861-convert-prefix-sweep
    // A52 (T-3975). With a binary prefix on either unit, a decimal prefix on the target enters as
    // the rounded reciprocal of its constant (× 2^30 × (1/1E-21), not × 2^30 × 1E21).
    let mut model = new_empty_model();
    model._set("A1", r#"=CONVERT(4.73E-10,"Gibyte","zbyte")"#);
    model.evaluate();
    let CellValue::Number(n) = model.get_cell_value_by_ref("Sheet1!A1").unwrap() else {
        panic!("A1 is not a number");
    };
    assert_eq!(n.to_bits(), 5.078_798_827_520_000_7e20_f64.to_bits());
    assert_ne!(n.to_bits(), 5.078_798_827_52e20_f64.to_bits());
}

#[test]
fn kelvin_prefix_is_dropped_toward_rank_and_reau() {
    // Excel (excel-graph): probes/t4615-convert-prefixed-kelvin (T-4615). The prefix on K or kel
    // is ignored from K to Rank, from K to Reau and from Reau to K, but applies from Rank to K and
    // against C and F.
    let cases: [(&str, f64); 8] = [
        (r#"=CONVERT(1000,"YK","Rank")"#, 1800.0),
        (r#"=CONVERT(1000,"ykel","Rank")"#, 1800.0),
        (r#"=CONVERT(1000,"mkel","Reau")"#, 581.48),
        (r#"=CONVERT(1000,"Reau","YK")"#, 1523.15),
        (
            r#"=CONVERT(-56.8944444765809,"ukel","Reau")"#,
            -264.035_555_581_264_76,
        ),
        (r#"=CONVERT(1000,"Rank","YK")"#, 5.555_555_555_555_556e-22),
        (r#"=CONVERT(1000,"mK","C")"#, -272.15),
        (r#"=CONVERT(1000,"C","kK")"#, 1.27315),
    ];
    let mut model = new_empty_model();
    for (row, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, expected)) in cases.iter().enumerate() {
        let CellValue::Number(n) = model
            .get_cell_value_by_ref(&format!("Sheet1!A{}", row + 1))
            .unwrap()
        else {
            panic!("{formula} is not a number");
        };
        assert_eq!(n.to_bits(), expected.to_bits(), "{formula}");
    }
}
