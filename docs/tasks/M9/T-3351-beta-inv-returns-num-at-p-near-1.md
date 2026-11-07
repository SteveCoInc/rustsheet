---
id: T-3351
title: "BETA.INV returns #NUM! at p near 1-1E-8 with a small b, where the answer is 1"
milestone: M9
lane: functions-stat
status: blocked
priority: P0
size: S
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
BETA.INV (and BETAINV) return Excel's value at p ≈ 1 − 1e-8 with a small b, where the engine returns `#NUM!` today and
the answer is 1.0.

## Context
- Owner's 2026-10-31 assessment (decision 0188, "The engine is wrong where Excel is right", item 4). Users hit
  this: it is an ordinary input, Excel returns the right number and the engine does not. Highest-priority work of the
  accuracy stage.
- Code: `fn_beta_inv` in crates/rustsheet-engine/src/functions/statistical/beta.rs and inverse_beta.rs (the statrs
  port: Newton steps, step adjustments and their caps). The quantile rounds to 1 in doubles; the search must return it
  instead of failing.
- Cells already failing (gap triage 2026-10-31, `make exit-owners MILESTONE=M9`, eval on a64f963e; Excel 1, ours
  `#NUM!`): probes-numeric/m9-beta-inv Calc!A7 `BETA.INV(0.9999999999,0.5,0.5)`, A42 (a = 1), A77 (a = 2.5), A112
  (a = 10) and A147 (a = 100), all at b = 0.5. statistics.md "Incomplete beta sweep (T-1044)" already records
  `BETA.INV(0.9999999999,0.5,0.5)` as exactly 1. The new probe below still covers the rest of the region.
- Related: T-1871 (statrs saturation and panics), T-2511 (crossing of the ported CDF), T-2555 (BETA.INV's early
  Newton stop at a huge shape), T-2633 (BETA.INV chained on BETA.DIST). Name any of their cells this
  moves in the Outcome.
- Gate A (T-3578): accuracy/BETA.INV/s1 A83, A86, A98, A100, A103, A106 and A123 (Excel 1 - 2^-53 where the quantile
  rounds to 1) and A88, A97 and A108 (Excel 2^-53 by the flip) are known failures owned by this task; remove their lines
  from eval/accuracy/known-failures.tsv when they pass (statistics.md "BETA.INV's Gate A table").

## Acceptance criteria
- [ ] BETA.INV at p from 1 − 1e-6 to 1 − 1e-15 with b from 1e-3 to 1 returns Excel's value (1.0 where Excel gives it).
      Not met: met at every probe and sweep point of that region except the 8 where Excel answers 1 at a zero or
      subnormal flipped y (probe A199, A209-A211, C199; sweep A88, A89, A122), which follow no rule found (Blocked
      because).
- [x] A new public probe case under eval/suites/probes/cases/ (one milestone tag, M9; decision 0057), filled by the
      oracle (excel-graph), reproduces the bug; its cells are listed in this file before the fix. Include the inputs in
      Context and points on both sides of the failing region.
