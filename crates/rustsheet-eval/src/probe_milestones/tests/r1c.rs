//! Probe rows split by R1(c) class (decision 0188, T-3427).

use super::*;

/// Decision 0188 R1(c): each visibly-off cell lands in the row's count its class names; the
/// row's `visibly_off` keeps them all (T-3427).
#[test]
fn visibly_off_cells_are_split_by_their_r1c_class() {
    let rules = rules("");
    let off = |a1: &str| CellResult {
        actual: Value::Number(2.0),
        ..cell(a1, "", Verdict::NumericMismatch)
    };
    let results = [case(
        "probes/r.case.json",
        &["M9"],
        CaseStatus::Fail,
        vec![
            off("S!A1"),
            off("S!A2"),
            off("S!A3"),
            off("S!A4"),
            off("S!A5"),
        ],
    )];
    let r1c = R1cCells::from_classes([
        ("probes/r.case.json!S!A1", R1c::Phase1b),
        ("probes/r.case.json!S!A2", R1c::R3Pass),
        ("probes/r.case.json!S!A3", R1c::M14Owned),
        ("probes/r.case.json!S!A4", R1c::Counts),
    ]);
    let rows = summarize(&rules, &results, BaselineMode::Cells, &r1c);
    let m9 = rows.iter().find(|r| r.milestone == "M9").expect("M9");
    assert_eq!(m9.visibly_off, 5);
    // A5 was never classified, so it counts too.
    assert_eq!(
        (m9.r1c_off, m9.phase1b, m9.r3_pass, m9.m14_owned),
        (2, 1, 1, 1)
    );
    assert_eq!(
        m9.r1c_off_cells,
        ["probes/r.case.json!S!A4", "probes/r.case.json!S!A5"]
    );
}

/// Decision 0355 item 4: the row's tiers count every non-deviation cell but the M14-owned ones,
/// with the cells R1(c) counts as misses, a wrong type or a number outside the envelope.
#[test]
fn the_row_tiers_count_r1c_misses_and_leave_m14_owned_cells_out() {
    let rules = rules("");
    let off = |a1: &str| CellResult {
        actual: Value::Number(2.0),
        ..cell(a1, "GEOMEAN(1)", Verdict::NumericMismatch)
    };
    let results = [case(
        "probes/r.case.json",
        &["M9"],
        CaseStatus::Fail,
        vec![
            pass("S!A1", "GEOMEAN(1)"),
            off("S!A2"),
            off("S!A3"),
            off("S!A4"),
            off("S!A5"),
            cell("S!A6", "GEOMEAN(1)", Verdict::Deviation),
            cell("S!A7", "1+1", Verdict::NumericMismatch),
        ],
    )];
    let r1c = R1cCells::from_classes([
        ("probes/r.case.json!S!A2", R1c::Phase1b),
        ("probes/r.case.json!S!A3", R1c::M14Owned),
        ("probes/r.case.json!S!A4", R1c::Counts),
    ])
    .with_outside("probes/r.case.json!S!A5");
    let rows = summarize(&rules, &results, BaselineMode::Cells, &r1c);
    let m9 = rows.iter().find(|r| r.milestone == "M9").expect("M9");
    // GEOMEAN is expert in the test catalog: A1, A2, A4, A5; A7 calls nothing (foundational)
    // and is within 15 digits of Excel (1 against 1), so no miss.
    assert_eq!(
        m9.tiers.expert,
        crate::usage_gates::Tally {
            cells: 4,
            wrong_type: 1,
            outside: 1
        }
    );
    assert_eq!(m9.tiers.foundational.cells, 1);
    assert!(!m9.tiers.meets_bars());
}
