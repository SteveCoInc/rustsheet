---
id: T-4045
title: "F.DIST.RT is exactly 0 in Excel at true tails of 4E-297 and 9E-278 (11 t4017 cells; find Excel's underflow cut)"
milestone: M9
lane: functions-stat
status: todo
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

## Acceptance criteria
- [ ] A probes-numeric case (excel-graph, valid arity, milestone M9) brackets the zero cut on at least two shape
      families; its cells are listed in this file before the fix.
- [ ] The 11 cells above and the probe's zero cells are exactly 0; every probe and t4017 cell where Excel returns a
      number stays a number with an unchanged or better verdict (cell by cell, `rustsheet-eval show --all`).
- [ ] If the cut is a constant read off cells, it carries a `FITTED` tag with its bracket and range (decision 0188
      R4.3), with the R4 guards of reviewer-excel-fidelity check 7: the hold-out counts, a fresh sweep scored once after
      the freeze, the Gate A envelope result, the reproducibility floor, and no keep-matched branch.
- [ ] The rule in docs/excel-semantics/statistics.md.
- [ ] No gated cell regresses (`make eval-baseline`); new failing cells have owners.
- [ ] `make check` green; reviewers approve.

## Out of scope
- `#NUM!` at df1 79, df2 1E10 (T-4044); Excel's digits at huge df2 (T-3209, T-4018).

## Visible-parity stage
Decision 0083 and 0114 (`visible`): done when every cell this task owns is exactly Excel's 0 or within half a unit of
the 15th significant digit of Excel's value with the right type.

## Decision 0267
The owner's cut-loss plan (decision 0267, 2026-11-04) makes this a Tier A task, worked first: its cells have the wrong type or error, or are first wrong at the 11th digit or earlier. Decision 0056's time-box applies. If two attempts do not move the cells, block the task as time-boxed, and decision 0267 accepts the cells that are left.
