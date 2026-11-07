---
id: T-4971
title: "Wire the M16 exit row's corpus render measure to the pivot-corpus fresh-render scores"
milestone: M16
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The M16 exit row's corpus render measure (`corpus_render_met`, T-4127) reads the pivot-corpus suites' fresh-render scores from report.json, instead of `None`.

## Context
T-4906 scores the re-render of every fresh pivot of the pivot-corpus suites (plan check (b)) and reports the counts per suite in report.json's `fresh_pivot_render` (`crates/rustsheet-eval/src/pivot_freshness/summary.rs`, `FreshRenderSuite`): `fresh`, `stale`, `fresh_unscored`, `fresh_rendered_exactly`, `render_cells`, `render_passed`, `render_visibly_off`, and the refresh checks. eval-harness.md "Pivot freshness" proposes the measure: met when the `pivot-corpus-lo` and `pivot-corpus-closedxml` rows both have `render_cells > 0` and `render_visibly_off == 0`. T-4127 adds the M16 exit row with the measure `None` pending T-4906; T-4906 did not edit milestone_exit*.rs.

On 2026-11-07 (T-4906's run): 2,463 render cells of 79 scored fresh pivots, 1,945 passing, 518 visibly off, every one owned (T-4906's Outcome).

The fetched suites are skipped when absent, so the measure must read an absent row as not measured, not as met (decision 0146 item 2(b)).

## Acceptance criteria
- [x] `corpus_render_met` reads `fresh_pivot_render` as above, absent rows reading as not measured, with tests.
- [x] The plan (docs/research/pivot-tables-2026-11-04/plan.md, M16 exit) and eval-harness.md name the measure's source.
- [x] Depends on T-4127 being merged.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing what the pivot-corpus scores (T-4906).

## Outcome
`corpus_render_met` (crates/rustsheet-eval/src/milestone_exit/pivot.rs, `corpus_render`) reads report.json's
`fresh_pivot_render`: met when both `pivot-corpus-lo` and `pivot-corpus-closedxml` have `render_cells > 0` and
`render_visibly_off == 0`; `None` (not measured, never met) when either suite's row is absent; a suite that ran with
no render cell is not met. Test `the_corpus_render_measure_reads_both_corpus_suites_fresh_render_scores` covers met,
a visibly-off cell, an empty suite and an absent suite. eval-harness.md ("Pivot freshness", "Milestone exit"),
docs/milestones.md (M16) and plan.md §6 "Gate and exit" name the source. With T-4906's scores (518 of 2,463 render
cells visibly off) the measure reads not met where the corpora are fetched, and not measured in PR CI, which does not
fetch them.
