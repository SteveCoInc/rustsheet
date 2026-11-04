---
id: T-3347
title: "IRR returns #NUM! on conventional cash-flow series whose IRR is near 0 or near -10%"
milestone: M9
lane: functions-financial
status: todo
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
