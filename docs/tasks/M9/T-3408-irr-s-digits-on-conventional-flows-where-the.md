---
id: T-3408
title: "IRR's long secant runs: Excel finishes runs of up to 200 steps on their own digits"
milestone: M9
lane: functions-financial
status: done
priority: P0
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
IRR returns Excel's value to 15 significant digits on the conventional cash-flow series where the secant model fails
(near r = 0 and near -10%): today T-3347's last stage returns the exact root (within 1.5e-11), which is 1e-13 to
2.2e-11 relative from Excel's double on 21 of the 162 cells of the T-3347 cases.

## Context
- Corrected 2026-11-06 (claim): the Goal above assumed T-3347's Newton stage `irr_conventional` landed. It did not:
  draft #2529 closed unmerged, and T-3347 merged (#3580) blocked, time-boxed (decision 0056), with only its
  `#DIV/0!` flat-step rule (decision 0316). There is no `irr_conventional`/`irr_newton` stage; the engine runs the
  secant, then its fallbacks (crates/rustsheet-engine/src/functions/financial/financial_util.rs). T-3347 no longer
  gates this task (`depends_on` cleared).
- State on origin/main 73e09221b (`rustsheet-eval run --suite probes --suite probes-numeric --filter irr`):
  - probes/t3347-irr-conventional-num A10-A13, A16, A17, A19, A37-A40 (11): all `#NUM!` for Excel's root.
  - probes-numeric/t3347-irr-conventional-sweep: 23 cells `#NUM!` for Excel's root; A33 and A82 numeric_close.
  - The `#NUM!` cells are T-3347's blocked cells, accepted under decision 0267 item 4 (T-3347's "Blocked
    because"). They are out of this task's scope.
- In scope: the cells where the engine returns a root on digits other than Excel's (numeric_close, and
  t3411-irr-r4-fresh A53 numeric_mismatch): the sweep's A33 and A82, the hold-out's four below and the t3411 cells.
- Digit cells of T-3347's hold-out, right type, beyond 15 digits (gap triage 2026-11-01):
- probes-numeric/t3347-irr-conventional-holdout.case.json (4): P!A12, A20, A22, A41
- Related: T-2928 (blocked: the v = 0 fallback's digits, the same question for non-conventional flows).
- Any fit follows decision 0188 R4 (hold-out split, fresh sweep, `FITTED` tag).

## Acceptance criteria
- [ ] ~~Every cell of the two T-3347 cases is within 15 significant digits of Excel's value, with the right
      type.~~ Rescoped at the claim: their `#NUM!` cells are T-3347's accepted block. Of the in-scope digit cells, 20
      of 36 are fixed (Outcome); the 16 noise-floor cells move to T-4213.
- [x] A fresh hold-out sweep (decision 0188 R4.1) shows the same, and the Gate A floor still holds.
- [ ] ~~Oracle cells for the shapes the stage takes~~: obsolete, the Newton stage from v = 1 never landed.
- [x] No gated cell regresses.
- [ ] `make check` green; reviewers approve.

## Cells of T-3411's fresh sweeps
T-3411's R4 sweeps 3 and 4 (cases added by PR #3063, scored on its round-4 model), where the engine returns the root on digits other than Excel's:
- probes-numeric/t3411-irr-r4-fresh-3.case.json (7): Calc!A53, A58, A62, A66, A73, A106, A134
- probes-numeric/t3411-irr-r4-fresh-4.case.json (2): Calc!A58, A75
- probes-numeric/t3411-irr-residual-scale.case.json (7): Calc!A10, A22, A23, A31, A32, A48, A49

## Cells left by T-3411's close (gap triage 2026-11-04b)
T-3411's R4 sweeps 1 and 2, which T-3411 owned until it closed and no open task named after it: the engine returns
the root on digits more than 15 significant digits from Excel's (`make eval` on origin/main a7b115e9e; e.g. fresh
A53 Excel -0.00016023194079306524, engine -0.00016023193850578377). Lifting the fallback's Newton cap moves none.
- probes-numeric/t3411-irr-r4-fresh.case.json (7): Calc!A53, A63, A67, A68, A74, A75, A146
- probes-numeric/t3411-irr-r4-fresh-2.case.json (7): Calc!A53, A60, A64, A75, A119, A125, A147

## Out of scope
- Bit-level parity (Phase 1b, decision 0188 R5).

## Decision 0267
The owner's cut-loss plan (decision 0267, 2026-11-04) makes this a Tier A task, worked first: its cells have the wrong type or error, or are first wrong at the 11th digit or earlier. Decision 0056's time-box applies. If two attempts do not move the cells, block the task as time-boxed, and decision 0267 accepts the cells that are left.

