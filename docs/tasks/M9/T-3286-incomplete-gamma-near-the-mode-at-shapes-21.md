---
id: T-3286
title: "Incomplete gamma near the mode at shapes 21 to 1E8: POISSON.xlsx D55 is 16 ulp off where Excel is correctly rounded (accuracy)"
milestone: M9
lane: functions-stat
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
campaign: accuracy
---

## Goal
The incomplete gamma near the mode (|x - a| within the engine's `NEAR_MODE` band) at a large shape is correctly rounded,
as Excel's is, so POISSON.DIST's and GAMMA.DIST's cdf there give Excel's double; first of all ironcalc-calc
POISSON.xlsx D55, which is visibly off.

## Visible-parity stage
`accuracy`: against 60-digit mpmath Excel is within half an ulp at every near-mode cdf cell below with a shape from 101
to 1E6, and ours is 1.5 to 16 ulp off, so the fix is a more accurate near-mode kernel, never a fit to Excel's bits.
Target: POISSON.xlsx D55 within half a unit of the 15th significant digit, with no cell of any suite regressing. The
bit-level cells below are this task's too: report how many become bit-exact.

## Context
- Gap triage 2026-10-30 (`make eval` on origin/main 73b187224). Q(k + 1, mean) and P(a, x), ulps of Excel's value against
  mpmath `gammainc` at 60 digits:

  | cell | formula | Excel | ours | Excel vs exact | ours vs exact |
  |---|---|---|---|---|---|
  | ironcalc-calc POISSON.xlsx POISSON.DIST!D55 | `POISSON.DIST(46040,46071,TRUE)` | 0.4438020698095389 | 0.4438020698095398 | -0.2 | +15.8 |
  | POISSON.xlsx POISSON.DIST!D31 | `POISSON.DIST(1000000,1000000,TRUE)` | 0.5002659614862837 | 0.5002659614862832 | +0.1 | -3.9 |
  | POISSON.xlsx POISSON.DIST!D19 | `POISSON.DIST(100,100,TRUE)` | 0.5265621985299984 | 0.5265621985299986 | -0.5 | +1.5 |
  | probes-numeric/t1048-poisson A9, B9 | `POISSON(20,25,TRUE)`, `POISSON.DIST(20,25,TRUE)` | 0.18549230269414177 | 0.18549230269414194 | +1.5 | +7.5 |
  
  D55 is visibly off; the others are within 15 digits (D19 and D31 are gated known failures in
  eval/baseline/ironcalc-calc.txt; `scripts/exit_owners.py --all-off` shows D55 with no open owner).
- The same loss just below Temme's shape, owned by T-3070 ("Temme's tails at the 1E8 cut, A144-A146"; gated in
  eval/baseline/probes.txt): probes/t2982-gamma-inv-huge-shape A144 `GAMMA.DIST(99990000,99999999,1,TRUE)` Excel
  0.15867945221382798 (-1.2 ulp from exact), ours 0.15867945221382823 (+7.8); A146 `GAMMA.DIST(99950000,99999999,1,TRUE)`
  Excel 2.8561234160188864E-07 (-13.6), ours 2.856123416018922E-07 (+53.4); A145 `GAMMA.DIST(99950000,1E8,1,TRUE)`
  (Temme's side) Excel +18.1, ours -0.9. A kernel that fixes D55 should move A144 and A146 toward Excel; report them in
  the Outcome and leave their ownership with T-3070.
- Path: `statistical/incomplete_gamma/mod.rs`. `gamma_ur` (POISSON.DIST's cdf) takes `Exponent::Accurate`, so a shape
  from 20 uses TOMS 654's `rcomp` prefactor and then the series or continued fraction near x = a; Temme's uniform
  expansion (`temme_pair`) starts only at `TEMME_SHAPE_MIN` = 1E8, a bound chosen for the series' term count (T-1891),
  not for accuracy. statistics.md "Incomplete gamma: accurate kernel (T-0971)" records that Excel near the mode
  "matches Temme's expansion" at t2982 A144, and that a log1p exponent loses about 1E-12 at shape 1E8 near the mode.
  Candidate remedies: Temme's expansion near the mode from a much smaller shape (TOMS 654 and Boost use it from a few
  hundred), or the series in double-double; test each on the cells above and on the passing near-mode cells.
- T-3252 (research) probes where Excel's ratio form starts at shapes 50 and 200; this task does not move
  `NEAR_MODE` or `RATIO_FORM_MIN`.

Cells this task owns:
- ironcalc-calc/STATISTICAL/POISSON.xlsx (3): POISSON.DIST!D19, D31, D55
- probes-numeric/t1048-poisson.case.json (2): Calc!A9, B9

## Acceptance criteria
- [ ] POISSON.xlsx D55 matches Excel to 15 significant digits; its known-failure line leaves
      eval/baseline/ironcalc-calc.txt if it becomes bit-exact, and no new line appears in any eval/baseline/ file.
- [ ] A unit test in `incomplete_gamma/tests.rs` checks P and Q within 1 ulp of 50-digit values at shapes 21, 101,
      1E3, 46041, 1E6 and 99999999 for x from 0.6a to 1.4a.
- [ ] No Excel-computed cell of POISSON, POISSON.DIST, GAMMA.DIST, GAMMADIST, CHISQ.DIST, CHISQ.DIST.RT, CHIDIST,
      GAMMA.INV or CHISQ.INV goes from pass to fail (main-vs-HEAD per cell); the Outcome gives the before/after count of
      bit-exact cells among the five owned above and t2982 A144-A146.
- [ ] docs/excel-semantics/statistics.md "Incomplete gamma: accurate kernel (T-0971)" records that Excel is correctly
      rounded near the mode at these shapes, citing the cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Where Excel's large-shape ratio form starts (T-3252, T-3270). The deep upper tail (T-3269). Shapes 1.5 and 2.5 (T-3285).

## Landing notes (resumed 2026-11-03 by an opus run)
- State (2026-11-03, session window closed): WIP on branch agent/functions-stat/T-3286-near-mode-temme, draft PR. Not
  reviewed. `make check-fast` not run; `cargo test -p rustsheet_engine --lib incomplete_gamma` passes with the
  measuring test below (it prints, it does not yet assert the 1-ulp bound).
- Change so far: `pair_with` takes `temme_pair` when `a >= TEMME_NEAR_MODE_MIN` (1E3) and |x - a| <= NEAR_MODE·a.
  `make eval-check` on it: ironcalc-calc 298 -> 296 (fixed 2, presumably POISSON.xlsx D55 and D31), probes 4354 ->
  4353 (fixed 1, presumably t2982 A144), 0 regressions in every suite. `a_call_past_its_cap_is_nan` was moved to shape
  999 (the series is one Temme term near the mode from 1E3 now). T-3985 (follow-up of T-3964) is filed on this branch.
- Measured (ulps from mpmath 60 digits, `near_mode_pair_is_within_an_ulp_from_shape_1e3`, new vs main): at the centre
  Temme wins (a=1000, x=1000: 0 vs 9; a=46041, x=a: 0 vs 6; x=1.01a: 0.97 vs 4.8; a=1E6, x=a: 0 vs 1), but in the
  band's tails both are tens to hundreds of ulps off and Temme is sometimes worse: a=1000 x=600 P 59 vs 17, x=1200 Q
  9.4 vs 2.9, x=1300 Q 13 vs 0, x=1400 Q 98 vs 82; a=46041 x=0.9a P 230 vs 157, 0.95a P 82 vs 52, 1.05a Q 59 vs 29.
  The tail loss looks like the rounding of a·phi in exp(-a·phi) (relative error about a·phi·eps), shared by both paths.
- Next step: restrict the Temme switch by a criterion derived from the error model, not fitted to these points (for
  example a·phi below a bound where erfc's and the correction's rounding dominate), or form a·phi in double-double so
  both tails improve; then assert the 1-ulp bound only where it holds, and say in the Outcome which acceptance points
  (shapes 21 and 101 are out of reach of the 4-term expansion: 3E-13 at 101) need a follow-up. mpmath's `gammainc`
  does not converge at shape 99999999 for most x (NoConvergence); the grid has only one such point.
- Do not: pick the a·phi cut by scoring these grid points without decision 0056's hold-out (it would be a FITTED cut).
- CI on 94e8eda2e: hygiene fails, `check_duplication.py` 2912 lines against baseline 2880 (+32, the size of the
  `NEAR_MODE_EXACT` table in incomplete_gamma/tests.rs): shrink that table (fewer points, or one row per shape with
  the x factors computed) rather than updating any baseline. Other jobs were green or pending at the window's close.
