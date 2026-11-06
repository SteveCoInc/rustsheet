---
id: T-4502
title: "HYPGEOM.DIST and HYPGEOMDIST pmf is #NUM! past the walk where Excel computes it, and at an empty population"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
HYPGEOM.DIST and HYPGEOMDIST give Excel's value on every point of their Gate A tables: the pmf is a number past the
double-double walk, the empty population is 1, and the cdf past the walk is within the Gate A envelope.

## Context
- T-4105's tables (eval/accuracy/tables/HYPGEOM.DIST/s1, HYPGEOMDIST/s1, excel-graph 2026-11-06) carry 76 known failures
  owned here (eval/accuracy/known-failures.tsv):
  - 66 pmf points from a population of about 2E4 up (`HYPGEOMDIST(4478,5613,21568,27873)`, A199): past the walk
    (hypegeom.rs `hypergeometric_pmf`, 2^12 steps) the fallback is statrs' `Hypergeometric::pmf`,
    `binomial(K, x) * binomial(N - K, n - x) / binomial(N, n)`, which is inf / inf = NaN, so `#NUM!`. Excel's value is the
    math to within 1E-10 relative; at A95 of HYPGEOM.DIST it is 0 (statistics.md "HYPGEOM.DIST of an empty population is
    1, and a deep-tail pmf can be 0 (T-4105)").
  - 4 points at the empty population, `HYPGEOMDIST(0,0,0,0)` and `HYPGEOM.DIST(0,0,0,0,FALSE)`: Excel 1, ours `#NUM!`
    (`number_pop > 0.0` in `hyp_geom_dist_of`).
  - 6 cdf points past the walk (`HYPGEOM.DIST(43,976865,29261,643257045,TRUE)`, A164, 1.2E-6 relative): statrs'
    ln-binomial sum is off where Excel matches the math.
- A log-space pmf (ln-gamma or Loader's saddle point, as R's `dhyper`) avoids the overflow; Excel's own formulation is
  open (statistics.md "HYPGEOM.DIST is not exact for small integer arguments", T-2481), so the target is the Gate A
  envelope, not Excel's last digits.

## Acceptance criteria
- [ ] All 76 T-4502 lines are gone from eval/accuracy/known-failures.tsv, accuracy/HYPGEOM.DIST/s1!Calc!A95 included,
      and `rustsheet-eval accuracy` reports 0 failing.
      - At A95 the bug is the engine's `#NUM!`. Once the engine computes the pmf, its 8.1E-186 is within the envelope of
        the math, so the point passes Gate A on the math side and its line is removed like the others.
      - Excel's 0 there stays Excel's value (decision 0300).
- [ ] Unit tests cover the empty population and a pmf past the walk.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Reproducing Excel's 0 at `HYPGEOM.DIST(41966,46221,65797,74726,FALSE)` (an intermediate underflow of Excel's,
  unpinned): that needs Excel's formulation (T-2481). Removing A95's known-failure line is in scope (see above).
- The cdf's per-call cap DEVIATION (T-1896).
