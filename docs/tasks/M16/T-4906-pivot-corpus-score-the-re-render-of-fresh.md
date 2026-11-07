---
id: T-4906
title: "pivot-corpus: score the re-render of fresh pivots and the refreshOnLoad refresh (plan checks b and c)"
milestone: M16
lane: harness
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
The pivot-corpus suites score plan check (b), the re-render of each fresh pivot from its saved cache against
Excel's saved cells and `rowItems`/`colItems`, and check (c), a full refresh from the source range of a
`refreshOnLoad` pivot; stale pivots stay reported and unscored for render.

## Context
- T-4131 added `pivot-corpus-lo` and `pivot-corpus-closedxml` (fetched, `gate = false`) and classifies
  every pivot of a trusted Excel-saved workbook fresh or stale (`crates/rustsheet-eval/src/pivot_freshness.rs`,
  report.md "Pivot freshness", report.json `pivot_freshness`). On 2026-11-07: 46 fresh and 79 stale of
  125 classified in the two suites; calipers 13 fresh, 6 stale; poi 1 fresh, 4 stale.
- Freshness is decided by the engine's own layout (`Workbook::pivot_table_cells`), so a stale pivot today is
  either one whose cache cannot reproduce its cells (no records part, `saveData="0"`; a non-worksheet or
  calculated-field cache the importer skips; a non-English Excel's captions, docs/excel-semantics/pivots.md
  "A non-English Excel saves its own UI language's default captions") or one the engine does not yet lay out
  as Excel did (missing `Row Labels`/`Values`/column-field headers, manual group labels shown as `Group1`,
  `""` where Excel leaves a cell blank, sort order, a time item's caption). Scoring render needs a
  freshness test that does not depend on the renderer under test: for example, the classification at the
  time a pivot first passes, recorded and ratcheted, or a check of the cache records against the source.
- docs/research/pivot-tables-2026-11-04/plan.md, "Excel-saved corpora"; SPEC-0032 "Corpora".
- The M16 exit row waits on this task: its `pivot.corpus_render_met` (T-4127,
  crates/rustsheet-eval/src/milestone_exit/pivot.rs) stays not measured, so M16 is not met, until fresh
  corpus pivots are scored with a renderer-independent freshness test.
- The nightly workflow fetches calipers, poi and corpus-bench only; the two pivot-corpus suites need the same
  to be measured outside a local run.

## Acceptance criteria
- [ ] Fresh pivots' cells (and their `rowItems`/`colItems`, once T-4147 emits them) are scored, with a
      freshness criterion independent of the render being scored; stale ones are listed, not scored.
- [ ] `refreshOnLoad` pivots of the corpus are refreshed from their source and compared (check (c)).
- [ ] The nightly eval fetches and reports both pivot-corpus suites.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fixing the render gaps the classification shows (their PV1 tasks own them).
