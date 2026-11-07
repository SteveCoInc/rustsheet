---
id: T-4213
title: "IRR at a root within 0.03 of zero: Excel's last iterate and ours sit a different 1E-16 from the root (9 unowned t3411 fresh-sweep cells, 2 at the 11th digit, decision 0267 Tier A)"
milestone: M9
lane: functions-financial
status: blocked
priority: P1
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
On conventional series whose root is close to zero, IRR returns Excel's value: the 9 listed cells match Excel within
half a unit of the 15th significant digit. Excel is not the accurate side there, so the task finds which iterate or
stop Excel returns and reproduces it. If two attempts fail, the task blocks as time-boxed (decision 0056).

## Context
- Gap triage 2026-11-04 (`make eval` on origin/main b450050f8, then `make unowned-off ALL=1` and a per-cell owner scan).
  These 9 IRR cells are visibly off and no open task lists them. T-3548 lists 160 other cells of the same three cases,
  and T-3411 (done) built the sweeps. All 9 cells are numbers in both Excel and the engine, and both find the root.
- The roots are 4.4E-6 to 0.029. Both sides are within 3E-17 to 1.5E-15 absolute of the 50-digit root (mpmath
  `findroot` on the literal flows, gap triage). Near zero, a 1E-16 absolute step becomes an 11th to 15th-digit
  relative difference. Examples:

  | cell | root (50-digit) | Excel | ours | first wrong digit |
  |---|---|---|---|---|
  | fresh-3 A38 | -7.8420403629003E-6 | -7.842040362748826E-6 (+1.5E-16) | -7.84204036297087E-6 (-7.1E-17) | 11th |
  | fresh-3 A45 | -4.4300532125970E-6 | -4.430053212489682E-6 (+1.1E-16) | -4.4300532127117265E-6 (-1.1E-16) | 11th |
  | fresh-3 A109 | 2.2378224888395E-4 | 2.2378224888375264E-4 (-2.0E-16) | 2.2378224888242038E-4 (-1.5E-15) | 12th |
  | fresh-2 A32 | 1.6845316336509E-4 | 1.684531633652231E-4 (+1.3E-16) | 1.6845316336500105E-4 (-8.9E-17) | 13th |
  | fresh-2 A43 | -2.2000456147424E-3 | -2.2000456147424075E-3 (+2.7E-17) | -2.2000456147435177E-3 (-1.1E-15) | 13th |
  | fresh-4 A110 | 1.1271176460087E-4 | 1.1271176460070365E-4 (-1.7E-16) | 1.1271176460092569E-4 (+5.3E-17) | 13th |

  The other three cells (fresh-2 A30, A37 and A44) are first wrong at the 14th or 15th digit.
- Excel's error against the 50-digit root is 8E-15 to 2.4E-11 relative. That is below decision 0188 R3's G = 1E-4, so
  Excel stays the target. Excel's absolute error is under 2.4E-16 on all 9 cells. It looks like the last step of a
  Newton or secant iteration whose NPV is evaluated at its rounding noise. Ours is up to 1.5E-15 off (A109, A43). That
  suggests the engine stops one step earlier, or takes its last step from a noisier residual.
- Things to check first:
  - which of `irr_secant`, `compute_irr` and `irr_fallback` returns each cell, and at which step;
  - whether Excel's value is the model's next or previous iterate;
  - financial.md "RATE and IRR secant constants past their fit sets" and T-3411's round-4 notes
    (docs/tasks/M9/T-3411-irr-on-conventional-flows-model-what-excel-does.md): the residual stop, `IRR_ROOT_RESIDUAL`,
    and the noise-floor findings ("in the noise floor Excel's iterates differ from the model's", T-3850);
  - the RATE noise-floor work in T-3963 and T-3424 (in flight), which may share a residual model.
- Coordinate with T-3548 (P0, claimed): it changes the same solver for the `#NUM!` and `#DIV/0!` cells of these
  sweeps. Re-measure these 9 cells on main after it lands before fitting anything.
- Code: crates/rustsheet-engine/src/functions/financial/financial_util.rs (`irr_secant`, `compute_irr`, `irr_fallback`).

## Acceptance criteria
- [ ] Every cell in "Decision 0267 cells" is within half a unit of Excel's 15th significant digit. Otherwise two
      attempts are recorded and the task blocks as time-boxed (decision 0056), and decision 0267 accepts what is left.
- [ ] docs/excel-semantics/financial.md records what Excel returns at a near-zero root and cites these cells. The
      engine change carries a comment citing the same evidence.
