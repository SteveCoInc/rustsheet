---
id: T-3424
title: "RATE's type at the noise floor: Gate A sweep cells where one side errs"
milestone: M9
lane: functions-financial
status: in-progress
priority: P0
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
RATE gives Excel's type, a number or `#NUM!`, on the 16 Gate A sweep cells of the T-3348 sweeps where the secant
ends in the residual's rounding noise and the engine and Excel disagree on the type. Decision 0188 R2 allows 0 such
cells, so RATE's Gate A fails until they are settled.

## Context
- T-3348 made a flat secant hold at its iterate (financial.md "RATE holds on a flat secant and runs on below -1").
  The iterate returns when |f| < 1e-7 or when it passes the cap's looser test (|f| < 1e-4); otherwise the run is
  `#NUM!`. Decision 0198 records the trade.
- **Cells T-3348 lost (13).** The old model's `#NUM!` matched Excel; the hold returns a number. Returning `#NUM!`
  on them is a goal of this task, through a model of Excel's noise, not through a rule on the hold (decision 0198):
  - probes-numeric/t3348-rate-gate-a-sweep Calc!A296:
    `=RATE(351,-1684985188.67608,607019725001.509,-15796156082.0366,0)`. Excel `#NUM!`; the engine gives
    1.8814890660971136e-6, 2.4e-8 relative off the math (0.0000018814890215042125897557712909148668011313466560856).
    From step 56 the iterates bounce in the noise (|f| 1 to 40 on terms of 6e11) until a flat secant at step 119
    lands on |f| = 2.1e-5 and holds.
  - probes-numeric/t3348-rate-gate-a-fresh (the sweep drawn after the freeze) Calc!A59, A70, A74, A77, A89, A90,
    A95, A157, A162, A173, A275 and A292. The cells, Excel's `#NUM!`, the engine's values and the 50-digit roots are
    listed in T-3348's Outcome. They hold at steps 11 to 89 on |f| 2.4e-7 to 5.5e-5.
- **Cells neither model gets right (3).** Excel returns a number; the engine gives `#NUM!`:
  - t3348-rate-gate-a-sweep Calc!A272: `=RATE(4,-70299017520.6175,254784242189.789,-1503672.27062336,0)`, Excel
    0.040657819686895275, math 0.040657819686896052522303945460859198781648510530346. It holds at step 8 on
    |f| = 1.286e-4, above the bound. Excel's value is about 2 ulps from the model's held iterate
    0.04065781968689529, so Excel's last steps differ from the model's.
  - t3348-rate-gate-a-fresh Calc!A251 and A272 (in T-3348's Outcome). They hold at |f| 2.4e-4 and 9.5e-4.
- All 16 have |pv| and |fv| from 1e6 to 1e12. Across the 1,796 RATE oracle cells, Excel returns 133 holds and errs
  on 13 below the 1e-4 bound, and errs on 95 and returns 3 above it. No rule on the hold gives them all Excel's type
  (decision 0198):
  - an absolute bound: Excel errs at 2.4e-7 and returns up to 9.5e-4;
  - a relative bound: errors and returns overlap from 5e-20 to 1.4e-16;
  - zero step against distinct iterates: Excel errs on 12 of 125 zero-step holds and 1 of 21 distinct ones;
  - a step cut: the losses run from step 11 to step 119.

  A bound below 2.1e-5 alone would turn 5 gated cells Excel returns into `#NUM!` (probes/t3348-rate-fv-nonzero
  A113, A117, A126, A167, A183, held |f| 3.9e-5 to 7.2e-5).
- **Gate A RATE table (2, from T-3585).** accuracy/RATE/s1 Calc!A278
  `RATE(29,622792052.26538,-18060969526.6562,179.964221458841,0)` and A291
  `RATE(43,-1768871036.99479,76057734597.4655,-42.6330901500337,0)` are `#NUM!` in Excel; the engine holds on a flat
  secant at |f| 5.0e-7 and 3.0e-6 (steps 59 and 41) and returns -9.45E-10 (root 6.24E-10) and 2.2231784E-6. They are
  carried in eval/accuracy/known-failures.tsv with this task as owner. probes/t3585-rate-num-near-minus-one columns G
  and H scale their pv and pmt by 2^-k and vary fv: Excel's error comes and goes with the scale (A291 errs at k = 0
  and 6, not 2 or 4), and the engine has Excel's type on the other 35 cells (financial.md "RATE is #NUM! when
  1 + r is 1e-7 or less").
