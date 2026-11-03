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
- T-3850 (financial.md "RATE flat secants: no rule on the model's state separates Excel's restarts from its holds";
  docs/research/t3850/): calls whose model run reaches the same flat secant (same args, r0, r1 and f1), differing only
  in the guess and so in the step, get different Excel values in no step order.
  - On probes/t3850-rate-same-state (240 calls in 40 such groups, drawn blind), 25 groups get more than one Excel
    value, and 21 mix the hold, the restart or neither. For example, steps 9, 11, 13, 15, 17 and 21 give R, H, R, R, H
    and R.
  - Excel's value is the hold on 29 cells, the restart from r0 on 111, and neither on 100. Today's rule gets 55.
  - So Excel's own iterates differ from the model's before the flat step. The likely cause is the residual's
    arithmetic in the noise floor (term order, or how the power or the sum is rounded). It is not a rule applied at
    that step.
- A wider restart (to step 200, |f1| unbounded) gains 70 and loses 14 cells on the new probe, but on the fit set it
  gains 8 and loses 12. The sampling decides the count, so no widening is a rule.
- Candidates: reproduce Excel's residual bit for bit at noise-floor points (probe the residual itself, for example
  through a run of 1 step), then rerun the secant on it. Ranking candidates owes decision 0188 R4's hold-out and fresh
  sweep.
- Code: `rate_secant` and `rate_residual` (crates/rustsheet-engine/src/functions/financial/financial_util.rs).
- Known failures owned (from T-3850, which owned them before it closed):
  - every cell of probes/t3698-rate-flat-restart that is off Excel;
  - every cell of probes/t3698-rate-flat-restart-fresh that is off Excel;
  - every cell of probes/t3906-rate-restart-oos that is off Excel;
  - every cell of probes/t3850-rate-same-state that is off Excel.
  - The cases are named here on lines with no cell address, so this task owns every cell of them. It overlaps with
    T-3424 (the noise floor's type) on the error-against-number cells and with T-3916 (the owner's ruling on T-3698's
    restart).

## Acceptance criteria
- [ ] A probe or finding that pins Excel's residual (or another cause) at the noise floor of at least two of the
      t3850-rate-same-state groups.
- [ ] A model that fits it, implemented with no RATE cell regressing, R4 hold-out and fresh sweep recorded; or a
      recorded finding that none on the evidenced inputs does.
- [ ] financial.md updated; `make check` green; reviewers approve.

## Out of scope
- The narrow-or-revert ruling on T-3698's restart (T-3916, needs owner).
