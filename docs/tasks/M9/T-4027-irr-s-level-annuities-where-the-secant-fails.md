---
id: T-4027
title: "IRR's level annuities where the secant fails: find Excel's path (t3406 A1-A96; uncapped Newton plus the residual cut breaks 39 of them)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-3411]
---

## Goal
IRR gives Excel's type on the level annuities `IRR({-P,1,...,1})` of probes/t3406-irr-num-boundary (A1-A96) where
both secant runs fail, from a model of Excel's path there that holds on a fresh sweep, so that the fallback's
50-step Newton cap can be lifted without breaking them.

## Context
- T-3411 (round 4) made `irr_fallback` `#NUM!` when |`irr_npv`| one Newton step past its stop is 1E-4 or more
  (financial.md "IRR's fallback is `#NUM!` when NPV at its root is 1E-4 or more"). That is the type rule only for
  roots the 50-step fallback reaches.
- With the Newton search uncapped plus the residual check, 219 cells would be fixed but 71 that main gets right break;
  39 of them are t3406 level annuities A12-A24, A34-A48, A56-A65 and A79 (residuals 2E-13 to 3E-7, Excel `#NUM!`).
  Their cutoff in Newton steps (n = 20: 125 returns, 133 `#NUM!`; n = 30: 223 / 241; n = 40: 313 / 344; n = 60:
  577 / 649) is no fixed cap and no fixed multiple of n, and most returning annuities (A1-A10, A25-A30) are not
  Newton iterates from v = 0 (T-3411 progress notes, 2026-11-04).
- Refuted already (T-3411 task file, do not repeat): restarts at a flat secant, damping the far first step, a
  last-ulp NPV difference at d = 0.0919, Illinois, any secant step cap alone, |NPV| at the far first step, v0^n.
- Lead from T-3411: try Newton started at the secant's far point, or at v = 1/(1 + guess), each with the residual
  check, scored on type and digits on t3406 A1-A96 only; raise the fallback cap only once the 71-cell list is empty.

## Acceptance criteria
- [ ] A model that gives Excel's type on t3406 A1-A96 and loses no cell main gets right over every IRR case in eval
      (scored cell by cell, probes-numeric included), with the R4 hold-out split and a fresh sweep after the freeze.
- [ ] The rule in docs/excel-semantics/financial.md; the engine implements it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The last digits (T-3408) and `#DIV/0!` (T-3407).
