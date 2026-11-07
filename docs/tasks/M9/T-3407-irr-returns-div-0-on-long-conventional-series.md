---
id: T-3407
title: "IRR returns #DIV/0! on long conventional series near -0.2 to -0.5 (9 hold-out cells; the engine is #NUM!)"
milestone: M9
lane: functions-financial
status: done
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
IRR returns `#DIV/0!` where Excel does on conventional cash flows, instead of `#NUM!`.

## Context
- probes-numeric/t3347-irr-conventional-holdout A123, A126, A130, A131, A132, A134, A135, A138 and A143 (9 cells of
  the neg-long stratum): long series (36-60
  flows, uneven or level, scales 1 to 1e10) after one outlay, exact root r = -0.23 to -0.47. Excel (excel-graph,
  2026-10-31) is `#DIV/0!`; the engine is `#NUM!`, with or without T-3347's stage. The other 15 cells of the stratum
  are `#NUM!` in both.
- A Python replica of the secant (T-3347 Outcome) shows that on 7 of the 9 the first secant step lands where NPV
  overflows (no finite value), and on A131 and A138 the secant goes flat at step 3 and both Newton runs overflow; the
  `#NUM!` cells of the stratum never overflow. A first guess: an overflowing NPV inside Excel's iteration surfaces as
  `#DIV/0!` instead of failing the run. Probe it with short series built to overflow at the first step, and with a
  guess that avoids it.
- docs/excel-semantics/financial.md "IRR on conventional flows near 0 and -10%" (the hold-out paragraph).
- Code: `irr_npv`, `irr_secant`, `compute_irr` (functions/financial/financial_util.rs).

## Acceptance criteria
- [x] A probe case (excel-graph) that isolates when IRR is `#DIV/0!`, its cells listed here before the fix. (Met by
      existing cells instead of a new probe; see the Outcome.)
- [ ] The engine returns `#DIV/0!` on those cells and on the 9 hold-out cells, with no gated cell regressing.
      (Partly: 7 of the 9 hold-out cells; A131 and A138 move to T-3900. No gated cell regresses.)
- [x] The rule in docs/excel-semantics/financial.md.
- [x] `make check` green; reviewers approve.


## Cells of T-3411's fresh sweeps
T-3411's R4 sweeps 3 and 4 (cases added by PR #3063, scored on its round-4 model), where Excel is #DIV/0!, the engine #NUM!:
- probes-numeric/t3411-irr-r4-fresh-3.case.json (12): Calc!A6, A18, A19, A21, A23, A24, A78, A79, A81, A85, A89, A93
- probes-numeric/t3411-irr-r4-fresh-4.case.json (16): Calc!A2, A3, A4, A11, A13, A14, A15, A54, A68, A77, A85, A86, A89, A94, A103, A147

