---
id: T-4896
title: "Expose the pivot plan's result and layout so the drill-down does not compute and lay out twice"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
`Workbook::pivot_cell_records` computes and lays out a pivot once, not twice.

## Context
T-4184's `Workbook::pivot_cell_records` (crates/rustsheet/src/pivot_view.rs) calls `Model::plan_pivot_render`
to learn whether the pivot fits or collapses to `#SPILL!`, then runs `compute` and `lay_out` again for the
`PivotResult` and `PivotLayout` the plan built and dropped (`Model::place`, crates/rustsheet-engine/src/pivot/grid.rs),
and feeds them to `pivot::records_behind`. That doubles the work under one `PivotBudget`, so a pivot near
`MAX_PIVOT_WORK` can lay out in `pivot_table_cells` and fail here with `PivotError::Limit`. Design review of
T-4184. SPEC-0032 "API".

## Acceptance criteria
- [ ] A read-only engine entry point returns a pivot's placement with its `PivotResult` and `PivotLayout` (or
      the records behind a cell directly), without changing what `plan_pivot_render` returns or writes.
- [ ] `pivot_cell_records` uses it: one compute and one layout per call; its doc no longer says the budget is
      spent twice; the existing pivot_records tests pass unchanged.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the records returned or the errors.
