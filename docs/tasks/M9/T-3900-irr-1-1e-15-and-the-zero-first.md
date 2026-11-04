---
id: T-3900
title: "IRR({-1,1E+15}) and the zero-first-flow gen-m9 set are #DIV/0! in Excel where ours is #NUM! (T-2802 remainder)"
milestone: M9
lane: functions-financial
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
IRR returns Excel's `#DIV/0!` for the cells T-2802 left unexplained.

## Context
T-2802 explained `IRR({-1,1E+300})` (the secant's iterate lands on d = 1, v = 0; financial.md "IRR's far-root
bound"). The rest stays `#NUM!` in the engine:
- probes/irr-guess-edges-2 A20, A28, A29 and probes/irr-guess-edges A38: `IRR({-1,1E+15})` from guesses 0.1, 1E+15,
  5E+14 and 0.05. Neighbours `IRR({-1,9E+14})` (irr-guess-edges-2 A19) and `IRR({-1,2E+15})` (A21) are `#NUM!` as
  ours. A plain-double trace of the fitted secant (`irr_secant`, functions/financial/financial_util.rs) stalls
  at the far root in all three (d = 0.999999999999999 for 1E+15, 0.9999999999999989 for 9E+14,
  0.9999999999999994 for 2E+15; v = 1 - d is 9, 10 and 5 steps of 2^-53) with NPV +0.055, -0.0008 and +0.11,
  and never reaches d = 1, so the separating rule is something else. Candidates to probe: X = 1E+15 +- a few ulp
  and other X giving 9 grid steps, a check on 1/v or on (1+r) after the stop, the sign of the last NPV.
- probes/irr-guess-edges A27-A29 and gen-m9/bc04c1f478c9896c Calc!A56: IRR over Data!A1:D12 (first flow 0,
  values 1E+15 among them) from guesses 0.05, 0.1, 0.5; `IRR({0,1E+15,2,42,1E+15,-0.0001})` (A34, A35) is `#NUM!`.
- gen-m9/530c636749a85209 Calc!A338 (`IRR({-1,1E+15},143)` inside HYPGEOM.DIST, `#DIV/0!` in Excel) is the same
  shape; T-3312 lists it and keeps it.
Cells:
- gen-m9/bc04c1f478c9896c.case.json (1): Calc!A56
- probes/irr-guess-edges-2 A20, A28, A29; probes/irr-guess-edges A27, A28, A29, A38

- T-3407 (2026-11-04): gen-m9/bc04c1f478c9896c Calc!A56 and probes/irr-guess-edges A27 now pass. The secant's step
  there lands where NPV cannot be evaluated, and the call is `#DIV/0!` (financial.md "IRR is #DIV/0! where its secant
  cannot evaluate NPV").

## Cells left by T-3407
Excel is `#DIV/0!` and the engine `#NUM!`, though every NPV the engine's secant evaluates is in range, so T-3407's rule
does not reach them (`make eval` on T-3407's branch, 2026-11-04). The overflowing step in Excel is not one of the
engine's secant iterates:
- probes/t3406-irr-num-boundary.case.json (6): P!A66, A67, A68, A81, A82, A95
- probes-numeric/t3411-irr-r4-fresh.case.json (3): Calc!A83, A85, A91
- probes-numeric/t3411-irr-r4-fresh-2.case.json (6): Calc!A1, A6, A19, A82, A92, A98
- probes-numeric/t3411-irr-r4-fresh-3.case.json (2): Calc!A24, A78
- probes-numeric/t3411-irr-r4-fresh-4.case.json (6): Calc!A4, A68, A86, A89, A94, A103

## Acceptance criteria
- [ ] Probe the d = 1 rule T-2802 implemented from one cell (probes/irr-guess-edges A39), per the T-2802
      Excel-fidelity review: `IRR({-1,1E+300},g)` from guesses other than 0.1 (does the call stop at `#DIV/0!`
      or retry from 0.1?), other flows whose secant lands on d = 1 exactly, and flows whose iterate passes d > 1
      (kept as a failed run from the fit, unprobed); then reword financial.md's Status line to what the cells show.
- [ ] The cells above pass (`make eval-baseline` drops them) with no new known-failure lines; the rule is in financial.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- IRR's returned digits (T-2131).
