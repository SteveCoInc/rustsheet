---
id: T-4431
title: "RATE is #NUM! on a flat secant at a tiny noise or constant residual (t3876 A22, fresh A1, A16, A26, A31)"
milestone: M9
lane: functions-financial
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
RATE matches Excel's `#NUM!` on probes/t3876-rate-noise-residual A22, `RATE(1,0,0,1E-300,0,0.3)`, without
losing the flat-secant holds of T-3348 (financial.md "RATE holds on a flat secant and runs on below -1").

## Context
- With pv 0 and pmt 0 the residual is fv at every rate, so the first secant is flat on a nonzero |f| far below
  1e-7. The model holds at r1 = 0.299 and returns it; Excel is `#NUM!`. A23 (same, guess -0.9999) is `#NUM!` in
  both (the 1 + r floor).
- Candidate rules to separate with a probe before changing code: an up-front `#NUM!` when pv and pmt are both 0
  (then `RATE(n,0,0,fv)` is `#NUM!` for every fv), a hold that needs r1 != r0 *and* a step the run actually took,
  or a hold that needs |f1| above some floor. Probe pv = 0 with pmt tiny, pmt = 0 with pv tiny, fv from 1E-300 to
  1E-8, and nper 1 against 5.
- The same shape on noise, not a constant: probes/t3876-rate-noise-residual-fresh A1, A16, A26, A31 (nper 1,
  every residual rounding noise, a nonzero f at the guess) are `#NUM!` in Excel; the model's first secant is flat on a
  nonzero noise value and it holds at r1 = guess - 0.001 and returns it. A5 of that case takes another noise path:
  Excel 0.49085317146390595, the model 0.490603171463906. Check whether one rule (no hold below some |f| floor, or
  no hold on the first step) covers all five with A22, without losing the T-3348 and T-3698 holds.
- Code: `rate_secant` in crates/rustsheet-engine/src/functions/financial/financial_util.rs (the flat branch).
- Found by T-3876 (financial.md "RATE steps up from a zero residual").

## Acceptance criteria
- [ ] A probe separating the candidate rules, oracle-filled.
- [ ] t3876-rate-noise-residual A22 and t3876-rate-noise-residual-fresh A1, A5, A16, A26, A31 pass (or are
  explained in financial.md) and no RATE cell regresses; financial.md updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The zero-residual step direction (T-3876).
