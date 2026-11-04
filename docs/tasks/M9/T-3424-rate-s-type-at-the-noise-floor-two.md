---
id: T-3424
title: "RATE's type at the noise floor: Gate A sweep cells where one side errs"
milestone: M9
lane: functions-financial
status: blocked
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

## Progress (2026-11-04, opus run, third attempt)
- **Re-scored on current `main`:** all 16 cells still have the wrong type, with T-3698's restart and T-3585's floor
  in place. The engine returns a number on sweep A296 and fresh A59, A70, A74, A77, A89, A90, A95, A157, A162,
  A173, A275 and A292, and gives `#NUM!` on sweep A272 and fresh A251 and A272.
- **Probe:** probes/t3424-rate-noise-trace (433 cells, excel-graph, 0 fill failures). Generator and scorers are in
  docs/research/t3424/.
  - FV at 355 iterates of the model's run on the 16 cells reads Excel's FV (the residual less + fv) along the path.
  - RATE seeded with the last 12 iterates of each nper 1 and 2 cell (78 cells) reads Excel's run from there.
- **Finding:** no residual difference shows in Excel's FV, but Excel's RATE path still leaves the model's in the noise
  floor, so RATE's own residual, step or stop test differs somewhere, unobserved (financial.md "Excel's FV along
  the path is the model's annuity value; how RATE decides in the noise floor is not").
  - Excel's FV is the model's annuity value on 355 of 355 cells, at nper 1 to 351.
  - The model's step form is bit-exact on 1,951 of the 2,475 literal RATE cells. Six other forms get 1,039 to
    1,945, and 80-bit iterates get 1,111 and 1,048.
  - Seeded runs that hold at the same iterate get different Excel outcomes, a number on one and `#NUM!` on the other
    (fresh A275).
  - No single quantity at the hold separates Excel's 549 returned holds from its 60 errors.
- No engine change was made, so no gated cell regresses. T-3963's context records this probe's result.
- **Follow-up:** T-4146 (opus). Excel's FV is the twice-rounded form on all 355 probe cells and on all 587 literal FV
  oracle cells. The engine's plain-double FV misses probe A132 (587 against 586).
- **Eval delta:** probes +1 case (433 cells: 394 pass, 39 new-coverage known failures, 0 regressions;
  `make eval-baseline`). No other case changes.

## Decision 0267 cells
Decision 0267 (the owner's cut-loss plan, 2026-11-04) gives this task 17 probes-numeric cells that no open task listed: 15 with the wrong type or error, 2 first wrong at the 12th or 13th digit.
These come from the run at main 985d511b. They are Tier A (a wrong type or error, or the 11th digit or earlier) or Tier B (the 12th or 13th digit). Tier A is worked first.
Decision 0056's time-box applies. If two attempts do not move these cells, block the task as time-boxed: decision 0267 then accepts the cells that are left, and no follow-up is filed for them.

- probes-numeric/t3348-rate-gate-a-fresh.case.json (16): Calc!A59, A70, A74, A77, A89-A90, A95, A157, A162, A173, A251, A272, A275, A280, A285, A292
- probes-numeric/t3348-rate-gate-a-sweep.case.json (1): Calc!A272

- [ ] Every cell listed above matches Excel within half a unit of the 15th significant digit, or has Excel's type and error. Otherwise the task is blocked under decision 0056 and the rest is accepted (decision 0267).

## Visible-parity stage
Every cell in "Decision 0267 cells" is within half a unit of Excel's 15th significant digit, or has Excel's type, error and text (decisions 0083, 0267).

## Blocked because: time-boxed (decision 0056)
Three attempts have not moved any of the 16 type cells: the power-form experiment (2026-10-31), the residual-precision
experiment (2026-11-01; both in "Progress notes") and this run's probe (2026-11-04, "Progress" above). Decision 0267
item 4 applies: the cells this task lists, including its "Decision 0267 cells", are accepted, and no follow-up is
filed for them. RATE's Gate A stays failed on them.

Candidates tried, none of which moves a cell:
- the power P = (1+r)^n: Excel's `^` model (shipped), correctly rounded pow, exp(n·log(1+r)), exp(n·log1p(r)), the x87
  sequence;
- each residual operation twice-rounded, kept at 64 bits, or P-1 exact (64 and 96 combinations), k·(P-1) before /r,
  the expm1 factor and the PV form (T-3389);
- six step forms, a flat secant as `#NUM!`, and iterates kept at 80 bits (this run);
- single thresholds on 13 quantities at the hold (this run).

Reopen only on a new hypothesis recorded here (decision 0056). This run's probe found no difference in Excel's FV along the
path and ranked the model's step first, yet Excel's RATE path leaves the model's: seeded runs that hold at the same
iterate get different Excel outcomes. A direct read of RATE's own residual or step (a seeded or one-step RATE probe)
is the next hypothesis. T-3963 studies that for the flat-secant cells it owns.

## Landing notes (2026-11-04; adopted as an orphan, work-loop step 0.3)
- Round 4 (full panel on 119d188f9): security and design APPROVE; fidelity REQUEST_CHANGES on one stale pointer in
  docs/research/t3424/README.md, which still named the old financial.md heading. Fixed in the next commit; the panel
  re-runs on that head before the PR is enqueued.
- Do not repeat: do not state that RATE's residual or step is Excel's; the probe reads FV, and the step result is a
  ranking. Keep T-3424 `blocked` (time-boxed, decision 0267 item 4), not `done`, and keep its cells with it.
