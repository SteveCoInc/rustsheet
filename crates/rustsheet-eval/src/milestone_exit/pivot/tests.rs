use super::*;
use crate::catalog::Catalog;
use crate::closeness::Closeness;
use crate::milestone_exit::{summarize, write_markdown, ExitRow};
use crate::pivot_features::{PivotCellTally, PivotFeatures};
use crate::pivot_freshness::{FreshRenderSuite, PivotFreshness};
use crate::pivot_scoring::PivotLayout;
use crate::probe_milestones::MilestoneRow;
use crate::report::SuiteSummary;
use crate::runner::{CaseResult, CaseStatus};

fn feature(id: &str, stage: &str, web: &str, cells: usize, passed: usize) -> FeatureRow {
    FeatureRow {
        id: id.into(),
        web: web.into(),
        scope: "in".into(),
        stage: stage.into(),
        status: "todo".into(),
        cells,
        passed,
        ..FeatureRow::default()
    }
}

fn suite(name: &str, cells: usize, visible: usize) -> SuiteSummary {
    SuiteSummary {
        name: name.into(),
        cells,
        cells_passed: cells - visible,
        closeness: Closeness {
            visible_other: visible,
            ..Closeness::default()
        },
        ..SuiteSummary::default()
    }
}

fn pivot_case(id: &str, suite: &str) -> CaseResult {
    let mut c = CaseResult::empty(id, suite, CaseStatus::Pass);
    c.pivot_cells = PivotCellTally {
        cells: 40,
        passed: 40,
    };
    c
}

/// A run where every M16 measure holds: two stages at the floor (a PV2 row the web cannot render
/// is below it but does not block), an out-of-scope row with no cell, every suite clean, the M16
/// probe row clean, and sidecar pivots with no layout entry.
fn met() -> RunReport {
    let mut r = RunReport::build("t", &[], &[], &Catalog::from_functions([]));
    r.pivot_features = PivotFeatures {
        rows: vec![
            feature("PV-0001", "PV1", "yes", 30, 30),
            feature("PV-0002", "PV2", "unknown", 45, 45),
            feature("PV-0003", "PV2", "no", 0, 0),
            FeatureRow {
                scope: "out (decision 0274)".into(),
                ..feature("PV-0004", "out", "unknown", 0, 0)
            },
        ],
        ..PivotFeatures::default()
    };
    r.suites = vec![
        suite(GEN_SUITE, 500, 0),
        suite(CORPUS_SUITES[0], 80, 0),
        suite(CORPUS_SUITES[1], 20, 0),
    ];
    let mut corpus =
        CaseResult::empty("pivot-corpus-lo/a.xlsx", CORPUS_SUITES[0], CaseStatus::Pass);
    corpus.pivot_freshness = vec![PivotFreshness {
        sheet: "S".into(),
        name: "Pv".into(),
        location: "A3:B4".into(),
        freshness: Freshness::Stale,
        compared: 4,
        differing: 1,
        reason: None,
        render: None,
        render_unscored: None,
        refresh: None,
    }];
    r.cases = vec![
        pivot_case("gen-pivot/a", GEN_SUITE),
        pivot_case("probes/p", "probes"),
        corpus,
    ];
    r.probe_milestones = vec![MilestoneRow {
        milestone: "M16".into(),
        cells: 40,
        ..MilestoneRow::default()
    }];
    r
}

fn m16(r: &RunReport) -> ExitRow {
    summarize(r)
        .into_iter()
        .find(|row| row.milestone == "M16")
        .unwrap_or_default()
}

fn layout(verdict: LayoutVerdict) -> PivotLayout {
    PivotLayout {
        sheet: "S".into(),
        name: "Pv".into(),
        excel_location: "A3:B4".into(),
        engine_location: None,
        excel_lines: (2, 1),
        engine_lines: None,
        verdict,
    }
}

