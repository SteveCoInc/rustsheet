---
id: T-3252
title: "Probe Excel's large-shape gamma prefactor window: near-mode bound, x below a/10, POISSON.DIST and large-shape BETA.DIST"
milestone: M9
lane: research
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence pins where Excel's large-shape gamma prefactor takes its ratio form (T-2638) and where it does not,
so the engine's bracketed bounds (`incomplete_gamma::NEAR_MODE` = 0.4, `RATIO_FORM_MIN` = 0.005) and the functions that
use the form are set by evidence.

## Context
T-2638 found Excel's prefactor from shape 20 away from the mode: sqrt(a/2pi) exp(L + (a - x) - correction(a)) with L
the log of the ratio that is at least 1 (docs/excel-semantics/statistics.md, T-2638 paragraph). The evidence covers
GAMMA.DIST and CHISQ.DIST densities and cdfs at shapes 25, 50 and 500 for x from a/10 to 4a. Bracketed only:
- Near the mode: Excel is accurate at x = 0.99a (`POISSON.DIST(100,100,TRUE)`) and at 1E6; it takes the ratio form at
  1.5a and a/10; both agree at a/2. The engine switches at |x - a| = 0.4a (TOMS 654's bound, not evidence).
- Below a/10: the form is Excel's at a/21 (gen-m9/8dd2312c96effb1b A333); at a/100 Excel is closer to it than to `rcomp` (t2753 A48, B48; gen-m9/68d8ffb69ff739d8 A341) and at a/500
  closer to `rcomp` (probes/t2753-gamma-density-ratio-sweep A47, B47); the engine keeps `rcomp` below a/200. T-3219 owns those cells' fit.
- POISSON.DIST cumulative (`gamma_ur`) and the large-shape BETA.DIST / BETA.INV / F.INV paths keep `rcomp`: no cell
  probes the ratio form for them.
Probe (excel-probe skill): GAMMA.DIST(x,50,1,FALSE) and (x,200,1,FALSE) at x/a = 0.55 to 0.99 and 1.01 to 1.45 in
steps of 0.05; x/a = 0.002 to 0.05; POISSON.DIST(k,λ,TRUE) for k from 19 to 200 with λ/k from 0.1 to 3; BETA.DIST with
b in [20, 40) and a/b at most 1E-6.

## Acceptance criteria
- [x] A probe case with oracle values lands, and statistics.md records the bounds it shows (or that the form does not
      apply), with the engine's constants and callers set to match (no regression).
- [x] `make check` green; reviewers approve.

## Out of scope
- The shapes below 20 (T-3219).

## Outcome
- Probes (excel-graph, 2026-11-04, M9): probes/t3252-gamma-prefactor-window (188 cells: GAMMA.DIST densities and
  cdfs at x/a 0.55 to 1.45 and 0.002 to 0.05, POISSON.DIST cumulative at k 19 to 200, BETA.DIST with b 20.5 to 39 and a
  1E-5 or 1E-6), with its Gate A math sidecar; probes/t3252-gamma-prefactor-holdout (42) and -holdout-2 (36).
- Finding: below the mode, Excel takes the ratio form (0 to 2 ulp) at every probed x <= 0.664a for shapes 30 to 500;
  from 0.668a to 0.99a, and above the mode from 1.05a to 1.45a, it is neither form (follow-up T-4255,
  campaign visible; the tiny-ratio cells A61:A70, A73:A80, A85:A93 are added to T-3219). POISSON.DIST's cdf is closer to `rcomp` near and above the mean; BETA.DIST's
  large-shape cells do not depend on the form. The tiny-ratio switch lies in (0.002a, 0.003a), outside the current
  1/200 (`RATIO_FORM_MIN`); that fit stays with T-3219. statistics.md has the rules with cells.
- Engine: `gamma_prefactor` takes the ratio form below the mode from (a - x) > 0.334a (`NEAR_MODE_BELOW`, new) at
  shapes 30 to 500 (`NEAR_MODE_BELOW_SHAPES`, the evidenced range; R4.3), and from 0.4a elsewhere as before; above the
  mode `NEAR_MODE` 0.4 stays. Decision 0281 records the fitted lock 2 to 3.
- R4 guards (round 2, after the Excel-fidelity review). Freezes: 0.3 failed -holdout (0.68a: 86 to 208 ulp on the
  ratio form against 43 on `rcomp`); 0.33 lay outside -holdout-2's bracket (0.332, 0.336). So -holdout and -holdout-2
  are training data now. 0.334, the bracket's middle, was frozen at becb4501c before the fresh sweep
  probes/t3252-gamma-prefactor-sweep was written: 40 points, shape log-uniform in [30, 500], x/a uniform in
  [0.55, 0.75], seed 3252, density and cdf. Margin stated before scoring: the fit must beat correctly rounded at
  every level on the sweep. Scores (bit-exact / within 4 ulp / within 15 digits; 50-digit mpmath for correctly
  rounded):

  | case | ours | correctly rounded |
  |---|---|---|
  | window, GAMMA.DIST (96) | 17 / 47 / 57 | 9 / 31 / 44 |
  | holdout (42, train) | 6 / 19 / 27 | 0 / 6 / 15 |
  | holdout-2 (36, train) | 7 / 14 / 20 | 0 / 7 / 10 |
  | sweep (80, fresh) | 22 / 61 / 69 | 3 / 15 / 35 |

  The sweep's exact rate (28%) is above training's (17 to 19%). FITTED tag and range are on the constant, and there
  is no keep-matched branch. `rustsheet-accuracy math` writes no Gate A sidecar for the hold-out and sweep cases (0
  candidate cells), so the envelope count is from the table above, not from Gate A.
- Eval delta: in the window case 74 of 188 cells pass, against 69 on main's engine. Each newly passing cell was 9 to
  72 ulp off on `rcomp` (A2, A3, A22, A23, A42, A43). No regressions. The three cases add 179 known failures
  (77 visibly off), owned by T-4255 and T-3219.
