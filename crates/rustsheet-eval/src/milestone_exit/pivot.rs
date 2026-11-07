//! M16's measures beyond the suite and probe columns (T-4127, decision 0274 item 6; docs/research/
//! pivot-tables-2026-11-04/plan.md §6 "Gate and exit"). Per catalog stage (`PV1`, `PV2`...):
//!
//! - **evidence**: every in-scope row of eval/catalog/pivot-features.tsv has at least
//!   [`MIN_ORACLE_CELLS`] Excel-evidence pivot cells, counted as the report's "Pivot features" rows
//!   count them (every scored sidecar pivot-area cell of each case naming the id, its edge pools'
//!   cases included). A row Excel for the web cannot render (`web` = `no`) is M14's and never
//!   blocks (decision 0274 item 5); it is counted apart.
//! - **owners** (T-4952): every remaining row (below the floor, or with a failing cell) is owned.
//!   A row is owned when the catalog's hand-filled `owner` column names a task;
//!   `scripts/tasks.py validate` (`make hygiene`) fails while one names a task that is not an
//!   open M16 task, so a done or missing task never owns a row on main. A row with an `out`
//!   scope is owned by the decision its scope cites (`out (decision 0274)`): it is not in
//!   scope, so it is never remaining. The remaining rows no task owns are listed by id.
//!
//! Milestone-wide, since a case's cells are not split by stage:
//!
//! - **layout**: no sidecar pivot is missing from the engine or at another location
//!   (`pivot_scoring`); a pivot whose `rowItems`/`colItems` the engine does not emit yet leaves the
//!   criterion not measured.
//! - **freshness**: the pivot-corpus pivots fresh and stale (`pivot_freshness`), shown only: a stale
//!   pivot is reported, never scored (plan §6 check (b)).
//! - **corpus render** (plan §6 check (b)): every fresh corpus pivot re-rendered from its saved cache
//!   reproduces Excel's saved cells. `pivot_freshness` decides "fresh" by the engine's own render, so
//!   a fresh file the engine renders wrongly is called stale and nothing is scored: the check is not
//!   measured (`None`) until freshness is decided without the renderer (T-4906), and M16 waits on it.
//!
//! The probe measure leaves out the visibly-off cells decision 0188 calls passing or never blocking:
//! R3 cells where the engine returns the math and desktop-owned (M14) cells ([`probe_off`]).
//!
//! The `pivots`, `gen-pivot` and fresh pivot-corpus cells are the row's suite and probe columns
//! (`criteria("M16")`): the pivot probes are the `probes` cases tagged M16. Every input is a
//! section the report already computes; a missing one is `None`, never met.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::pivot_features::FeatureRow;
use crate::pivot_freshness::Freshness;
use crate::pivot_scoring::LayoutVerdict;
use crate::probe_milestones::MilestoneRow;
use crate::r1c::R1c;
use crate::report::RunReport;

/// The milestone these measures belong to.
pub(super) const MILESTONE: &str = "M16";
/// M16's generated suite (T-4132).
pub const GEN_SUITE: &str = "gen-pivot";
/// The fetched pivot-corpus suites (T-4131); `gate = false`, so a run may lack them.
pub const CORPUS_SUITES: [&str; 2] = ["pivot-corpus-lo", "pivot-corpus-closedxml"];
/// Fewest oracle cells an in-scope catalog row needs (decision 0274 item 6).
pub const MIN_ORACLE_CELLS: usize = 30;
/// The `web` value of a row Excel for the web cannot render (decision 0274 item 5).
const WEB_CANNOT_RENDER: &str = "no";

/// One catalog stage's measures.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PivotStageExit {
    /// `PV1`, `PV2`...
    pub stage: String,
    /// In-scope catalog rows of the stage.
    pub rows: usize,
    /// Of those, the rows Excel for the web cannot render: M14's, not blocking.
    pub web_cannot_render: usize,
    /// Of the blocking rows, those with at least [`MIN_ORACLE_CELLS`] oracle cells.
    pub evidenced: usize,
    /// The blocking rows below the floor, by id.
    #[serde(default)]
    pub below_floor: Vec<String>,
    /// No blocking row is below the floor.
    pub evidence_met: Option<bool>,
    /// Blocking rows below the floor or with a failing oracle cell: those the owner criterion reads.
    pub remaining: usize,
    /// The remaining rows with no `owner`, by id.
    #[serde(default)]
    pub unowned: Vec<String>,
    /// Every remaining row is owned (`unowned` is empty).
    pub owners_met: Option<bool>,
}

