---
id: T-3698
title: "RATE on a flat secant: when Excel restarts from the older iterate instead of holding"
milestone: M9
lane: functions-financial
status: done
priority: P2
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
A rule, fitted on oracle evidence, for when RATE's secant restarts from the older iterate on a flat secant (f1 = f0)
instead of holding at the newer one, so that probes/t3579-rate-residual-nper1 A1, A2, A41 and A42 and
t3348-rate-fv-nonzero A171 match Excel without losing the held cells.

## Context
- T-3579 (financial.md "RATE holds on a flat secant and runs on below -1", the "residual is not the cause" bullet):
  at nper 1 Excel's residual has the model's bits (49/53 bit-exact). The 4 misses are flat secants the model holds,
  where Excel's value is the model's run restarted from the flat step's older iterate
  (`RATE(1,2560574.93,-2543613.41,-11634.76,0,0.0020941704345913)`, A6, is Excel's A1 bit for bit; scale 4, A42,
  fits too).
- A blanket restart loses 32 held cells of t3348-rate-fv-nonzero (and 31 Gate A sweep cells; A43/A46 here), so
  Excel holds on some flat secants and restarts on others. Candidates to test: the iterate count when the flat
  secant occurs (A171's comes at step 3, right after the first near-root step), whether the flat step's two
  iterates differ (zero step vs distinct iterates), |f| against a bound, a restart budget (once per run), or the
  restart point (guess vs older iterate).
- Code: `compute_rate` (crates/rustsheet-engine/src/functions/financial/financial_util.rs); fit lab `rate`
  (tools/src/rustsheet_tools/fitlab/kernels/solvers.py). A structural model chosen among ranked candidates owes the
  hold-out and fresh sweep (decision 0188 R4).
- Known failures owned: probes/t3579-rate-residual-nper1 Sheet1!A1, A2, A41, A42.

## Acceptance criteria
- [x] A probe that separates the restart from the hold (e.g. flat secants at different steps and |f|, with and
      without a prior restart), oracle-filled, and a rule that fits it.
- [x] `compute_rate` implements the rule; the 4 cells above pass and no RATE cell regresses
      (t3348-rate-fv-nonzero, the Gate A sweeps, t3478-rate-r4-wide); R4 hold-out and fresh sweep recorded.
- [x] financial.md updated; `make check` green; reviewers approve.

## Out of scope
- RATE's residual form (ruled out at nper 1 by T-3579).

## Outcome
- **Rule** (financial.md "RATE restarts once on an early flat secant"). A flat secant (f1 = f0, f1 ≠ 0) restarts
  the run once as a fresh run from the older iterate r0 when all of these hold:
  - r1 ≠ r0;
  - it comes at step 3 to 16;
  - 1.3e-7 <= |f1| < 1e-4;
  - r0 > -1.

  Otherwise it holds as before. The restarted run holds on its own flat secants.
- **Code:** `compute_rate` now calls `rate_secant(args, period_start, guess, restart)`, which recurses at most
  once. `RATE_RESTART_STEPS = 16` carries `FITTED(T-3698, 1477 cells, RATE flat secant at steps 3..16 with |f1|
  1.3E-7..1E-4)`. Outside that range the run holds (R4.3: steps 1 and 2 and |f1| below 1.3e-7 have no evidence).
  `quality/fitted.json` is updated by `check_fitted.py --update`.
- **Probes** (excel-graph, 2026-11-02):
  - probes/t3698-rate-flat-restart, 54 cells where the hold and the restart differ. The first flat secant comes at
    steps 3 to 154, and the nper-1 example is scaled by 2^3 to 2^11.
  - probes/t3698-rate-flat-restart-fresh, 51 cells, drawn after a first freeze. Because that freeze was scored on
    every cell, it is now part of the fit set.
  - probes-numeric/t3698-rate-r4-fresh, 300 cells, the R4.2 fresh sweep (below).
- **R4.1 hold-out** (review round 1 asked for a train-only choice):
  - Kind: (a) fit for the step bound and the |f1| cut; (c) structural for the restart.
  - Fit set: every `=RATE(` oracle cell of probes and probes-numeric except t3478-rate-r4-wide.
  - Split: the fit lab's hash (`holdout.py`, 30% test). The 5-point margin was stated before scoring.
  - Candidates: K in {3, 8, 12, 14..18, 20, 22, 30, 60, 200} × cut in {1e-5, 3e-5, 1e-4, 3e-4, 1e-3}, all with
    step >= 3 and |f1| >= 1.3e-7. They were ranked on the 1,477 train cells only. A candidate that loses a cell
    main passes was excluded (the acceptance criterion).
  - Chosen: K 16 with cut 1e-4.
    - Train bit-exact / 4 ulp / 15 digits: 1,395 / 1,422 / 1,422 of 1,477. The hold gets 1,378 / 1,412 / 1,413.
    - Test: 551 / 561 / 561 of 581, against 546 / 554 / 557 for the hold.
    - Rates: 94.4% train, 94.8% test.
  - K 17 ties; the smaller bound is taken.
  - Cut 3e-4 ties on train bit-exact and is +1 within 4 ulp, but it loses t3579-rate-residual-nper1 A43 (a main
    pass), so it is excluded. Note: I froze 3e-4 by the tie-break first and drew the fresh sweep under it, then
    found the loss and moved to 1e-4. The fresh sweep scores the same under both (below).
  - Correctly rounded root, on the cells with 50-digit math: train 35 / 80 / 98 of 637, test 16 / 33 / 44 of 251.
    The rule is bit-exact on 610 and 239 of those cells, so it beats the root on test.
  - Caveat: these cells had all been scored under the first freeze. So this is a train-only choice on a fixed
    split, not a blind hold-out. The fresh sweep is the out-of-sample check.
- **R4.2 fresh sweep** (probes-numeric/t3698-rate-r4-fresh, generated after the freeze):
  - 300 points of Gate A's RATE draw (`rustsheet-accuracy` sampler, batch seed `r4-T-3698-fresh`, every stratum),
    each with a guess log-uniform in ±1e-4..0.9 (80%) or 0.9..10.
  - `math.tsv` commits the 50-digit roots: one sign change makes the root unique, so the guess does not matter.
    Scored with docs/research/t3478/score_sweep.py.
  - Results: bit-exact 298, within 4 ulp 299, within 15 digits 299. Correctly rounded: 22 / 29 / 34.
- **Gate A envelope** on the fresh sweep: 299 of 300 inside, under both the 1e-4 and the 3e-4 build.
  - The one outside is A201, `RATE(1,13501.684222927,-13501.684222927,0,1,1.74874265618351)`. Excel returns the
    guess, the model `#NUM!`.
  - Every residual there is rounding noise (about 1e-12, from pv + pmt = 0), far below the restart's 1.3e-7
    floor. So the rule cannot move it, and it is the same under the hold.
  - T-3876 owns it; financial.md records the quirk.
  - The sweep rarely makes a flat secant. It shows the rule moves nothing elsewhere, not the restart itself.
- **R4.4:** the gains are 5 ulp to 3e-11 relative, above the 4-ulp web-desktop floor.
- **Eval delta:**
  - probes: 6 fixed (t3579-rate-residual-nper1 A1, A2, A41, A42; t3348-rate-fv-nonzero A171, A175), 0
    regressions.
  - probes-numeric: t3348-rate-gate-a-fresh A296 fixed.
  - New cases: t3698-rate-flat-restart, t3698-rate-flat-restart-fresh and t3698-rate-r4-fresh add 405 cells. The
    noise-floor misses are owned by T-3850 and A201 by T-3876.
- **Follow-ups:**
  - T-3850: late restarts and noise-floor paths.
  - T-3876: a noise-only residual returns the guess.