#[test]
fn every_measure_but_the_unmeasured_corpus_render_holds() {
    let r = met();
    let row = m16(&r);
    assert_eq!(row.gen_suite, GEN_SUITE);
    assert_eq!(row.gen_met, Some(true));
    let more: Vec<_> = row
        .more_suites
        .iter()
        .map(|s| (s.name.as_str(), s.met))
        .collect();
    assert_eq!(
        more,
        vec![
            (CORPUS_SUITES[0], Some(true)),
            (CORPUS_SUITES[1], Some(true))
        ]
    );
    assert_eq!(row.probes_met, Some(true));
    let p = row.pivot.clone().unwrap_or_default();
    assert_eq!(p.stages.len(), 2, "{p:?}");
    let pv2 = &p.stages[1];
    assert_eq!(
        (
            pv2.rows,
            pv2.web_cannot_render,
            pv2.evidenced,
            pv2.remaining
        ),
        (2, 1, 1, 0)
    );
    assert_eq!(
        (p.evidence_met, p.owners_met, p.layout_met),
        (Some(true), Some(true), Some(true))
    );
    assert_eq!((p.corpus_fresh, p.corpus_stale), (Some(0), Some(1)));
    // No `fresh_pivot_render` row for the corpus suites: plan check (b) is not measured (T-4971).
    assert_eq!(p.corpus_render_met, None);
    assert!(!p.met && !row.exit, "{row:?}");
    assert!(all_met(&PivotExit {
        corpus_render_met: Some(true),
        ..p.clone()
    }));
    assert!(!all_met(&PivotExit {
        corpus_render_met: Some(false),
        ..p
    }));
    let mut md = String::new();
    write_markdown(&summarize(&r), &mut md);
    assert!(md.contains("| PV2 | 2 | 1 | 1/1 | met | 0 | met |"), "{md}");
    assert!(
        md.contains("M16 evidence met, owners met; layouts met (0 missing, 0 misplaced, 0 with lines not emitted, 0 with other lines); corpus render not measured; pivot-corpus pivots 0 fresh, 1 stale (shown, not a criterion); probes 0 visibly off counted, 0 left out"),
        "{md}"
    );
    assert!(
        md.contains(
            "| M16 | n/a/n/a n/a | gen-pivot: 500, 1.00000, 0/0 [0] met; pivot-corpus-lo: 80"
        ),
        "{md}"
    );
}

#[test]
fn a_row_below_the_floor_fails_evidence_and_with_no_owner_fails_owners() {
    let mut r = met();
    r.pivot_features.rows[0].cells = 29;
    r.pivot_features.rows[0].passed = 29;
    let row = m16(&r);
    let p = row.pivot.clone().unwrap_or_default();
    assert_eq!(p.stages[0].below_floor, vec!["PV-0001"]);
    assert_eq!(p.stages[0].evidence_met, Some(false));
    assert_eq!(p.stages[0].unowned, vec!["PV-0001"]);
    assert_eq!(p.stages[0].owners_met, Some(false));
    // The other stage has nothing remaining: its owners hold.
    assert_eq!(p.stages[1].owners_met, Some(true));
    assert_eq!((p.evidence_met, p.owners_met), (Some(false), Some(false)));
    assert!(!row.exit);
    let mut md = String::new();
    write_markdown(&summarize(&r), &mut md);
    assert!(
        md.contains("| PV1 | 1 | 0 | 0/1 | not met | 1 | not met |"),
        "{md}"
    );
    assert!(
        md.contains("PV1 remaining rows with no open M16 owner (1 of 1): PV-0001."),
        "{md}"
    );
    assert!(md.contains("M16 evidence not met, owners not met;"), "{md}");
}

#[test]
fn a_remaining_row_an_open_m16_task_owns_meets_owners() {
    let mut r = met();
    r.pivot_features.rows[0].cells = 29;
    r.pivot_features.rows[0].owner = "T-0001".into();
    let p = m16(&r).pivot.unwrap_or_default();
    assert_eq!((p.stages[0].remaining, p.stages[0].unowned.len()), (1, 0));
    assert_eq!(
        (p.stages[0].owners_met, p.owners_met),
        (Some(true), Some(true))
    );
    // Evidence still fails: an owner is not evidence.
    assert_eq!(p.evidence_met, Some(false));
}

