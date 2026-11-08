// LINT-DEBT-BEGIN (inherited from the IronCalc fork, decision 0335: entries may only be removed, never added)
#![expect(clippy::float_cmp)]
// LINT-DEBT-END
#![allow(clippy::unwrap_used)]

use crate::cell::CellValue;
use crate::test::util::new_empty_model;

fn num(model: &mut crate::model::Model, cell: &str) -> f64 {
    model.get_text_a1(cell).parse::<f64>().unwrap()
}

// VDB documented examples
// (https://support.microsoft.com/en-us/office/vdb-function-dde4e207-f3fa-488d-91d2-66d55e861d73)
#[test]
fn fn_vdb_microsoft_examples() {
    let mut model = new_empty_model();
    // First period of the double-declining-balance method (factor defaults to 2):
    // 2400 * 2 / 10 = 480
    model.set_a1("A1", "=VDB(2400, 300, 10, 0, 1)");
    // Depreciation of the first 7/8 of a year with factor 1.5: 315
    model.set_a1("A2", "=VDB(2400, 300, 10, 0, 0.875, 1.5)");
    // Months 6..18 with the default factor: ~396.31
    model.set_a1("A3", "=VDB(2400, 300, 120, 6, 18)");
    model.evaluate();

    assert!((num(&mut model, "A1") - 480.0).abs() < 1e-6);
    assert!((num(&mut model, "A2") - 315.0).abs() < 1e-6);
    assert!((num(&mut model, "A3") - 396.306_138).abs() < 1e-3);
}

// The no_switch flag keeps the calculation on declining balance. With a slow
// (factor 1) declining balance the straight-line switch matters.
#[test]
fn fn_vdb_no_switch() {
    let mut model = new_empty_model();
    // First year is the same either way (240 = factor-1 declining balance).
    model.set_a1("A1", "=VDB(2400, 300, 10, 0, 1, 1, TRUE)");
    // Whole life, allowing the switch, recovers cost - salvage = 2100.
    model.set_a1("A2", "=VDB(2400, 300, 10, 0, 10, 1)");
    // Whole life with no switch stays on declining balance, recovering less.
    model.set_a1("A3", "=VDB(2400, 300, 10, 0, 10, 1, TRUE)");
    model.evaluate();

    assert!((num(&mut model, "A1") - 240.0).abs() < 1e-6);
    assert!((num(&mut model, "A2") - 2100.0).abs() < 1e-6);
    assert!((num(&mut model, "A3") - 1_563.171_74).abs() < 1e-4);
    assert!(num(&mut model, "A3") < num(&mut model, "A2"));
}

#[test]
fn fn_vdb_errors() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=VDB(2400, 300, 10, 5, 3)"); // start > end
    model.set_a1("A2", "=VDB(2400, 300, -10, 0, 1)"); // life <= 0
    model.set_a1("A3", "=VDB(2400, 300, 10)"); // too few args
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#NUM!");
    assert_eq!(model.get_text_a1("A2"), *"#NUM!");
    assert_eq!(model.get_text_a1("A3"), *"#ERROR!");
}

/// An end period of 1e12 is past the period cap, so VDB returns `#NUM!` at once instead of walking
/// 1e12 periods, in both modes (DEVIATION "DB and VDB period cap", T-1820, T-1977).
#[test]
fn fn_vdb_huge_period_is_capped() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=VDB(1,0,1e12,0,1e12)");
    model.set_a1("A2", "=VDB(1,0,1e12,0,1e12,2,TRUE)");
    model.set_a1("A3", "=VDB(1,0,1e12,999999999999,1e12)");
    let started = std::time::Instant::now();
    model.evaluate();
    assert!(started.elapsed() < std::time::Duration::from_secs(5));

    for cell in ["A1", "A2", "A3"] {
        assert_eq!(model.get_text_a1(cell), *"#NUM!", "{cell}");
    }
}

