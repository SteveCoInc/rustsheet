---
id: T-4972
title: "Capture Excel's on-open render of the pivot-corpus refreshOnLoad pivots for check (c)"
milestone: M16
lane: harness
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Check (c) of the pivot-corpus compares the refresh of every `refreshOnLoad` pivot with Excel's own render on open, not only those whose saved cells happen to be one.

## Context
T-4906 refreshes each `refreshOnLoad` pivot of the corpus from its source and compares it with Excel's saved cells only when the pivot is fresh and its source still holds the cache's records (eval-harness.md "Pivot freshness", check (c)): Excel re-renders such a pivot on open (pivots.md "Excel for the web re-renders a refreshOnLoad pivot when it opens the file"), so its saved cells predate that refresh. That equality of records implies equal renders is an inference. Where the source changed, the saved cells are no evidence of the refresh and the check reports `not_comparable`.

On 2026-11-07 the corpus has 14 `refreshOnLoad` caches (8 pivots the importer loads, all in pivot-corpus-closedxml); the engine refreshes none of them: 7 read a Table or a name (T-4506) and 1 meets its own merged cells (T-4677). The unit fixture crates/rustsheet-xlsx/tests/pivot_tables.xlsx compares exactly.

The oracle can record what Excel shows on open: fill each corpus workbook with a `refreshOnLoad` pivot (`rustsheet-oracle fill`, which captures the pivot areas, tools/src/rustsheet_tools/snapshot.py) and score the refresh against that sidecar's area. The corpus is fetched, so the sidecars need a home (a vendored suite of copies, as eval/roundtrip/excel-saved does for downloads, decision 0342) and a decision record.

## Acceptance criteria
- [x] The `refreshOnLoad` pivots of both corpora have Excel's on-open area as committed oracle evidence, and check (c) compares against it.
- [x] Check (c) scores those cells, or says why not, and eval-harness.md says which.
- [x] `make check` green; reviewers approve.

## Out of scope
- Making the engine refresh Table sources (T-4506) or merged areas (T-4677).

## Outcome
- All 14 `refreshOnLoad` caches of the corpus are in four pivot-corpus-closedxml workbooks (MIT). pivot-corpus-lo has none.
- The new vendored, gated suite `eval/suites/pivot-on-open` holds byte-for-byte copies of those workbooks. Its `suite.toml` records each file's upstream path and pin, and `LICENSE-ClosedXML` carries the MIT notice. Decision 0349 records why the copies are committed: Excel's on-open render needs committed sidecars, the fetched corpus cannot hold them, and its suites are ungated.
- The four excel-graph sidecars were filled on 2026-11-07 with `rustsheet-oracle fill -b excel-graph --suite pivot-on-open --include-cached`. They capture 21 pivot areas and 1449 formula values. One web-vs-desktop conflict was left pending: TemplateWithTableSourcePivotTables BondsOverview!L2, where the web gives `#NAME?` and the desktop `#VALUE!`. The fill now also takes an xlsx case that has pivots but no formula cell: `rustsheet-eval cases` lists `pivots`.
- Check (c) (`pivot_freshness::attach_on_open`, `refresh_on_open`) compares each refreshed `refreshOnLoad` pivot with the sidecar's area whatever its freshness, and scores the cells. A failed refresh scores the area against nothing. That pivot's check (b) cells are reported and not scored. report.json adds `refresh_on_open`, `refresh_cells` and `refresh_passed`, and report.md adds an evidence column.
- Result: all 20 pivots over a `refreshOnLoad` cache are scored: 597 cells. Before T-4506 merged, 224 passed (the empty ones); with its Table refresh, 371 pass. The case's 225 failing cells are known failures in `eval/baseline/pivot-on-open.txt` (one more failing area cell is a formula cell, which its formula expectation scores). Of the 7 pivots over a Table, T-5075 owns 52 cells where the refresh writes `""` and Excel leaves the cell blank, and T-5076 owns one missing `Values` caption (StressedAssetsPivot!D4). T-4677 owns PivotSubtotalsSource Pivot1 (59 cells, its own merged cells), and T-5065 owns the 12 pivots over a `saveData="0"` cache, which the importer does not load (113 cells). Each lists its cells. The fidelity review found the unloaded pivots unscored at first; they now score as failed refreshes, so an importer that stops loading a pivot regresses its cells. All 1449 formula cells of TemplateWithTableSourcePivotTables pass.
- Finding (pivots.md, new section "Excel's on-open render of the corpus refreshOnLoad pivots is their saved render (T-4972)"): every on-open cell equals the saved cell for all 21 pivots, so the saved-cells comparison was right for these files. Excel did rebuild 3 caches on open, dropping some unused shared items and renumbering `rowItems`. Which items it drops is open and filed as T-5057.