- [ ] R4 guards (decision 0188, reviewer-excel-fidelity check 7) for any fitted or structurally chosen step or stop:
  - [ ] Hold-out: train and hold-out counts side by side (bit-exact, within 4 ulp, within 15 digits) for the chosen
        candidate and for the correctly rounded root, from the fit lab's split or a stated manual split of about 30%
        by case or cell hash, with the margin fixed before scoring. The chosen candidate beats the correctly rounded
        root on hold-out, and its hold-out exact rate is within the margin of its train rate.
  - [ ] Fresh sweep: after the fit is frozen, a new log-uniform sweep of conventional series with roots between 1E-7
        and 0.05 in absolute value, oracle-filled. Its result is recorded in the Outcome.
  - [ ] Gate A envelope: the fresh sweep's count inside the envelope, with every point outside it listed, and no IRR
        Gate A point moved out of it (`rustsheet-eval accuracy`).
  - [ ] `FITTED(T-4213, n_cells, range)` on each fitted piece. Outside its range, the code uses the accurate root.
  - [ ] Reproducibility floor: no step targets differences below 4 ulp web against desktop, or bits that vary
        between fills (decision 0112).
  - [ ] No keep-matched branch: no band or special case is defined by the set of cells that already match.
- [ ] No gated cell regresses. IRR's 263-cell secant fit set stays bit-exact. `make check-fast` is green locally,
      CI is green, and reviewers approve (excel-fidelity included).

## Out of scope
- The type mismatches and roots the engine misses (T-3548, T-3407, T-3408, T-3347, T-4027).
- RATE's noise floor (T-3963, T-3424).

## Decision 0267 cells
Gap triage gives this task 9 visibly-off IRR cells that no open task listed: 2 Tier A (first wrong at the 11th
digit), 4 Tier B (12th or 13th), and 3 Tier C/D (14th or 15th), which come along as part of the same cluster
(decision 0267 item 4). Measured on origin/main b450050f8. Decision 0056's time-box applies.

- probes-numeric/t3411-irr-r4-fresh-2.case.json (5): Calc!A30, A32, A37, A43, A44
- probes-numeric/t3411-irr-r4-fresh-3.case.json (3): Calc!A38, A45, A109
- probes-numeric/t3411-irr-r4-fresh-4.case.json (1): Calc!A110

## Visible-parity stage
Every cell in "Decision 0267 cells" is within half a unit of Excel's 15th significant digit (decisions 0083, 0267).

## Time-box note (salvaged onto main by T-4386)
Draft PR #3211 (branch `agent/functions-financial/T-4213-irr-near-zero-root`) holds the IRR change (the fallback returns
the Newton iterate one step past its stop), its R4 guards and a blocked, time-boxed status (decision 0056): two attempts
ran, the second lands 2 of the 9 cells, and the other 7 stay for decision 0267. Its Excel evidence is on main: the fresh
sweep probes-numeric/t4213-irr-r4-fresh (T-4386) and the findings in financial.md "IRR's returns where the secant
fails are Newton iterates". The visibly-off cells of the sweep are owned by T-4232. The branch's fidelity review asked
for the hold-out from the fit lab's split and for the sweep as a Gate A batch (`eval/accuracy/tables/IRR` r4-T-4213);
neither is done.

## Cells from T-3408 (2026-11-06)
T-3408 (done, decision 0320) fixed its long-run cells with a 200-step secant cap. Its remaining digit cells are this
task's class: both secant runs go flat or stall at NPV's noise floor, and Excel's value is a secant iterate before the
flat step or a fallback Newton iterate past the stop. Measured on origin/main 73e09221b plus T-3408's cap; all are
numbers on both sides, off at the 10th to 15th significant digit. Draft #3211's one-step-past model fixes holdout A41
and residual-scale A31, A32, A48, A49 (financial.md "IRR's returns where the secant fails are Newton iterates").
- probes-numeric/t3347-irr-conventional-sweep.case.json (2): P!A33, A82
- probes-numeric/t3347-irr-conventional-holdout.case.json (4): P!A12, A20, A22, A41
- probes-numeric/t3411-irr-r4-fresh.case.json (1): Calc!A146
- probes-numeric/t3411-irr-r4-fresh-2.case.json (3): Calc!A64, A119, A125
- probes-numeric/t3411-irr-residual-scale.case.json (6): Calc!A22, A23, A31, A32, A48, A49
- Gate A: accuracy/IRR/r4-T-3411!Calc!A286 (known-failures.tsv; two flows within 5E-10 of a zero root).

## Outcome (round 2, 2026-11-07: no code change, blocked)
Round 2 ported draft #3211's change onto main 22a804e85 (after T-3408), re-derived and not copied: `irr_fallback`
returns the Newton iterate one step past its stop where that iterate is a positive discount factor. It was then
measured and not shipped, because three cells move away from Excel. Main's code is unchanged.
- Measured with two release builds of `rustsheet-eval` (main and the change). Each ran `run` over every suite and
  `accuracy --json`, and every IRR cell with an Excel value was compared: 3,436 eval/suites cells and Gate A's 900
  points, 4,336 in all. Cells absent from report.json's failing list count as bit-exact.