/// VDB's `no_switch` reads as a boolean ("TRUE"/"FALSE" text in any case; other text, numeric too,
/// is `#VALUE!` before the range checks), a factor of 0 is straight line, an empty window at 0 is 0,
/// and DDB's factor takes a boolean and numeric text (probes/t2492-vdb-ddb-arguments, excel-graph,
/// 2026-10-25, T-2492).
#[test]
fn vdb_and_ddb_switch_and_factor_arguments() {
    let mut model = new_empty_model();
    model.set_a1("B1", "TRUE");
    for (a1, formula) in [
        ("A1", "=VDB(17334,389,36,0.2,9,3,\"true\")"),
        ("A2", "=VDB(17334,389,36,0.2,9,3,\"1\")"),
        ("A3", "=VDB(17334,389,36,0.2,9,3,\"x\")"),
        ("A4", "=VDB(1,-0.0001,23,1,9,10,\"true\")"),
        ("A5", "=VDB(1000,100,10,0,1,0)"),
        ("A6", "=VDB(1000,100,10,0,1,0,TRUE)"),
        ("A7", "=VDB(1000,100,10,0,1,-1)"),
        ("A8", "=VDB(1000,100,10,0,0)"),
        ("A9", "=VDB(43478,488,1E+9,3,7,0)"),
        ("A10", "=DDB(1000,100,10,1,TRUE)"),
        ("A11", "=DDB(1000,100,10,1,B1)"),
        ("A12", "=DDB(1000,100,10,1,FALSE)"),
        ("A13", "=DDB(1000,100,10,1,\"3\")"),
        ("A14", "=DDB(1000,100,10,1,\"TRUE\")"),
    ] {
        model.set_a1(a1, formula);
    }
    model.evaluate();
    assert!((num(&mut model, "A1") - 9_123.703_644_445_597).abs() < 1e-6);
    assert_eq!(model.get_text_a1("A2"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A3"), *"#VALUE!");
    assert_eq!(model.get_text_a1("A4"), *"#NUM!");
    assert_eq!(num(&mut model, "A5"), 90.0);
    assert_eq!(num(&mut model, "A6"), 0.0);
    assert_eq!(model.get_text_a1("A7"), *"#NUM!");
    assert_eq!(num(&mut model, "A8"), 0.0);
    assert!((num(&mut model, "A9") - 0.000_171_96).abs() < 1e-12);
    assert_eq!(model.get_text_a1("A10"), *"$100.00");
    assert_eq!(model.get_text_a1("A11"), *"$100.00");
    assert_eq!(model.get_text_a1("A12"), *"#NUM!");
    assert_eq!(model.get_text_a1("A13"), *"$300.00");
    assert_eq!(model.get_text_a1("A14"), *"#VALUE!");
}

// AMORLINC documented example
// (https://support.microsoft.com/en-us/office/amorlinc-function-7d417b45-f7f5-4dba-a0a5-3451a81079a8)
#[test]
fn fn_amorlinc_microsoft_example() {
    let mut model = new_empty_model();
    model.set_a1(
        "A1",
        "=AMORLINC(2400, DATE(2008,8,19), DATE(2008,12,31), 300, 1, 0.15, 1)",
    );
    model.evaluate();

    assert!((num(&mut model, "A1") - 360.0).abs() < 1e-6);
}

// AMORDEGRC documented example
// (https://support.microsoft.com/en-us/office/amordegrc-function-a14d0ca1-64a4-42eb-9b3d-b0dededf9e51)
#[test]
fn fn_amordegrc_microsoft_example() {
    let mut model = new_empty_model();
    model.set_a1(
        "A1",
        "=AMORDEGRC(2400, DATE(2008,8,19), DATE(2008,12,31), 300, 1, 0.15, 1)",
    );
    model.evaluate();

    assert!((num(&mut model, "A1") - 776.0).abs() < 1e-6);
}

// AMORLINC: period 0 returns the (prorated) first-period depreciation.
#[test]
fn fn_amorlinc_first_period() {
    let mut model = new_empty_model();
    model.set_a1(
        "A1",
        "=AMORLINC(2400, DATE(2008,8,19), DATE(2008,12,31), 300, 0, 0.15, 1)",
    );
    model.evaluate();

    // 134 / 366 * 0.15 * 2400
    let expected = 134.0 / 366.0 * 0.15 * 2400.0;
    assert!((num(&mut model, "A1") - expected).abs() < 1e-6);
}

#[test]
fn fn_amorlinc_errors() {
    let mut model = new_empty_model();
    // salvage > cost (equal is 0: `fn_amor_salvage_equal_to_the_cost`)
    model.set_a1(
        "A1",
        "=AMORLINC(2400, DATE(2008,8,19), DATE(2008,12,31), 2401, 1, 0.15, 1)",
    );
    // basis 2 (Actual/360) is not allowed
    model.set_a1(
        "A2",
        "=AMORLINC(2400, DATE(2008,8,19), DATE(2008,12,31), 300, 1, 0.15, 2)",
    );
    // date_purchased >= first_period
    model.set_a1(
        "A3",
        "=AMORLINC(2400, DATE(2009,1,1), DATE(2008,12,31), 300, 1, 0.15, 1)",
    );
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#NUM!");
    assert_eq!(model.get_text_a1("A2"), *"#NUM!");
    assert_eq!(model.get_text_a1("A3"), *"#NUM!");
}

// A salvage equal to the cost is 0 at every period, fractions and period 0 included, but only
// after every argument check: the basis's error, a basis of 2 and AMORDEGRC's asset life still
// win, and a cost of 0 stays #NUM!. The walk keeps comparing `remain < salvage`: a remain equal to
// a lower salvage still charges (financial.md "AMORLINC and AMORDEGRC check order";
// probes/amor-salvage-equal-cost, T-4504).
#[test]
fn fn_amor_salvage_equal_to_the_cost() {
    let mut model = new_empty_model();
    let cases = [
        ("=AMORDEGRC(2400,39679,39813,2400,0,0.15,1)", "0"),
        ("=AMORDEGRC(2400,39679,39813,2400,1,0.15,1)", "0"),
        ("=AMORDEGRC(2400,39679,39813,2400,5,0.15,1)", "0"),
        ("=AMORDEGRC(2400,39679,39813,2400,6,0.15,1)", "0"),
        ("=AMORDEGRC(2400,39679,39813,2400,1.5,0.15,1)", "0"),
        ("=AMORDEGRC(2400,39679,39813,2400,1,0.4,1)", "0"),
        ("=AMORLINC(2400,39679,39813,2400,0,0.15,1)", "0"),
        ("=AMORLINC(2400,39679,39813,2400,0.5,0.15,1)", "0"),
        ("=AMORLINC(2400,39679,39813,2400,2,0.15,1)", "0"),
        ("=AMORLINC(2400,39679,39813,2400,1,2,1)", "0"),
        ("=AMORDEGRC(2400,39679,39813,2400,1,0.5,1)", "#NUM!"),
        ("=AMORDEGRC(2400,39679,39813,2400,0.5,0.5,1)", "#NUM!"),
        ("=AMORDEGRC(2400,39679,39813,2400,1,0.5,NA())", "#N/A"),
        ("=AMORDEGRC(2400,39679,39813,2400,1,0.15,2)", "#NUM!"),
        ("=AMORLINC(2400,39679,39813,2400,1,0.15,NA())", "#N/A"),
        ("=AMORLINC(2400,39679,39813,2400,1,0.15,5)", "#NUM!"),
        ("=AMORLINC(2400,39813,39679,2400,1,0.15,1)", "#NUM!"),
        ("=AMORDEGRC(0,39679,39813,0,1,0.15,1)", "#NUM!"),
        ("=AMORLINC(0,39679,39813,0,0,0.15,1)", "#NUM!"),
        ("=AMORDEGRC(2400,39679,39813,2070,1,0.15,1)", "776"),
        ("=AMORDEGRC(2400,39679,39813,1293.75,2,0.15,1)", "485"),
    ];
    for (row, (formula, _)) in cases.iter().enumerate() {
        model.set_a1(&format!("A{}", row + 1), formula);
    }
    model.evaluate();
    for (row, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model.get_text_a1(&format!("A{}", row + 1)),
            *expected,
            "{formula}"
        );
    }
}

#[test]
fn fn_forecast_ets_family_computes() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=FORECAST.ETS(5, {1,2,3}, {1,2,3})");
    model.set_a1("A2", "=FORECAST.ETS.CONFINT(5, {1,2,3}, {1,2,3})");
    model.set_a1("A3", "=FORECAST.ETS.SEASONALITY({1,2,3}, {1,2,3})");
    model.set_a1("A4", "=FORECAST.ETS.STAT({1,2,3}, {1,2,3}, 1)");
    model.evaluate();

    // The family computes (T-3599 to T-3601, unverified, decision 0221): the line continues
    // with no error, three points show no season, and α is the first grid point (0.05).
    assert_eq!(model.get_text_a1("A1"), *"5");
    assert_eq!(model.get_text_a1("A2"), *"0");
    assert_eq!(model.get_text_a1("A3"), *"0");
    assert_eq!(model.get_text_a1("A4"), *"0.05");
}