## Cells left by T-3411's close (gap triage 2026-11-04b)
T-3411 owned these until it closed (PR #3063); no open task named them after it. Excel is `#DIV/0!`, the engine
`#NUM!` (`make eval` on origin/main a7b115e9e; the engine stays `#NUM!` on all 38 with the fallback's Newton cap
lifted, so they are not a cap effect). On t3406 these are the level annuities where T-3411's replica put
log10 |NPV(d2)| above 228 (T-3411 progress notes, 2026-11-02):
- probes/t3406-irr-num-boundary.case.json (21): P!A66, A67, A68, A69, A70, A71, A72, A81, A82, A84, A85, A86, A87, A88, A89, A90, A91, A92, A93, A94, A95
- probes-numeric/t3411-irr-r4-fresh.case.json (7): Calc!A3, A5, A77, A83, A85, A88, A91
- probes-numeric/t3411-irr-r4-fresh-2.case.json (10): Calc!A1, A6, A11, A16, A19, A52, A82, A91, A92, A98

## Out of scope
- Which series return a number (T-3406).

## Decision 0267
The owner's cut-loss plan (decision 0267, 2026-11-04) makes this a Tier A task, worked first: its cells have the wrong type or error, or are first wrong at the 11th digit or earlier. Decision 0056's time-box applies. If two attempts do not move the cells, block the task as time-boxed, and decision 0267 accepts the cells that are left.

## Outcome
Done, first attempt: a secant step whose NPV cannot be evaluated ends IRR with worksheet NPV's `#DIV/0!`.
- Engine: `irr_secant` maps an NPV that `irr_npv` cannot compute (a discount power out of the double range, a sum that
  is not finite) to `SecantStop::DivZero`, which ends the call as `#DIV/0!` without the retry from 0.1. Before this,
  it was a failed run (retry, fallback, then `#NUM!`). Unit test `irr_is_div_zero_where_its_secant_cannot_evaluate_npv`
  (t3406 P!A69, P!A84 `#DIV/0!`; P!A83 `#NUM!`).
- Candidates scored cell by cell over every IRR cell of probes, probes-numeric and gen-m9 against main: (A) any secant
  NPV that cannot be evaluated: 59 better, 0 worse, Gate A unchanged (shipped); (B) |NPV| above 1E+228 at an iterate
  (T-3411's observation): 20 better, 8 worse (rejected).
- Criterion 1, met by existing cells instead of a new probe: t3406 P!A83 and P!A84 are the same 60-flow annuity, and
  only A84's secant step leaves the double range. Across the scored cells, Excel's `#DIV/0!` appears where the
  engine's step overflows, and no cell where it overflows is anything else in Excel. Not isolated by any cell: an
  unevaluable NPV at the guess point (d0 or d1) itself, and an overflow in the retry from 0.1 after a failed guess run.
  That the error comes from NPV's evaluation is inferred, not observed (financial.md's status line).
- Criterion 2, partly met: 7 of the 9 hold-out cells pass (P!A123, A126, A130, A132, A134, A135, A143). A131 and A138
  (the secant goes flat at step 3, and both Newton runs overflow) stay `#NUM!` and move to T-3900. Of the 75 cells
  listed above, 52 pass, and the other 23 (no step of the engine's secant overflows there) move to T-3900 too, under
  "Cells left by T-3407". The rule also fixes t3478-irr-r4-wide A261, A275, A277, A281, A289, A290 and A298
  (T-3548's), probes/irr-guess-edges A27, and gen-m9/bc04c1f478c9896c A56 (T-3900's).
- Gate A ownership: this task's three known-failure lines (accuracy/IRR/r4-T-3411 A213, A232, A233; Excel `#DIV/0!`,
  the engine `#NUM!`) and the new batch's A237 (the same symptom) move to T-3900.
- R4 guards (structural rule, kind c; frozen before the draws):
  - R4.1, fit lab split (`holdout.is_holdout`), 212 cases calling IRR, main -> rule: train 52,909 cells, bit-exact
    51,253 -> 51,296 (96.87% -> 96.95%); test 23,002 cells, 22,289 -> 22,305 (96.90% -> 96.97%), within the 5-point
    margin; type/error mismatches 586 -> 543 and 262 -> 246; 0 cells worse on either side.
  - R4.2 fresh probes-numeric sweep (docs/research/t3411/gen_sweep.py, now seeded per task: `gen_sweep.py <repo>
    t3407-irr-r4-fresh`, seed "T-3407/r4-fresh"), oracle-filled from excel-graph and scored once: 7 better (A19, A23,
    A70, A76, A82, A86, A99), 0 worse, `#DIV/0!` mismatches 10 -> 3. Not committed: decision 0267's probe freeze allows
    a new probes-numeric case only for owned cells.
  - Gate A batch accuracy/IRR/r4-T-3407 (300 points, `rustsheet-accuracy generate IRR --batch r4-T-3407`, excel-graph):
    25 points outside the envelope with and without the rule, the same points. None of its points reaches the new
    path: no step overflows there, so the batch shows only that nothing moves. The fresh probes sweep above is the
    out-of-sample test that runs through it (7 cells). Known-failure lines: T-3548 17 (the families of its s1 and
    r4-T-3411 lines), T-3347 7 (roots near 0), T-3900 1 (A237). `rustsheet-eval accuracy`: 0 failing.
- Eval delta: probes 16 and gen-m9 1 baseline cells fixed (`make eval-baseline`), probes-numeric (ungated) 42 cells
  fixed, 0 regressions.
- financial.md: "IRR is #DIV/0! where its secant cannot evaluate NPV" (new), and the t3478 / Gate A section's links.
