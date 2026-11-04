---
id: T-4045
title: "F.DIST.RT is exactly 0 in Excel at true tails of 4E-297 and 9E-278 (11 t4017 cells; find Excel's underflow cut)"
milestone: M9
lane: functions-stat
status: done
priority: P0
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
F.DIST.RT returns Excel's exact 0 on the huge-df2 upper tails where Excel underflows early, from a cut that a probe
brackets, while every tail Excel returns as a number stays a number.

## Context
- Excel (excel-graph, T-4017) is exactly 0 where the true tail is far above the double minimum; the engine returns
  the (correct) small tail (`make eval` on origin/main a7b115e9e; gap triage 2026-11-04b):
  - probes-numeric/t4017-f-huge-df2-above-mean-1e7.case.json (2): Calc!A175, A221
  - probes-numeric/t4017-f-huge-df2-above-mean-3e7.case.json (2): Calc!A175, A221
  - probes-numeric/t4017-f-huge-df2-above-mean-1e8.case.json (2): Calc!A175, A221
  - probes-numeric/t4017-f-huge-df2-above-mean-3e8.case.json (2): Calc!A175, A221
  - probes-numeric/t4017-f-huge-df2-above-mean-1e9.case.json (2): Calc!A175, A221
  - probes-numeric/t4017-f-huge-df2-above-mean-1e10.case.json (1): Calc!A175
  - A175 is `F.DIST.RT(50,30,df2)` (engine 3.96E-297 to 4.18E-297), A221 `F.DIST.RT(20,79,df2)` (engine 9.45E-278 to
    9.99E-278; at df2 1E10 it is `#NUM!`, T-4044).
