---
id: T-3908
title: "RATE early flat-secant restart: blind out-of-sample check where the rule fires (R4.1, R4.2)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
T-3698's early flat-secant restart in RATE (`rate_secant`, `RATE_RESTART_STEPS`, merged in #2858) is
tested out of sample on points where the rule actually fires, and is kept, narrowed or removed by
that result.

## Context
- #2858 merged with split Excel-fidelity verdicts on the same head 7f51edc. One adopter's
  fidelity reviewer returned REQUEST_CHANGES: the step and |f1| cuts were scored on every cell
  before the train/test split (R4.1), and the post-freeze sweeps never fire the restart (R4.2):
  with an instrumented build, the Gate A tables (r4-T-3698 included) show 121 flat secants and 0
  restarts, and probes-numeric/t3698-rate-r4-fresh 102 flat secants and 0 restarts; the rule fires
  only on cells of the fit set (21 in each t3698-rate-flat-restart probe). A second adopter's
  fidelity reviewer approved, and that adopter enqueued it.
- The steward (2026-11-03 run) files this so the unresolved R4 finding has an owner. T-3850 owns
  late restarts and the noise floor, not this check.
- MINOR findings from the same reviews: the |f1| cut reuses `RATE_CAP_TOLERANCE` instead of its own
  constant; the 1.3e-7 floor's evidence inside steps 3..16 is 1.67e-7; decision 0255 and the T-3698
  Outcome classify the cut differently.

## Acceptance criteria
- [ ] A fresh oracle-filled probes-numeric set, drawn after #2858 with the constants untouched,
      filtered to annuities and guesses whose run reaches a flat secant on distinct iterates at
      steps 1..30 with |f1| 1e-7..1e-3, so the restart fires on a share of it.
- [ ] Hold-out (R4.1) and fresh-sweep (R4.2) results recorded: bit-exact, within 4 ulp and within 15
      digits for the rule, the plain hold and the correctly rounded root; Gate A envelope count with
      every point outside listed; the R4.3 `FITTED` range and R4.4 floor checked (reviewer-excel-fidelity
      check 7).
- [ ] If the rule does not beat the hold out of sample, it is narrowed or removed, with a regression
      decision record if cells are lost (decision 0082's precedent); otherwise financial.md records
      the out-of-sample result.
- [ ] The |f1| cut gets its own constant under the T-3698 `FITTED` tag; decision 0255 and the T-3698
      Outcome agree.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Late restarts and the noise floor (T-3850); the every-flow-zero guess return (T-3876).