/// M16's catalog, layout and freshness measures.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PivotExit {
    /// One per stage of the catalog's in-scope rows, in stage order; empty without a catalog.
    pub stages: Vec<PivotStageExit>,
    /// Every stage's evidence is met; `None` without a catalog.
    pub evidence_met: Option<bool>,
    /// Every stage's owners are met; `None` without a catalog.
    pub owners_met: Option<bool>,
    /// Sidecar pivots the engine does not have.
    pub layout_missing: usize,
    /// Sidecar pivots the engine places elsewhere.
    pub layout_location_differs: usize,
    /// Sidecar pivots at Excel's location whose `rowItems`/`colItems` the engine does not emit.
    pub layout_lines_not_emitted: usize,
    /// Sidecar pivots at Excel's location whose saved `rowItems`/`colItems` are not Excel's
    /// (engine-export cases, T-4177).
    pub layout_lines_differ: usize,
    /// The missing, misplaced and line-differing pivots as `case!Sheet!Name`.
    #[serde(default)]
    pub layout_not_excel: Vec<String>,
    /// No sidecar pivot is missing or misplaced and every one's lines are compared; `None` while
    /// lines are not emitted or no case has a sidecar pivot.
    pub layout_met: Option<bool>,
    /// Fresh pivots of the pivot-corpus suites; `None` when neither ran.
    pub corpus_fresh: Option<usize>,
    /// Stale pivots of the pivot-corpus suites.
    pub corpus_stale: Option<usize>,
    /// Plan check (b): the fresh corpus pivots' re-render reproduces Excel's saved cells. Always
    /// `None` (not measured) until freshness is decided without the engine's renderer (T-4906).
    pub corpus_render_met: Option<bool>,
    /// Of the M16 probe row's visibly-off cells, those the measure counts: all but R3 passes and
    /// M14-owned cells ([`probe_off`]); `None` without a probe row.
    pub probe_off: Option<usize>,
    /// The visibly-off probe cells left out: R3 passes and M14-owned cells.
    pub probe_left_out: Option<usize>,
    /// Evidence, owners, layout and corpus render are met (the suite and probe columns are the
    /// row's).
    pub met: bool,
}

/// The M16 probe row's visibly-off cells as (counted, left out): an R3 cell where the engine
/// returns the math passes (docs/milestones.md "Phase 1 exit", decision 0188 R3) and an M14-owned
/// cell never blocks (decision 0274 item 5); every other class counts, Phase 1b evidence included,
/// since decision 0188 R1(c) is not M16's reading.
pub(super) fn probe_off(report: &RunReport, row: &MilestoneRow) -> (usize, usize) {
    let left_out = row
        .visibly_off_cells
        .iter()
        .filter(|c| matches!(report.r1c.class(c), R1c::R3Pass | R1c::M14Owned))
        .count();
    (row.visibly_off.saturating_sub(left_out), left_out)
}

/// Whether evidence, owners, layout and corpus render are all met.
fn all_met(p: &PivotExit) -> bool {
    [
        p.evidence_met,
        p.owners_met,
        p.layout_met,
        p.corpus_render_met,
    ]
    .iter()
    .all(|v| *v == Some(true))
}

/// The stage measures of the report's pivot feature rows.
fn stages(rows: &[FeatureRow]) -> Vec<PivotStageExit> {
    let mut by_stage: BTreeMap<&str, PivotStageExit> = BTreeMap::new();
    for r in rows.iter().filter(|r| r.in_scope()) {
        let s = by_stage
            .entry(r.stage.as_str())
            .or_insert_with(|| PivotStageExit {
                stage: r.stage.clone(),
                ..PivotStageExit::default()
            });
        s.rows += 1;
        if r.web == WEB_CANNOT_RENDER {
            s.web_cannot_render += 1;
            continue;
        }
        if r.cells >= MIN_ORACLE_CELLS {
            s.evidenced += 1;
        } else {
            s.below_floor.push(r.id.clone());
        }
        if r.cells < MIN_ORACLE_CELLS || r.passed < r.cells {
            s.remaining += 1;
            if r.owner.is_empty() {
                s.unowned.push(r.id.clone());
            }
        }
    }
    by_stage
        .into_values()
        .map(|mut s| {
            s.evidence_met = Some(s.below_floor.is_empty());
            s.owners_met = Some(s.unowned.is_empty());
            s
        })
        .collect()
}

/// Every stage's verdict, `None` when there is no stage or one is unmeasured.
fn all(stages: &[PivotStageExit], met: impl Fn(&PivotStageExit) -> Option<bool>) -> Option<bool> {
    if stages.is_empty() {
        return None;
    }
    let verdicts: Vec<Option<bool>> = stages.iter().map(met).collect();
    if verdicts.contains(&Some(false)) {
        Some(false)
    } else if verdicts.contains(&None) {
        None
    } else {
        Some(true)
    }
}

