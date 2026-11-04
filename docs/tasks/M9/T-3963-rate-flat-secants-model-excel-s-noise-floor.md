---
id: T-3963
title: "RATE flat secants: model Excel's noise-floor iterates, since no rule on the model's flat-secant state fits (after T-3850)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3850]
campaign: accuracy
---

## Goal
RATE's flat-secant cells come out as in Excel. That needs a model of Excel's own iterates in the noise floor,
because T-3850 showed that no rule on the model's state at the flat step does it.

## Context
- T-3992 (excel-graph fill-to-fill stability, 2026-11-03): measured on its own probe t3850-rate-same-state: 0 of its 240 cells changed over three fills (its cells there); see docs/research/t3992/README.md.
- T-3850 (financial.md "RATE flat secants: no rule on the model's state separates Excel's restarts from its holds";
  docs/research/t3850/): calls whose model run reaches the same flat secant (same args, r0, r1 and f1), differing only
  in the guess and so in the step, get different Excel values in no step order.
  - On probes/t3850-rate-same-state (240 calls in 40 such groups, drawn blind), 25 groups get more than one Excel
    value. 12 are solid: A55:A60, A109:A114, A175:A180 and A229:A234 mix `#NUM!` with numbers, and 8 groups spread
    over more than 4 ulp. The other 13 differ by 1 to 4 ulp.
  - Differences of 4 ulp or less are under the reproducibility floor of decision 0112 and reviewer check 7 (R4.4), so
    they are not targets. A second fill on 2026-11-03 reproduced every bit, but one refill doesn't clear the floor.
  - Excel's value is the hold on 29 cells, the restart from r0 on 111, and neither on 100. Today's rule gets 55.
  - So Excel's own iterates differ from the model's before the flat step. The likely cause is the residual's
    arithmetic in the noise floor (term order, or how the power or the sum is rounded). It is not a rule applied at
    that step.
- A wider restart (to step 200, |f1| unbounded) gains 70 and loses 14 cells on the new probe, but on the fit set it
  gains 8 and loses 12. The sampling decides the count, so no widening is a rule.
- T-3424 (2026-11-04, probes/t3424-rate-noise-trace, docs/research/t3424/): no residual difference found in FV.
  - Excel's FV is the model's annuity value at 355 noise-floor iterates of T-3424's 16 cells, at nper 1 to 351.
  - The model's step form beats six others and two extended-iterate forms.
  - Seeded runs that hold at the same iterate get different Excel outcomes.
  - No single quantity at the hold separates Excel's returned holds from its errors.
  - Excel's returned holds equal the model's held iterate on 202 of 549 cells, are 1 to 4 ulp off on 182 and further
    off on 165, so Excel's path still leaves the model's in the noise floor (financial.md "Excel's FV along the path is the model's annuity value; how RATE decides in the noise floor is not").
- Candidates: RATE's own residual (FV's arithmetic matched, but RATE's was not read directly), the step's
  arithmetic, a guard on tiny or zero steps, a nudge of an iterate that repeats, or a stop test of the step against
  the iterate. The direct test is a seeded or one-step RATE probe whose model trace stalls after one or two steps. Ranking candidates owes decision 0188
  R4's hold-out and fresh sweep.
- Code: `rate_secant` and `rate_residual` (crates/rustsheet-engine/src/functions/financial/financial_util.rs).
- Known failures owned (from T-3850, which owned them before it closed):
  - every cell of probes/t3698-rate-flat-restart that is off Excel;
  - every cell of probes/t3698-rate-flat-restart-fresh that is off Excel;
  - every cell of probes/t3906-rate-restart-oos that is off Excel;
  - every cell of probes/t3850-rate-same-state that is off Excel.
  - every cell of probes/t3424-rate-noise-trace that is off Excel (from T-3424's probe; its FV cell is T-4146's too).
  - The cases are named here on lines with no cell address, so this task owns every cell of them. It overlaps with
    T-3424 (the noise floor's type) on the error-against-number cells and with T-3916 (the owner's ruling on T-3698's
    restart, decision 0265: kept).

## Acceptance criteria
- [ ] A probe or finding that pins Excel's residual (or another cause) at the noise floor of at least two of the
      12 solid t3850-rate-same-state groups (an error against a number, or more than 4 ulp apart). Splits of 4 ulp
      or less are not targets (decision 0112, R4.4).
- [ ] A model that fits it, implemented with no RATE cell regressing, R4 hold-out and fresh sweep recorded; or a
      recorded finding that none on the evidenced inputs does.
- [ ] financial.md updated; `make check` green; reviewers approve.

## Out of scope
- The narrow-or-revert ruling on T-3698's restart: decided, the rule is kept (decision 0265, T-3916). A194 (and A26,
  if a fallback is tried) stay this task's.
