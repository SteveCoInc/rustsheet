---
id: T-4497
title: "Oracle sweep: F.TEST and the df1 = 2 F quantiles where their unswapped a = 1 tail has x between 4E-4 and 0.1, to place A1_EXPM1_MAX_X"
milestone: M9
lane: research
status: done
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
- [x] A probes-numeric case (for example probes-numeric/t4497-f-a1-cut-sweep), oracle-filled by
      excel-graph, with:
      - at least 150 F.TEST calls whose a = 1 tail x is log-uniform over 4E-4..0.2. Use a
        3-value sample on the a side and data scaled to reach each target x (x <= a/(a+b) holds for
        F.TEST). Spread b over 1..50, and record the x of each call in the case description or a
        sidecar TSV;
      - at least 150 F.INV(p,2,df2) and F.INV.RT(p,df1,2) calls whose crossing x falls in the same
        range, df 1..50;
      - a dense block of at least 40 points around any cut the first 300 suggest (a second oracle
        round is fine).
- [x] 50-digit math values for every point (a `.math.tsv` sidecar or the Gate A math tooling), so
      T-4498 can apply the Gate A envelope and the R3 check.
- [x] A research note (docs/research/t4497/README.md) gives the engine's current score on the sweep
      (bit-exact, within 4 ulp, within 15 digits, visibly off) under four forms: the 0.1 cut, the cut
      the data suggests, libm kernels and Excel kernels. No engine change here.
- [x] The split for T-4498 is fixed before any candidate is scored: about 30% of the points by case
      or cell hash held out, with the margin stated (decision 0188 R4.1).
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing the constant or the kernels (T-4498).
- The oriented closed-form cut at y0 = 0.9 and BETA.DIST's own tails.

## Outcome
- Cases `probes-numeric/t4497-f-a1-cut-sweep` (480 cells: 160 each of F.TEST, F.INV(p,2,df2) and F.INV.RT(p,df1,2), a = 1
  tail x log-uniform 4E-4..0.2, b 0.5..50) and `t4497-f-a1-cut-dense` (round 2, 180 cells, x 0.095..0.105), filled by
  excel-graph. `docs/research/t4497/`: `mkcase.py` (cases, split), `points.tsv` (x, b, 50-digit math value and the
  hold-out flag per cell), `score.py` / `score.out`, README. The `.math.json` sidecars cover the F.INV cells (the Gate A
  tooling has no F.TEST or F.INV.RT definition); `points.tsv` has math for all 660.
- Split fixed before scoring: sha256(case!cell) mod 100 < 30, 194 of 660 held out, margin 5 points.
- Scores (train | hold-out, bit-exact): today's 0.1 cut with libm 146/466 | 59/194; the data's cut (plateau
  0.099798..0.10004, so 0.1) the same; 0.1 cut with Excel's expm1/log1p below 192/466 | 88/194; libm everywhere
  119 | 48; Excel's kernels everywhere 171 | 77.
- Findings: the cut lies in (0.0996048, 0.100089] on F.TEST, so 0.1 stands; below it F.TEST takes Excel's kernels
  (`excel_one_minus_exp` of b times `excel_ln_one_minus`): 220 of 220 F.TEST cells bit-exact (71/71 held out) against
  142 today. The df 2 quantiles do not place the cut (search-stop noise); above 0.1 they lean slightly against the
  cancelling form, for T-4498 to check. statistics.md's T-0974 entry updated.
- Eval delta: +660 probe cells (probes-numeric, ungated): 205 bit-exact, 9 visibly off, all F quantile cells, listed in
  T-3485 (amended). No engine change.
