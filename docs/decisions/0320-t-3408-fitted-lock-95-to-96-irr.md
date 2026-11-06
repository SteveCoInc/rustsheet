# 0320 — T-3408 fitted lock 95 to 96: IRR's secant runs up to 200 steps, and its fresh sweep under the probe freeze

- **Status:** accepted (2026-11-06, T-3408). It applies decision 0188 R4 to one new fitted constant and raises
  quality/fitted.json by one tag (financial_util.rs, 11 -> 12).
- **Context:**
  - T-3408 (decision 0267 Tier A) owns IRR cells where the engine returns a root on digits other than Excel's. An
    engine replay of every IRR literal cell with an oracle value (3,233 cells, eval/suites and Gate A) showed that 21
    of them are runs from the guess (mostly guesses near -0.99) that reach the residual stop after their 50th step.
    Excel's value is that run's stop iterate, bit for bit. The engine's 50-step cap sent them to the retry from 0.1.
  - Over all 3,233 cells, the 83 runs that reach the residual stop (|step| and |NPV| under 1e-7) at their 51st to
    200th step are Excel's value bit for bit. From the 208th step Excel has the retry's digits instead. Every such
    cell, with its side, step count and stop, is in docs/research/t3408/long_runs.tsv. `docs/research/t3408/bounds.py`
    derives the bounds below from that file, and replay_test.rs is the uncapped replay of `irr_secant` that wrote it.
  - **Bracket and range from train only (R4.1).** These come from the fit lab's train side only
    (`fitlab.holdout.is_holdout`, keys `suite/case` and `Sheet!A1`; Gate A cells keyed `accuracy/IRR/<batch>`).
    - Cap bracket 200..216. The last train run Excel finishes stops at its 200th step (t3411-irr-r4-fresh-3 A53).
      The first train run it does not finish needs its 217th (t3411-irr-r4-fresh-2 A63). The test side's
      t3411-irr-r4-fresh-4 A70 (208th) and fresh A55 (215th) agree and narrow it to 200..207. 200 is in both.
    - Range (R4.3), from the 57 returning train runs:
      - 6 flows (t2927-irr-fallback-sweep A93) to 54 (t3411-irr-r4-fresh-4 A148);
      - largest |flow| 5.62E-9 (t3478-irr-r4-wide A270) to 9.52E+8 (A278);
      - guess -0.99970 (t3411-irr-r4-fresh-3 A57) to 9.81E+7 (t3478-irr-r4-wide A234).
    - The 26 test runs fall inside that range. So the gate (6..54, 5.6E-9..9.6E+8, -0.9997..9.81E+7, rounded
      outward) is the train-only range, and the hold-out result below does not change.
  - **The stall bound.** A stalled zero step is d2 = d1 with |NPV| above 1e-7 (T-3120's stop).
    - Excel returns a stalled stop up to the 41st step (t3478-irr-r4-wide A260). It is `#NUM!` on all 6 stalled
      stops from there to its cap:
      - probes/irr-guess-edges-2 A25, A26, A27 (70th, 74th, 125th step);
      - Gate A r4-T-3411 A228 (129th) and A215 (132nd);
      - t3478-irr-r4-wide A227 (189th).
    - So the stall bound is bracketed 41..69. The engine keeps it at `IRR_SHORT_RUN_STEPS` = 50, the old cap, and
      does not fit it.
    - All six cells lie outside the cap's range, by flow count or largest |flow|, where runs stop at 50 steps
      anyway. No scored cell tests the stall bound inside the range, and none of the six counts toward the fit.
    - They do explain why a plain cap failed in T-3548 and T-3806: those caps applied to every flow set, and A227 and
      A228 stalled under them.
  - **Hold-out (R4.1)**, over the 2,333 IRR literal cells of eval/suites with oracle values, margin 5 points, fixed
    in the freeze commit. Bit-exact, within 4 ulp and within 15 digits:

    | candidate | side | cells | bit-exact | within 4 ulp | within 15 digits |
    |---|---|---|---|---|---|
    | main (cap 50) | train | 1,620 | 1,126 (69.51%) | 1,156 (71.36%) | 1,159 (71.54%) |
    | main (cap 50) | test | 713 | 507 (71.11%) | 513 (71.95%) | 514 (72.09%) |
    | model | train | 1,620 | 1,175 (72.53%) | 1,203 (74.26%) | 1,206 (74.44%) |
    | model | test | 713 | 529 (74.19%) | 534 (74.89%) | 535 (75.04%) |
    | correctly rounded | train | 951 scorable, 669 not | 71 (7.47%) | 171 (17.98%) | 226 (23.76%) |
    | correctly rounded | test | 424 scorable, 289 not | 37 (8.73%) | 79 (18.63%) | 103 (24.29%) |

    - "Not scorable" means an Excel error, or no 50-digit root in the case's `.math.json` or `.math.tsv`.
    - The model's test rate is 1.66 points above its train rate, inside the margin, and it beats the correctly
      rounded root on test.
    - 49 train and 22 test cells become bit-exact. None is worse, and none changes while still failing.
  - **Fresh sweep (R4.2).** probes-numeric/t3408-irr-r4-fresh: 150 series in six strata of long runs
    (docs/research/t3411/gen_sweep.py, seed "T-3408/r4-fresh"). It was generated and filled (excel-graph,
    2026-11-06) after the freeze commit. Main 76 -> model 119 of 150: 43 cells become bit-exact and none move away.
    Gate A envelope against its 50-digit roots: 108 -> 119 inside, per stratum 21/24/18/11/20/14 ->
    23/25/22/14/21/14. All 31 outside are `#NUM!` where Excel returns the root: 11 in stratum 5 (outside the range),
    20 inside it. T-3548 owns them.
  - **Basis under decision 0267 item 5.** The probe freeze allows new probes-numeric cases only for a task's own
    cells. This case is the R4.2 fresh sweep that the fit on T-3408's own Decision 0267 cells owes. It follows
    decision 0316 (T-3347) and the practice it cites. Its cells are evidence, not new T-3408 cells. This decision
    names probes-numeric/t3408-irr-r4-fresh as an exception to decision 0267 item 5's probe freeze, as decision 0316
    did for t3347-irr-r4-fresh.
  - `rustsheet-eval accuracy`: 0 failing. Four T-3548 known-failure lines (r4-T-3411 A208 and A278, r4-T-3407 A128
    and A194) are now inside the envelope and are removed. No point moves out.
  - **R4.4.** The fixed cells are bit-exact, not ulp-close. No step targets bits below the reproducibility floor.
  - Not a keep-matched branch: the domain is the run's step count and the flows' shape, not the set of matched
    cells.
- **Decision:** `irr_secant` runs up to `IRR_LONG_RUN_STEPS` (200) steps inside `IRR_LONG_RUN_RANGE` and up to
  `IRR_SHORT_RUN_STEPS` (50) elsewhere. A stalled zero step stops a run only within its first 50 steps. That bound
  is bracketed 41..69, is not fitted, and is untested inside the range. Past it, the run goes flat and fails (the
  retry, then the fallback). The constant carries one `FITTED(T-3408, ...)` tag,
  and `scripts/check_fitted.py --update` records it.
- **Consequences:**
  - eval/baseline/probes.txt loses the lines of the gated cells that now pass (`make eval-baseline`).
  - Excel's own cap lies between 200 and 207 steps. A probe that pins it, or a returning run outside the range
    (stratum 5 of the sweep has 11 `#NUM!` cells on 55..148 flows), may widen the range. Widening it needs new
    evidence, not this sweep.
  - The remaining T-3408 digit cells are flat-secant and fallback stops at the noise floor. They move to T-4213,
    which owns that class.
