---
id: T-4216
title: "GAMMA.DIST at shape 1E8 just below the mode: Excel 14 to 18 ulp off the math either side of Temme's cut (t2982 A144-A146; needs an oracle sweep, Phase 1b)"
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
After Phase 1, `GAMMA.DIST(x,a,1,TRUE)` at a shape of about 1E8 and x 0.05% to 0.01% below the mode gives Excel's
value on t2982 A144 to A146. Excel is the inaccurate side there, and the task finds which form Excel evaluates on each
side of its 1E8 shape cut.

## Context
- Gap triage 2026-11-04 (`make eval` on origin/main b450050f8). No open task lists these three cells. T-3286 (done)
  measured them, and T-3272 names them only under "Out of scope". Values (Excel, ours, error in ulp against the
  60-digit math, from T-3286's Outcome and this run):
  - probes/t2982-gamma-inv-huge-shape S!A144 `GAMMA.DIST(99990000,99999999,1,TRUE)`: Excel 0.15867945221382798
    (-1.2 ulp), ours 0.15867945221382804. Within 15 digits.
  - S!A145 `GAMMA.DIST(99950000,1E8,1,TRUE)`: Excel 2.8546421399586357E-07 (+18.1 ulp), ours 2.8546421399586256E-07
    (-0.9). First wrong at the 15th digit.
  - S!A146 `GAMMA.DIST(99950000,99999999,1,TRUE)`: Excel 2.8561234160188864E-07 (-13.6 ulp), ours
    2.856123416018894E-07. First wrong at the 14th digit.
- A145 and A146 are visibly off under the 15-digit bar (GAMMA.DIST is not in decision 0267's 13-digit family). Both
  sides are within 1E-9 of the math, so under decision 0188 R1(c) they are Phase 1b evidence, not counted. They are
  Tier C/D, which decision 0267 cuts from Phase 1 work, hence `defer: phase-1b`. For Phase 1's owner criterion, T-4076
  lists A145 and A146 as Phase 1b evidence. This task owns the fix.
- Needs an oracle sweep. Two visibly-off cells cannot pin a form. The sweep should cover:
  - shapes 5E7 to 2E8, closer around 1E8 (99999998, 99999999, 1E8, 100000001);
  - x/a from 0.999 to 0.99999 on both sides of the mode;
  - a hold-out.

  Put it in the `probes` suite: probes-numeric is frozen to decision 0267 tasks, and this is not one. File it as a
  research task when this task is undeferred.
- statistics.md "Incomplete gamma: accurate kernel (T-0971)", the T-3286 paragraph. Below 1E8 Excel matches Temme's
  expansion near the mode (A144). The engine takes Temme within `NEAR_MODE` from a shape of 1E3, with a·phi in
  double-double. Excel's error of 18 ulp at a = 1E8 and -13.6 at 99999999 suggests a different exponent or series on
  each side of the cut. Code: crates/rustsheet-engine/src/functions/statistical/incomplete_gamma/.
- Related, not duplicated: T-4000 (shapes 21 to 999 near the mode, accuracy), T-3252 (prefactor window probe at shapes
  50 and 200), T-3272 (gamma quantiles from shape 1E9).

Cells:
- probes/t2982-gamma-inv-huge-shape.case.json (3): S!A144, A145, A146

## Acceptance criteria
- [ ] The three known-failure lines disappear from eval/baseline/probes.txt with no new ones, or the Outcome records
      each remaining cell with its ulp and why (decision 0056 time-box).
- [ ] statistics.md records Excel's form at shape 1E8 near the mode, with the sweep as evidence.
- [ ] R4 guards (decision 0188, reviewer-excel-fidelity check 7):
  - [ ] hold-out counts from the sweep's split, stated before scoring;
  - [ ] a fresh sweep after the freeze;
  - [ ] the Gate A envelope result for GAMMA.DIST, CHISQ.DIST and POISSON.DIST;
  - [ ] `FITTED(T-4216, n_cells, range)` on fitted pieces;
  - [ ] the reproducibility floor;
  - [ ] no keep-matched branch.
- [ ] `make check-fast` is green locally, CI is green, and reviewers approve.

## Out of scope
- Shapes of 1E9 and up (T-3272). Near-mode accuracy at shapes below 1E3 (T-4000).
