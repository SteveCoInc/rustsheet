---
id: T-4906
title: "pivot-corpus: score the re-render of fresh pivots and the refreshOnLoad refresh (plan checks b and c)"
milestone: M16
lane: harness
status: done
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
- [x] Fresh pivots' cells (and their `rowItems`/`colItems`, once T-4147 emits them) are scored, with a
      freshness criterion independent of the render being scored; stale ones are listed, not scored.
- [x] `refreshOnLoad` pivots of the corpus are refreshed from their source and compared (check (c)).
- [x] The nightly eval fetches and reports both pivot-corpus suites.
- [x] `make check` green (the gates this change can break, run locally; CI runs the rest).
- [x] Reviewers approve.

## Out of scope
- Fixing the render gaps the classification shows (their PV1 tasks own them).

## Outcome
Done on 2026-11-07. Freshness is now a property of the file, not of the renderer: each value cell is placed by
Excel's own saved `rowItems`/`colItems` and read from the engine's result tree over the saved records
(`compute_on`, never the layout); the pivot is fresh when every one equals Excel's saved cell
(`crates/rustsheet-eval/src/pivot_freshness/records.rs`). The workbook is loaded a second time without a
recalculation so `refreshOnLoad` caches are still Excel's (`saved_model.rs`). T-4131's renderer comparison is the
render score (check (b)); a fresh pivot's whole saved area is scored as cells with the cell baseline, and its engine
`rowItems`/`colItems` (T-4147) are compared with Excel's. A fresh pivot whose `dataCaption` is neither `Values` nor
`Data` (a non-en-US Excel's) is reported and not scored (decision 0030). report.json `fresh_pivot_render` holds the
per-suite counts for the M16 exit row (wiring: T-4971); eval-harness.md "Pivot freshness" documents both.

Counts (2026-11-07, both corpora):
- Fresh by the new test: 92 of 125 classified pivots (lo 75 of 86, closedxml 17 of 39), against 46 under T-4131's
  renderer test. Stale: 33 (the importer skips 21 of them, 7 have no data field, 5 differ in values: four where Excel
  shows a blank and the engine 0, as T-4918 owns (three of column-style-autofilter's, closedxml
  PivotTableWithTableSource), and column-style-autofilter Age!PivotTable4, whose saved values are all empty). 13 fresh pivots are
  non-en-US (Hungarian, German, Spanish) and unscored.
- Render-scored cells: 2,463 of 79 fresh pivots (lo 1,879, closedxml 584); 1,945 pass (79.0%), 518 visibly off, all
  owned. 41 pivots render exactly; 77 of 79 have Excel's `rowItems`/`colItems`.
- Render failures by cause: x14 `hideValuesRow` with several data fields on columns, 303 cells of 11 pivots
  (T-4966, new); `""` under an empty `missingCaption`, 52 of 6 (T-4967, new); a merged label range the engine refuses,
  59 of 1 (T-4677); value-sort ties under a show-values-as field, 76 of 1 (T-4970, new); item `n` captions and a
  renamed group, 16 of 7 (T-4915); outline and tabular header captions, 8 of 8 (T-4914); a data field without
  `name`, 2 of 2 (T-4968, new); a page field's `(All)`, 1 (T-4969, new); one visible page item, 1 (T-4921).
- The calipers and poi runs score their fresh pivots too: calipers 16 fresh (483 cells, 15 off), poi 4 fresh (68
  cells, 60 off); both off pivots are the `hideValuesRow` cause (T-4966). Their baselines are ratcheted.
- Check (c): of 14 `refreshOnLoad` caches, the importer loads 8 pivots (all closedxml) and the engine refreshes
  none: 7 read a Table or a name (T-4506) and 1 meets its own merged cells (T-4677). poi's one `refreshOnLoad` pivot
  is compared and exact, as are the two of crates/rustsheet-xlsx/tests/pivot_tables.xlsx (unit test). Excel's
  saved cells predate its on-open refresh, so check (c) is compared only where the source still holds the saved
  records; T-4972 captures Excel's on-open render for the rest.
- Nightly: fetches, runs (artifact `pivot-corpus/`) and checks both suites (decision 0346). Decision 0245's gate
  (`scripts/new_case_owners.py`) skips fetched suites; the owner tasks above list every off cell anyway.
- Baseline growth is new coverage, not regressions: the pivot-corpus, poi and calipers baselines gain the newly scored
  fresh-render cells (`update-baseline`: 0 regressions). For example poi `DataTableCities.xlsx` goes from 14 to 74
  checked cells, all its 60 new failures render cells (T-4966); calipers gains 7 fresh-pivot cases and 15 failing
  render cells (`officejs/pivot_two_data`, T-4966), and its 9 removed failure lines (`formula_stress_test`,
  `formulas_engineering`) are fixes that predate this change, recorded by the same ratchet.
- A non-en-US pivot's value cells are reported, not scored, with its labels (fidelity review's optional item, left
  as is: the whole render of such a pivot waits on decision 0030's evidence rule).
- A compact or outline parent line Excel saved empty is skipped only when its field's flags leave it blank
  (`subtotalTop` or `defaultSubtotal` off, custom subtotals, or the Values field below it on the axis); the counts above did not
  change with that rule.
- Quirks recorded: docs/excel-semantics/pivots.md "What the Excel-saved pivot corpus shows (T-4906)".
