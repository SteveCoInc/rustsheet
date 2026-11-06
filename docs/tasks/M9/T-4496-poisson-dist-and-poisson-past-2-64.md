---
id: T-4496
title: "POISSON.DIST and POISSON at a count past 2^53 and 2^64: probe the pmf, a negative mean and where the cdf stops computing, and match"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Excel evidence fixes what POISSON.DIST and POISSON return at a huge count x, in four places: the pmf,
a negative mean, where the cdf turns into its limit 1, and what happens between 2^53 and 2^64. The
engine matches every new cell. The note "UNVERIFIED: the pmf there (ours `#NUM!`)" in statistics.md
becomes evidence.

## Context
- Gap triage 2026-11-06 (`make eval` on origin/main 4bd55c6c1). No visibly-off cell is unowned, so
  this comes from the semantics notes. No open task probes this: T-2752 covers POISSON's EXP overflow
  at x 0 with a very negative mean, and T-3295 covers the saddle point at large k with an ordinary
  mean.
- statistics.md, the T-2475 subexpression entry (near "POISSON.DIST(2E+29,7,-1)"):
  `POISSON.DIST(2E+29,7,-1)` is 1 (probes/t2475-stat-subexpressions P!A216, gen-m9
  5919660b7bb4d195 A88), "a cumulative count past 2^64 is not rejected. UNVERIFIED: the pmf there
  (ours `#NUM!`)." The same entry's Status says "one cell each; the boundaries are not probed".
- Engine (`fn_poisson_dist` in crates/rustsheet-engine/src/functions/statistical/poisson.rs): for
  x > u64::MAX it returns 1 when cumulative with a mean >= 0, and `#NUM!` otherwise, including the pmf
  and any negative mean. Below that, k = trunc(x) as u64. The pmf is the saddle point
  (`excel_poisson_pmf`, through f64 k, which rounds above 2^53), and its `excel_bd0` overflows to
  `#NUM!` when k / mean does. The cdf is `poisson_upper(k + 1, mean)`. The 2^64 cut is the engine's
  integer type, not evidence.
- Questions for the probe:
  - The pmf at x = 2E+29, 1E+20, 2^64 and 2^64 times (1 ± 2^-52), against means of 1E-300, 7,
    1E+15, 1E+20 and 1E+29. Is it 0, `#NUM!`, or a value when the mean is about x?
  - The cdf over the same grid. Where does a huge x stop computing Q(k+1, mean) and give 1? Excel may
    cut at 2^31, 2^53, 2^64 or 1E+15, or never cut and compute.
  - A negative mean at x past 2^64: is it `#NUM!` (ours), or EXP(-mean) as at x 0 (statistics.md
    "POISSON at x 0 with a negative mean")?
  - Fractional and boolean-adjacent forms of the same x (for example 2^53 + 0.5 is not
    representable, so use 2^52 + 0.5) so that the truncation is visible.
  - POISSON (the legacy name) on every row. Earlier evidence shows it is the same kernel, but this
    region has no cell for it.
- Write it with the excel-probe skill (probes/t4496-poisson-huge-count). Bisect whichever cut appears
  over at least two oracle rounds until adjacent doubles bracket it. Commit the engine's predictions
  before each fill.

## Acceptance criteria
- [ ] A probes case of at least 60 cells over the grid above, POISSON.DIST and POISSON, cumulative
      and not, filled by excel-graph. Each cut that appears is bracketed by adjacent doubles, or by
      the finest bracket two bisection rounds give, stated in the case description.
- [ ] The engine matches the type (error kind included) of every new cell, and the value to Excel's
      15 digits wherever Excel returns a number. If a cut is placed on these cells, it carries
      `FITTED(T-4496, n_cells, range)` unless it is an exact IEEE or integer-type boundary that
      statistics.md proves (decision 0188 R4: a boundary at 2^53 or 2^64 is kind (b), with the
      argument written down).
- [ ] No POISSON or POISSON.DIST cell in any suite regresses (`make eval`). Known failures in
      eval/baseline/gen-m9.txt and probes.txt only shrink.
- [ ] statistics.md: the T-2475 entry's POISSON.DIST UNVERIFIED note is replaced by a rule citing
      the new cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- POISSON's EXP overflow at x 0 (T-2752) and the saddle-point bits at an ordinary large k (T-3295,
  T-3288 sweep).
- The normal family's huge-z cdf, which statistics.md already verifies at 1 or 0 (probes/t2435-dist-extremes).
