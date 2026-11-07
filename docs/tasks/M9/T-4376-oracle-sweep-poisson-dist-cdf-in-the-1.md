---
id: T-4376
title: "Oracle sweep: POISSON.DIST cdf in the 1.3a to 1.5a window and far above the mean, fresh points for a fraction fit (Phase 1b)"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
The Phase 1b fits of Excel's own continued fraction in POISSON.DIST's cdf (T-4377 inside the window, T-4337 far
above the mean) have Excel points nobody has tuned on yet. They need them to train and score a candidate, because
every cached point there is already spent.

## Context
- Above 1.3a (a = k + 1, x = the mean, k 30 to 300) Excel's cdf is a prefactor times a fraction. Inside the window,
  1.3a to 1.5a, the prefactor is T-4322's form with T-4308's window exponent. Above 1.5a it is T-4302's form. The
  prefactors are settled. What is left is Excel's own fraction, 1 to 9 ulp off the accurate one. Neither T-4302 nor
  T-4322 fitted it (statistics.md, the T-4302 and T-4322 paragraphs). Cephes' continued fraction in doubles is not
  it either (T-4345 Outcome: 47/57 fresh within 4 ulp, against 51/57 with the exact fraction).
- Where the cached points stand:
  - Window: T-4322's train set (15 T-4302 points) and its fresh sweep (probes-numeric/t4322-poisson-window-r4-fresh,
    57 window points) were both scored on.
  - Far above the mean: T-4302's train and hold-out sets (probes/t4302-poisson-cdf-beside-pmf,
    probes/t4302-poisson-cdf-holdout-2) and its fresh sweep (probes-numeric/t4302-poisson-cdf-r4-fresh) were used
    the same way.
  So a fit has no clean hold-out for a new fraction model. T-4337's acceptance criteria ask for this sweep first.
- probes-numeric is frozen to new cases (decision 0267 item 5), so the case goes in the probes suite.
- Generator precedent: docs/research/t4322/make_sweep.py (seeded, column-major, means to 6 significant digits, no
  selection).

## Acceptance criteria
- [ ] `docs/research/t4376/make_sweep.py` writes `eval/suites/probes/cases/t4376-poisson-fraction-sweep.case.json`
      (`rustsheet-case/v1`, sheet `Calc`, one `POISSON.DIST(k,mean,TRUE)` per cell, a `hypotheses` label naming k
      and x/a). It uses seed 43760, k log-uniform on [30, 300] rounded to an integer, and the mean cut to 6
      significant digits. It draws 96 points with mean / (k + 1) uniform on [1.3, 1.5] (rows 1:12, columns A:H) and
      96 with mean / (k + 1) uniform on [1.5, 2.5] (rows 14:25), with no selection.
- [ ] The split is written in the generator's docstring before the oracle fill: odd rows are train, even rows are
      hold-out. A fit task scores its hold-out only after it freezes the model. This case is not that fit's R4.2
      fresh sweep. The fit draws its own after the freeze.
- [ ] The `.oracle.json` sidecar is filled by `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes`
      (excel-probe skill) with no pending cells, and the `.math.json` holds the 50-digit values (as the t4302
      cases do).
- [ ] statistics.md, after the T-4345 paragraph, records how many points are bit-exact and how many are within
      4 ulp of the current engine forms (excel_tail.rs `excel_window_upper` and `excel_far_above_mean`), per region.
      It cites the case. T-4377 and T-4337 link the case.
- [ ] The case's failing cells are owned. Each region's cells are named in T-4377 (window) or T-4337 (far above),
      so `make unowned-off ALL=1` and the all-cells ownership check add no unowned cell. A visibly-off cell, if
      any, goes to a non-deferred task.
- [ ] The fixtures hold only formulas and Excel values. No RUSTSHEET_ORACLE_* values go in logs or commits. If the
      oracle is unavailable, the task is `status: blocked` with the reason.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Fitting or changing the engine (T-4377, T-4337). The cdf below 1.3a (T-4357, T-4368). CHISQ.DIST.RT (T-4346,
  T-4320).
