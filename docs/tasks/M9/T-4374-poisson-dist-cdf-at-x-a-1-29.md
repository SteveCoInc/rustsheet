---
id: T-4374
title: "POISSON.DIST cdf at x/a 1.29 to 1.5: the 52 last-digit cells of T-4322's fresh sweep that no task owns (window-form residue)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
defer: phase-1b
depends_on: [T-4357]
---

## Goal
The POISSON.DIST cdf cells of T-4322's fresh sweep that miss Excel by 1 to 12 ulp match Excel's bits, by a form
that passes the R4 guards. If no such form exists, the rejected forms and their scores are recorded.

## Context
- Gap triage 2026-11-05 (`make eval` on origin/main e84e073bb; calipers, corpus-bench and poi not fetched). A
  cell-level owner scan (`scripts/exit_owners.py` `owned` over every failing cell, with deferred tasks counted)
  found one case whose failing cells no open task names:
  probes-numeric/t4322-poisson-window-r4-fresh, 52 of its 57 failing cells. T-4322's Outcome passed "below the cut,
  the residue" to T-4345. T-4345 (done) and then T-4357 took only the 5 visibly-off cells (G3, B8, B9, A10, C11).
  The rest were left with no owner.
- Semantics: docs/excel-semantics/statistics.md, the T-4322 and T-4345 paragraphs. From 1.3a to 1.5a (a = k + 1,
  integer shapes 31 to 301), Excel's cdf is sqrt(a / 2 pi) exp(-B - c(a)) times the fraction, with T-4308's window
  exponent B. The engine takes that form (crates/rustsheet-engine/src/functions/statistical/ excel_tail.rs
  `excel_window_upper`). Its residue against Excel is 0 to 7 ulp: 51/57 fresh points within 4 ulp, and only 8/57
  bit-exact. Below 1.3a no model reproduces Excel (T-4345: 14 models scored). T-4357 (research, in flight) probes
  the companion functions there.
- Cells, x/a = mean/(k + 1), ulp = ours minus Excel's:
  - Above the cut (window form, 47 cells, 1 to 7 ulp; B2 6, B13 5, F5 -5 and G6 7 are past 4 ulp):
    probes-numeric/t4322-poisson-window-r4-fresh Calc!A1, Calc!A2, Calc!A3, Calc!A5, Calc!A6, Calc!A11, Calc!A12, Calc!A13, Calc!B1, Calc!B2, Calc!B3, Calc!B4, Calc!B5, Calc!B6, Calc!B12, Calc!B13, Calc!C1, Calc!C2, Calc!C3, Calc!C4, Calc!C6, Calc!C12, Calc!C13, Calc!D1, Calc!D4, Calc!D5, Calc!D6, Calc!E1, Calc!E2, Calc!E3, Calc!E4, Calc!E5, Calc!E6, Calc!F1, Calc!F2, Calc!F3, Calc!F5, Calc!F6, Calc!G1, Calc!G2, Calc!G4, Calc!G5, Calc!G6, Calc!H1, Calc!H2, Calc!H5, Calc!H6
  - Below the cut (5 cells; A8 2, A9 -2, B10 -1, C8 12, C9 9 ulp), whatever T-4357 finds there:
    probes-numeric/t4322-poisson-window-r4-fresh Calc!A8, Calc!A9, Calc!B10, Calc!C8, Calc!C9
  - Examples: A1 `POISSON.DIST(44,66.7884,TRUE)`, Excel 0.0019680482489483836, ours 0.001968048248948383;
    B1 `POISSON.DIST(120,174.512,TRUE)`, Excel 7.830504504258226E-06, ours 7.830504504258229E-06.
- Why deferred (decision 0114; decision 0188 R4.4 and R5): 46 of the 52 cells are within 15 digits. The other one,
  B5 `POISSON.DIST(64,85.26,TRUE)`, is 3 ulp off but crosses a 15-digit rounding boundary. Both values are within
  1e-9 of the 50-digit math, so R1(c) leaves it out of every Phase 1 count as Phase 1b evidence. 47 of the 52 cells
  are within 4 ulp, which is the oracle's reproducibility floor (web against desktop). Fitting there is barred
  until desktop evidence exists. The 5 cells past 4 ulp (B2, B13, F5, G6 above the cut; C8, C9 below it) are the
  only fit targets.
- Needs an oracle sweep if a candidate form is found. The 72 cached points (15 T-4302 train, 57 fresh) are the
  train set by then. Before any candidate is accepted, a fresh sweep must be drawn after the freeze.
- Probes-numeric is ungated (`gate = false`, no baseline file), so progress is measured on report.json's cells.

## Acceptance criteria
- [ ] Each cell above matches Excel's bits, or the Outcome records why the form that fits it was rejected. No other
      POISSON.DIST, CHISQ.DIST.RT or GAMMA.DIST cell regresses: the known-failure lines of
      eval/baseline/probes.txt and eval/baseline/gen-m9.txt only shrink, and no t4322 fresh cell goes from pass
      to fail.
- [ ] R4 guards (reviewer-excel-fidelity check 7, decision 0188): train and hold-out counts are reported side by
      side, with the split fixed before scoring (R4.1). A fresh log-uniform sweep is drawn and filled after the
      freeze, and its counts are recorded (R4.2). The Gate A envelope holds on accuracy/POISSON.DIST, with its
      result recorded (R2, R4.5). Every new constant or cut carries a `FITTED(T-4374, n cells, range)` tag, and the
      fitted piece is not applied outside that range (R4.3). No cell within 4 ulp is a fit target (R4.4,
      reproducibility floor). There is no branch that keeps a matched cell by special case (no keep-matched branch).
- [ ] statistics.md's T-4322 paragraph is updated with the outcome.
- [ ] `make check` green; reviewers approve (excel-fidelity included).

## Out of scope
- The 5 visibly-off cells of the same case (G3, B8, B9, A10, C11), which T-4357 owns.
- CHISQ.DIST.RT in the window (T-4346, T-4320).
