---
id: T-3806
title: "IRR's secant runs up to 213 steps, not 50 (Excel's cap is 136..214)"
milestone: M9
lane: functions-financial
status: dropped
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
IRR's secant takes as many steps per run as Excel's does, so the slow runs Excel finishes (long level annuities,
long series near a root of 0) return Excel's digits instead of falling through to `#NUM!`.

## Context
- Split out of T-3411 (IRR on conventional flows after the secant fails), whose full model is still open: this is
  the one part of it the evidence pins. Findings and refuted candidates: T-3411's task file and financial.md
  "IRR's secant runs up to 214 steps".
- `irr_secant` capped a run at 50 steps as a directive-5 bound. In an engine-faithful replica (twice-rounded
  operators), runs of 51 to 136 steps stop on Excel's exact digits (7 probe cells), and three cells put Excel's cap
  below 215 (t3411-irr-r4-fresh A55, t3478-irr-r4-wide A240, m9-fin-irr-rate A125: Excel returns the retry's digits).

## Acceptance criteria
- [ ] `IRR_SECANT_STEPS` replaces the 50-step cap, with its evidence and FITTED range. (Dropped, see Outcome.)
- [ ] No eval cell regresses; the R4 guards are met and recorded below. (Failed: A227 below.)
- [x] financial.md records the findings, T-3411's included.
- [ ] `make check` green; reviewers approve.

## Out of scope
- What Excel does after both secant runs fail (T-3411), `#DIV/0!` (T-3407), the last digits (T-3408).

## Outcome
Dropped: no step cap is Excel's rule, so none ships. The engine keeps its 50-step cap; PR #2822 lands only the research.
- Round 2 froze a cap of 213 (from a bracket of 136..=214) before the fresh sweep t3411-irr-r4-fresh-2 (83/150 against
  76/150 with 50). The Excel-fidelity review found what the per-case counts hid: probes-numeric/t3478-irr-r4-wide A227
  is `#NUM!` in Excel and on main, while with cap 213 the guess run converges at step 189 to -0.6781680690607276
  (confirmed with `rustsheet-eval show` on the cap-213 build). A201 and A221 match Excel only with a cap of 195 and 200
  or more, so no single cap fits all three; the review's totals over the six IRR cases were 50 = 594, 136 = 630,
  170 = 639, 200..214 = 641, 215 = 640, 230 = 637. A cap chosen from that table would be a best-scoring fit, not an
  Excel fact, and a fit that moves a point out of the Gate A envelope fails R4.
- The Gate A envelope counts first recorded here also counted cells where Excel is `#NUM!` and the engine a number as
  inside (sweep 1 A31, A39, A45, A144, A148; sweep 2 A50, A132, A146): with cap 213 the counts were at most 89 and 92
  of 150, not 94 and 95.
- What lands: the two fresh sweeps (probes-numeric/t3411-irr-r4-fresh and t3411-irr-r4-fresh-2, excel-graph
  2026-11-02, 50-digit roots in their `.math.tsv`) and their generator (`cd tools && uv run python -I
  ../docs/research/t3411/gen_sweep.py <repo> [case name]`, seeds "T-3411/r4-fresh" and "T-3411/r4-fresh-2"), and the
  findings in financial.md "IRR's secant runs longer than 50 steps, but no step cap is Excel's rule" and T-3411's task
  file. Eval delta: 0 gated cells (no engine change); the two new probes-numeric cases pass 80/150 and 76/150 on main's
  engine (probes-numeric has no eval/baseline file).
- The model of Excel's long runs stays with T-3411.
