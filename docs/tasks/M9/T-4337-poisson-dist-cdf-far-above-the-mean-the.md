---
id: T-4337
title: "POISSON.DIST cdf far above the mean: the 75 one-to-nine-ulp cells of t4291 and T-4302's hold-out 2 no task names (Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
defer: phase-1b
---

## Goal
Every POISSON.DIST cdf cell listed below either matches Excel bit for bit or is listed in the Outcome with Excel's
value, ours, the 50-digit math and the reason Excel's last digits cannot be derived.

## Context
- Gap triage 2026-11-05 (`make eval` on origin/main 4e3993c9e). Every visibly-off cell has an open owner
  (`make unowned-off ALL=1` prints 0). Checking every failing cell of report.json against the open tasks (the
  `scripts/exit_owners.py` ownership rule) leaves these 75 POISSON.DIST cells with no open owner. T-4291 and T-4302
  are done; T-4302's Outcome routed only the visibly-off cells (to T-4322, T-4319, T-4320 and T-3295).
- All 75 are `POISSON.DIST(k,mean,TRUE)` at k 30 to 300, 69 NumericUlp and 6 NumericClose, 1 to 9 ulp (median 2).
  The report counts none of them as visibly off, so decisions 0114 item 3 and 0188 R5 put the task under
  `defer: phase-1b`.
- Two regions, with a = k + 1 and x = mean:
  - **Above 1.5a (65 cells: t4291 rows 3 to 6, all 39 of hold-out 2).** T-4302's structural model applies: Excel's
    gamma-density prefactor times x times the accurate fraction (excel_tail.rs `excel_far_above_mean`). T-4302's
    Outcome ("Reproducibility floor") says what is left is Excel's own fraction, a few ulp (up to 8) from the
    accurate one, and that it was not fitted because it lies within the 4-ulp web-desktop floor. Example
    (`make eval-show CASE=probes/t4302-poisson-cdf-holdout-2.case.json`): A1 `POISSON.DIST(199,392.613,TRUE)`,
    Excel 2.501665641787806E-27, ours 2.5016656417878044E-27.
  - **Inside the gamma window, x/a 1.27 to 1.49 (10 cells: t4291 B1 to H1, A2, C2 and E2).** These are T-4322's
    region (its live owner, `campaign: visible`, for the visibly-off window cells B2, D2, F2, G2 and H2 of the same
    probe). Whatever T-4322 lands should move these too; re-measure them after it.
- docs/excel-semantics/statistics.md has the T-4302 paragraph; docs/research/t4302/score.py scores the cdf against
  Excel's density times x times the fraction. No new Excel point is needed to re-measure.
- Split (T-3229): T-4322 owns this probe's B2, D2, F2, G2 and H2; this task owns none of them.
- Cells:
  - probes/t4291-poisson-far-above-mean.case.json (36): Calc!A2, A3, A4, A5, A6, B1, B3, B4, B5, B6, C1, C2, C3, C5, D1, D3, D4, D5, D6, E1, E2, E3, E4, E5, E6, F1, F4, F5, F6, G1, G5, H1, H3, H4, H5, H6
  - probes/t4302-poisson-cdf-holdout-2.case.json (39): Calc!A1, A2, A3, A4, A5, A6, B1, B2, B3, B5, B6, C1, C2, C3, C6, D1, D2, D4, D6, E1, E4, E5, E6, F1, F3, F4, F5, F6, G1, G2, G3, G5, G6, H1, H2, H3, H4, H5, H6

## Acceptance criteria
- [ ] After T-4322 is done, the 10 window cells are re-measured and each is bit-exact or listed in the Outcome.
- [ ] Each of the 65 cells above 1.5a is bit-exact or listed in the Outcome with Excel, ours, Excel's density times x
      times the accurate fraction, and the 50-digit math.
- [ ] Any change to Excel's fraction there is a fit only if it meets the R4 guards of reviewer-excel-fidelity check 7
      (decision 0188): hold-out counts stated before scoring, a fresh sweep, Gate A envelope result, a `FITTED` tag and
      range in `quality/fitted.json`, the reproducibility floor (T-4302's 4-ulp web-desktop floor), and no
      keep-matched branch. Needs an oracle sweep: the cached probes are already T-4302's train and hold-out sets,
      and its fresh sweep (probes-numeric/t4302-poisson-cdf-r4-fresh) is spent, so a fit first files a
      research-lane sweep task (`make new-task`, probes suite: probes-numeric is frozen by decision 0267 item 5) and
      depends on it.
- [ ] The fixed cells' lines disappear from eval/baseline/probes.txt with no new ones.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The window's visibly-off cells and its model (T-4322), the density (T-4319), CHISQ.DIST.RT (T-4320), the pmf
  (T-3295).
