---
id: T-4497
title: "Oracle sweep: F.TEST and the df1 = 2 F quantiles where their unswapped a = 1 tail has x between 4E-4 and 0.1, to place A1_EXPM1_MAX_X"
milestone: M9
lane: research
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Enough Excel points exist to place the cut where F.TEST's and the F quantile search's unswapped
a = 1 tail switches from the expm1/log1p form to the cancelling 1 - exp(b ln(1 - x)). They also
show which expm1 and log1p kernels sit below the cut. T-4498 fits on these points.

## Context
- Gap triage 2026-11-06 (`make eval` on origin/main 4bd55c6c1). No visibly-off cell is unowned, so
  this is an unprobed fitted constant. No open task covers it: T-3150 is the w1 below 2^-53, T-3764
  is b = 1, and T-2830 and T-4097 are F.TEST's last digits and its Gate A table.
- statistics.md "The a = 1 closed form cancels only from x = 0.1 up (T-0974)", Status: "VERIFIED
  (excel-graph) at x from 1E-11 to 4E-4 below the cut and at 0.1; the cut itself between 4E-4 and
  0.1 is UNVERIFIED (assumed T.DIST.RT's)." The engine constant is `A1_EXPM1_MAX_X = 0.1`
  (crates/rustsheet-engine/src/functions/statistical/incomplete_beta.rs), tagged
  `FITTED(T-0974, 11 cells, x 1E-11..4E-4 and 0.1)`.
- Where the cut is used: `unoriented_shape_one_tail` in fisher.rs, called by `f_test_upper_tail`
  (F.TEST and FTEST) and by `f_quantile_tail` (the crossing F.INV, F.INV.RT and FINV search: the
  lower tail at df1 = 2 and the upper at df2 = 2). The oriented BETA.DIST/F.DIST path
  (`shape_one_tails`) is tied to the separate closed-form cut at y0 = 0.9 (`A1_CLOSED_FORM_MAX_Y`,
  803 cells), so it is not in question here.
- Kernel question for the same sweep: below the cut `unoriented_shape_one_tail` uses libm's
  `ln_1p`/`exp_m1`. `shape_one_tails` uses Excel's own expm1 (Kahan's form) and log1p (the stored
  atanh series), which T-3170 found 71 eval cells closer for the oriented tails. Points below the
  cut separate the two kernels.
- Size of the effect: at x near 0.01 to 0.1 the cancelling form loses about eps/|b ln(1-x)|
  relative, roughly 1E-14 at b ln(1-x) = -0.02. That is near the half-unit of the 15th digit, so the
  cut can be visible as well as bitwise. It is not a defer case.

## Acceptance criteria
- [ ] A probes-numeric case (for example probes-numeric/t4497-f-a1-cut-sweep), oracle-filled by
      excel-graph, with:
      - at least 150 F.TEST calls whose a = 1 tail x is log-uniform over 4E-4..0.2. Use a
        3-value sample on the a side and data scaled to reach each target x (x <= a/(a+b) holds for
        F.TEST). Spread b over 1..50, and record the x of each call in the case description or a
        sidecar TSV;
      - at least 150 F.INV(p,2,df2) and F.INV.RT(p,df1,2) calls whose crossing x falls in the same
        range, df 1..50;
      - a dense block of at least 40 points around any cut the first 300 suggest (a second oracle
        round is fine).
- [ ] 50-digit math values for every point (a `.math.tsv` sidecar or the Gate A math tooling), so
      T-4498 can apply the Gate A envelope and the R3 check.
- [ ] A research note (docs/research/t4497/README.md) gives the engine's current score on the sweep
      (bit-exact, within 4 ulp, within 15 digits, visibly off) under four forms: the 0.1 cut, the cut
      the data suggests, libm kernels and Excel kernels. No engine change here.
- [ ] The split for T-4498 is fixed before any candidate is scored: about 30% of the points by case
      or cell hash held out, with the margin stated (decision 0188 R4.1).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the constant or the kernels (T-4498).
- The oriented closed-form cut at y0 = 0.9 and BETA.DIST's own tails.
