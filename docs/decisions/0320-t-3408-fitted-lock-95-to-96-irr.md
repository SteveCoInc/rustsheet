# 0320 — T-3408 fitted lock 95 to 96: IRR's secant runs up to 200 steps, and its fresh sweep under the probe freeze

- **Status:** accepted (2026-11-06, T-3408). It applies decision 0188 R4 to one new fitted constant and raises
  quality/fitted.json by one tag (financial_util.rs, 11 -> 12).
- **Context:**
  - T-3408 (decision 0267 Tier A) owns IRR cells where the engine returns a root on digits other than Excel's. An
    engine replay of every IRR literal cell with an oracle value (3,233 cells, eval/suites and Gate A) showed that 21
    of them are runs from the guess (mostly guesses near -0.99) that reach the residual stop after their 50th step.
    Excel's value is that run's stop iterate, bit for bit. The engine's 50-step cap sent them to the retry from 0.1.
  - Over all 3,233 cells, the 83 runs that reach the residual stop (|step| and |NPV| under 1e-7) at their 51st to
    200th step are Excel's value bit for bit.
  - Past 200 steps Excel's outcome is not a clean function of the step count:
    - train runs of 241, 256, 281 and 291 steps return their own value: t3411-irr-r4-fresh A2,
      t2927-irr-fallback-sweep A97, t3411-irr-r4-fresh-4 A63 and t3411-irr-r4-fresh A70;
    - train runs of 217 to 287 steps give the retry's value: t3411-irr-r4-fresh-2 A63, A54, A74 and A57,
      t3411-irr-r4-fresh A142, t3478-irr-r4-wide A240 and t3411-irr-r4-fresh-3 A74;
    - on test, A70 of t3411-irr-r4-fresh-4 (208th step) and A55 of t3411-irr-r4-fresh (215th) give the retry's value.

    So no step cap is Excel's rule, and 200 is not Excel's cap.
  - The evidence files are in docs/research/t3408/:
    - long_runs.tsv: the replay's runs of 51..300 steps, with each cell's fit lab side, how it stops, what Excel
      shows, and whether the run and the retry give the same bits;
    - cap_runs.tsv: the engine's result at each candidate cap inside the shipped range, for the cells whose result
      depends on it;
    - stalls.tsv: every stalled stop;
    - replay_test.rs: the replay that wrote them;
    - bounds.py: prints everything below from them.
  - **Range and cap from train only (R4.1, R4.3).** Both come from the fit lab's train side only
    (`fitlab.holdout.is_holdout`, keys `suite/case` and `Sheet!A1`; Gate A cells keyed `accuracy/IRR/<batch>`).
    - **Range.** It is the range of the 53 train runs of 51..200 steps that return their own value and whose run and
      retry give different bits. A cell where both give the same bits cannot tell the cap apart, so it is left out.
      - flows: 6 (t2927-irr-fallback-sweep A93) to 54 (t3411-irr-r4-fresh-4 A148);
      - largest |flow|: 5.62E-9 (t3478-irr-r4-wide A270) to 2.15E+8 (t2927-irr-fallback-sweep A95);
      - guess: -0.99933 (t3478-irr-r4-wide A230) to 9.81E+7 (t3478-irr-r4-wide A234).
    - The gate is that range rounded outward: 6..54 flows, 5.6E-9..2.2E+8, -0.99934..9.81E+7. The rounding was chosen
      before any held-out cell was scored.
    - With the run = retry cells included, the range would reach |c| 9.52E+8 (t3478-irr-r4-wide A278) and a guess of
      -0.99970 (t3411-irr-r4-fresh-3 A57). Review round 2 shipped that wider band and kept it on test and fresh-sweep
      evidence. Round 3 asked to narrow it, because keeping it was tuning on held-out cells.
    - **Cap.** It is chosen by train score inside that range. Over the 87 train cells whose result depends on the
      cap, Excel is matched by:

      | cap | 50 | 100 | 150 | 190 | 200..216 | 217 | 241 | 281 | 291 | 400 |
      |---|---|---|---|---|---|---|---|---|---|---|
      | train cells | 12 | 39 | 53 | 61 | 64 | 63 | 62 | 62 | 61 | 56 |

      200 is the low end of the train plateau 200..216. On test (36 cells), 200 and 207 match 34, and 216 matches 32.
  - **The stall bound.** A stalled zero step is d2 = d1 with |NPV| above 1e-7 (T-3120's stop).
    - Excel returns a stalled stop up to the 41st step (t3478-irr-r4-wide A260, stalls.tsv). It is `#NUM!` on all 6
      stalled stops from the 70th to the 189th step:
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
    | model | test | 713 | 528 (74.05%) | 533 (74.75%) | 534 (74.89%) |
    | correctly rounded | train | 951 scorable, 669 not | 71 (7.47%) | 171 (17.98%) | 226 (23.76%) |
    | correctly rounded | test | 424 scorable, 289 not | 37 (8.73%) | 79 (18.63%) | 103 (24.29%) |

    - "Not scorable" means an Excel error, or no 50-digit root in the case's `.math.json` or `.math.tsv`.
    - The model's test rate is 1.52 points above its train rate, inside the margin, and it beats the correctly
      rounded root on test.
    - 49 train and 21 test cells become bit-exact. None is worse, and none changes while still failing.
    - Narrowing the range in round 3 cost one test cell: t3478-irr-r4-wide A206 (|c| 4.7E+8) goes back to main's
      `#NUM!`.
  - **Fresh sweep (R4.2).** probes-numeric/t3408-irr-r4-fresh: 150 series in six strata of long runs
    (docs/research/t3411/gen_sweep.py, seed "T-3408/r4-fresh"). It was generated and filled (excel-graph,
    2026-11-06) after the freeze commit.
    - Main 76 -> model 115 of 150: 39 cells become bit-exact, and none moves away.
    - Before round 3 narrowed the range the model scored 119. A2, A13, A31 and A48, whose guesses or flows are now
      outside the range, return main's retry digits again.
    - Gate A envelope against its 50-digit roots: 108 -> 119 inside, per stratum 21/24/18/11/20/14 ->
      23/25/22/14/21/14. The four cells above are inside it on main's digits too.
    - All 31 cells outside the envelope are `#NUM!` where Excel returns the root. T-3548 owns them, and the four
      digit cells above.
  - **Basis under decision 0267 item 5.** The probe freeze allows new probes-numeric cases only for a task's own
    cells. This case is the R4.2 fresh sweep that the fit on T-3408's own Decision 0267 cells owes. It follows
    decision 0316 (T-3347) and the practice it cites. Its cells are evidence, not new T-3408 cells. This decision
    names probes-numeric/t3408-irr-r4-fresh as an exception to decision 0267 item 5's probe freeze, as decision 0316
    did for t3347-irr-r4-fresh.
  - `rustsheet-eval accuracy`: 0 failing. Four T-3548 known-failure lines (r4-T-3411 A208 and A278, r4-T-3407 A128
    and A194) are now inside the envelope and are removed. No point moves out. The count is the same after round 3.
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
  - 200 is not Excel's cap: past 200 steps Excel's outcome does not follow the step count. A model of what decides
    it, or a returning run outside the range
    (stratum 5 of the sweep has 11 `#NUM!` cells on 55..148 flows), may widen the range. Widening it needs new
    evidence, not this sweep.
  - The remaining T-3408 digit cells are flat-secant and fallback stops at the noise floor. They move to T-4213,
    which owns that class.
