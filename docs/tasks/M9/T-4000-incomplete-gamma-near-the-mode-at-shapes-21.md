---
id: T-4000
title: "Incomplete gamma near the mode at shapes 21 to 999: POISSON(20,25,TRUE) and POISSON.DIST(100,100,TRUE) a few ulp off where Excel is within 1.5 (accuracy)"
milestone: M9
lane: functions-stat
status: done
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
- [x] P and Q at shapes 21 and 101, x from 0.6a to 1.4a, within 1 ulp of 50-digit values (a unit test in
      `incomplete_gamma/tests.rs`, extending `near_mode_pair_is_within_an_ulp_from_shape_1e3`).
- [x] POISSON.xlsx D19 and t1048-poisson A9, B9 within half a unit of Excel's 15th digit; no Excel-computed cell of the
      gamma family (POISSON, POISSON.DIST, GAMMA.DIST, GAMMADIST, CHISQ.DIST, CHISQ.DIST.RT, CHIDIST, GAMMA.INV,
      CHISQ.INV) goes from pass to fail, cell by cell against main.
- [x] `make check` green; reviewers approve.

## Out of scope
- Where Excel's ratio form starts away from the mode (T-3252, T-3270); the deep upper tail (T-3269).

## Outcome
- New `incomplete_gamma/near_mode.rs`: within `NEAR_MODE` at shapes 21 to 1E3, the pair is computed in double-double
  and rounded once.
  - The prefactor is exp(ln(a)/2 - ln(2 pi)/2 - a phi - mu(a)), with Temme's phi and `stirlerr_exact`.
  - Below a + 1 it takes the series; from a + 1 on, the backward fraction. `fraction.rs`'s backward fraction now takes
    any shape, and stays bit-identical at the half-integer shapes, whose low words are 0.
- It runs for `Exponent::Accurate` only: POISSON's cdf (`gamma_ur`) and the quantiles.
  - On GAMMA.DIST and CHISQ.DIST it made 7 cells visibly off, because Excel is 2 to 35 ulp from the truth there
    (T-3252's window, and below the mode).
  - Shape 20 stays on the double kernel. Excel's Q(20, 20.9) is the double kernel's bits, 2.3 ulp below the truth (gated
    probes t3252-gamma-prefactor-window A103).
- Unit test `near_mode_pair_is_within_an_ulp_from_shape_21`: P and Q at shapes 21 and 101, x from 0.6a to 1.4a, within
  1 ulp of 60-digit mpmath. It also checks D19 and Q(21, 25).
- Eval delta: 5 known failures fixed, all bit-exact now:
  - ironcalc-calc POISSON.xlsx D19;
  - gen-m9 bc40d37cb4d8301d A371 and ede0323c389fc703 A371;
  - probes t3252-gamma-prefactor-window A113, A114.
  No regressions. t1048-poisson A9/B9 are within w15, 1.5 ulp from Excel at the correctly rounded value.
- Cell by cell against origin/main 70d01bb1 (`docs/research/t3007/cell_diff.py`):
  - 14 cells move: 10 closer to Excel, 4 further, no verdict changes.
  - The 4: t3252 window A126 and A137, where Excel is 3.5 and 6.0 ulp from the truth; m9-legacy-chiinv A39/A40, 1 ulp
    on either side.
- Also carries T-4070's two RESULTS.md figure fixes (reviewer MINORs on #3289).
