---
id: T-3998
title: "Incomplete gamma near the mode at shapes 21 to 999: more Temme terms for POISSON.xlsx D19 and t1048 A9, B9 (accuracy)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
The incomplete gamma near the mode (within 0.4a of x = a) at shapes 21 to 999 is within an ulp of 60-digit values,
as Excel's is, so POISSON.DIST's and GAMMA.DIST's cdf there give Excel's double; first POISSON.xlsx D19 and
probes-numeric/t1048-poisson A9, B9. Also lift P at shape 1E3 from 0.8a down to within a double (T-3286 left it two
doubles low).

## Context
- Follow-up of T-3286, which took Temme's uniform expansion (`temme_pair`, 4 coefficient rows `TEMME_C`) with a phi in
  double-double (`temme_exponent`) near the mode from shape 1E3 (`TEMME_NEAR_MODE_MIN`). Below 1E3 the 4 rows are not
  enough: the first dropped term is about 3E-13 relative at shape 101; at shape 1E3, P from 0.8a down is two doubles
  low from the expansion's truncation (2.6 eps at 0.6a, checked in 60-digit arithmetic with the same rows).
- Cells (Excel vs 60-digit mpmath; ours on main after T-3286):
  ironcalc-calc POISSON.xlsx D19 `POISSON.DIST(100,100,TRUE)` Excel 0.5265621985299984 (-0.5 ulp), ours
  0.5265621985299986 (+1.5); t1048-poisson A9, B9 `POISSON(20,25,TRUE)` Excel 0.18549230269414177 (+1.5), ours
  0.18549230269414194 (+7.5). statistics.md "Incomplete gamma: accurate kernel (T-0971)" has the evidence.
- Candidate remedy: more rows of DiDonato and Morris's `C_k(z)` coefficients (TOMS 654 and Boost's 53-bit
  `igamma_temme_large` carry 10, and use the expansion from shape 20 within a band). Copy the constants from a cited
  source and check them end to end against mpmath; do not fit. Or a double-double series near the mode.
- Path: `statistical/incomplete_gamma/mod.rs` (`pair_with`, `temme_pair`, `TEMME_C`); test
  `near_mode_pair_is_within_an_ulp_from_shape_1e3` in tests.rs.

Cells this task owns:
- ironcalc-calc/STATISTICAL/POISSON.xlsx (1): POISSON.DIST!D19
- probes-numeric/t1048-poisson.case.json (2): Calc!A9, B9

## Acceptance criteria
- [ ] D19, A9 and B9 match Excel to 15 significant digits; report which become bit-exact; no new line in any
      eval/baseline/ file.
- [ ] A unit test checks P and Q within a double of rounded 60-digit values at shapes 21, 101 and 1E3 for x from 0.6a
      to 1.4a (no two-double exception at 1E3).
- [ ] No Excel-computed cell of POISSON, POISSON.DIST, GAMMA.DIST, GAMMADIST, CHISQ.DIST, CHISQ.DIST.RT, CHIDIST,
      GAMMA.INV or CHISQ.INV goes from pass to fail.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Where Excel's large-shape ratio form starts (T-3252, T-3270); the deep upper tail (T-3269).