// Excel's exact doubles for VDB's two charge orders (docs/excel-semantics/financial.md
// "VDB charge order"; probes/fin-vdb-modes, fin-vdb-schedule, T-1975).
#[test]
fn fn_vdb_charge_order_bits() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=VDB(2400,300,10,7,10,2,FALSE)");
    model.set_a1("A2", "=VDB(2400,300,10,7,10,2,TRUE)");
    model.set_a1("A3", "=VDB(2400,300,10,0,10,3,TRUE)");
    model.set_a1("A4", "=VDB(2400,0,10,5,6)");
    model.set_a1("A5", "=VDB(1234.56,78.9,12,4,9,1.75,FALSE)");
    model.set_a1("A6", "=VDB(2400,3000,10,0,1)");
    model.evaluate();
    let bits = |c: &str| match model.get_cell_value_by_ref(&format!("Sheet1!{c}")) {
        Ok(CellValue::Number(n)) => n.to_bits(),
        other => panic!("expected number, got {other:?}"),
    };

    assert_eq!(bits("A1"), 203.316_480_000_000_07_f64.to_bits());
    assert_eq!(bits("A2"), 203.316_480_000_000_4_f64.to_bits());
    assert_eq!(bits("A3"), 2_099.999_999_999_999_5_f64.to_bits());
    assert_eq!(bits("A4"), 157.286_400_000_000_01_f64.to_bits());
    assert_eq!(bits("A5"), 379.885_448_741_749_9_f64.to_bits());
    assert_eq!(bits("A6"), (-600.0_f64).to_bits());
}

