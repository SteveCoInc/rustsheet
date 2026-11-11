//! The run's pivot freshness, re-render and refresh counts by suite: report.json's
//! `fresh_pivot_render` and report.md's "Pivot freshness" section (T-4131, T-4906).

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use super::{Freshness, PivotFreshness, RefreshOutcome};
use crate::runner::CaseResult;

/// One suite's pivots: how many are fresh, how their re-render scores (check (b)) and what the
/// refresh of the `refreshOnLoad` ones found (check (c)). report.json's `fresh_pivot_render`
/// holds one per suite with a classified pivot; the M16 exit row's corpus render measure reads
/// the `pivot-corpus-*` rows (met when `render_cells > 0` and `render_visibly_off == 0`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FreshRenderSuite {
    pub suite: String,
    /// Pivots whose saved records give every value Excel saved.
    pub fresh: usize,
    /// The other classified pivots, listed and never scored for render.
    pub stale: usize,
    /// Fresh pivots whose render is reported and not scored: saved by a non-en-US Excel
    /// (decision 0030), or scored by check (c) against Excel's on-open render instead (T-4972).
    /// The render counts below leave them out.
    pub fresh_unscored: usize,
    /// Fresh pivots whose re-render gives every cell of Excel's saved area.
    pub fresh_rendered_exactly: usize,
    /// Scored fresh pivots whose engine `rowItems` and `colItems` are the ones Excel saved.
    pub fresh_lines_equal: usize,
    /// The scored render cells of the fresh pivots (every cell of each saved area, labels
    /// included, and any cell the engine adds outside it).
    pub render_cells: usize,
    /// Of those, the ones that pass.
    pub render_passed: usize,
    /// Of those, the ones visibly off (decision 0083).
    pub render_visibly_off: usize,
    /// `refreshOnLoad` pivots whose refresh was compared with Excel's saved cells, and of those
    /// the ones that matched every cell.
    pub refresh_compared: usize,
    pub refresh_exact: usize,
    /// `refreshOnLoad` pivots whose saved cells are no evidence of the refresh.
    pub refresh_not_comparable: usize,
    /// `refreshOnLoad` pivots the engine did not refresh.
    pub refresh_failed: usize,
    /// `refreshOnLoad` pivots checked against Excel's render on open, an oracle sidecar's area
    /// (T-4972; counted in `refresh_compared` or `refresh_failed` too), and their scored cells.
    #[serde(default)]
    pub refresh_on_open: usize,
    #[serde(default)]
    pub refresh_cells: usize,
    /// Of those, the ones that pass.
    #[serde(default)]
    pub refresh_passed: usize,
}

/// Every classified pivot of `cases` with its case.
fn rows(cases: &[CaseResult]) -> Vec<(&CaseResult, &PivotFreshness)> {
    (cases.iter())
        .flat_map(|c| c.pivot_freshness.iter().map(move |p| (c, p)))
        .collect()
}

/// The counts by suite, in suite order.
#[must_use]
pub(crate) fn summarize(cases: &[CaseResult]) -> Vec<FreshRenderSuite> {
    let mut by_suite: BTreeMap<&str, FreshRenderSuite> = BTreeMap::new();
    for (case, p) in rows(cases) {
        let s = by_suite
            .entry(case.suite.as_str())
            .or_insert_with(|| FreshRenderSuite {
                suite: case.suite.clone(),
                ..FreshRenderSuite::default()
            });
        match (p.freshness, &p.render) {
            (Freshness::Fresh, Some(_)) if p.render_unscored.is_some() => {
                s.fresh += 1;
                s.fresh_unscored += 1;
            }
            (Freshness::Fresh, Some(render)) => {
                s.fresh += 1;
                s.fresh_rendered_exactly += usize::from(render.differing == 0);
                s.fresh_lines_equal += usize::from(render.lines_equal == Some(true));
                s.render_cells += render.compared;
                s.render_passed += render.compared - render.differing;
                s.render_visibly_off += render.visibly_off;
            }
            (Freshness::Fresh, None) => s.fresh += 1,
            (Freshness::Stale, _) => s.stale += 1,
        }
        if let Some(r) = p.refresh.as_ref().filter(|r| r.on_open) {
            s.refresh_on_open += 1;
            s.refresh_cells += r.compared;
            s.refresh_passed += r.compared - r.differing;
        }
        match p.refresh.as_ref().map(|r| (r.outcome, r.differing)) {
            Some((RefreshOutcome::Compared, differing)) => {
                s.refresh_compared += 1;
                s.refresh_exact += usize::from(differing == 0);
            }
            Some((RefreshOutcome::NotComparable, _)) => s.refresh_not_comparable += 1,
            Some((RefreshOutcome::Failed, _)) => s.refresh_failed += 1,
            None => {}
        }
    }
    by_suite.into_values().collect()
}