/// M16's measures from `report`.
pub(super) fn measure(report: &RunReport) -> PivotExit {
    let stages = stages(&report.pivot_features.rows);
    let mut out = PivotExit {
        evidence_met: all(&stages, |s| s.evidence_met),
        owners_met: all(&stages, |s| s.owners_met),
        stages,
        ..PivotExit::default()
    };
    for case in &report.cases {
        for p in &case.pivot_layout {
            match p.verdict {
                LayoutVerdict::Missing => out.layout_missing += 1,
                LayoutVerdict::LocationDiffers => out.layout_location_differs += 1,
                LayoutVerdict::LinesDiffer => out.layout_lines_differ += 1,
                LayoutVerdict::LinesNotEmitted => {
                    out.layout_lines_not_emitted += 1;
                    continue;
                }
            }
            out.layout_not_excel
                .push(format!("{}!{}!{}", case.id, p.sheet, p.name));
        }
    }
    // `pivot_layout` lists only the pivots whose layout is not Excel's, so a run with sidecar pivot
    // cells and no such entry matched every one.
    let any_pivot = report.cases.iter().any(|c| c.pivot_cells.cells > 0);
    out.layout_met = if !out.layout_not_excel.is_empty() {
        Some(false)
    } else if out.layout_lines_not_emitted > 0 || !any_pivot {
        None
    } else {
        Some(true)
    };
    let corpus = || {
        report
            .cases
            .iter()
            .filter(|c| CORPUS_SUITES.contains(&c.suite.as_str()))
    };
    if corpus().next().is_some() {
        let pivots: Vec<Freshness> = corpus()
            .flat_map(|c| c.pivot_freshness.iter().map(|p| p.freshness))
            .collect();
        let fresh = pivots.iter().filter(|f| **f == Freshness::Fresh).count();
        out.corpus_fresh = Some(fresh);
        out.corpus_stale = Some(pivots.len() - fresh);
    }
    // Plan check (b) is not scored until T-4906 decides freshness without the renderer.
    out.corpus_render_met = None;
    out.met = all_met(&out);
    out
}

fn verdict(met: Option<bool>) -> &'static str {
    match met {
        Some(true) => "met",
        Some(false) => "not met",
        None => "not measured",
    }
}

fn count(n: Option<usize>) -> String {
    n.map_or_else(|| "n/a".to_string(), |n| n.to_string())
}

/// report.md's M16 lines below the milestone exit table.
pub(super) fn write_markdown(p: &PivotExit, md: &mut String) {
    let _ = writeln!(
        md,
        "\nM16 catalog measures (decision 0274): each in-scope row of the pivot catalog needs at least \
         {MIN_ORACLE_CELLS} oracle cells (a row Excel for the web cannot render is M14's and does not \
         block), and each remaining row (below the floor or with a failing cell) an owner: its \
         catalog `owner` an open M16 task (an `out` row is its scope decision's and never remains). \
         Layouts: no sidecar pivot missing or \
         misplaced, not measured while the engine emits no `rowItems`/`colItems`. Corpus render \
         (plan check (b), fresh pivots re-rendered against Excel's saved cells): not measured until \
         freshness is decided without the engine's renderer (T-4906).\n"
    );
    let _ = writeln!(
        md,
        "| stage | in scope | web cannot render | at the floor | evidence | remaining | owners |"
    );
    let _ = writeln!(md, "|---|---|---|---|---|---|---|");
    for s in &p.stages {
        let _ = writeln!(
            md,
            "| {} | {} | {} | {}/{} | {} | {} | {} |",
            s.stage,
            s.rows,
            s.web_cannot_render,
            s.evidenced,
            s.rows - s.web_cannot_render,
            verdict(s.evidence_met),
            s.remaining,
            verdict(s.owners_met)
        );
    }
    for s in p.stages.iter().filter(|s| !s.unowned.is_empty()) {
        let _ = writeln!(
            md,
            "\n{} remaining rows with no open M16 owner ({} of {}): {}.",
            s.stage,
            s.unowned.len(),
            s.remaining,
            s.unowned.join(", ")
        );
    }
    let _ = writeln!(
        md,
        "\nM16 evidence {}, owners {}; layouts {} ({} missing, {} misplaced, {} with lines not \
         emitted, {} with other lines); corpus render {}; pivot-corpus pivots {} fresh, {} stale (shown, not a criterion); \
         probes {} visibly off counted, {} left out (R3 passes, M14-owned).",
        verdict(p.evidence_met),
        verdict(p.owners_met),
        verdict(p.layout_met),
        p.layout_missing,
        p.layout_location_differs,
        p.layout_lines_not_emitted,
        p.layout_lines_differ,
        verdict(p.corpus_render_met),
        count(p.corpus_fresh),
        count(p.corpus_stale),
        count(p.probe_off),
        count(p.probe_left_out)
    );
}

#[cfg(test)]
mod tests;
