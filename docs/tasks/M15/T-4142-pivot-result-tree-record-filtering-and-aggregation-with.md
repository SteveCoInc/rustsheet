---
id: T-4142
title: "Pivot result tree: record filtering and aggregation with subtotals and grand totals"
milestone: M15
lane: engine
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-4139, T-4135]
---

## Goal
`PivotResult` computes, from definition and cache, the filtered records (page field selections, hidden items) and the accumulators for every (row tuple, column tuple, data field), its subtotals and grand totals, with Excel's rules for the eleven aggregates.

## Context
SPEC-0032; evidence from `p-aggregates`; aggregate kernels reused from GETPIVOTDATA's folds and excel_variance. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274. Since the plan (2026-11-06): M6 added `crates/rustsheet-engine/src/pivot/layout.rs` (a compact layout of one row field and one data field, used for grouped spec pivots, T-4235), `pivot/number_groups.rs`, `pivot/date_groups.rs` and `rustsheet-xlsx/src/import/pivots/groups.rs`; build on them, do not add a parallel layout or grouping path.

## Acceptance criteria
- [ ] Unit tests; GETPIVOTDATA answers unchanged.
- [ ] Bench on 100k records (crates/rustsheet/benches).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Layout.
