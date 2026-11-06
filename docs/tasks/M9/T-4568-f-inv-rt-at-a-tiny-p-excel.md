---
id: T-4568
title: "F.INV.RT at a tiny p: Excel's #NUM! at (9, 51) and its off-math quantile at p = DBL_MIN"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
F.INV.RT and FINV return Excel's value or error at the two Gate A points where the engine returns the math and Excel
does not, or the task records why Excel's search lands there and is blocked as time-boxed (decision 0056).

## Context
- Gate A known failures this task owns (T-4103, eval/accuracy/known-failures.tsv):
  - accuracy/F.INV.RT/s1 A142, `F.INV.RT(1.53687678555259E-247,9,51)`: Excel `#NUM!`, the math and the engine
    38774983971.6707. No domain rule explains it: every other p > 0 point of the F.INV.RT and FINV tables, down to
    p = 2.2E-308, is a number in Excel.
  - accuracy/F.INV.RT/s1 A112, `F.INV.RT(2.22507385850721E-308,29,57)`: Excel 285298388679.62225, 0.9% below the math
    287879096822.68036; the engine is 287879123912.896 (9.4E-8 off the math).
- At p = DBL_MIN and below 1E-130 Excel's right-tail quantile is often far off the math (statistics.md "F.INV.RT and
  FINV at a tiny probability: Excel's own search, far from the math (T-4103)"), and the engine already matches Excel
  at the other such points, so the gap here is in where the engine's search departs from Excel's (T-2555's BETA.INV
  stop, T-0974's crossing).
- Decision 0300: Excel stays the target; no R3 mark.

## Acceptance criteria
- [ ] A142 is `#NUM!` and A112 within half a unit in the 15th digit of Excel (decision 0083), with the two lines
      removed from known-failures.tsv, or the task is blocked as time-boxed with what was tried.
- [ ] No other Gate A point leaves the envelope.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The F.INV.RT far-tail saturation at a df2 below 2 (T-2808) and the engine's deep-tail misses where Excel matches the
  math (T-1871).
