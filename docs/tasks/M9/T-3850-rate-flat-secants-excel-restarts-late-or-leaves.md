---
id: T-3850
title: "RATE flat secants Excel restarts late or leaves in the noise floor (after T-3698)"
milestone: M9
lane: functions-financial
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-3698]
campaign: accuracy
---

## Goal
A rule for the RATE flat secants that T-3698's restart rule leaves to the hold, or to neither path: late restarts
Excel takes, and noise-floor runs where Excel's value is neither the hold nor the restart.

## Context
- T-3698 (financial.md "RATE restarts once on an early flat secant") restarts once from the older iterate when the
  flat secant is on distinct iterates at step 16 or earlier with |f1| < 1e-4. It gains 22 of 1,573 RATE cells and
  loses none.
- Excel also restarts late on some series (probes/t3698-rate-flat-restart A35 at step 27, A39 at step 33, A42 at
  step 41), while it holds early on others (A28 at step 18). On many cells Excel's value is neither path: its
  iterates leave the model's in the noise floor (financial.md "RATE holds on a flat secant", the T-3389 notes).
- Candidates the T-3698 probes did not separate: a restart count within a run, a bound relative to the residual's
  terms instead of the step, and a restart point other than r0.
- Code: `rate_secant` (crates/rustsheet-engine/src/functions/financial/financial_util.rs). A rule chosen by scoring
  owes decision 0188 R4's hold-out and fresh sweep.
- Known failures owned: all cells of probes/t3698-rate-flat-restart and probes/t3698-rate-flat-restart-fresh that
  are off Excel.
  - The cases are named here on lines with no cell address, so this task owns every cell of them.
  - Overlaps with T-3424 (noise-floor type) on the error-against-number cells.
- Also owned: every cell of probes/t3906-rate-restart-oos that is off Excel. It holds 240 fresh flat-secant calls
  (T-3906, 2026-11-03), and on 180 of them Excel matches neither the hold nor the restart. They are mostly noise-floor
  runs of large annuities, so they are this task's evidence.
  - The case is named on a line with no cell address, so this task owns every cell of it.
- R4 decision owed (T-3906): on that case T-3698's rule moves A152 and A194 out of the Gate A envelope (A194 becomes
  `#NUM!`), which fails decision 0188 R4.2's guard, though it brings five other points in.
  - Narrow the rule so that neither point leaves, or revert it.
  - Also compute the train-side bit-exact rate on firing cells only (R4.1), to compare with this probe's 48 of 240.

## Acceptance criteria
- [ ] A probe that separates late restarts from holds, oracle-filled, and a rule that fits it, or a recorded finding
      that no rule on the run's visible state does.
- [ ] Any rule is implemented with no RATE cell regressing; R4 hold-out and fresh sweep recorded.
- [ ] financial.md updated; `make check` green; reviewers approve.

## Out of scope
- RATE's residual form (ruled out at nper 1 by T-3579).
