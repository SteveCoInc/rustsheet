---
id: T-4315
title: "Match which calls empty WEEKNUM's stale state: T-4289 probes show it is not every dynamic anchor"
milestone: M6
lane: functions-date
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
The engine empties WEEKNUM's start-weekday state only where Excel does, per the T-4289 probes, instead of at every dynamic-array anchor.

## Context
T-4289 (dates.md "Which calls empty WEEKNUM's state") found that a dynamic-array formula as such does not empty the
state; particular calls do, and not the same ones as AMORDEGRC's. `run_excel_order` (excel_order.rs) still empties it at
every dynamic anchor, so these cells are off (the reader in A1 expects 16 and gets `#NUM!`, or the reverse):
- probes/weeknum-stale-between-calls: `S02!A1`, `S04!A1`, `S06!A1`, `S08!A1`, `S10!A1`, `S12!A1`, `S13!A1`, `S15!A1`,
  `S16!A1`, `S18!A1`, `S20!A1`, `S21!A1`, `S22!A1`, `S23!A1`, `S24!A1`, `S27!A1`.
- probes/weeknum-stale-between-ranges: `S01!A1`, `S03!A1`, `S13!A1`, `S17!A1`, `S18!A1`, `S19!A1`, `S20!A1`,
  `S21!A1`, `S22!A1`, `S23!A1`.
The evidence is not one simple rule per function (SORT({3,1}) keeps the state but SORT(C1:D2) empties it; TEXTJOIN
and CONCAT with scalar arguments empty it but with C1:D2 keep it), so probe the argument shape that decides it first
(constant array, range, scalar), then keep the gen-m6 cells passing.

## Acceptance criteria
- [x] The cells listed above pass, and no gen-m6 or other WEEKNUM stale-state cell regresses.
- [x] dates.md records the rule and the shape probes.
- [x] `make check` green; reviewers approve.

## Out of scope
- Further WEEKNUM return_type or ordering work.

## Outcome
- Probes (excel-graph, 2026-11-05): probes/weeknum-stale-call-shapes and -call-shapes-2, 30 calls each in the T-4289
  layout. Together they separate the argument shapes: text and number scalars, constant arrays, one cell, one-column
  and two-dimensional ranges, nested and IF-skipped calls, and ten more array functions. The rule is per function, not
  per shape class. Eleven array functions always empty the state. SORT and UNIQUE empty it only with a multi-cell
  range, SORTBY unless its array is one cell, and ARRAYTOTEXT only with a range of more than one row and column.
  CONCAT and TEXTJOIN empty it unless a joined argument is a reference. FILTER keeps it. A nested call empties it too,
  and an untaken IF branch does not. dates.md "Which calls empty WEEKNUM's state" records the table and what is
  unprobed.
- Engine:
  - `Model::weeknum_effect_of_call` (weeknum.rs) holds the table and runs after every function call
    (`evaluate_function`).
  - `run_excel_order` no longer empties the state at every dynamic anchor. An anchor whose run emptied it does so at
    its place in the order. `AnchorAmor` is now `AnchorEffects`, which records that alongside AMORDEGRC's value.
- Tests: test_weeknum_stale_state.rs replaces "a spilling formula empties the state" (the T-2068 reading that T-4289
  disproved) with two tests of the evidenced table.
- Eval delta: probes fixed 26: the 16 between-calls and 10 between-ranges cells the task lists. Two new cases (189 cells)
  pass, and regressions are 0 in every suite, gen-m6 included.