/// A table cell's text: `|` escaped.
fn cell(text: Option<&str>) -> String {
    text.unwrap_or("-").replace('|', "\\|")
}

/// report.md's "Pivot freshness" section: the counts by suite, every stale pivot, every fresh
/// pivot whose render is off, and every refresh check.
pub(crate) fn write_markdown(cases: &[CaseResult], md: &mut String) {
    let rows = rows(cases);
    if rows.is_empty() {
        return;
    }
    let _ = writeln!(
        md,
        "## Pivot freshness\n\nA pivot of an Excel-saved workbook is fresh when its saved cache \
         records give every value Excel saved, at the cells Excel's own `rowItems`/`colItems` \
         place them; the renderer plays no part (T-4906). A fresh pivot's re-render is scored, \
         every cell of its saved area against Excel's (check (b)); a stale one is listed here and \
         never scored for render. A `refreshOnLoad` pivot is also refreshed from its source and \
         compared, where its saved cells are a render of the records the source still holds, or \
         scored against Excel's render on open where an oracle sidecar captured it (check (c)).\n"
    );
    let _ = writeln!(
        md,
        "| suite | fresh | stale | fresh, render unscored | fresh rendered exactly | Excel's lines \
         | render cells | passed | visibly off | refresh compared | exact | not comparable | failed \
         | on open | refresh cells | passed |\n|---|---|---|---|---|---|---|---|---|---|---|---|---|---|\
         ---|---|"
    );
    for s in summarize(cases) {
        let _ = writeln!(
            md,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            s.suite,
            s.fresh,
            s.stale,
            s.fresh_unscored,
            s.fresh_rendered_exactly,
            s.fresh_lines_equal,
            s.render_cells,
            s.render_passed,
            s.render_visibly_off,
            s.refresh_compared,
            s.refresh_exact,
            s.refresh_not_comparable,
            s.refresh_failed,
            s.refresh_on_open,
            s.refresh_cells,
            s.refresh_passed
        );
    }
    let _ = writeln!(
        md,
        "\nStale pivots:\n\n| case | pivot | location | differing / compared values | reason |\n\
         |---|---|---|---|---|"
    );
    for (case, p) in rows.iter().filter(|(_, p)| p.freshness == Freshness::Stale) {
        let _ = writeln!(
            md,
            "| {} | {}!{} | {} | {} / {} | {} |",
            case.id,
            p.sheet,
            p.name,
            p.location,
            p.differing,
            p.compared,
            cell(p.reason.as_deref())
        );
    }
    let _ = writeln!(
        md,
        "\nFresh pivots whose re-render is off (scored), or not scored and why:\n\n| case | pivot \
         | location | differing / compared cells | first |\n|---|---|---|---|---|"
    );
    for (case, p) in &rows {
        let Some(render) = p
            .render
            .as_ref()
            .filter(|r| r.differing > 0 || r.compared == 0)
        else {
            continue;
        };
        if p.freshness == Freshness::Fresh {
            let _ = writeln!(
                md,
                "| {} | {}!{} | {} | {} / {} | {} |",
                case.id,
                p.sheet,
                p.name,
                p.location,
                render.differing,
                render.compared,
                cell(p.render_unscored.as_deref().or(render.first.as_deref()))
            );
        }
    }
    let refreshed: Vec<_> = rows.iter().filter(|(_, p)| p.refresh.is_some()).collect();
    if !refreshed.is_empty() {
        let _ = writeln!(
            md,
            "\nRefresh of `refreshOnLoad` pivots (against Excel's saved cells, or its render on \
             open, scored):\n\n| case | pivot | outcome | evidence | differing / compared cells \
             | first |\n|---|---|---|---|---|---|"
        );
        for (case, p) in refreshed {
            let Some(r) = &p.refresh else { continue };
            let outcome = match r.outcome {
                RefreshOutcome::Compared => "compared",
                RefreshOutcome::NotComparable => "not comparable",
                RefreshOutcome::Failed => "failed",
            };
            let _ = writeln!(
                md,
                "| {} | {}!{} | {outcome} | {} | {} / {} | {} |",
                case.id,
                p.sheet,
                p.name,
                if r.on_open { "on open" } else { "saved" },
                r.differing,
                r.compared,
                cell(r.first.as_deref())
            );
        }
    }
    let _ = writeln!(md);
}