- **probes-numeric/t3478-rate-r4-wide Calc!A285 (from T-3547).** `RATE(3,-27701521124.6443,83584699115.003,
  -1296.87658562967,0,228.952995156186)` is `#NUM!` in Excel; the engine returns -0.0028749021254309863 (math
  -0.00287490212542650...), |pv| 8E+10.
- The lead is the residual's bits near the root, the same cause as T-3389's misses. A171's second iterate needs
  Excel's f to be about -2.71e-7 where the model has -1.67e-7. A residual form that reproduces Excel's noise likely
  settles both tasks; it will be a structural model under reviewer check 7, with its own hold-out and fresh sweep.
- Code: `compute_rate` and `rate_residual` in crates/rustsheet-engine/src/functions/financial/financial_util.rs; fit
  lab `rate` (tools/src/rustsheet_tools/fitlab/kernels/solvers.py).

## Acceptance criteria
- [ ] The 16 cells above have Excel's type, and the numbers are within 15 significant digits of Excel. If Excel's
      noise cannot be reproduced, the task records the residual forms tried and the cells stay known misses;
      RATE's Gate A then stays failed (no exception exists for it).
- [ ] No gated cell regresses, and the 5 fv-nonzero holds above stay numbers.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The cells of the right type that are not Excel's bits (T-3389).

## Progress notes (2026-10-31, opus run; resumed 2026-11-01 after 3 h idle)
- First experiment: the power form inside the residual. The T-3348 hold model (the engine's `compute_rate`) was re-run
  in Python with P = (1+r)^n from five forms, every other operation Excel's twice-rounded one, on the 1,260 literal
  RATE cells of t3348-rate-gate-a-sweep, t3348-rate-gate-a-fresh, t3348-rate-fv-nonzero and the fit lab's
  `RATE_CASES` (number or `#NUM!` type agreement / bit-exact numbers):
  | P | type | bits |
  |---|---|---|
  | Excel's `^` model (`caret_pow`, shipped) | 1211 | 990 |
  | correctly rounded pow of Excel's 1+r | 1203 | 256 |
  | correctly rounded (1+r)^n, r exact | 1204 | 201 |
  | `exp(n*log(1+r))` in doubles | 1234 | 252 |
  | `exp(n*log1p(r))` in doubles | 1239 | 190 |
- So the `^` model is Excel's for the bits (990 against at most 256): RATE's noise does not come from a different
  power routine. The log/exp forms win type on the noise-floor cells only by moving the noise, at the cost of most
  bits, so they are not Excel's either. Next: per-cell comparison on the 16 cells (the experiment's watch list did not
  match the cell keys), then the residual's sum order and fused forms (pv·P + pmt·A as one extended sum before + fv,
  or the 80-bit sum kept across all three terms) under the same hold.
- Second experiment (2026-11-01, opus run that did T-3389): the precision of each residual operation, on the same
  hold model. The operations are 1+r, the power's products, P-1, the quotient by r, the two products and the two
  sums. Each was twice-rounded or kept at 64 bits as in an x87 register, and P-1 was also tried exact: 64
  combinations. They were scored on the 600 cells of t3348-rate-gate-a-sweep and t3348-rate-gate-a-fresh,
  exploration only, no fit taken from them. The shipped form gives 584 types and 562 bits, and 0 of the 16. The best
  on the 16 keeps 1+r and the power's products at 64 bits with P-1 exact: 5 of 16, with 581 types and 126 bits. The
  best on all types keeps the two products at 64 bits: 588 types and 4 of 16, but 177 bits. Every variant loses 176
  to 436 bits. So no precision change in the residual is Excel's noise. This also covers the fused forms the first
  experiment named next: the two sums kept at 64 bits. T-3389 adds the algebraic forms and the step forms, also
  negative, in financial.md "Not reproduced".

## Unblocked (2026-11-04, decision 0264 stale-draft triage)
T-3579, the blocker, is `done` on `main` (#2728): Excel's residual bits at nper 1 are probed, and the misses are
flat-secant restarts. RATE has also changed on `main` since this branch's experiments: T-3585 (#2816) makes RATE
`#NUM!` when a converged iterate has 1 + r at or below 1e-7. The next run re-scores the 16 cells on current `main`
before it uses the two experiments above, then fits the residual from T-3579's cells.
