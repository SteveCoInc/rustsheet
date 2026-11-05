---
id: T-4377
title: "POISSON.DIST cdf window-form residue and below-cut tail: the 52 within-15-digit t4322 fresh-sweep cells no task names (Phase 1b)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4376]
defer: phase-1b
---

## Goal
Each POISSON.DIST cdf cell listed below matches Excel bit for bit, or the Outcome lists it with Excel's value, ours,
the 50-digit math and the reason Excel's last digits cannot be derived.

## Context
- Gap triage 2026-11-05 (`make eval` on origin/main 304f2d006). `make unowned-off ALL=1` prints 0. The
  `scripts/exit_owners.py` ownership rule, run over every failing cell of report.json and every open task, leaves
  52 cells with no owner. All 52 are in probes-numeric/t4322-poisson-window-r4-fresh (66 checked, 57 failing).
  T-4322 (done) drew that case as its R4.2 fresh sweep. Its Outcome routed only the 5 visibly-off cells (G3, B8,
  B9, A10, C11, now T-4357, then T-4368 in open PR #3387). T-4345 (done) used A8:C10 as its hold-out and owned
  only those 5 too.
- None of the 52 is visibly off. They are 46 NumericUlp and 6 NumericClose, all within half a unit in the 15th
  digit. So decisions 0114 item 3 and 0188 R5 put the task under `defer: phase-1b`. Function POISSON.DIST, catalog
  milestone M9, prior 0.05.
- Two groups, with a = k + 1, x = the mean, and the ulp shown as ours minus Excel's:
  - **Window form residue, x/a 1.3 to 1.5 (47 cells, median 2 ulp, max 7).** The engine uses T-4322's form
    (excel_tail.rs `excel_window_upper`: sqrt(a / 2 pi) exp(-B - c) times the accurate fraction, with T-4308's
    window exponent). The residue is Excel's own fraction against the accurate one (T-4345 Outcome: it is not
    Cephes' continued fraction in doubles). Examples from `make eval-show
    CASE=probes-numeric/t4322-poisson-window-r4-fresh.case.json`:
    - B2 `POISSON.DIST(30,42.513,TRUE)`: Excel 0.027789674656578233, ours 0.027789674656578254 (+6).
    - G6 `POISSON.DIST(158,229.08,TRUE)`: Excel 4.1469652897892623E-07, ours 4.146965289789266E-07 (+7).
  - **Below the 1.3a cut (5 cells: A8, C8, A9, C9, B10; x/a 1.29 to 1.298; median 2 ulp, max 12).** Excel there is
    1 to 17 ulp from the correctly rounded Q, and no model is known (T-4345, T-4357, T-4368). Example: C8
    `POISSON.DIST(300,388.29,TRUE)`, Excel 1.8072885627226788E-06, ours 1.8072885627226814E-06 (+12). Re-measure
    these once T-4368 (or its successor) lands a below-cut model. They are listed here so that each one has an
    owner.
- The same window residue is in T-4337's t4291 window cells (B1:H1, A2, B2, C2, E2, G2). Those stay T-4337's. A
  fraction model found here should move them too, so this task's Outcome reports them.
- Needs an oracle sweep. T-4322's 15 train points and its 57-point fresh sweep have both been scored on, so a
  new fraction model has no clean hold-out. T-4376 supplies train and hold-out points. The fit then draws its own
  R4.2 fresh sweep after the freeze, in the probes suite (probes-numeric is frozen, decision 0267 item 5).
- docs/excel-semantics/statistics.md has the T-4322 and T-4345 paragraphs. docs/research/t4322/score.py scores
  the window form.

The cells this task owns (all of probes-numeric/t4322-poisson-window-r4-fresh's failing cells except G3, B8, B9, A10
and C11):
- probes-numeric/t4322-poisson-window-r4-fresh.case.json window: Calc!A1, B1, C1, D1, E1, F1, G1, H1, A2, B2, C2,
  E2, F2, G2, H2, A3, B3, C3, E3, F3, B4, C4, D4, E4, G4, A5, B5, D5, E5, F5, G5, H5, A6, B6, C6, D6, E6, F6, G6,
  H6, A11, A12, B12, C12, A13, B13, C13
- probes-numeric/t4322-poisson-window-r4-fresh.case.json below the cut: Calc!A8, C8, A9, C9, B10

## Acceptance criteria
- [ ] Each of the 47 window cells is bit-exact, or the Outcome lists it with Excel's value, ours, the window form
      with the accurate fraction, and the 50-digit math.
- [ ] Each of the 5 below-cut cells is re-measured after the below-cut model task (T-4368 or its successor), and it
      is bit-exact or listed in the Outcome the same way.
- [ ] A change to the fraction is a fit only if it meets the R4 guards of reviewer-excel-fidelity check 7
      (decision 0188):
      - the train and hold-out counts are stated before scoring (T-4376's odd and even rows; T-4322's 72 window
        points are train only);
      - a fresh sweep is drawn after the model is frozen and committed with its seed;
      - the Gate A envelope result is reported (`rustsheet-eval accuracy`, 0 failing);
      - a `FITTED` tag and range are entered in `quality/fitted.json`;
      - the reproducibility floor is respected: no fit inside T-4302's 4-ulp web-desktop floor unless the
        hold-out beats it;
      - there is no keep-matched branch.
- [ ] probes-numeric/t4322-poisson-window-r4-fresh's failing count (57 at 304f2d006) goes down only, by the cells
      fixed. No other case gains a failing cell (`make eval`, compared per case with the 304f2d006 report). The
      gated suites gain no known-failure line (`make eval-baseline` refuses regressions).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The visibly-off cells G3, B8, B9, A10 and C11, and the below-cut model itself (T-4357, T-4368).
- Far above the mean (T-4337). CHISQ.DIST.RT in the window (T-4346). The density (T-4319).
