---
id: T-3926
title: "Oracle sweep: F.DIST density where Y = density·x is subnormal, at power-of-two and general x, with a hold-out, for T-2647"
milestone: M9
lane: research
status: in-progress
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The fit task T-2647 gets dense excel-graph evidence of F.DIST's density where Excel's stored Y = density·x is subnormal.
The evidence must tell its candidate Y/x steps apart and give a separate hold-out (decision 0188 R4.1), so the fit is
not attempted blind.

## Context
- statistics.md, F density entry ("At a tiny x the stored Y = density·x decides (t1864-f-pdf-tiny-x, 20 cells)"):
  - Excel stores Y and returns F = fl(Y/x).
  - A subnormal Y keeps only its own bits, so Excel can be several percent off the exact density.
  - Excel's value then differs from fl(Y/x) by about 1E-14 relative (T-2647).
- T-2647 lists the candidates: an extended-precision Y, a division by a rounded 1/x, or the EXP of a log difference.
  It asks for cells with a known subnormal Y at an x that is a power of two, where Y/x is exact in double, to tell
  them apart.
- The cached evidence is 90 counted cells over six df pairs. Every cell is at a general x:
  - probes-numeric t1864-f-pdf-tiny-x: A6, A10, A17, A20;
  - t2586-r5-f-edges-a: B771-B783, B811-B823, B851-B863, B891-B903, at (3,5);
  - t2586-r5-f-edges-hold: B217-B229, B257-B269, at (5,3);
  - t2586-verify-r5-f-edges-hold: B519, B520, B534, B535, B549, B550, B564, B565, at (3,1000) and (5,130).
  These are T-2647's cells. Excel is up to 41% off the 50-digit density there, and the engine matches Excel to within
  5E-14.
- Region: with d1 ≥ 3, Y ∝ x^(d1/2) near 0, so Y is subnormal for x below a df-dependent cut. For (3,5) the cut is
  about 1E-205: t2586-r5-f-edges-a A771, x = 2.43E-216, gives Y ≈ 2·2^-1074. Keep d1 ≥ 3. At d1 = 1 or 2 the
  density's limit at 0 is infinite or 1, and a subnormal x is the engine's `#DIV/0!` (statistics.md, `f_dist_of`).
- Model for the case layout and the mpmath side file: docs/research/t3285/mksweep.py.
- Decision 0245: the PR that adds the cases also makes their visibly-off cells owned, here by amending T-2647's cell
  list (or by naming the cases on a line with no cell token, which owns them whole).

## Acceptance criteria
- [ ] A new probes-numeric case `eval/suites/probes-numeric/cases/t3926-f-density-subnormal-y.case.json`
      (`rustsheet-case/v1`, sheet `Calc`, one formula per cell, a `hypotheses` label per cell naming d1, d2, x and the
      mpmath Y/2^-1074), filled with `cd tools && uv run rustsheet-oracle fill -b excel-graph --suite probes-numeric`
      (excel-probe skill). Grid:
  - df pairs (3,5), (5,3), (3,1000), (4,1000), (5,130), (10,1000), (3,3) and (6,20);
  - per pair, 30 x values whose exact Y/2^-1074 is log-uniform in [1, 2^52], seed stated in the description;
  - half of the x values are powers of two (`2^-k`, written as such), half are general literals of at most 10
    significant digits;
  - every point as `F.DIST(x,d1,d2,FALSE)` (240 cells).
- [ ] A second case `t3926-f-density-subnormal-y-hold.case.json` follows the same rule with another seed and no shared
      point (120 cells). Its description says it is never used for fitting.
- [ ] Both `.oracle.json` sidecars are committed with no pending cells, and their math sidecars are generated with
      `cd tools && uv run rustsheet-accuracy math --suite probes-numeric` and committed.
- [ ] statistics.md's F density entry cites both cases in its Evidence. It gives counts against today's engine: cells
      bit-exact, within 4 ulp and within 15 digits. It also counts the power-of-two cells where Excel's F·x/2^-1074 is
      an integer.
- [ ] T-2647 owns the new cases' visibly-off cells (decision 0245), and `make check` is green (probes-numeric is
      ungated, so no eval/baseline line). Reviewers approve.

## Out of scope
- Any engine change or fit (T-2647).
- The huge-x side, where F is below 2^-1022 (statistics.md, already evidenced by t2586-r5-f-edges).