- Excel (and the engine) are also 0 at `F.DIST.RT(50,2,df2)` (true 1.9E-22) for all six df2, while Excel returns
  `F.DIST.RT(50,20,1E8)` = 3.914E-199 and `F.DIST.RT(20,60,1E8)` = 1.164E-211. So the cut is not on the result's size
  alone (1.9E-22 is 0, 1.2E-211 is not), which points at an intermediate of Excel's incomplete beta (a bgrat or
  prefactor term underflowing, a log-space threshold, or a series that stops at its first term). docs/excel-semantics/
  statistics.md "Incomplete beta at a huge first shape" (the T-4017 paragraph, "Excel returns exactly 0 where the true
  tail is far above the double minimum") records the cells; `reference.py` of T-4017 prints every such cell.
- Code: crates/rustsheet-engine/src/functions/statistical/fisher.rs (F.DIST.RT), bratio/ (bgrat.rs, bpser.rs, mod.rs);
  find where the df1 = 2 zero comes from first (it may be the same branch at another shape).
- These cells were reachable only through deferred T-3209 (`defer: phase-1b`), so `make unowned-off ALL=1` listed them
  as unowned; an exact 0 against a number is visibly off (decision 0083), not a last-digit residue.
- This is a semantic cut, not a numeric fit, but the cached cells do not bracket it: the task starts with an
  excel-probe case (excel-graph) of F.DIST.RT and BETA.DIST at the matching shapes, walking x (or df1) from the last
  returning tail to the first zero at two or three df2, listed here before the fix.

- Probe (2026-11-04, before the fix): probes-numeric/t4045-f-dist-rt-zero-cut (excel-graph, 192 cells). F.DIST.RT
  walks z = df1 x / 2 from 690 to 770 at (df1, df2) = (30, 1E8) in column A, (20, 1E7) in B and (31, 1E8) in C. Column D
  is FDIST at A's cut and column E is FDIST at T-4044's band shapes. Zero cells (Excel exactly 0):
  - probes-numeric/t4045-f-dist-rt-zero-cut.case.json: Calc!A18:A55, B19:B55, C15:C55, D1:D21
  The last numbers are A17 (z 714), B18 (z 715) and C14 (z 711). E1, E4 and E6 (FDIST `#NUM!` in T-4044's band) pass
  with T-4044's `f_bup_sum_overflows`.

## Acceptance criteria
- [x] A probes-numeric case (excel-graph, valid arity, milestone M9) brackets the zero cut on at least two shape
      families; its cells are listed in this file before the fix.
- [x] The 11 cells above and the probe's zero cells are exactly 0; every probe and t4017 cell where Excel returns a
      number stays a number with an unchanged or better verdict (cell by cell, `rustsheet-eval show --all`).
- [x] If the cut is a constant read off cells, it carries a `FITTED` tag with its bracket and range (decision 0188
      R4.3), with the R4 guards of reviewer-excel-fidelity check 7: the hold-out counts, a fresh sweep scored once after
      the freeze, the Gate A envelope result, the reproducibility floor, and no keep-matched branch.
- [x] The rule in docs/excel-semantics/statistics.md.
- [x] No gated cell regresses (`make eval-baseline`); new failing cells have owners.
- [x] `make check` green; reviewers approve.

## Out of scope
- `#NUM!` at df1 79, df2 1E10 (T-4044); Excel's digits at huge df2 (T-3209, T-4018).

## Visible-parity stage
Decision 0083 and 0114 (`visible`): done when every cell this task owns is exactly Excel's 0 or within half a unit of
the 15th significant digit of Excel's value with the right type.

## Decision 0267
The owner's cut-loss plan (decision 0267, 2026-11-04) makes this a Tier A task, worked first: its cells have the wrong type or error, or are first wrong at the 11th digit or earlier. Decision 0056's time-box applies. If two attempts do not move the cells, block the task as time-boxed, and decision 0267 accepts the cells that are left.

## Outcome (2026-11-04, opus run)
- Rule: Excel's L140 bup+bgrat tail is exactly 0 when bup's prefactor ret = P·x0·y0/c (P the Beta(c, a0) density at
  y0) is below DBL_MIN. The threshold is the double range's own bound (no `FITTED` tag), but the quantity it applies to
  was chosen from ranked candidates (e^-z, P, ret), a structural choice under decision 0188 R4. Its guards:
  - Hold-out: leave-one-out over t4045's three families. ln ret at each family's last number and first 0 (60-digit
    mpmath) brackets as (-707.42, -708.42), (-708.38, -709.38) and (-707.59, -708.59). Any two meet in an interval that
    holds ln(DBL_MIN) = -708.396, and that predicts the third family's edge. e^-z (708.4, 745.1) and P (z near 726,
    724, 722) miss every family. The step of about 1 in ln cannot tell ret from ret·c (a factor of 2 at c = 1/2).
  - Fresh sweep after the freeze: probes/t4045-f-dist-rt-zero-cut-fresh (102 cells, excel-graph; generated from the
    frozen rule at untouched shapes df2 1E4, 1E5 and 1E6, c = 1 and c = 1/2, stepping about 0.5 in ln ret). Excel's
    first 0 is within one step of the predicted row in all 6 families. Scored once: 51 of 51 zeros exact, 51 numbers
    NumericClose (last digits, T-3209), 0 type or zero mismatches. Gate A: F.DIST.RT has no accuracy table, and
    `rustsheet-eval accuracy` stays at 0 failing (F.INV A146 is in its envelope).
  - No keep-matched branch: the predicate reads only ret.
- t4045 column E (FDIST at T-4044's band shapes) is the FDIST evidence T-4044's excel-fidelity review asked for
  (#3237: FDIST unprobed there). E1, E4 and E6 pass. E3's digits go to deferred T-3209.
- Code: `bratio/bup.rs` `l140_bup_prefactor_underflows` (bup's prefactor ret is one helper, `l140_bup_ret`, shared
  with `l140_bup_bgrat`), asked for F's tails only by `bratio::f_closed_or_ported_tails` at any a0. Applying it to
  BETA.DIST as well moved `F.INV(1.43795578761177E-262,107702,58)` (accuracy/F.INV/s1 A146, which searches BETA.DIST's
  cdf) 0.6% off the math, out of the accuracy envelope, so BETA.DIST and the t keep today's kernel until T-4254
  probes them. Unit test `upper_tail_is_zero_where_bups_prefactor_underflows` (fisher.rs).
- Eval: every Excel 0 of t4045 (A18:A55, B19:B55, C15:C55, D1:D21) and of t4017 (this task's 11 cells) is now exact.
  With T-4044 merged in, t4045's only off cell is E3 (`FDIST(1.5,80,1E10)` digits, owned by deferred T-3209).
  Gated suites: fixed 0, regressions 0 (`make eval-baseline`).
- statistics.md "Incomplete beta at a huge first shape": a new "Where the exact 0 starts" item. The df1 = 2 zeros come
  from Excel's closed form and stay unexplained (the engine already returns 0 there).
