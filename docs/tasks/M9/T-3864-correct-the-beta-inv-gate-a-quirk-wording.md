---
id: T-3864
title: "Correct the BETA.INV Gate A quirk wording from T-3578's review"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: [T-3578]
---

## Goal
statistics.md "BETA.INV's Gate A table", the BETA.INV s1 lines of eval/accuracy/known-failures.tsv, T-3862's Context and
`beta.quantile`'s docstring say what the oracle and math sidecars of accuracy/BETA.INV/s1 show.

## Context
- MINOR findings of the Excel-fidelity review of PR #2873 (T-3578), all checked against
  eval/accuracy/tables/BETA.INV/s1.{oracle,math}.json:
  1. Class 4 (huge alpha, tiny p; A180, A181, A187, A188, A190-A194, A196-A198): the math is exactly 1 (the true
     quantile rounds to 1), not 1 - 2^-53; the engine returns 1 - 2^-53 and Excel `#NUM!`. Reword the entry, the 12
     T-3862 known-failure notes and T-3862's Context.
  2. A123 (`BETA.INV(0.999999999999999,6.87566761020923,0.991674132119787)`): its math 0.99999999999999989157 rounds to
     1 - 2^-53, so Excel's 0.9999999999999999 is the correctly rounded quantile and only the engine's `#NUM!` fails.
     Take A123 and "or p = 0.999999999999999" out of class 1; record it as an engine `#NUM!` near 1 where Excel matches
     the math (still owned by T-3351); fix its known-failure note.
  3. tools/src/rustsheet_tools/accuracy/defs/beta.py `quantile`: the sub-double bound applies only for p <= 1/2; above
     it the flipped branch records 0 (A88, A97, A108, A115), consistent with class 2. Say so in the docstring.
- Only the note column of known-failures.tsv changes (no point is added or removed); decision 0199.

## Acceptance criteria
- [ ] The three corrections are made; `rustsheet-eval accuracy` still reports 0 failing and the same 86 known failures.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine fixes (T-3351, T-2808, T-3862).
