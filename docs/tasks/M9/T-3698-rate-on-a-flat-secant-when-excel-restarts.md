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
  - it comes at step 16 or earlier;
  - |f1| < 1e-4;
  - r0 > -1.

  Otherwise it holds as before. The restarted run holds on its own flat secants.
- **Code:** `compute_rate` now calls `rate_secant(args, period_start, guess, restart)`, which recurses at most
  once. `RATE_RESTART_STEPS = 16` carries a `FITTED` tag, and `quality/fitted.json` is updated by
  `check_fitted.py --update`.
- **Probes** (excel-graph, 2026-11-02):
  - probes/t3698-rate-flat-restart, 54 cells where the hold and the restart differ. The first flat secant comes at
    steps 3 to 154, and the nper-1 example is scaled by 2^3 to 2^11. The rule was chosen on this probe.
  - probes/t3698-rate-flat-restart-fresh, 51 cells, drawn and filled after the rule was frozen.
- **Candidates scored** on the 1,522 pre-fresh RATE cells, as bit-exact gains/losses against the hold:
  - unconditional restart: +8/-108;
  - distinct iterates only: +25/-15;
  - adding |f1| < 1e-4: +21/-10;
  - adding step ≤ 3: +11/-0;
  - step bound K with distinct iterates and |f1| < 1e-4: K = 16 or 17 gives +18/-0, K = 18 to 22 gives +18/-1
    (t3698 A28), K = 30 gives +19/-3;
  - restart from the guess instead of r0: +0, because it repeats the run.
- **R4:**
  - Kind: (a) fit for the step bound, (c) structural for the restart.
  - Hold-out: the fit lab's hash split (`holdout.py`, 30%, 5-point margin), applied after scoring, so it is not
    blind. Train is 1,016/1,077 bit-exact (94.3%; 1,038 within 4 ulp) and test 422/445 (94.8%; 432). The hold
    has 1,007 and 413. On the 201 test cells with 50-digit math, the rule has 189 and the correctly rounded root
    18.
  - Fresh sweep (t3698-rate-flat-restart-fresh): the rule is bit-exact on 21/51 and the hold on 17/51. Within
    4 ulp they are 26 and 29, and Excel's type matches on 43 under both.
  - Gate A envelope on the fresh sweep (within 15 significant digits of Excel, or 1e-9 relative of the 50-digit
    root from mpmath `findroot` on the FV residual at 50 digits; an error against a number fails): 42 of 51 for the
    rule and for the hold. The same 9 are outside under both (A15, A20, A22, A34, A37, A39, A41, A46, A48), all
    number against error: the noise-floor type that T-3424 and T-3850 own. The rule moves no point out.
    `rustsheet-accuracy math` skips RATE calls with a guess ("argument count"), so the sidecars hold only the
    first probe's 9 guess-free cells. `make eval-math-check` passes.
  - The fresh sweep is a probes case, not a probes-numeric/Gate A batch. Gate A's T-3357/T-3358 batches have not
    landed.
  - Reproducibility floor: the gains are 5 ulp to 3e-11 relative, above the 4-ulp web-desktop floor.
- **Eval delta:** probes 6 fixed (t3579-rate-residual-nper1 A1, A2, A41, A42; t3348-rate-fv-nonzero A171,
  A175), 0 regressions. probes-numeric t3348-rate-gate-a-fresh A296 is also fixed.
  - The two new probe cases add 105 cells. The noise-floor cells among them are new-coverage known failures owned
    by T-3850.
- **Follow-up:** T-3850 (late restarts and noise-floor paths).
