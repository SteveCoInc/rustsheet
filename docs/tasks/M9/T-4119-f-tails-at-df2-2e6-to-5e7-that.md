---
id: T-4119
title: "F tails at df2 2E6 to 5E7 that T-3754's bup cap leaves on Cephes: widen large_shape_beta's b/a cut to its own bounds (decision 0267 accuracy)"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
F.DIST and F.DIST.RT at a huge df2 are accurate to the 50-digit math where T-3754's L140 bup+bgrat port does not
reach: df1 of 20 and more (more than 9 bup terms), and a0 above 1E7. Excel is within about 1E-13 of the math there
(statistics.md, the T-4017 sweep), so the target is the math, not Excel's bits (decision 0114 `accuracy`).

## Context
- Decision 0267 made our F accuracy at a huge df2 a P0 Tier A item. T-3754 (merged on main) raised bup+bgrat's
  a0 cap to 1E7, which fixed df1 4 to 19 up to df2 2E7.
- On main after T-3754, a 640-point grid against 60-digit mpmath still had errors up to 7.0E-10. All 94 points past
  5E-14 were at df1 40 to 79. These calls fell back to Cephes' continued fraction, because `large_shape_beta`
  refused b/a above 1E-6.
- Code: `large_shape_beta.rs` `large_shape_lower`, and `fisher.rs` `f_tail`.

## Visible-parity stage
Stage `accuracy`. Target: the engine's worst relative error against 60-digit math on the grid is under 2.5E-14.
Excel's own error there is 1E-14 to 1E-13, so a cell left visibly off Excel is Excel's noise, which decision 0267
counts as residue at 13 digits.

## Acceptance criteria
- [x] The grid and its before/after table are committed (docs/research/t4119). Worst error 7.0E-10 → 2.1E-14 (target
      2.5E-14), and points past 5E-14 94 → 0.
- [x] A unit test checks six t4017 tails against references at the double arguments, within 2.5E-14. It fails on
      main and passes here.
- [x] `rustsheet-eval check`: no gated cell changes, so no baseline line changes.
- [x] statistics.md records the gap T-3754 left and the cut's removal.
- [x] `make check` green; reviewers approve.

## Out of scope
- Matching Excel's own 1E-14 to 1E-13 error in these tails (decision 0267: Phase 1b residue).
- Excel's exact-zero tails (T-4045), its `#NUM!` at df1 77 to 79 (T-4044) and df1 = 2 (T-4053).

## Outcome
- **Fix.** `large_shape_beta` drops its b/a <= 1E-6 condition. The remaining bounds (a >= 1E6, b < 40) imply
  b/nu <= 4E-5, where the series converges in a few terms. The constant is gone; nothing else read it.
- **Grid** (docs/research/t4119): worst relative error 7.0E-10 → 2.1E-14, median 5.0E-16 → 3.4E-16.
  - The references are taken at the exact ratio d2/(d2 + d1·x). At 1E-14 that includes up to about 2E-15 of argument
    rounding (README).
  - Per df1 the worst point is now 2.0E-15 (df1 3) to 2.1E-14 (df1 79).
- **probes-numeric against main.** 54 t4017 cells change, 31 at df2 1E7 and 23 at 3E7.
  - 51 move toward Excel and none moves away. 5 of them come within 15 digits (`numeric_close` to `numeric_ulp`), and
    none becomes bit-exact.
  - The other 3 are cells where Excel is an exact 0 (T-4045): `F.DIST.RT(20,79,1E7)`, `F.DIST.RT(20,79,3E7)` and
    `F.DIST.RT(50,30,1E7)`. Their tails shift in about the 12th digit. For the first, the true value is
    9.98932061401838E-278, and ours goes from 5E-13 off it to 2.7E-15 off.
  - Counts are from the Excel-fidelity review's cell-by-cell comparison.
- **BETA.DIST check.** The widened region also serves BETA.DIST and the quantile searches. A 145-point BETA.DIST grid
  at non-half-integer b and a up to 3.9E7 (README) goes from a worst error of 4.9E-10 to 4.9E-15, and no point gets
  worse.
- **Gated suites.** No cell moves.
- **How the cut was chosen.** It is not fitted to Excel (R4): the expansion is extended to the bounds it already had,
  and the choice is checked only against 60-digit math. No fresh sweep was run, because the change only moves values
  toward the math. Gate A: every grid point was already inside its 1E-9 envelope.
- **Superseded branch.** agent/functions-stat/T-3754-f-huge-df2-accuracy did the same widening on top of the pre-T-3754
  engine. Its review asked for honest criteria, a pin with a stated bound, and a stated reference convention; this task
  takes all three up. That branch is superseded and no PR was opened from it.
