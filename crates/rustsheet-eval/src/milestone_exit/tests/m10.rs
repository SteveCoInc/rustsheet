//! M10's exit row: gen-m10's text items and value cells, and the M10 probe row (decision 0146
//! item 4).

use super::*;

/// A run with gen-m10's `items` text items (`failing` of them failing, listed in one case) and
/// 1,690 value cells (`gen_off` visibly off), and an M10 probe row with `probe_off` visibly-off
/// cells and `probe_text_failing` failing text items of 415.
fn m10_report(
    items: usize,
    failing: usize,
    gen_off: usize,
    probe_off: usize,
    probe_text_failing: usize,
) -> RunReport {
    let mut r = report(None);
    r.suites.push(SuiteSummary {
        text_cells: items,
        text_passed: items - failing,
        ..suite("gen-m10", 1_690, gen_off)
    });
    let mut g = crate::report::tests::case("gen-m10/abc.case.json", 1);
    g.text = (0..failing)
        .map(|i| crate::text_expect::TextResult {
            sheet: "S".into(),
            cell: format!("B{}", i + 1),
            expected: "1.0".into(),
            actual: Some("1".into()),
            pass: false,
        })
        .collect();
    r.cases.push(g);
    r.probe_milestones.push(MilestoneRow {
        milestone: "M10".into(),
        visibly_off: probe_off,
        r1c_off: probe_off,
        tiers: crate::usage_gates::TierTallies {
            foundational: crate::gate_tiers::Cells {
                cells: 415,
                wrong_type: probe_off,
                outside: 0,
            },
            ..Default::default()
        },
        text_items: 415,
        text_failing: probe_text_failing,
        ..MilestoneRow::default()
    });
    r
}

fn m10(r: &RunReport) -> ExitRow {
    row(r, "M10")
}

#[test]
fn m10_needs_ten_thousand_gen_m10_text_items_none_failing_and_clean_probes() {
    let r = m10_report(10_400, 0, 0, 0, 0);
    let row = m10(&r);
    assert_eq!(row.gen_suite, "gen-m10");
    assert_eq!(
        (row.gen_text_items, row.gen_text_failing, row.gen_met),
        (Some(10_400), Some(0), Some(true))
    );
    assert_eq!(
        (row.probe_text_items, row.probe_text_failing, row.probes_met),
        (Some(415), Some(0), Some(true))
    );
    assert_eq!((row.coverage_met, row.calc_met), (None, None));
    assert!(row.exit, "{row:?}");
    let mut md = String::new();
    write_markdown(&summarize(&r), &mut md);
    assert!(
        md.contains(
            "| M10 | n/a/n/a n/a | 1690, 1.00000, 0/0 [0], text 0 failing of 10400 met | n/a [n/a] n/a | 0 [0], text 0 failing of 415, tiers F 0/0 of 415, E 0/0 of 0 met | n/a n/a | n/a | met |"
        ),
        "{md}"
    );
    // Other rows carry no text measure.
    assert_eq!(m5(&r).gen_text_items, None);
}

#[test]
fn m10_a_failing_gen_m10_text_item_fails_alone() {
    let r = m10_report(10_400, 1, 0, 0, 0);
    let row = m10(&r);
    assert_eq!((row.gen_text_failing, row.gen_met), (Some(1), Some(false)));
    assert_eq!(row.gen_text_failing_cells, ["gen-m10/abc.case.json!S!B1"]);
    assert_eq!(row.probes_met, Some(true));
    assert!(!row.exit);
}

#[test]
fn m10_gen_m10_short_of_ten_thousand_text_items_fails_alone() {
    let row = m10(&m10_report(M10_MIN_TEXT - 1, 0, 0, 0, 0));
    assert_eq!(row.gen_met, Some(false));
    assert_eq!(row.probes_met, Some(true));
    assert!(!row.exit);
    let row = m10(&m10_report(M10_MIN_TEXT, 0, 0, 0, 0));
    assert!(row.exit, "{row:?}");
}

#[test]
fn m10_a_visibly_off_gen_m10_value_cell_fails_alone() {
    let row = m10(&m10_report(10_400, 0, 1, 0, 0));
    assert_eq!((row.gen_text_failing, row.gen_met), (Some(0), Some(false)));
    assert!(!row.exit);
}

#[test]
fn m10_a_visibly_off_probe_value_cell_fails_alone() {
    let row = m10(&m10_report(10_400, 0, 0, 1, 0));
    assert_eq!((row.gen_met, row.probes_met), (Some(true), Some(false)));
    assert!(!row.exit);
}

#[test]
fn m10_a_failing_probe_text_item_fails_alone() {
    let row = m10(&m10_report(10_400, 0, 0, 0, 1));
    assert_eq!(row.probe_visibly_off, Some(0));
    assert_eq!(
        (row.gen_met, row.probe_text_failing, row.probes_met),
        (Some(true), Some(1), Some(false))
    );
    assert!(!row.exit);
}

#[test]
fn m10_without_gen_m10_is_not_applicable() {
    let mut r = m10_report(10_400, 0, 0, 0, 0);
    r.suites.retain(|s| s.name != "gen-m10");
    let row = m10(&r);
    assert_eq!((row.gen_met, row.gen_text_items), (None, None));
    assert!(!row.exit);
}
