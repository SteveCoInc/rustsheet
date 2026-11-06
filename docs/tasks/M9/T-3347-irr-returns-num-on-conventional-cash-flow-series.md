---
id: T-3347
title: "IRR returns #NUM! on conventional cash-flow series whose IRR is near 0 or near -10%"
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
IRR returns Excel's root for conventional cash-flow series (one sign change) whose internal rate is near 0 (1e-5 to
3e-4) or near -10%, where the engine returns `#NUM!` today. Together with T-3348 (RATE) this is the first task of the
accuracy stage.

## Context
- Owner's 2026-10-31 assessment (decision 0188, "The engine is wrong where Excel is right", item 1). Users hit
  this: it is an ordinary input, Excel returns the right number and the engine does not. Highest-priority work of the
  accuracy stage.
- Code: `fn_irr` in crates/rustsheet-engine/src/functions/financial/mod.rs and the solver in
  financial/financial_util.rs (the secant, the far-root bound and `irr_fallback`, a Newton run from v = 0 fitted in
  T-2727). Find which stop or bound refuses the root: near r = 0 the NPV's slope and value are both small, and near
  r = -10% the discount factors grow.
- Related: T-2131 (IRR's returned iterate to 15 digits), T-2928 (blocked: the fallback's digits), T-2927 (done: the
  fallback-path sweep), T-3205 (done: secant-in-d residues), T-3246 (a calipers
  ulp regression). This task is about the `#NUM!`, not their last digits; name any of their cells this fix moves in
  the Outcome.
- The IRR fallback is one of the fitted pieces of decision 0188's code audit: if the fix changes it, tag it per R4.3
  (T-3361) and keep it inside its measured range.

- Gate A (T-3586) carries six accuracy/IRR/s1 points this task owns: A271, A280, A286, A288, A290 and A298.
  These are conventional series with roots from -2.9E-4 to 2.6E-4, where the engine is `#NUM!` and Excel returns the
  root. They are `known-failures.tsv` lines; remove them once they pass.
- Excel itself is `#NUM!` on four other near-zero points of that table, and the engine already matches: A273, A276,
  A279 and A293 (roots -5.0E-7 to 2.4E-5). A276's root, 2.4E-5, is inside this task's band, so a fix must keep
  `#NUM!` there.

Training probes from draft #2529 (salvaged 2026-11-04; the hold-out case landed in #2562; T-3408 cites cells of both).
Visibly-off cells on main: `#NUM!` against Excel's number on all 19 probe cells and on 23 of the 25 sweep cells; sweep
A33 and A82 return a number off from about the 10th significant digit. T-3408 owns the digits of probe A10-A13, A16,
A17, A19 and A37-A40 once they return a number, and of sweep A33 and A82:
- probes/t3347-irr-conventional-num.case.json (19): P!A10, A11, A12, A13, A14, A15, A16, A17, A18, A19, A20, A21, A36, A37, A38, A39, A40, A41, A42
- probes-numeric/t3347-irr-conventional-sweep.case.json (25): P!A1, A4, A7, A17, A33, A35, A38, A45, A47, A51, A56, A58, A61, A62, A64, A71, A76, A82, A87, A88, A96, A97, A108, A109, A119

## Acceptance criteria
- [ ] Conventional series with an IRR between 1e-5 and 3e-4, and near -10%, match Excel's result (number or error, within 15 significant digits for a number) on the oracle-filled cells.
- [ ] A new public probe case under eval/suites/probes/cases/ (one milestone tag, M9; decision 0057), filled by the
      oracle (excel-graph), reproduces the bug; its cells are listed in this file before the fix. Include the inputs in
      Context and points on both sides of the failing region.
