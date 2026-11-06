---
id: T-4027
title: "IRR's level annuities where the secant fails: find Excel's path (t3406 A1-A96; uncapped Newton plus the residual cut breaks 39 of them)"
milestone: M9
lane: functions-financial
status: in-progress
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

## Cells (gap triage 2026-11-04b)
The span "A1-A96" names only its ends for `make unowned-off` (decision 0136), so the t3406 cells T-3411 left open
when it closed (its criterion 1 moved here) are listed by name. Excel returns the root, the engine is `#NUM!`
(`make eval` on origin/main a7b115e9e). The level annuities (this task's goal):
- probes/t3406-irr-num-boundary.case.json (28): P!A5, A6, A7, A8, A9, A10, A11, A26, A27, A28, A29, A30, A31, A32, A33, A49, A50, A51, A52, A53, A54, A55, A73, A74, A75, A76, A77, A78

The outlay families of the same case (a level annuity whose outlay is scaled; Excel alternates number and `#NUM!`
inside one family, T-3411 progress notes 2026-11-02), scored with the annuities since T-3411's criterion 1 covered
every t3406 cell; split them to a follow-up if the annuity model does not reach them:
- probes/t3406-irr-num-boundary.case.json (26): P!A97, A98, A102, A105, A107, A108, A117, A122, A123, A124, A126, A128, A129, A132, A133, A134, A135, A136, A137, A138, A139, A140, A141, A142, A143, A144

Measured for this triage (local, not committed): with `IRR_FALLBACK_STEPS` 50 -> 100000 and the residual cut kept,
t3406 gains A11, A31 and 15 outlay cells (A98, A102, A105, A108, A117, A122, A123, A124, A126, A128, A132, A134,
A138, A139, A142) at Excel's exact value and loses the 39 annuities named in Context; the other annuities above
return a number 11 to 14 digits from Excel's (A5: Excel -0.09999761744847369, Newton -0.09999761744841507), which
agrees with "most returning annuities are not Newton iterates from v = 0". The `#DIV/0!` cells A66-A95 are T-3407's.

## Out of scope
- The last digits (T-3408) and `#DIV/0!` (T-3407).
