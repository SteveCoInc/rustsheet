---
id: T-3876
title: "RATE returns the guess when every residual is rounding noise (T-3698 fresh sweep A201)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
RATE matches Excel on annuities whose residual is rounding noise at every rate (Excel returned the guess on
probes-numeric/t3698-rate-r4-fresh A201), without losing the exact-zero cells of probes/rate-zero-residual-guess.

## Context
- probes-numeric/t3698-rate-r4-fresh A201, `RATE(1,13501.684222927,-13501.684222927,0,1,1.74874265618351)`: nper 1,
  type 1, fv 0, pmt = -pv, so pv + pmt = 0 at t = 0 and every residual is rounding noise. Excel returns the guess
  1.74874265618351; the engine returns `#NUM!`. `RATE(1,0,0,0,0,0.3)` (residual exactly 0 everywhere) is `#NUM!` in
  Excel (probes/rate-zero-residual-guess A2).
- financial.md "RATE restarts once on an early flat secant" ("Found in passing"), and "RATE is a secant on the FV
  residual" (the zero-residual paragraph).
- Code: `rate_secant` in crates/rustsheet-engine/src/functions/financial/financial_util.rs. Trace the model's
  residuals on A201 first: is f(guess) exactly 0 while f(r1) is not, or the reverse?
- Probe noise-only annuities with other scales, guesses and nper 1 against 2 before changing the code.

## Acceptance criteria
- [ ] A probe that separates a noise-only residual from an exactly-zero one, oracle-filled.
- [ ] t3698-rate-r4-fresh A201 passes and no RATE cell regresses; financial.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The flat-secant restart rule (T-3698).
