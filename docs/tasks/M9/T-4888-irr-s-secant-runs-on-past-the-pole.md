---
id: T-4888
title: "IRR's secant runs on past the pole: continue the run at d > 1 instead of failing, with an absolute stall bound"
milestone: M9
lane: functions-financial
status: done
priority: P1
size: S
tier: opus
depends_on: [T-4844]
---

## Goal
IRR's secant goes on past the pole (an iterate at d > 1) as Excel's does, instead of failing there, so the calls whose run crosses the pole return Excel's bits.

## Context
- docs/research/irr-2026-11-07/plan.md (T-4833) names the pole crossing as one of the three events the replica treated
  as a dead end. RATE's fitted solver runs on below -1 ("RATE holds on a flat secant and runs on below -1").
- Scratch evidence (T-4844 session): the run-on secant gives Excel's bits on 65 cells of
  probes-numeric/t2927-irr-fallback-sweep and probes/irr-fallback-roots, where the T-2928 fallback is 15 to 35 ulp off.
- Code: `irr_secant` (crates/rustsheet-engine/src/functions/financial/financial_util.rs) failed at d > 1.

## Acceptance criteria
- [x] A fresh sweep with predictions committed before the fill (probes/t4838-irr-pole-runon): the run-on secant is
      bit-exact on every cell where it converges.
- [x] `irr_secant` runs on past d = 1; a result past the pole fails the run; unit tests.
- [x] No eval regression; Gate A unchanged or better; financial.md "IRR's secant runs on past the pole".
- [x] `make check` green; reviewers approve.

## Out of scope
- What Excel does where the run-on secant does not converge (t4838 A116:A145; T-4840).
- The flat secant (T-4838).

## Outcome
`irr_secant` runs on past the pole. A step at d > 1 is evaluated at the negative v and the run carries on, d = 1
stays `#DIV/0!`, and a result past the pole fails the run. Two changes follow from the fresh sweep and the eval:
- the zero-step stall bound is absolute, |NPV| < 1e-4 (`IRR_STALL_RESIDUAL`, RATE's held-flat-secant bound, inside
  T-3120's absolute 1E-5..0.1 bracket). The relative bound returned gen-m9/fec39efaba9284c3 Calc!A283 once that run
  went on past the pole;
- `IRR_LONG_RUN_RANGE` covers 4 and 5 flows too: 12 fresh cells of 4 and 5 flows need 52..197 steps (no cell evidences
  2 or 3 flows, which keep the 50-step cap).

Evidence (decision 0188 R4):
- The rule was found on t2927-irr-fallback-sweep and irr-fallback-roots (65 cells bit for bit) as the train side.
- The fresh sweep probes/t4838-irr-pole-runon (160 cells, seed "T-4838/pole", excel-graph 2026-11-07) was drawn after
  that. Each model's prediction depends only on the seed and was generated before the fill (its `.predictions.tsv`,
  rewritten by the committed replay.py with the same values). The run-on prediction is
  Excel's bits on all 130 cells where it converges in 200 steps, against 15 for the old Newton fallback.
- The engine now passes 134 of the 160. The other 26 are mostly where the run-on fails; T-4840 owns them. A131 is
  `#NUM!` now, where the old fallback was 2 ulp from Excel.
- The cap widening is measured on the same fresh sweep: its 12 cells were predicted with a 200-step cap.
- The stall change was chosen on one eval cell (gen-m9/fec39efaba9284c3 Calc!A283), so its held-out measure is Gate A,
  whose tables no fit reads:
  - with the absolute bound, 16 accuracy/IRR points (s1, r4-T-3411, r4-T-3407; flows near 1E+11 to 1E+14) are in the
    envelope, with or without the run-on;
  - with the relative bound and the run-on, the same 16 fail.
  - So the stall bound moves the 16 points whose known-failure lines go; 0 failing either way.
  The t4838 predictions model no stall rule, so the fresh sweep does not test it.
- Attribution, from scratch builds: the run-on alone gives the t4838 bits and the 4 eval fixes below; the absolute stall
  alone gives the 16 Gate A points.

Eval (full run, external suites not fetched): 0 regressions. Known failures 4 fewer outside the new case:
gen-m9/a91bad4e84f49739 Calc!A369, probes/irr-guess-edges P!A28 and P!A29, and probes/t2565-irr-guess-sweeps P!C157,
the outlier T-2565 and T-3120 left. IRR cells off Excel outside t4838 and t4844 go from 727 to 663 (ulp 34 -> 18,
visibly-off numbers 113 -> 73). Also: the three t4844 math sidecars were stale (their case descriptions changed after
the sidecars were written, in #3910) and are regenerated.
Follow-ups: none new. T-4840 owns t4838's 26 cells; the flat secant stays T-4838's.

