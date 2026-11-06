---
id: T-4027
title: "IRR's level annuities where the secant fails: find Excel's path (t3406 A1-A96; uncapped Newton plus the residual cut breaks 39 of them)"
milestone: M9
lane: functions-financial
status: blocked
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

The level annuities of T-4027's own probe (attempt 2), where Excel returns the root and the engine is `#NUM!`:
- probes/t4027-irr-annuity-scale-guess.case.json (35): P!A1:A4, A7:A10, A13:A16, A19:A22, A25:A28, A34, A35, A43:A46, A49:A51, A54:A56, A59:A61

## Out of scope
- The last digits (T-3408) and `#DIV/0!` (T-3407).

## Progress notes
2026-11-06, opus run, attempt 1 of 2 (decision 0056; no code changed). The replica was a scratch test inside the engine
crate that called `irr_npv` and `extended_quotient` directly, so its arithmetic is the engine's twice-rounded one. It
scored t3406 A1-A96 on Excel's digits (the returning cells) and on type.
- Main's secant from 0.1, by cell: A1-A4 and A25 converge. A5-A9, A26-A29 and A49-A51 hit the 50-step cap, oscillating
  between d of about 0.0919 and a far point. The rest stall flat at step 6: the far point's NPV (1.3E+20 at A11) makes
  d3 equal d4 at 0.091909 with the same NPV. Uncapped, the secant gives Excel's digits on A5 (138 steps) and A26 (120).
  It gives other digits on A6 (1719 steps, 9E-14 off), A7, A27 and A49, it never converges on A8, A9, A28, A29 and A50,
  and it stalls flat on A10. So Excel's path leaves the plain secant somewhere after step 138 on these cells.
- Exact digits: no candidate reproduces Excel on A6, A7, A9, A27-A30, A33, A49, A51, A54, A76 or A77. The candidates
  were Newton in v (from 0, 1/(1 + guess), 1 or the far point) and Newton in d (from the guess or the far point). They
  also include a secant in d, r or v from 200 guesses in -0.99..1 with offsets ±1E-3, 1E-4 and 1E-2, and a secant
  restarted from its last or best iterate after a cap of 20 to 500. Every Newton variant converges to the exact root.
  Excel's returns are 1E-15 to 9E-12 from it, so they are secant-like iterates stopped on a 1E-7 step, not
  Newton-converged roots. The digit route is therefore unlikely to find the path; T-3408 owns the digits anyway.
- Type, Newton step counts at each n's cut (last returning cell | first `#NUM!` cell). From v = 0: 126|134, 224|242,
  314|345, 578|650 for n = 20, 30, 40, 60. From v = 1/1.1: 45|52, 91|109, 130|160, 295|366. From v = 1: 27|33, 49|64,
  58|84, 137|205. From the far point: 44|51, 90|108, 129|160, 294|366. No constant cap separates all four n for any
  start. A cap of C·n^p fits for p in 1.33..1.45 from v = 0, but that is numerology on 4 brackets, not a rule.
- Type, other cut quantities (none is common to all four n): the root (-0.22|-0.24, -0.18|-0.20, -0.14|-0.16,
  -0.12|-0.14), P/n, P/n^1.5, P/n^2, log P/n, P^n (Newton's first iterate from v = 0 is v = P), the far point v (10.1|15.1,
  23.4|43.5, 28.4|62.8, 153|519) and its NPV (refuted in T-3411).
- Bracketing stage, tried in the same run. Every failed run from 0.1 on A5-A68 and A73-A83 leaves a sign change
  between its last negative point (d of about 0.0919) and its last positive point (the far point). Regula falsi on
  that bracket does not converge on A6-A68 or A73-A83, even in 100,000 steps. Bisection to a width or |NPV| of 1E-7
  takes, at each n's cut (last return | first `#NUM!`), 28|29, 29|30, 30|31 and 32|34 steps. That is the closest so
  far, but no single cap fits all four n, because the bracket width is the far point's |d|: 9.1|14.1, 22.4|42.5,
  27.4|61.8 and 152|519. Width alone does not separate the four n either: n = 20 is `#NUM!` at 14.1, n = 30 returns
  at 22.4.
- For attempt 2: A8-A10 return in Excel although no secant run converges on them. So Excel has a second stage after
  the secant fails, and how far it reaches decides the type. The four t3406 brackets cannot pin that stage. A probe
  would help: annuities whose far point and root are moved independently, for example the same P and n with a
  guess other than 0.1, which moves the far point, or a level annuity with flows of 2. Score candidates on type
  over A1-A96 first, then over every IRR cell in eval, cell by cell.

2026-11-06, opus run, attempt 2 of 2 (decision 0056; no engine change). It added a probe that moves the far point and
the flows' scale independently of the root: probes/t4027-irr-annuity-scale-guess (excel-graph, 68 cells; the engine's
predictions were committed before the fill, docs/research/t4027/predictions.md). Findings (financial.md "IRR on level
annuities where the secant fails: the type cut ignores the flows' scale and guesses 0 to 0.3"):
- The cuts are n = 20: returns through r = -0.23, `#NUM!` from -0.24; n = 40: returns through -0.15, `#NUM!` from -0.16.
- Scaling every flow by 2, 1000 or 0.001 moves no type, so no absolute NPV tolerance is the cut (unlike the 1E-4
  residual). Guesses 0 and 0.3 give the default's exact results, so the stage that decides the type does not see the guess.
  Guess -0.1 returns at -0.23 and -0.24 with digits of its own.
- Refuted on this run (a scratch engine build, scored on t4027 and t3406): a secant restart from any fixed guess in
  -0.6..0.7 (step 0.005; no guess gives Excel's digits on A1, A2, A4 or A49); any secant cap from 50 to 100000 (at most
  3 cells move); a fallback cap of 100, 200, 400 or uncapped, with a secant cap of 50 or 200 (uncapped, it returns at
  -0.24 and -0.25, where Excel is `#NUM!`).

## Blocked because
Blocked because: time-boxed (decision 0056)
- Two attempts found no model of Excel's path that gives the type on t3406 A1:A96. The candidates tried are in the
  Context's refuted list and in both progress notes: Newton from v = 0, 1/(1 + guess), 1 or the far point, under any
  cap or C*n^p; secant in d, r or v from fixed guesses, restarted or capped; regula falsi and bisection on the failed
  run's bracket; cut quantities (root, P/n powers, far point, its NPV); absolute NPV tolerances (refuted by the scale
  probe).
- What is known: the type stage runs after the retry from 0.1, ignores the guess and the scale, and its cut moves with
  n (-0.23|-0.24 at n = 20, -0.15|-0.16 at n = 40). Its digits are neither Newton-converged roots nor iterates of a
  secant from a fixed guess.
- The cells stay owned here: t3406's 54 above and t4027's 35.