#[test]
fn owners_are_per_stage_and_the_milestone_needs_every_stage() {
    // PV1's remaining row is owned, PV2's is not. A done or missing owner cannot reach here:
    // `scripts/tasks.py validate` refuses it (scripts/test_tasks.py).
    let mut r = met();
    r.pivot_features.rows[0].cells = 29;
    r.pivot_features.rows[0].owner = "T-0001".into();
    r.pivot_features.rows[1].passed = 44;
    let p = m16(&r).pivot.unwrap_or_default();
    assert_eq!(
        (p.stages[0].owners_met, p.stages[1].owners_met),
        (Some(true), Some(false))
    );
    assert_eq!(p.stages[1].unowned, vec!["PV-0002"]);
    assert_eq!(p.owners_met, Some(false));
}

#[test]
fn an_out_of_scope_row_is_never_remaining_or_unowned() {
    // PV-0004 is `out (decision 0274)` with no cell and no owner.
    let r = met();
    assert!(r.pivot_features.rows[3].owner.is_empty());
    let p = m16(&r).pivot.unwrap_or_default();
    assert!(p
        .stages
        .iter()
        .all(|s| s.stage != "out" && s.unowned.is_empty()));
    assert_eq!(p.owners_met, Some(true));
}

#[test]
fn a_failing_cell_at_the_floor_is_a_remaining_row() {
    let mut r = met();
    r.pivot_features.rows[1].passed = 44;
    let p = m16(&r).pivot.unwrap_or_default();
    assert_eq!(p.stages[1].evidence_met, Some(true));
    assert_eq!(
        (p.stages[1].remaining, p.stages[1].owners_met),
        (1, Some(false))
    );
}

#[test]
fn a_visibly_off_suite_cell_fails_its_suite() {
    let mut r = met();
    r.suites[0] = suite(GEN_SUITE, 500, 3);
    let row = m16(&r);
    assert_eq!((row.gen_visibly_off, row.gen_met), (Some(3), Some(false)));
    assert!(!row.exit);
}

#[test]
fn every_visibly_off_probe_cell_counts_not_only_r1c() {
    let mut r = met();
    r.probe_milestones[0].visibly_off = 1;
    let row = m16(&r);
    assert_eq!((row.probe_visibly_off, row.probe_r1c_off), (Some(1), None));
    assert_eq!(row.probes_met, Some(false));
    assert!(!row.exit);
}

#[test]
fn r3_passes_and_m14_owned_probe_cells_do_not_count_but_phase1b_cells_do() {
    use crate::r1c::{R1c, R1cCells};
    let mut r = met();
    let cells = ["probes/p!S!A1", "probes/p!S!A2", "probes/p!S!A3"];
    r.probe_milestones[0].visibly_off = 2;
    r.probe_milestones[0].visibly_off_cells = cells[..2].iter().map(|c| (*c).into()).collect();
    r.r1c = R1cCells::from_classes([(cells[0], R1c::R3Pass), (cells[1], R1c::M14Owned)]);
    let row = m16(&r);
    let p = row.pivot.clone().unwrap_or_default();
    assert_eq!((p.probe_off, p.probe_left_out), (Some(0), Some(2)));
    assert_eq!(
        (row.probe_visibly_off, row.probes_met),
        (Some(2), Some(true))
    );
    // A Phase 1b cell is R1(c)'s leave-out, not M16's: it counts.
    r.probe_milestones[0].visibly_off = 3;
    r.probe_milestones[0].visibly_off_cells = cells.iter().map(|c| (*c).into()).collect();
    r.r1c = R1cCells::from_classes([
        (cells[0], R1c::R3Pass),
        (cells[1], R1c::M14Owned),
        (cells[2], R1c::Phase1b),
    ]);
    let row = m16(&r);
    let p = row.pivot.clone().unwrap_or_default();
    assert_eq!((p.probe_off, p.probe_left_out), (Some(1), Some(2)));
    assert_eq!(row.probes_met, Some(false));
}

