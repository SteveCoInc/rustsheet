---
id: T-4972
title: "Capture Excel's on-open render of the pivot-corpus refreshOnLoad pivots for check (c)"
milestone: M16
lane: harness
status: todo
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
- [ ] The `refreshOnLoad` pivots of both corpora have Excel's on-open area as committed oracle evidence, and check (c) compares against it.
- [ ] Check (c) scores those cells, or says why not, and eval-harness.md says which.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Making the engine refresh Table sources (T-4506) or merged areas (T-4677).
