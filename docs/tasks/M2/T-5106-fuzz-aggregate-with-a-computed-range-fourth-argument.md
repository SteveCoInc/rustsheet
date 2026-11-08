---
id: T-5106
title: "Fuzz: AGGREGATE with a computed range fourth argument (-A1:B2, A1:B2%) reaches a scalar-context cell (debug_assert panic)"
milestone: M2
lane: functions-math
status: done
priority: P0
size: S
tier: opus
usage: foundational
depends_on: []
issue: 4105
issue_author: github-actions
---

## Goal
Evaluating `=AGGREGATE(7,2,Data!B3:B9,Data!C6:E11%` in a cell never panics, in debug or release
builds, and gives Excel's result; the same for any AGGREGATE whose fourth argument is computed from
a range (`-A1:B2`, `(A1:B2)%`).

## Context
Nightly fuzz finding, https://github.com/SteveCoInc/rustsheet/issues/4105 (target `formula_eval`,
artifact `crash-6c2689f475d0f2d1082501410f12547397edb3dd`, 37 bytes, escaped:
`AGGREGATE(7,2,Data!B3:B9,Data!C6:E11%`). Reproduced on 9dbff3a0 with a debug facade test:

    panicked at crates/rustsheet-engine/src/model.rs:797:25:
    Larger-than-1x1 array reached scalar-context cell (sheet=0, row=3, column=3, 3x6);
    implicit intersection was expected to collapse it.

Minimised in the engine: `=AGGREGATE(9,0,B3,-C6:E11)`, `=AGGREGATE(9,0,B3,(C6:E11)%)` and
`=AGGREGATE(14,0,B3,C6:E11%)` panic; `C6:E11+0` and `{1,2;3,4}%` spill. Cause:
`aggregate_static_result` (static_analysis/aggregate.rs) skips a fourth argument whose static
shape is a `Range`, as if it were a reference, but a unary minus or percent of a range has a range
shape and evaluates to an array, which `Model::fn_aggregate` lifts. Same cause as T-0296 (a shape
disagreement), a different argument. `lifted.rs` already guards the same case with `yields_cells`.

## Acceptance criteria
- [x] The reproducer is a unit test (decision 0035) that passes in a debug build.
- [x] Static analysis treats a fourth argument with a range shape as read only when it yields cells
      (`yields_cells`), so the shapes agree; the spilled values match excel-graph evidence from a
      probe case (`probes/aggregate-computed-range-fourth`), or the case is left pending and the task
      blocked.
- [x] `make check` green; reviewers approve.

## Out of scope
- Other functions that may reach the same assert (file them as found).

## Outcome
`aggregate_static_result` now keeps a range-shaped fourth argument unlifted only when it yields
cells (`yields_cells`, as `lifted_static_result` does), so `-range`, `range%` and `--range` there
spill the array `Model::fn_aggregate` already lifts. Excel agrees
(probes/aggregate-computed-range-fourth, excel-graph 2026-11-08, 21/21 cells passing): the reference
form spills `#VALUE!` per element, the array form one LARGE/SMALL per k, and bare ranges stay read.
Unit test: `computed_range_fourth_argument_spills` (the fuzz reproducer). Recorded in
docs/excel-semantics/numbers.md "AGGREGATE".