// Switching from a fractional start lays the periods on the start's fraction (financial.md
// "VDB charge order"; probes/fin-vdb-fractional-start, fin-vdb-fractional-start-edges, T-2185).
#[test]
fn fn_vdb_fractional_start_bits() {
    let mut model = new_empty_model();
    // The same schedule wherever the start lies past the switch.
    model.set_a1("A1", "=VDB(2400,100,10,6.5,7.5)");
    model.set_a1("A2", "=VDB(2400,100,10,8.5,9.5)");
    // Past the switch from 8.3: straight line from the 5.3 grid point.
    model.set_a1("A3", "=VDB(2400,300,10,8.3,9.9,1.5)");
    // A clipped last piece stays declining; an unclipped one smaller than straight line switches.
    model.set_a1("A4", "=VDB(2400,300,10,9.25,10)");
    model.set_a1("A5", "=VDB(1000,5,10,9.25,10,4)");
    // The first partial charge is clipped at cost - salvage, and at 0 with salvage above cost.
    model.set_a1("A6", "=VDB(1000,900,5,0.5,1)");
    model.set_a1("A7", "=VDB(2400,3000,10,0.5,1.5)");
    // A window that is a hair above one period ends at its last grid point.
    model.set_a1("A8", "=VDB(2400,300,10,7.3,8.3)");
    model.evaluate();
    let bits = |c: &str| match model.get_cell_value_by_ref(&format!("Sheet1!{c}")) {
        Ok(CellValue::Number(n)) => n.to_bits(),
        other => panic!("expected number, got {other:?}"),
    };

    assert_eq!(bits("A1"), 133.208_868_571_428_58_f64.to_bits());
    assert_eq!(bits("A2"), 133.208_868_571_428_58_f64.to_bits());
    assert_eq!(bits("A3"), 244.075_770_638_297_8_f64.to_bits());
    assert_eq!(bits("A4"), 4.512_314_880_000_062_f64.to_bits());
    assert_eq!(bits("A5"), 4.069_926_399_999_998_f64.to_bits());
    assert_eq!(bits("A6"), 25.0_f64.to_bits());
    assert_eq!(bits("A7"), (-600.0_f64).to_bits());
    assert_eq!(bits("A8"), 94.623_498_24_f64.to_bits());
}