- R4.1 tiered table on the fit lab's split (`holdout.is_holdout`, keys `suite/case` and `Sheet!A1`). The split and
  the default 5-point margin were fixed before scoring. Each cell gives bit-exact / within 4 ulp / within 15 digits:

  | side | cells | main | one step past | correctly rounded |
  |---|---|---|---|---|
  | train | 3,038 | 2,474 / 2,503 / 2,506 | 2,486 / 2,513 / 2,516 | 669 / 857 / 964 |
  | test | 1,298 | 1,060 / 1,065 / 1,066 | 1,061 / 1,067 / 1,068 | 306 / 386 / 429 |

  The change beats the correctly rounded root on both sides. Its test exact rate, 81.74%, is within the margin of
  train's 81.83%. So the accept rule would take it. The no-regression rule refuses it.
- 65 cells change and no type changes:
  - 15 become bit-exact: t2927-irr-fallback-sweep A79, A80, A135, A137; t3347-irr-conventional-holdout A41;
    t3411-irr-r4-fresh-2 A43, A44; t3411-irr-residual-scale A31, A32, A48, A49; t3478-irr-r4-wide A292;
    t4213-irr-r4-fresh A95; Gate A r4-T-3407 A285 and r4-T-3411 A286.
  - 4 t2927 cells come within 4 ulp: A8, A59, A65, A142.
  - 3 move away from Excel:
    - t3411-irr-r4-fresh A129 (train), bit-exact to 2.2E-16 off;
    - Gate A s1 A275 (test), bit-exact to 2.2E-16 off, still inside the envelope. The first attempt's measurement
      covered eval/suites only and did not see this one;
    - t2927-irr-fallback-sweep A46 (test), from 4 ulp to 6 ulp, past the 15th digit.
  - The other 43 stay in their tier.
- Narrowing. The only rule found on the train side that keeps A129 is to take the step past only when it keeps the
  stopping step's direction. It still loses A275 on test and drops A286, so nothing narrower can ship. The step past
  is 1 ulp of v on both losers and on most winners, and it lowers |P| on both. The step count, flow count, largest
  |c| and stopping step do not separate them either (financial.md "IRR's returns where the secant fails are Newton
  iterates", re-measured bullet).
- The fresh Gate A batch `r4-T-4213` was not drawn. It exists to test a frozen model, and no model ships. The A275
  loss already refuses this one on Gate A's own points. Decision 0267 item 5's probe allowance was not used. No
  probes-numeric case was added.
- Eval delta: 0, since no engine change was made. `rustsheet-eval accuracy` on main: 0 failing, and A286 stays a
  known failure owned here. Each case's `passed` is unchanged.

## Blocked because
Blocked because: time-boxed (decision 0056)
- Three attempts have run: draft #3211's two, and this re-measurement with a train-side narrowing. None reproduces
  which Newton iterate Excel returns at the noise floor. Returning the iterate one step past the stop fixes 8 of
  this task's cells, but it moves t3411-irr-r4-fresh A129, Gate A s1 A275 and t2927 A46 away from Excel. Decision
  0267 item 6 does not cover IRR.
- Refuted models, so they are not tried again:
  - a relative noise-floor stop on the secant (#3211);
  - one step past, as is or narrowed to same-direction steps (above);
  - stop rules on the step size or |P| (financial.md).
- Decision 0267 accepts the cells left, and all 26 stay off on main:
  - probes-numeric/t3411-irr-r4-fresh-2 (8): Calc!A30, A32, A37, A43, A44, A64, A119, A125
  - probes-numeric/t3411-irr-r4-fresh-3 (3): Calc!A38, A45, A109
  - probes-numeric/t3411-irr-r4-fresh-4 (1): Calc!A110
  - probes-numeric/t3347-irr-conventional-sweep (2): P!A33, A82
  - probes-numeric/t3347-irr-conventional-holdout (4): P!A12, A20, A22, A41
  - probes-numeric/t3411-irr-r4-fresh (1): Calc!A146
  - probes-numeric/t3411-irr-residual-scale (6): Calc!A22, A23, A31, A32, A48, A49
  - Gate A: accuracy/IRR/r4-T-3411!Calc!A286
- Unblocks on a model that tells A129 and A275 (stop iterate) apart from A43, A44 and A286 (one past). The
  replica's own iterates do not do it, so it would need Excel's own Newton arithmetic, or evidence of what Excel
  tests at the stop.
