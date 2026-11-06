# 0320 — T-3408 fitted lock 95 to 96: IRR's secant runs up to 200 steps, and its fresh sweep under the probe freeze

- **Status:** accepted (2026-11-06, T-3408). It applies decision 0188 R4 to one new fitted constant and raises
  quality/fitted.json by one tag (financial_util.rs, 11 -> 12).
- **Context:**
  - T-3408 (decision 0267 Tier A) owns IRR cells where the engine returns a root on digits other than Excel's. An
    engine replay of every IRR literal cell with an oracle value (3,233 cells, eval/suites and Gate A) showed that 21
    of them are runs from the guess (mostly guesses near -0.99) that reach the residual stop after their 50th step.
    Excel's value is that run's stop iterate, bit for bit. The engine's 50-step cap sent them to the retry from 0.1.
  - Over all 3,233 cells, the runs that stop at their 51st..207th step split cleanly. The 83 that stop by the
    residual stop (|step| and |NPV| under 1e-7) are Excel's value bit for bit, up to the 200th step
    (t3411-irr-r4-fresh-3 A53, t3478-irr-r4-wide A221). The 6 that stop on a stalled zero step (d2 = d1 with |NPV|
    above 1e-7, T-3120's stop) are all `#NUM!` in Excel (probes/irr-guess-edges-2 A25-A27, t3478-irr-r4-wide A227,
    Gate A r4-T-3411 A215, A228). From the 208th step Excel has the retry's digits (t3411-irr-r4-fresh-4 A70; fresh
    A55 at 215, fresh-2 A63 at 217).
  - T-3548 and T-3806 refuted a plain step cap because of A227 and A228. Both are stalled stops, so they do not
    refute a cap once the stall stop is limited to the first 50 steps. T-3120's evidence for the stall stop is all
    inside that limit.
  - **Hold-out (R4.1).** The fit lab's split (`rustsheet_tools.fitlab.holdout.is_holdout`, keyed on the case id
    without `.case.json` and `Sheet!A1`) over the 2,333 IRR literal cells of eval/suites with oracle values, at the
    default margin of 5 points, fixed in the freeze commit before scoring. Main -> model, cells passing: train
    1,126 -> 1,175 of 1,620 (69.51% -> 72.53%; type/error mismatches 352 -> 336); test 507 -> 529 of 713
    (71.11% -> 74.19%; 167 -> 154). 0 cells are worse or changed-still-failing on either side; 49 train and 22 test
    cells become bit-exact. The test rate is within the margin of train. The correctly rounded root is bit-exact on
    0 of the 28 changed cells that have a math sidecar; the model is bit-exact on all of them.
  - The bracket was read off cells the split puts on both sides, so it is a fit: one `FITTED` tag.
  - **Range (R4.3).** The 200-step cap applies only to 6..54 flows with a largest |flow| of 5.6E-9..9.6E+8 and a
    guess of -0.9997..9.81E+7, the range of the 83 returning runs. Elsewhere a run stops after 50 steps, as before.
  - **Fresh sweep (R4.2).** probes-numeric/t3408-irr-r4-fresh: 150 series in six strata of long runs
    (docs/research/t3411/gen_sweep.py, seed "T-3408/r4-fresh"). It was generated and filled (excel-graph,
    2026-11-06) after the freeze commit. Main 76 -> model 119 of 150: 43 cells become bit-exact and none move away.
    Gate A envelope against its 50-digit roots: 108 -> 119 inside, per stratum 21/24/18/11/20/14 ->
    23/25/22/14/21/14. All 31 outside are `#NUM!` where Excel returns the root: 11 in stratum 5 (outside the range),
    20 inside it. T-3548 owns them.
  - **Basis under decision 0267 item 5.** The probe freeze allows new probes-numeric cases only for a task's own
    cells. This case is the R4.2 fresh sweep that the fit on T-3408's own Decision 0267 cells owes. It follows
    decision 0316 (T-3347) and the practice it cites. Its cells are evidence, not new T-3408 cells.
  - `rustsheet-eval accuracy`: 0 failing. Four T-3548 known-failure lines (r4-T-3411 A208 and A278, r4-T-3407 A128
    and A194) are now inside the envelope and are removed. No point moves out.
  - **R4.4.** The fixed cells are bit-exact, not ulp-close. No step targets bits below the reproducibility floor.
  - Not a keep-matched branch: the domain is the run's step count and the flows' shape, not the set of matched
    cells.
- **Decision:** `irr_secant` runs up to `IRR_LONG_RUN_STEPS` (200) steps inside `IRR_LONG_RUN_FLOWS` and up to
  `IRR_SHORT_RUN_STEPS` (50) elsewhere. A stalled zero step stops a run only within its first 50 steps; past that,
  the run goes flat and fails (the retry, then the fallback). The constant carries one `FITTED(T-3408, ...)` tag,
  and `scripts/check_fitted.py --update` records it.
- **Consequences:**
  - eval/baseline/probes.txt loses the lines of the gated cells that now pass (`make eval-baseline`).
  - Excel's own cap lies between 200 and 207 steps. A probe that pins it, or a returning run outside the range
    (stratum 5 of the sweep has 11 `#NUM!` cells on 55..148 flows), may widen the range. Widening it needs new
    evidence, not this sweep.
  - The remaining T-3408 digit cells are flat-secant and fallback stops at the noise floor. They move to T-4213,
    which owns that class.
