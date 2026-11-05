---
id: T-4000
title: "Incomplete gamma near the mode at shapes 21 to 999: POISSON(20,25,TRUE) and POISSON.DIST(100,100,TRUE) a few ulp off where Excel is within 1.5 (accuracy)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
The incomplete gamma near the mode (|x - a| within `NEAR_MODE`) at shapes 21 to 999 is within an ulp of the math, as
Excel's is, so POISSON.DIST's cdf there gives Excel's double or is within half a unit of its 15th digit.

## Context
- Follow-up of T-3286, which took Temme's expansion near the mode from shape 1E3 with its exponent a·phi in
  double-double (POISSON.xlsx D31 and D55 bit-exact). Below 1E3 the pair is still `rcomp` times the series or the
  continued fraction (`statistical/incomplete_gamma/mod.rs` `pair_with`), and the 4-term expansion is no help there:
  its first dropped term is about 3E-13 relative at shape 101.
- Cells, ulps against 60-digit mpmath `gammainc` (T-3286's table), Excel / ours:
  - ironcalc-calc STATISTICAL/POISSON.xlsx POISSON.DIST!D19 `POISSON.DIST(100,100,TRUE)`: Excel 0.5265621985299984
    (-0.5), ours 0.5265621985299986 (+1.5); a gated known failure.
  - probes-numeric/t1048-poisson Calc!A9, B9 `POISSON(20,25,TRUE)`, `POISSON.DIST(20,25,TRUE)` = Q(21, 25): Excel
    0.18549230269414177 (+1.5), ours 0.18549230269414194 (+7.5).
- Candidates: the series and Lentz fraction with the `rcomp` prefactor in double-double (the half-integer path,
  `incomplete_gamma/half_integer.rs`, does this for CHISQ.DIST.RT, T-3285), or more Temme terms (TOMS 654 takes the
  expansion from a few hundred with more coefficients). Measure each cell by cell on the gamma family's cells, as T-3286
  did; Excel is within 1.5 ulp here, so this is an accuracy kernel, never a fit.

## Acceptance criteria
- [ ] P and Q at shapes 21 and 101, x from 0.6a to 1.4a, within 1 ulp of 50-digit values (a unit test in
      `incomplete_gamma/tests.rs`, extending `near_mode_pair_is_within_an_ulp_from_shape_1e3`).
- [ ] POISSON.xlsx D19 and t1048-poisson A9, B9 within half a unit of Excel's 15th digit; no Excel-computed cell of the
      gamma family (POISSON, POISSON.DIST, GAMMA.DIST, GAMMADIST, CHISQ.DIST, CHISQ.DIST.RT, CHIDIST, GAMMA.INV,
      CHISQ.INV) goes from pass to fail, cell by cell against main.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Where Excel's ratio form starts away from the mode (T-3252, T-3270); the deep upper tail (T-3269).