// Where straight line already wins at period 0, the whole window charges `(cost - salvage)/life`
// as one product `(end - start)·sl`, whatever the start (financial.md "VDB charge order";
// accuracy/VDB/s1, gen-m9/67a4c03afd63f988 A334 and probes/t4724-vdb-straight-from-zero W A7, a
// whole start with a fractional end; excel-graph, T-4111, T-4510, T-4724).
#[test]
fn fn_vdb_straight_line_from_period_zero_bits() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=VDB(7.00815007310244E-276,0,1793,53.25,1099,0.603)");
    model.set_a1(
        "A2",
        "=VDB(133385333.387961,232343.987648587,8,1.25,4.75,0.570826109457717,FALSE)",
    );
    model.set_a1(
        "A3",
        "=VDB(147.345559004139,0.00344462972942079,21,12.5,18.5,0.295807274286059)",
    );
    model.set_a1(
        "A4",
        "=VDB(121503138647.778,3864354372.11506,4,0.5,4,0.15833054798759,FALSE)",
    );
    model.set_a1(
        "A5",
        "=VDB(2802973.77427359,454.661570023026,24,6.5,18.75,0.101632427134068)",
    );
    model.set_a1(
        "A6",
        "=VDB(9.95316729348141,4.66310160542375,20,16.3,16.5,0.334748293043145)",
    );
    model.set_a1("A7", "=VDB(86661,45,37,0,6,0.2)");
    model.set_a1(
        "A8",
        "=VDB(4510.05703893478,2761.5898014473,19,14,17.5067437509898,0.236648520946425)",
    );
    model.evaluate();
    let bits = |c: &str| match model.get_cell_value_by_ref(&format!("Sheet1!{c}")) {
        Ok(CellValue::Number(n)) => n.to_bits(),
        other => panic!("expected number, got {other:?}"),
    };

    assert_eq!(bits("A1"), 4.087_436_106_495_748e-276_f64.to_bits());
    assert_eq!(bits("A2"), 58_254_432.862_636_68_f64.to_bits());
    assert_eq!(bits("A3"), 42.097_746_964_117_02_f64.to_bits());
    assert_eq!(bits("A4"), 102_933_936_241.205_06_f64.to_bits());
    assert_eq!(bits("A5"), 1_430_452.463_775_779_1_f64.to_bits());
    assert_eq!(bits("A6"), 0.052_900_656_880_576_415_f64.to_bits());
    assert_eq!(bits("A7"), 14_045.837_837_837_837_f64.to_bits());
    assert_eq!(bits("A8"), 322.706_660_993_137_8_f64.to_bits());
}

// RRI and DURATION use Excel's own operators (financial.md "DURATION and RRI operation order";
// probes/fin-mduration-rri-fit A13, A37, T-1975). RRI is `EXP(LN(fv/pv)*(1/nper))-1`, and
// `fv/pv-1` at nper 1 (RRI.xlsx D12, probes-numeric/m9-fin-rri-nper-one row 32, T-2217).
#[test]
fn fn_rri_and_duration_bits() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=RRI(10,1,200)");
    model.set_a1(
        "A2",
        "=DURATION(DATE(2020,3,15),DATE(2025,9,30),0,0.06,2,0)",
    );
    model.set_a1("A3", "=RRI(0.5,3,4)");
    model.set_a1("A4", "=RRI(1,826,40633)");
    model.evaluate();
    let bits = |c: &str| match model.get_cell_value_by_ref(&format!("Sheet1!{c}")) {
        Ok(CellValue::Number(n)) => n.to_bits(),
        other => panic!("expected number, got {other:?}"),
    };

    assert_eq!(bits("A1"), 0.698_646_464_634_247_2_f64.to_bits());
    assert_eq!(bits("A2"), 5.541_666_666_666_668_f64.to_bits());
    assert_eq!(bits("A3"), 0.777_777_777_777_777_5_f64.to_bits());
    assert_eq!(bits("A4"), 48.192_493_946_731_23_f64.to_bits());
}

