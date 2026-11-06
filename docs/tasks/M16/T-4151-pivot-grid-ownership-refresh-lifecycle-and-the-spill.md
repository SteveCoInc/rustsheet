---
id: T-4151
title: "Pivot grid ownership, refresh lifecycle and the #SPILL! collapse"
milestone: M16
lane: engine
status: done
priority: P1
size: M
tier: opus
depends_on: [T-4149, T-4128]
---

## Goal
Rendered cells are owned by their pivot: a refresh clears the old area and writes the new one; a layout that would meet a foreign cell collapses to `#SPILL!` at the anchor as Excel does; pivots refresh on load when `refreshOnLoad` is set and on `Txn::refresh_pivot`, never on source edits; on load without refreshOnLoad the saved cells are kept.

## Context
oracle-spike.md runs 1 and 3; SPEC-0032; formulas reading pivot cells need no new dependency machinery. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [x] Spike probes for re-render and `#SPILL!` pass through the engine.
- [x] Structural edits (T-2086) still hold.
- [x] `make check` green; reviewers approve.

## Out of scope
- Auto Refresh (task `autorefresh`).

## Outcome
- The grid writer is `crates/rustsheet-engine/src/pivot/grid.rs`: `Model::plan_pivot_refresh`
  (refresh the cache, remap each pivot over it by field name, compute, lay out, render) and
  `plan_pivot_render` change nothing; `apply_pivot_refresh` clears the cells the pivot owned
  (location and page area) and writes the new ones as constants, then swaps the cache and pivots
  in and returns a `PivotSwap` for undo. A layout meeting a foreign cell or another pivot's area
  collapses to `#SPILL!` at the anchor with a one-cell location and zero geometry (t4128 S).
- `PivotCache::refresh_on_load` is read by the importer; the facade's first
  `Workbook::recalculate` of a loaded workbook refreshes the marked caches in one edit and
  recalculates again. `Txn::refresh_pivot_table(sheet, name)` (doc-hidden, SPEC-0032's name) is
  the explicit refresh. Nothing refreshes on a source edit; pivots without `refreshOnLoad` keep
  their saved cells (UNVERIFIED, T-4586). Grouped spec pivots now lay out through the same writer.
- Tests: t4128 R and S against the sidecar through a real model (`pivot/grid/tests.rs`),
  growth, clearing, collapse, row inserts (T-2086), remapping; facade load/refresh/rollback on
  pivot_tables.xlsx (`crates/rustsheet/tests/pivot_lifecycle.rs`).
- Engine readings without evidence are in pivots.md "The grid writer's own readings", probe
  T-4664 filed. Eval wiring of the probes stays T-4152's.
- Review round 1 fixes: one `PivotBudget` (work and cells, the source read and every scan
  included) per plan and across every `refreshOnLoad` cache on open; the open path plans first and
  edits (and recalculates) only when a plan succeeds; merged ranges, Tables, array formulas and
  the sheet edge (`PivotComputeError::PastSheet`) collapse to `#SPILL!` (UNVERIFIED against
  PSPILL, T-4664 g to j); `apply_pivot_refresh` returns `PivotRefreshError` (`Write`,
  `OutsideSheet`); only the swapped pivots' GETPIVOTDATA indices are dropped. workbook.rs split
  is T-4669.