## R4 plan (frozen 2026-11-06, before the fresh sweep)
- Diagnosis (engine replay of every IRR literal cell with an oracle value, 3,233 cells of eval/suites and Gate A):
  21 of the in-scope digit cells are runs from the guess that reach the residual stop after their 50th step, and
  Excel's value is that run's stop iterate bit for bit; the engine's 50-step cap sent them to the retry from 0.1.
  Over all cells, the 83 runs that stop by the residual at their 51st..200th step are Excel's value bit for bit;
  from the 208th step Excel has the retry's digits (t3411-irr-r4-fresh-4 A70). The 6 runs that stop on a stalled zero
  step after step 50 (irr-guess-edges-2 A25-A27, t3478-irr-r4-wide A227, Gate A r4-T-3411 A215, A228) are all
  `#NUM!` in Excel: they are the counterexamples that refuted a cap in T-3548 and T-3806.
- Model, frozen here: a secant run takes up to 200 steps (`IRR_LONG_RUN_STEPS`, bracket 200..207) on 6..54 flows
  with the largest |c| in 5.6E-9..9.6E+8 and a guess in -0.9997..9.81E+7 (R4.3, the 83 runs' range), 50 elsewhere;
  a stalled zero step stops a run only within its first 50 steps.
- R4.1: the fit lab split (`holdout.is_holdout`, case id and Sheet!A1) over the IRR literal cells of eval/suites,
  margin 5 points, fixed now. Accept: no cell worse on either side and the test rate within the margin of train.
- R4.2: a fresh sweep drawn after this commit (docs/research/t3411/gen_sweep.py, seed "T-3408/r4-fresh"), scored
  once against main; it may not move any cell away from Excel.

## Outcome (2026-11-06, branch agent/functions-financial/T-3408-irr-conventional-digits)
- **Engine:** `irr_secant` runs up to `IRR_LONG_RUN_STEPS` = 200 steps (was 50) on 6..54 flows with the largest |c|
  in 5.6E-9..9.6E+8 and a guess in -0.9997..9.81E+7 (`IRR_LONG_RUN_RANGE`, R4.3), 50 elsewhere. A stalled zero step
  stops a run only within its first 50 steps (the old cap; Excel's stall bound is bracketed 41..69, not fitted). `FITTED(T-3408, ...)`, quality/fitted.json 95 -> 96, decision 0320.
  Unit tests: `irr_secant_runs_up_to_200_steps_on_the_evidenced_flows`, `the_long_secant_cap_applies_only_inside_its_evidence`.
- **Why:** Excel's value on 21 of the in-scope cells is the guess run's own stop iterate after 51..200 steps (see "R4
  plan"). The counterexamples that refuted a cap before (T-3548, T-3806: t3478-irr-r4-wide A227, Gate A A228, A215)
  are stalled stops past step 41, as are irr-guess-edges-2 A25-A27; all 6 are `#NUM!` in Excel. All six lie outside
  the cap's range, so they explain the earlier refutations but do not test the stall bound inside it.
- **Review rounds 1 and 2 (fidelity, R4.1):**
  - The R4 plan's bracket was read off all cells, and round 1's "train bracket 200..216" used a step filter derived
    from test cells.
  - Past 200 steps Excel's outcome is not a clean function of the step count. Train runs of 241, 281 and 291 steps
    return their own value (t3411-irr-r4-fresh A2, t3411-irr-r4-fresh-4 A63, t3411-irr-r4-fresh A70), while train
    runs of 217..287 steps give the retry's value.
  - So 200 is not Excel's cap but the low end of the train-score plateau. Of the 88 train cells whose result depends
    on the cap, 65 match at any cap from 200 to 216; a cap of 50 matches 13, 291 matches 61 and 400 matches 56. The
    FITTED text says so.
  - The range comes from the train runs of 51..200 steps that return their own value:
    - 6..54 flows (t2927 A93, t3411-irr-r4-fresh-4 A148);
    - |c| 5.62E-9..9.52E+8 (t3478-irr-r4-wide A270, A278);
    - guess -0.99970..9.81E+7 (t3411-irr-r4-fresh-3 A57, t3478 A234).
  - Counting only cells where run and retry give different bits narrows that range to |c| up to 2.15E+8 and guesses
    from -0.99933. The gate is kept wider: in that band, test A206 and fresh A2 and A48 match Excel only with the cap,
    and no cell moves away from Excel.
  - docs/research/t3408/ holds the replay (replay_test.rs), its cell lists (long_runs.tsv, cap_runs.tsv,
    stalls.tsv) and bounds.py, which prints every count.
  - The tiered R4.1 table (bit-exact / 4 ulp / 15 digits) for main, the model and the correctly rounded root is in
    decision 0320. `IRR_LONG_RUN_FLOWS` became the named `IRR_LONG_RUN_RANGE`.
  - Engine behaviour is unchanged by both rounds.
- **R4.1** (fit lab split, 2,333 IRR literal cells of eval/suites, margin 5 points): train 1,126 -> 1,175 of 1,620
  (69.51% -> 72.53%), test 507 -> 529 of 713 (71.11% -> 74.19%); type/error mismatches 352 -> 336 and 167 -> 154.
  49 train and 22 test cells fixed, 0 worse, 0 changed-still-failing. Within 4 ulp / 15 digits: train 1,156 -> 1,203
  and 1,159 -> 1,206; test 513 -> 534 and 514 -> 535. The correctly rounded root, where a math root exists, is
  bit-exact on 71 of 951 train and 37 of 424 test cells.
- **R4.2** fresh sweep probes-numeric/t3408-irr-r4-fresh (seed "T-3408/r4-fresh", committed after the freeze commit
  04bd27413, excel-graph 2026-11-06): passed 76 -> 119 of 150, 43 fixed, none away. Gate A envelope against its
  50-digit roots: 108 -> 119 in, per stratum 21/24/18/11/20/14 -> 23/25/22/14/21/14. The 31 left are all `#NUM!` for
  Excel's root (11 on 55..148 flows, outside the range), listed in T-3548. Math sidecar
  `t3408-irr-r4-fresh.math.json` (25 one-argument cells; `rustsheet-accuracy math` skips the guess calls).
- **Gate A:** `rustsheet-eval accuracy` 0 failing; 4 T-3548 known-failure lines are now inside the envelope and are
  removed (r4-T-3411 A208, A278; r4-T-3407 A128, A194). Line A286 (T-3408's, a two-flow zero root at the noise floor)
  moves to T-4213.
- **Eval delta** (full `rustsheet-eval run`, main 73e09221b vs branch; 71 cells fixed, 0 worse):
  - probes/t3347-irr-conventional-num: passed 23 -> 25 of 42 (A10, A11)
  - probes/t3406-irr-num-boundary: 88 -> 90 of 144 (A5, A26)
  - probes-numeric/t3347-irr-conventional-holdout: 110 -> 115 of 144 (A23, A47, A50, A56, A57)
  - probes-numeric/t3347-irr-conventional-sweep: 95 -> 97 of 120 (A38, A88)
  - probes-numeric/t3411-irr-r4-fresh: 91 -> 99 of 150; -2: 86 -> 93; -3: 90 -> 99; -4: 84 -> 87
  - probes-numeric/t3411-irr-residual-scale: 65 -> 66 of 72 (A10)
  - probes-numeric/t3478-irr-r4-wide: 241 -> 264 of 300
  - probes-numeric/t4213-irr-r4-fresh: 102 -> 108 of 150
  - probes-numeric/t2927-irr-fallback-sweep: 40 -> 43 of 165 (A22, A23, A96)
  - new probes-numeric/t3408-irr-r4-fresh: 119 of 150
  - `make eval-baseline` drops t3347-irr-conventional-num A10, A11 and t3406-irr-num-boundary A5, A26 from
    eval/baseline/probes.txt.
- **In-scope cells, per cell:** fixed (20): t3411-irr-r4-fresh A53, A63, A67, A68, A74, A75; fresh-2 A53, A60, A75,
  A147; fresh-3 A53, A58, A62, A66, A73, A106, A134; fresh-4 A58, A75; t3411-irr-residual-scale A10. Left (16), all
  numeric_close, unchanged: t3347-irr-conventional-sweep A33, A82; t3347-irr-conventional-holdout A12, A20, A22, A41;
  t3411-irr-r4-fresh A146; fresh-2 A64, A119, A125; t3411-irr-residual-scale A22, A23, A31, A32, A48, A49. They are
  not long runs: both secant runs go flat (or stall) at NPV's noise floor and Excel's value is a secant iterate before
  the flat step or a fallback Newton iterate past the stop, the class T-4213 owns (its relative-stop and one-step-past
  models, financial.md "IRR's returns where the secant fails are Newton iterates"). They move to T-4213's list rather
  than repeat those refuted or pending models here; the one-step-past model of draft #3211 fixes holdout A41 and
  residual-scale A31, A32, A48, A49.
- financial.md: "IRR's secant runs longer than 50 steps" now states the rule; "The IRR and XIRR solver caps" and
  "IRR on conventional flows near 0 and -10%" updated.
- Follow-up evidence, not filed: stratum 5 of the sweep suggests Excel also finishes long runs on 55..148 flows; a
  future widening of the range needs its own probe (decision 0320).