#[test]
fn a_missing_pivot_fails_layout_and_unemitted_lines_leave_it_unmeasured() {
    let mut r = met();
    r.cases[1].pivot_layout = vec![layout(LayoutVerdict::LinesNotEmitted)];
    let p = m16(&r).pivot.unwrap_or_default();
    assert_eq!((p.layout_lines_not_emitted, p.layout_met), (1, None));
    r.cases[1].pivot_layout.push(layout(LayoutVerdict::Missing));
    let row = m16(&r);
    let p = row.pivot.clone().unwrap_or_default();
    assert_eq!(p.layout_not_excel, vec!["probes/p!S!Pv"]);
    assert_eq!(p.layout_met, Some(false));
    assert!(!row.exit);
}

#[test]
fn absent_inputs_are_not_measured_never_met() {
    let mut r = met();
    // The fetched corpus suites did not run (CI), and neither did gen-pivot.
    r.suites.clear();
    r.cases.retain(|c| c.suite == "probes");
    let row = m16(&r);
    assert_eq!(row.gen_met, None);
    assert!(row.more_suites.iter().all(|s| s.met.is_none()), "{row:?}");
    let p = row.pivot.clone().unwrap_or_default();
    assert_eq!((p.corpus_fresh, p.corpus_stale), (None, None));
    assert!(!row.exit);
    let mut md = String::new();
    write_markdown(&summarize(&r), &mut md);
    assert!(
        md.contains("gen-pivot: n/a; pivot-corpus-lo: n/a; pivot-corpus-closedxml: n/a"),
        "{md}"
    );
    // No catalog, no probe row and no sidecar pivot: nothing is measured.
    let empty = m16(&RunReport::build(
        "t",
        &[],
        &[],
        &Catalog::from_functions([]),
    ));
    let p = empty.pivot.clone().unwrap_or_default();
    assert!(p.stages.is_empty());
    assert_eq!(
        (p.evidence_met, p.owners_met, p.layout_met),
        (None, None, None)
    );
    assert_eq!(empty.probes_met, None);
    assert!(!empty.exit);
}

#[test]
fn only_m16_carries_the_pivot_measures() {
    let rows = summarize(&met());
    assert!(rows
        .iter()
        .filter(|r| r.milestone != "M16")
        .all(|r| r.pivot.is_none()));
}

fn fresh_render(suite: &str, cells: usize, visibly_off: usize) -> FreshRenderSuite {
    FreshRenderSuite {
        suite: suite.into(),
        render_cells: cells,
        render_passed: cells - visibly_off,
        render_visibly_off: visibly_off,
        ..FreshRenderSuite::default()
    }
}

#[test]
fn the_corpus_render_measure_reads_both_corpus_suites_fresh_render_scores() {
    let mut r = met();
    // Both suites scored, none visibly off: met.
    r.fresh_pivot_render = vec![
        fresh_render(CORPUS_SUITES[0], 40, 0),
        fresh_render(CORPUS_SUITES[1], 9, 0),
    ];
    assert_eq!(measure(&r).corpus_render_met, Some(true));
    // A visibly-off render cell in either suite: not met.
    r.fresh_pivot_render[1] = fresh_render(CORPUS_SUITES[1], 9, 2);
    assert_eq!(measure(&r).corpus_render_met, Some(false));
    // A suite that ran but scored no render cell is not evidence: not met.
    r.fresh_pivot_render[1] = fresh_render(CORPUS_SUITES[1], 0, 0);
    assert_eq!(measure(&r).corpus_render_met, Some(false));
    // A fetched suite that is absent measures nothing: not measured, never met.
    r.fresh_pivot_render.truncate(1);
    assert_eq!(measure(&r).corpus_render_met, None);
    assert!(!measure(&r).met);
}
