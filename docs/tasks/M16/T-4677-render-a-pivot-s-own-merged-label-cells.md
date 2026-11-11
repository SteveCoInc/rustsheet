---
id: T-4677
title: "Render a pivot's own merged label cells (mergeItem) on refresh instead of refusing"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-4151, T-4664]
---

## Goal
A pivot saved with "Merge and center cells with labels" (`mergeItem="1"`) refreshes like any other pivot: the engine rewrites its own merged label ranges as Excel does, instead of refusing the refresh and keeping the saved cells.

## Context
T-4151's grid writer (crates/rustsheet-engine/src/pivot/grid.rs) refuses a refresh with `PivotRefreshError::MergedArea` when a merged range lies in the pivot's own area, because the cell writers cannot write through a merge (docs/excel-semantics/pivots.md "The grid writer's own readings", UNVERIFIED). Excel creates those merges itself (excel-docs.md, `mergeItem`). Needs T-4664 case (k) evidence for how Excel re-merges on refresh. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

Also owner of these pivot-corpus render cells (T-4906, Excel-saved files: docs/excel-semantics/pivots.md "What the Excel-saved pivot corpus shows (T-4906)"), 59 render cells of 1 fresh pivot, which the engine refuses to lay out (`a merged range lies in the pivot's own area`), so every saved cell is scored against nothing; its `refreshOnLoad` refresh fails the same way (check (c)):
- pivot-corpus-closedxml/Other/PivotTableReferenceFiles/PivotSubtotalsSource/input.xlsx Pivot1!B3:H14

Also owner of the check (c) cells of the same pivot scored against Excel's on-open render (T-4972, decision 0349): 59 of its 84 area cells, every non-empty one, fail because the refresh is refused for the merge and writes nothing:
- pivot-on-open/PivotSubtotalsSource.xlsx Pivot1!B3:H14

## Acceptance criteria
- [ ] A `mergeItem` pivot refreshed on load or by `Txn::refresh_pivot_table` matches the T-4664 (k) sidecar, merges included.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Merges the pivot did not make (they stay foreign: `#SPILL!`).