- [ ] After the fix every cell of that case is within 15 significant digits of Excel's value, with the right type and
      error (decision 0083's rule). Not met: 232 of 254; the 22 left are listed in the Outcome.
- [x] No gated cell regresses (`make eval-baseline` refuses regressions; decision 0083's ulp allowance only as it
      states).
- [ ] Gate A envelope on a fresh sweep of its own (decision 0188 R2, R4.2): a new probes-numeric case, filled, with
      points drawn log-uniform over the function's valid domain and stratified around the failing region, scored
      against a 50-digit mpmath reference. The Outcome reports, per stratum, the points within 15 digits of Excel and
      the points within 1e-9 relative of the math (absolute 1e-300 near 0): 0 points outside both, and 0 cells where
      one side is an error and the other a number. This is a floor only: the target stays Excel's value.
      Not met: the sweep is filled and scored (Outcome); 6 points are outside the envelope, none of them moved.
- [x] The rule (what Excel does, what was wrong) is in docs/excel-semantics/ in the same PR.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Last-digit (bit) parity on cells already within 15 digits: Phase 1b (decision 0188 R5).
- Fitting to Excel's bits; if a fit is needed it follows decision 0188 R4 (hold-out split, fresh sweep, `FITTED` tag).

## Visible-parity stage
Decision 0083 and 0114 (`accuracy`): done when every cell this task owns is within half a unit of the 15th significant
digit of Excel's value with the right type and error, and the fresh sweep shows the Gate A envelope (decision 0188 R2).
Bit-level residue moves to a follow-up with `defer: phase-1b`.

## Decision 0267 cells
Decision 0267 (the owner's cut-loss plan, 2026-11-04) gives this task 46 probes-numeric cells that no open task listed: 46 with the wrong type or error.
These come from the run at main 985d511b. They are Tier A (a wrong type or error, or the 11th digit or earlier) or Tier B (the 12th or 13th digit). Tier A is worked first.
Decision 0056's time-box applies. If two attempts do not move these cells, block the task as time-boxed: decision 0267 then accepts the cells that are left, and no follow-up is filed for them.

- probes-numeric/m9-beta-inv.case.json (4): Calc!A42, A77, A112, A147
- probes-numeric/t3566-beta-inv-tiny-shape-and-range.case.json (42): Calc!A19-A21, A91-A93, A130-A132, A145-A153, A157-A159, A163-A177, A187-A189, A193-A195

- [ ] Every cell listed above matches Excel within half a unit of the 15th significant digit, or has Excel's type and error. Otherwise the task is blocked under decision 0056 and the rest is accepted (decision 0267).
      Not met: the 4 m9-beta-inv cells and t3566 A19-A21, A91-A93, A130-A132 match Excel exactly; the other 33 t3566
      cells are accepted under decision 0267 item 4 (Blocked because).

## Blocked because
Blocked because: time-boxed (decision 0056)
- Two candidates were tried for the cells left, after the fix in the Outcome landed what the evidence supports:
  1. The flip at every end of (0, 1): 1 - 2^-53 where the quantile rounds to 1 at p of 1/2 or less, and 2^-53 where it
     underflows at p above 1/2 (PR #2573's rule plus its mirror). It moved about 60 cells to Excel but 14 Gate A points
     away from Excel's `#NUM!` (accuracy/BETA.INV/s1 A89, A92, A94, A95, A105, A109, A184, A189, A195, A199;
     BETAINV/s1 A82, A83, A91, A125), so it was not kept.
  2. A simulation of AS 109 as revised (the statrs port's start, step control and accuracy exponent) on the ported
     BETA.DIST kernel, over 370 evidenced end points. It ended at 1 - 2^-53 or 2^-53 at most of Excel's `#NUM!` points
     and did not separate the classes.
- Decision 0267 count: 4 m9-beta-inv cells and 9 t3566 cells (A19-A21, A91-A93, A130-A132) match Excel; 33 t3566 cells
  (A145-A153, A157-A159, A163-A177, A187-A189, A193-A195) are accepted under decision 0267 item 4, with no follow-up for
  them.
- What would reopen it: Excel's own BETA.DIST values at x = 1 - k 2^-53 and k 2^-53 for the shapes of these cells (a
  probe), or any rule that splits Excel's end values from its `#NUM!`s at the Gate A points above without moving one of
  them; or an owner ruling on these classes. The non-0267 cells (probe, Gate A lines, lower-tail mirror) are T-4865's.

## Probe cells (before the fix)
probes/t3351-beta-inv-p-near-1 (254 cells, excel-graph, 2026-10-31, from PR #2573): `BETA.INV(1-1E-k, a, b)` for a in
0.5, 2, 10, b in 1E-3, 3E-3, 0.01, 0.03, 0.1, 0.3, 1 and k in 6, 7, 8, 9, 10, 12, 14, 15, 16 (A1-A189); BETAINV at
a = 2 (A190-A195); the bounds form (A196); the underflow boundary at b = 0.03 for a = 2 and 0.5 (A197-A214, with the
lower-tail mirror `BETA.INV(q, 0.03, a)` in B197-B214) and for a = 10 (C197-C204); p at or below 1/2 (D197-D210). On
main at 22a804e85, 94 cells had the wrong type: every cell where Excel gives 1 (A28-A31, A37-A54, A91-A94, A100-A117,
A126, A154-A157, A163-A180, A189, A194, A195, A197-A199, A206-A211, C197-C199) and D197-D202, D204, D210 (Excel
0.9999999999999999), plus B199, B209-B211 (Excel 0).

## Outcome
Supersedes PR #2573 (closed unmerged; its branch had no usable merge base). Re-derived on current `beta.rs` and
narrowed after re-measuring on current main (Blocked because, candidate 1).
- `beta.rs` `upper_quantile_near_one`: where BETA.INV's upper-tail search rounds to 1 at p above 1/2, the flipped tail's
  y (lower quantile of Beta(beta, alpha) at the exact 1 - p, crossing `bratio::beta_search_tails` without a start; T-1538's
  `beta_lower_crossing` above a shape of 1E5, which no evidenced cell reaches: UNVERIFIED, probe listed in T-4865) gives
  1 - y for a normal y. A y of 0 or a subnormal one stays `#NUM!` as before, so no threshold was chosen and nothing was
  fitted (decision 0188 R4 does not apply). 130 eval cells move from `#NUM!` to Excel's value on this path (129
  distinct suite formulas, counted by the excel-fidelity review building main and this branch and diffing verdicts,
  plus Gate A s1 A123); none moves away. Lift weight unchanged (comment updated: 3.0 ms bound). Unit test
  `test_fn_beta_inv_upper_quantile_rounding_to_one`; statistics.md "BETA.INV where the quantile rounds to 1 (T-3351)",
  with the inconsistent classes Excel shows at the other ends.
- Eval delta, per case (`passed` before → after, main 22a804e85 against this branch):
  - probes/t3351-beta-inv-p-near-1 (new, gated): 147 → 224 of 254. Fixed: A28-A31, A37-A54, A91-A94, A100-A117, A126,
    A154-A157, A163-A180, A189, A194, A195, A197, A198, A206-A208, C197, C198 (77). Left (T-4865): A199, A209-A211,
    C199, B199, B209-B211, D197-D202, D204, D210 (type); B197, B198, B206-B208 (13th digit, Excel itself 1.5E-14 to
    7.2E-14 off the math); 8 last-digit cells (A118, A120, A125, A182, A183, A186, D205, D209), Phase 1b.
  - probes-numeric/m9-beta-inv: 80 → 85 of 175. Fixed A7, A42, A77, A112, A147 (Excel 1).
  - probes-numeric/t3566-beta-inv-tiny-shape-and-range: 110 → 119 of 249. Fixed A19-A21, A91-A93, A130-A132.
  - probes-numeric/t1543-inverse-cdf-tiny-p: A89 `BETA.INV(1-1E-16,1000,1)` fixed (Excel 1).
  - probes-numeric/t3351-beta-inv-sweep (new): 79 → 116 of 200; 37 of the 40 points where Excel is exactly 1 now
    match (all 40 were `#NUM!`).
  - gen-m9: 22656c225ff4f22a Calc!A179 fixed (a FORECAST.ETS.STAT over a BETAINV near 1, now Excel's `#N/A`).
  - Gate A: accuracy/BETA.INV/s1 A123 passes (Excel 1 - 2^-53, the correctly rounded quantile); its known-failures line
    is removed. The other 21 lines that named T-3351 now name T-4865. No point moved outside the envelope.
  - `make eval-baseline`: probes +1 case with 30 new-coverage known failures, 0 regressions in every suite;
    visible-parity gets the new case's line (22 cells). No cell of T-1871, T-2511, T-2555 or T-2633 moved; T-4663's
    DBL_MIN Gate A points are unchanged.
- Sweep basis: probes-numeric/t3351-beta-inv-sweep is this task's own decision 0188 R4.2 / Gate A sweep that its
  acceptance criteria require, landed as a one-off by a task holding a Decision 0267 section, as decisions 0316 and
  0320 did. Its case and oracle files are byte-identical to PR #2573's (excel-graph, filled 2026-10-31; 200 points,
  seed 3351, stratified around the failing region). Nothing was fitted, so it is a fresh sweep, not a hold-out. Math:
  the rustsheet-accuracy sidecar (126 points; it skips the 74 with a `1-q` argument) and a 50-digit mpmath scratch
  solve for those 74. Per stratum, within 15 digits of Excel / within 1E-9 of the math / outside the envelope (an
  error against a number counts as outside) / of which error against number:
  typical (p 0.01 to 0.99, shapes 0.1 to 30) 73 / 80 / 0 / 0 of 80;
  tail (p = 1 - q, q 3E-16 to 1E-6, alpha 0.1 to 10, beta 1E-3 to 1) 57 / 37 / 3 / 3 of 60 (before: 25 / 5 / 35 / 35);
  extreme (p near 0 or 1, shapes 1E-3 to 1E4) 34 / 31 / 1 / 1 of 40 (before: 31 / 28 / 4 / 4);
  log-uniform (shapes 1E-3 to 1E5) 14 / 14 / 2 / 1 of 20 (before: 12 / 12 / 4 / 3).
  Not clean, and none of the 6 moved: A88, A89, A122 (Excel 1 at a y far below the subnormals; they run
  `upper_quantile_near_one`, which returns `None`, so their `#NUM!` is unchanged) and A175, A188 (lower tail, p below
  1/2: one side an error) are T-4865's; A200 (`BETA.INV(1-1.1E-14,0.005,171)`, ours 2.9E-5 against 0.1294) is T-4866's.
- T-4052 (salvage of PR #2573's evidence) is dropped in this PR: both cases and the statistics.md entry land here.
- Follow-ups: T-4865 (P2: where Excel's search returns an end value instead of `#NUM!`, for the probe and Gate A cells
  and the unverified shape-above-1E5 branch), T-4866 (P1: sweep A200). PR #2573's T-3430 and T-3431 never reached main
  and were not reused.
