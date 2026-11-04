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
SPEC-0030; evidence from `p-aggregates`; aggregate kernels reused from GETPIVOTDATA's folds and excel_variance. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Unit tests; GETPIVOTDATA answers unchanged.
- [ ] Bench on 100k records (crates/rustsheet/benches).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Layout.
