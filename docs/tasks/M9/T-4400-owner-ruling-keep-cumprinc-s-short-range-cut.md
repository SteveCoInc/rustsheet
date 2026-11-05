---
id: T-4400
title: "Owner ruling: keep CUMPRINC's short-range cut GEOMETRIC_MIN_LEAD (decision 0083's order) or remove it (decision 0188 R4.3)"
milestone: M9
lane: functions-financial
status: blocked
priority: P2
size: S
tier: opus
depends_on: [T-4349]
---

## Goal
A ruling on CUMPRINC and CUMIPMT's short-range cut `GEOMETRIC_MIN_LEAD` (0.3, functions/financial/cumulative.rs):
keep it as decision 0295's recorded deviation from decision 0188 R4.3, or remove it (the geometric sum at every
leading exponent over a short range outside T-4349's band).

## Context
- Decision 0295 kept the cut while removing it puts at least as many gated cells visibly off as it fixes on fresh
  sweeps, and gave T-4349 the question: find a decider that replaces it, or ask the owner to rule between decision
  0083's order (visible parity first) and R4.3 (a fit that loses its fresh sweep goes).
- T-4349 (decision 0296) found none: where the cut's forms and the geometric sum differ, each is bit-exact on only
  265 and 286 of 923 cells, so Excel uses neither. It carved out one band that wins (at most 4 periods, rate at
  least 1E-3, lead below 0.095), which leaves the rest of the cut as it was. The evidence for the rest is decision
  0295's: removing it regresses 171 probes cells and fixes 158, regresses gen-m9 b72d952c5f52cb4d A257,
  d912671baa1364aa A300, e168be7f6d3f84c2 A379 and ironcalc-calc `CUMPRINC_CUMIPMT.xlsx` G6, H6, G22, H22, and moves
  21 gated cells visibly off against 21 fewer on the two T-3524 sweeps.
- T-4390 is looking for Excel's own short-range form; if it finds it, this ruling is moot.

## Blocked because
Blocked because: owner ruling. Choose one:
1. **Keep the cut** (decision 0295 stands as a standing deviation from R4.3 until T-4390 lands).
2. **Remove the cut.** An opus run removes `GEOMETRIC_MIN_LEAD` (keeping T-4349's band moot), records the
   regressions above in a decision record under decision 0083's conditions where they hold, and lands it.

## Acceptance criteria
- [ ] The owner's ruling recorded (a decision record, or this task's Outcome citing it).
- [ ] Option 2: the cut removed with its regression record; option 1: decision 0295's status line cites the ruling.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Excel's own short-range form (T-4390).
