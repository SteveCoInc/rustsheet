---
id: T-4502
title: "HYPGEOM.DIST and HYPGEOMDIST pmf is #NUM! past the walk where Excel computes it, and at an empty population"
milestone: M9
lane: functions-stat
status: done
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
- [x] All 76 T-4502 lines are gone from eval/accuracy/known-failures.tsv, accuracy/HYPGEOM.DIST/s1!Calc!A95 included,
      and `rustsheet-eval accuracy` reports 0 failing.
      - At A95 the bug is the engine's `#NUM!`. Once the engine computes the pmf, its 8.1E-186 is within the envelope of
        the math, so the point passes Gate A on the math side and its line is removed like the others.
      - Excel's 0 there stays Excel's value (decision 0300).
- [x] Unit tests cover the empty population and a pmf past the walk.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Reproducing Excel's 0 at `HYPGEOM.DIST(41966,46221,65797,74726,FALSE)` (an intermediate underflow of Excel's,
  unpinned): that needs Excel's formulation (T-2481). Removing A95's known-failure line is in scope (see above).
- The cdf's per-call cap DEVIATION (T-1896).

## Outcome
- **Pmf past the walk** (hypegeom.rs `ln_pmf`): R nmath's `dhyper` in log space, the sum ln b(x; K, p) +
  ln b(n - x; N - K, p) less ln b(n; N, p) with p = n / N, each term Loader's saddle point (new loader.rs
  `loader_ln_binomial`, R's `dbinom_raw`, on the existing exact `stirlerr_exact` and a new accurate `loader_bd0`, R's
  `bd0.c`). The means m p are double-doubles and enter bd0 through its exact first-order term, so p's rounding does not
  reach the result. Not Excel's `excel_bd0`, an x87 model FITTED to BETA.DIST and T.DIST cells, which check 7 keeps
  inside its range.
- **Cdf past the walk** (hypegeom.rs `sum_of_terms`): the skip and the per-call cap still read statrs' exponents, so the
  same calls are capped and T-1873's DEVIATION and the lifting weights are unchanged. The terms left are the largest
  one of the window in the log form above times the sum of the others relative to it, by the exact ratio of consecutive
  terms in double-double (the walk's step, about 19 ns, cheaper than statrs' term), each side stopping once a term is
  below 2^-110 of the sum; clamped at 1.
- **Empty population**: `number_pop > 0` dropped from the validity check; `HYPGEOMDIST(0,0,0,0)` and
  `HYPGEOM.DIST(0,0,0,0,FALSE)` are 1 through the existing min(n, K) = 0 rule. Counts above the population stay `#NUM!`.
- **Not a fit** (reviewer-excel-fidelity check 7, kind (b)): every constant is copied verbatim from R nmath (`bd0.c`'s
  0.1 cut and 1000-term bound, `dbinom.c`'s p < 0.1 and q < 0.1 branches) or is the exact ratio recurrence; nothing was
  scored against oracle cells, no `FITTED` piece, no hold-out owed. Results inside the walk are unchanged (T-2481).
- **Gate A**: all 76 T-4502 lines removed from eval/accuracy/known-failures.tsv (66 pmf, 4 empty population, 6 cdf);
  `rustsheet-eval accuracy`: 42827 points, 0 failing, 370 known failures. The pmf is within 1.3E-13 relative of the
  50-digit math on the nine table points of unit test `the_pmf_past_the_walk_is_the_math`, the cdf within 1E-12 on the
  six of `the_cdf_past_the_walk_is_the_math` (A164 was 1.2E-6 off). A95 is the math's 8.1E-186 (Excel 0, decision 0300).
- **Eval**: `make eval-baseline` 0 regressions, 0 fixed, no baseline change. No suite cell (gen-m9, probes,
  probes-numeric, ironcalc-calc) reaches the fallback: a temporary trace over a full `rustsheet-eval run` logged 0
  fallback calls (78 in the accuracy tables). Unit test `a_cdf_sum_is_bounded`
  (`HYPGEOM.DIST(25000000,50000000,50000000,100000000,TRUE)`) moves from statrs' 0.500079709 to the math's 0.500079788
  (1/2 + pmf(x)/2 by symmetry).
- statistics.md: the Ours lines of "HYPGEOM.DIST of an empty population is 1 ..." and "HYPGEOM.DIST is not exact ...",
  and the per-call cap entry, updated; the unprobed empty-population shapes (`HYPGEOM.DIST(0,0,0,0,TRUE)`,
  `HYPGEOMDIST(1,0,0,0)`, `HYPGEOMDIST(0,0,1,0)`) are recorded with what the general rules give.
