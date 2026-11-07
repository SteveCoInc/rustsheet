---
id: T-4206
title: "Excel's gamma inverse search at its step limit: the three cells the step-count model gets wrong"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
The three huge-shape GAMMA.INV cells where T-3409's step-count model of Excel's search has the wrong type get Excel's type, without moving any cell the model places now.

## Context
T-3409 models Excel's gamma inverse at a large shape as Newton's method on the odds P/Q from Excel's start, failing past
`SEARCH_STEP_LIMIT` = 207 steps at a relative step of 1E-8 (`incomplete_gamma/huge_shape.rs` `search_converges`;
docs/research/t3409/README.md). It gets every probed type right but three, all within a step or two of the limit:
- probes-numeric/fit-audit-t3070-band-upper S!A39 `GAMMA.INV(1.11E-10,9000000000,1)`: Excel 8999398045.572908, ours
  `#NUM!` (208 steps in the 40-digit replay, where four `#NUM!` cells also take 208).
- probes-numeric/t3409-r4-gamma-inv-search-sweep Calc!A177 `GAMMA.INV(0.997046030688198,4386000000,1)`: Excel
  4386182314.709858, ours `#NUM!`.
- probes-numeric/t3409-r4-gamma-inv-search-sweep Calc!A191 `GAMMA.INV(7.08757646342324E-96,3641000000,1)`: Excel
  `#NUM!`, ours 3639748494.964917.
So the count is right to about a step: Excel's own stopping rule, its cdf's rounding, or its step form (for example
Newton on another function with the same tail behaviour) may differ. Probe p either side of these cells at fixed
shapes to see how sharp the edge is, and compare stopping rules (a residual test, a step test, a count of cdf calls)
on the replay before changing the model. Decision 0056's time-box applies.

## Acceptance criteria
- [ ] fit-audit-t3070-band-upper S!A39 and t3409-r4-gamma-inv-search-sweep Calc!A177, Calc!A191 have Excel's type, with no type regression in the huge-shape probes, or the task is blocked as time-boxed (decision 0056).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
