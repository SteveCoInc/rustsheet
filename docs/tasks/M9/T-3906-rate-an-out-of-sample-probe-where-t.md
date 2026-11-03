---
id: T-3906
title: "RATE: an out-of-sample probe where T-3698's flat-secant restart fires (R4)"
milestone: M9
lane: functions-financial
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
T-3698's fitted restart rule (`RATE_RESTART_STEPS = 16`, the |f1| window 1.3e-7..1e-4) is tested on points the fit never
saw and where the restart actually fires. The rule is kept, narrowed, or reverted on that evidence.

## Context
- PR #2858 (T-3698) merged with the Excel-fidelity reviewer's MAJOR finding open (orphan re-review, 2026-11-03,
  PR comment):
  - The step bound and the |f1| cut were scored on every cell before the train/test split, so R4.1's hold-out is not blind.
  - The two sweeps drawn after the freeze fire the restart 0 times, so R4.2 says nothing about the fitted values:
    - Gate A RATE: 121 flat secants, 0 restarts;
    - probes-numeric/t3698-rate-r4-fresh: 102 flat secants, 0 restarts.
  - The rule fires only on cells of the fit set (21 in each t3698-rate-flat-restart probe).
- Fix the reviewer asked for:
  - draw fresh annuities and guesses after the freeze, filtered to model runs that reach a flat secant on distinct
    iterates near the rule's window;
  - oracle-fill them with the constants untouched;
  - report bit-exact / 4 ulp / 15 digits for the rule, the hold and the correctly rounded root.
- The reviewer's MINOR findings come along:
  - give the |f1| cut its own constant under the T-3698 FITTED tag instead of reusing `RATE_CAP_TOLERANCE`, and make
    decision 0255 agree;
  - `RATE_RESTART_F_MIN`'s comment: inside steps 3..16 the smallest evidenced |f1| is 1.67e-7, not 1.3e-7.
- Code: `rate_secant` in crates/rustsheet-engine/src/functions/financial/financial_util.rs; docs/excel-semantics/financial.md
  "RATE restarts once on an early flat secant".

## Acceptance criteria
- [ ] A probes case of fresh, oracle-filled (excel-graph) RATE calls where the frozen rule restarts, with the
      draw's seed and filter recorded. Scored rule vs hold vs correctly rounded root.
- [ ] The rule is kept if it beats the hold out of sample; otherwise narrowed or reverted, with no RATE cell regressing
      beyond what the evidence justifies, recorded in financial.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Late restarts and the noise floor (T-3850).