- [ ] After the fix every cell of that case is within 15 significant digits of Excel's value, with the right type and
      error (decision 0083's rule).
- [ ] No gated cell regresses (`make eval-baseline` refuses regressions; decision 0083's ulp allowance only as it
      states).
- [ ] Gate A envelope on a fresh sweep of its own (decision 0188 R2, R4.2): a new probes-numeric case, filled, with
      points drawn log-uniform over the function's valid domain and stratified around the failing region, scored
      against a 50-digit mpmath reference. The Outcome reports, per stratum, the points within 15 digits of Excel and
      the points within 1e-9 relative of the math (absolute 1e-300 near 0): 0 points outside both, and 0 cells where
      one side is an error and the other a number. This is a floor only: the target stays Excel's value.
- [ ] The rule (what Excel does, what was wrong) is in docs/excel-semantics/ in the same PR.
- [ ] `make check` green; reviewers approve.

## Cells of this task's hold-out (gap triage 2026-11-04b)
T-3411 listed these as "Type cells of T-3347's hold-out" and owned them until it closed (PR #3063); no open task
named them after it. Excel returns the root, the engine is `#NUM!` (`make eval` on origin/main a7b115e9e). With the
fallback's Newton cap lifted (local experiment, not committed) 10 come out at Excel's value (A13, A18, A21, A39,
A46, A49, A53, A61, A69, A105), 11 cells of the case that pass now break, and the other 20 return a number 7 to 14
digits from Excel's (A1: Excel 1.9690908018343833E-5, Newton from v = 0 1.9690910050940147E-5), so Excel's stop is
not the fallback's. T-4027 (the level annuities of probes/t3406) is the same path; say so in both if one rule fits.
- probes-numeric/t3347-irr-conventional-holdout.case.json (30): P!A1, A3, A7, A13, A18, A21, A23, A39, A44, A45, A46, A47, A49, A50, A53, A54, A56, A57, A61, A66, A67, A69, A71, A90, A95, A103, A105, A107, A110, A119

## Out of scope
- Last-digit (bit) parity on cells already within 15 digits: Phase 1b (decision 0188 R5).
- Fitting to Excel's bits; if a fit is needed it follows decision 0188 R4 (hold-out split, fresh sweep, `FITTED` tag).

## Visible-parity stage
Decision 0083 and 0114 (`accuracy`): done when every cell this task owns is within half a unit of the 15th significant
digit of Excel's value with the right type and error, and the fresh sweep shows the Gate A envelope (decision 0188 R2).
Bit-level residue moves to a follow-up with `defer: phase-1b`.

## Decision 0267 cells
Decision 0267 (the owner's cut-loss plan, 2026-11-04) gives this task 9 probes-numeric cells that no open task listed: 9 with the wrong type or error.
These come from the run at main 985d511b. They are Tier A (a wrong type or error, or the 11th digit or earlier) or Tier B (the 12th or 13th digit). Tier A is worked first.
Decision 0056's time-box applies. If two attempts do not move these cells, block the task as time-boxed: decision 0267 then accepts the cells that are left, and no follow-up is filed for them.

- probes-numeric/t3347-irr-conventional-holdout.case.json (9): P!A123, A126, A130-A132, A134-A135, A138, A143

- [x] Every cell listed above matches Excel within half a unit of the 15th significant digit, or has Excel's type and error. Otherwise the task is blocked under decision 0056 and the rest is accepted (decision 0267).

## Outcome (2026-11-06, branch agent/functions-financial/T-3347-irr-conventional-r2)
- **Decision 0267 cells: all 9 match Excel.** On main, T-3407's overflow rule already matched seven (A123, A126, A130,
  A132, A134, A135, A143). A131 and A138 were `#NUM!` for Excel's `#DIV/0!`. On both, the secant from 0.1 jumps to
  d = -5.5E+7 or -8.5E+6, where NPV is finite but 7.5E+275 or 1.8E+278. It comes back to d = 0.0919, takes a zero
  step and goes flat (0/0). The engine called that a failed run, and the fallback was `#NUM!`.
- **Attempt 1 (refuted, not shipped):** treat every 0/0 flat step as `#DIV/0!`. It moves A131 and A138 but breaks
  about 150 cells across t3406, t3411, t3478, solver-iteration-traces and the T-3347 cases, where Excel is `#NUM!` or
  returns the root.
- **Attempt 2 (shipped, fitted; decision 0316):** a run from 0.1 that goes flat after a zero step is `#DIV/0!` when
  its largest |NPV| is above 3E+226 (`IRR_FLAT_DIV_ZERO_NPV`, functions/financial/financial_util.rs).
  - Range (R4.3): only runs from 0.1 (the default guess or the retry from 0.1), with 27..144 flows and a largest
    |flow| of 1.3E+4..6.7E+14 (`IRR_FLAT_DIV_ZERO_FLOWS`). A run from another guess fails as before; the unit test
    checks A138's series from 0.05 and 0.2.
  - `FITTED(T-3347, ...)` tag; quality/fitted.json 93 -> 94.
  - R4.1, fit lab split (`holdout.is_holdout`), 209 cases with IRR cells, margin 5 points. The cut is placed from the
    train side: the largest train `#NUM!` run is 1.556E+226 (t3406 A65) and the smallest train `#DIV/0!` above it is
    6.57E+230 (t3406 A81). Main -> rule, cells passing:
    - train 2,086 -> 2,099 of 2,580 (80.85% -> 81.36%); type/error mismatches 344 -> 331
    - test 883 -> 889 of 1,080 (81.76% -> 82.31%); 155 -> 149
    - 0 worse on either side. Train fixes (13): holdout A138; t3406 A66, A67, A68, A81; t3411-irr-r4-fresh A83, A91;
      -2 A6, A92; -3 A78; -4 A4, A89, A103. Test fixes (6): holdout A131; t3406 A82; -2 A1, A19; -4 A86, A94.
  - R4.2 fresh sweep, drawn and filled after the freeze (probes-numeric/t3347-irr-r4-fresh, 150 cells, excel-graph
    2026-11-06): passed 82 -> 90. 8 cells move to Excel's `#DIV/0!` (A37, A49, A54, A61, A84, A131, A144, A148) and
    none away. Its 60 off cells are evidence, not this task's Decision 0267 cells; T-3548 owns them.
  - Gate A envelope per stratum (25 cells each), cells in the envelope before -> after:
    - n 20..40, root -0.5..-0.1: 14 -> 14
    - n 40..80, root -0.5..-0.1: 16 -> 18
    - n 80..147, root -0.4..-0.05: 14 -> 16
    - level, n 30..147, root -0.4..-0.05: 16 -> 17
    - n 27..147, root -0.1..-0.01: 3 -> 3
    - n 27..147, root -0.6..-0.3: 19 -> 22
  - Outside the envelope: 68 -> 60. All 60 are an error against Excel's number or another error, and no number is
    outside both 15 digits of Excel and 1e-9 of the math. No point moved out. The sweep's math sidecar is
    `t3347-irr-r4-fresh.math.json` (`rustsheet-accuracy math`).
  - Against the cut: on 11 runs past it, Excel returns a root from -4% to -11% (old t3411-irr-r4-fresh A15, A22; -4
    A17, A79, A88, A93; fresh A52, A53, A58, A74, A109). The engine's `#NUM!` there is now `#DIV/0!`, wrong before
    and after. Two `#DIV/0!` runs peak below the cut (t3411-irr-r4-fresh A85, -3 A24). So it is a proxy
    (financial.md "IRR is #DIV/0! on a flat secant after a far step").
  - R4.4: only the error type is scored, so the ulp floor does not arise. No keep-matched branch: the domain is the
    run's state, not a set of matched cells.
- **Eval delta** (full `rustsheet-eval run`, main 5304248f4 vs branch; report.json lists only failing cells):
  - probes/t3406-irr-num-boundary: passed 83 -> 88
  - probes-numeric/t3347-irr-conventional-holdout: passed 108 -> 110 of 144
  - t3411-irr-r4-fresh: 89 -> 91
  - t3411-irr-r4-fresh-2: 82 -> 86
  - t3411-irr-r4-fresh-3: 89 -> 90
  - t3411-irr-r4-fresh-4: 79 -> 84
  - No other case changes. The probe t3347-irr-conventional-num (19 cells) and t3347-irr-conventional-sweep (25
    cells) do not move, nor do the hold-out's 30 number cells or Gate A's A271-A298: their cells are `#NUM!` for a
    number, which this rule does not reach.
  - `make eval-baseline` drops the five t3406 lines from eval/baseline/probes.txt. `rustsheet-eval accuracy` passes
    unchanged: no table point reaches the rule.
- T-3548's six t3411 cells listed under "Against the cut" were `#NUM!` and are now `#DIV/0!`, still a type
  mismatch; T-3548's lists note it. No T-3408, T-3407 or T-2131 cell moves.

## Blocked because
Blocked because: time-boxed (decision 0056)
- The decision 0267 cells are done (above). The task's main acceptance criteria are not met: IRR's `#NUM!` for
  Excel's root on conventional series near 0 and near -10%. Those cells are the probe's 19, the sweep's 23 of 25, the
  hold-out's 30, and Gate A's A271, A280, A286, A288, A290 and A298.
- Two attempts on them have failed. Draft #2529's Newton stage from v = 1 fit its training cases but failed its own
  hold-out. T-3406 then refuted Newton under any cap, secant caps of 50, 100 and 200 steps, a Newton step past the flat
  step and secant retries from 17 fixed guesses, scored on 450 cells. This run's data adds one fact: on these series
  the engine's secant takes the same far step and flat 0/0 stall where Excel returns the root as where it errs (the 11
  roots past the cut), so Excel's path differs from the engine's before the stall.
- Under decision 0267 the remaining cells are accepted when this block stands; no follow-up is filed for them.