// DB keeps the remaining balance and DDB charges `min(old * rate, old - salvage)`, in Excel's
// operation order (financial.md "DB and DDB operation order"; ironcalc-calc DB_DDB.xlsx G20,
// G42, H10, probes/fin-argument-modes H95, T-2033).
#[test]
fn fn_db_ddb_operation_order_bits() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=DB(100,10,4,1,2)");
    model.set_a1("A2", "=DB(23,10,4,5,2)");
    model.set_a1("A3", "=DDB(100,10,4,3,1.3334)");
    model.set_a1("A4", "=DDB(10000,1000,6,2,2)");
    model.set_a1("A5", "=DDB(9695,282,20,8,1)");
    model.evaluate();
    let bits = |c: &str| match model.get_cell_value_by_ref(&format!("Sheet1!{c}")) {
        Ok(CellValue::Number(n)) => n.to_bits(),
        other => panic!("expected number, got {other:?}"),
    };

    assert_eq!(bits("A1"), 7.299_999_999_999_999_f64.to_bits());
    assert_eq!(bits("A2"), 1.868_731_396_396_230_9_f64.to_bits());
    assert_eq!(bits("A3"), 14.814_814_787_037_497_f64.to_bits());
    assert_eq!(bits("A4"), 2_222.222_222_222_222_f64.to_bits());
    assert_eq!(bits("A5"), 338.519_004_281_445_3_f64.to_bits());
}

// SYD and DB argument domains (financial.md "DB and SYD argument domains",
// probes/fin-syd-db-domain, T-2491).
#[test]
fn fn_syd_db_argument_domains() {
    let mut model = new_empty_model();
    // A negative salvage is #NUM!, even at cost 0; salvage 0 is a value.
    model.set_a1("A1", "=SYD(36746,-100,22,3)");
    model.set_a1("A2", "=SYD(0,-439103,37,9)");
    model.set_a1("A3", "=DB(0,-439103,27,10)");
    model.set_a1("A4", "=DB(100,-0.0001,4,1)");
    model.set_a1("A5", "=DB(0,0,4,1)");
    // SYD: life*(life+1) past the double range is #DIV/0!, an overflowing numerator #NUM!.
    model.set_a1("B1", "=SYD(34664,230,1E+300,5)");
    model.set_a1("B2", "=SYD(100,10,1.4E+154,1)");
    model.set_a1("B3", "=SYD(100,10,1.3E+154,1)");
    model.set_a1("B4", "=SYD(1E+300,-1E+300,10,1)");
    // DB: a life past i32::MAX is #NUM!, but for a period below 1 only up to u32::MAX - 1.
    model.set_a1("C1", "=DB(6912,0,1E+300,2,6)");
    model.set_a1("C2", "=DB(67486,100,1E15+2,6,8)");
    model.set_a1("C3", "=DB(100,10,2147483648,1)");
    model.set_a1("C4", "=DB(18057,894,FACT(13),0.5,8)");
    model.set_a1("C5", "=DB(100,10,2147483648,0.5)");
    model.set_a1("C6", "=DB(100,10,4294967294,0.5,8)");
    model.set_a1("C7", "=DB(100,10,4294967295,0.5,12)");
    model.set_a1("C8", "=DB(100,10,4294967296,1,8)");
    // From u32::MAX on, with a month other than 12, a period below 1 is the last year.
    model.set_a1("C9", "=DB(100,0,4294967296,0.5,8)");
    model.set_a1("C10", "=DB(100,0,1E10,0.5,6)");
    // DB: a period past i32::MAX is period 1's value whatever the life.
    model.set_a1("D1", "=DB(100,10,4,2147483648)");
    model.set_a1("D2", "=DB(100,10,4,1E+200)");
    model.set_a1("D3", "=DB(0.01,488,32,1E+200,10)");
    model.set_a1("D4", "=DB(100,10,4,2147483647)");
    model.evaluate();
    let value = |c: &str| model.get_cell_value_by_ref(&format!("Sheet1!{c}")).unwrap();

    for c in [
        "A1", "A2", "A3", "A4", "B4", "C1", "C2", "C3", "C5", "C6", "C7", "C8", "D4",
    ] {
        assert_eq!(model.get_text_a1(c), "#NUM!", "{c}");
    }
    assert_eq!(model.get_text_a1("B1"), "#DIV/0!");
    assert_eq!(model.get_text_a1("B2"), "#DIV/0!");
    assert_eq!(value("A5"), CellValue::Number(0.0));
    assert_eq!(value("B3"), CellValue::Number(1.384_615_384_615_384_8e-152));
    assert_eq!(value("C4"), CellValue::Number(0.0));
    assert_eq!(value("C9"), CellValue::Number(11.111_111_111_111_114));
    assert_eq!(value("C10"), CellValue::Number(25.0));
    assert_eq!(value("D1"), CellValue::Number(43.8));
    assert_eq!(value("D2"), CellValue::Number(43.8));
    assert_eq!(value("D3"), CellValue::Number(-0.003_341_666_666_666_667_3));
}
