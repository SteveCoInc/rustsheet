---
id: T-4232
title: "IRR on T-4213's near-zero-root sweep: 45 cells #NUM! where Excel returns a root, and A53/A149 at the noise floor (decision 0267)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
IRR gives Excel's type and value on the visibly-off cells of probes-numeric/t4213-irr-r4-fresh, T-4213's fresh sweep of
conventional series with roots 1E-7 to 0.05. If two attempts fail, the task blocks as time-boxed (decision 0056) and
decision 0267 accepts what is left.

## Context
- T-4213 drew the sweep after freezing its change (the fallback returns the Newton iterate one step past its stop) and
  filled it from excel-graph on 2026-11-04. Of the 150 cells, Excel and the engine give the same error on 46 and Excel
  returns a number on 104. The engine returns a number on 59 of those (57 bit-exact). The rest are visibly off:
  - 45 type mismatches. Excel returns a root and the engine is `#NUM!`: both secant runs fail and the fallback is
    `#NUM!`. This is T-3548's open type rule (blocked, time-boxed), "Excel errs on runs of 98 and 129 steps and
    returns runs of 182". Re-measure them against whatever T-3548's successor lands.
  - A53 `-1.1597865900991877E-7` (ours -1.1597865923196338E-7, math -1.1597865918101396E-7): first wrong at the 9th
    digit. The root sits 1.2E-7 from zero, so the engine's error is 2.2E-16 absolute.
  - A149 `9.855957131432724E-5` (ours 9.85595713141052E-5): first wrong at the 12th digit, also a 2.2E-16 absolute
    step.
- financial.md "IRR's returns where the secant fails are Newton iterates" (the T-4213 bullets): at this noise floor,
  Excel's value is sometimes a secant iterate and sometimes a Newton iterate, and no stop on |NPV| fits. That bullet
  lists the refuted relative stops.
- Code: crates/rustsheet-engine/src/functions/financial/financial_util.rs (`irr_secant`, `irr_fallback`).

## Acceptance criteria
- [ ] Every cell in "Decision 0267 cells" has Excel's type and is within half a unit of its 15th significant digit,
      or two attempts are recorded and the task blocks as time-boxed (decision 0056).
- [ ] The R4 guards (decision 0188; reviewer-excel-fidelity check 7) for any fitted or structurally chosen step.
- [ ] No gated cell regresses. `make check` is green and reviewers approve.

## Out of scope
- RATE's noise floor (T-3963, T-3424).

## Decision 0267 cells
- probes-numeric/t4213-irr-r4-fresh.case.json (47): Calc!A2, A4, A6, A8, A16, A17, A20, A23, A26, A36, A38, A41, A43,
  A45, A47, A48, A49, A50, A51, A52, A53, A54, A56, A58, A60, A62, A63, A64, A65, A68, A69, A70, A72, A73, A90, A92, A93,
  A98, A99, A127, A132, A137, A138, A140, A143, A145, A149
