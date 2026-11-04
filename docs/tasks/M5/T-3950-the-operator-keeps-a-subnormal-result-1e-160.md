---
id: T-3950
title: "The ^ operator keeps a subnormal at x^2 for x = 1E-160, 1E-155, 1.1E-155 (probes seriessum-subnormal-power-and-term)"
milestone: M5
lane: engine
status: done
priority: P0
size: S
tier: sonnet
depends_on: []
---

## Goal
The ^ operator returns Excel's value at x^2 for x = 1E-160, 1E-155 and 1.1E-155, where Excel keeps the subnormal, and keeps flushing where Excel flushes.

## Context
Found by T-3942's probe (probes/seriessum-subnormal-power-and-term, excel-graph, 2026-11-03; docs/excel-semantics/numbers.md
"SERIESSUM: a power or term below the smallest normal double is 0"):
- `=1E-160^2` keeps the subnormal 1E-320 in Excel: its `=0` companion B15 is FALSE and `=1E-160^2*1E+300` (A16) is
  9.99988867182683E-21, not 0. Ours flushes (B15 TRUE, A16 0).
- `POWER(1E-160,2)*1E+300` (A17) is 0: POWER flushes, `^` does not.
- `=1E-155^2` and `=(1.1E-155)^2` (term-sum case A3:B4) keep the subnormal too. Only base 1E-160, 1E-155 and 1.1E-155 with exponent 2 were probed;
  probe other positive and non-integer exponents first if the fix needs them.
- Positive-exponent chains flush and must keep passing: probes-numeric/power-large-integer-exponents B103, B107, B117, B119 (`0.5^1074`,
  `0.1^310`, `(-0.5)^1073`). Why x^2 differs is unknown: probe `1E-160*1E-160`, `1E-160^3` and `0.1^310` against `(0.1^155)^2` first. A plain product is not enough to explain it: `*` (`1E-307/10`) and `SUMSQ(1E-160)` flush (numbers.md "Subnormal results become 0", per function group).
- A negative exponent whose result is subnormal gives a true 0: `=(2^-1023)=0` (term-sum case B5, 2^1023 finite) and `=(SUM(2^-1074))=0`
  (subnormal-function-results C16, where 2^1074 overflows) are TRUE, and the engine matches. Keep term-sum B5 and C16 passing; do not
  remove the flush from `^` as a whole.
Code path: the `^` operator in crates/rustsheet-engine (arithmetic power).

Owned cells:
- probes/seriessum-subnormal-power-and-term.case.json (3): Sheet1!A16, B15, B16
- probes/seriessum-subnormal-term-sum-and-power-points.case.json (2): Sheet1!B3, B4

## Acceptance criteria
- [ ] probes/seriessum-subnormal-power-and-term Sheet1!B15, A16, B16 and term-sum Sheet1!B3, B4 pass, and no passing cell of any suite changes (subnormal-function-results C16 included).
- [ ] `make check` green; reviewers approve.

## Out of scope
- SERIESSUM's term (T-3949) and POWER, which already flush.

## Outcome
Added `arithmetic::operator_power`, used by the `^` operator only: at exponent 2 it keeps the subnormal square (`1E-160^2`, `1E-155^2`, `(1.1E-155)^2`); every other exponent, POWER and SERIESSUM still flush. Unit test `operator_keeps_a_subnormal_square_and_power_flushes_it`. Eval: probes/seriessum-subnormal-power-and-term 34/34 and seriessum-subnormal-term-sum-and-power-points 12/12 now pass (were 31/34 and 10/12); no regressions in any suite. Other exponents were not probed (stated in numbers.md and the code).
