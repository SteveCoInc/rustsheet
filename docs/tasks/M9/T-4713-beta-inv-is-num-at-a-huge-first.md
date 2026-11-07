---
id: T-4713
title: "BETA.INV is #NUM! at a huge first shape with a second shape above 1 from about 1E+16 (T-3862 split)"
milestone: M9
lane: functions-stat
status: done
priority: P3
size: M
tier: opus
depends_on: []
campaign: semantics
---

## Goal
BETA.INV and BETAINV return `#NUM!` where Excel does at a huge first shape with a second shape above 1, below the
sqrt(DBL_MAX) cut T-3862 implemented, by a rule derived from Excel's search (owner rule: no fitted cut).

## Context
- T-3862 found that Excel is `#NUM!` once AS 109's first shape squared overflows (pp > 1.3407807929942596E+154), but
  with beta above 1 it fails much lower: probes/t3862-beta-inv-sqrt-dbl-max G1:G25 `BETA.INV(1E-10,1E+k,10)` is
  0.9999999999999956 at k = 16 and `#NUM!` from k = 18; H1:H25 `BETA.INV(0.1,1E+k,2)` is 0.9999999999999611 at
  k = 14 and `#NUM!` from k = 16; t3862-beta-inv-huge-alpha D1:D11 (beta 10, alpha 1E+110 to 1E+160) is `#NUM!`
  throughout (D1:D9 are below the cut and off; D10:D11 pass by it). Gate A: accuracy/BETAINV/s1 A179 (alpha 1.55E+145, beta 4.23E+128) is `#NUM!`; the line in
  eval/accuracy/known-failures.tsv is owned here.
- Refuted (T-3862): "AS 109's start pp/(pp + qq exp(2w)) rounds to 1" (it is 0.9999999999999996 at H4, which is
  `#NUM!`); "the true quantile rounds to 1" (H4's is about 1 - 3.9E-16, representable).
- Ours: a number (the crossing, often 1 - 2^-53) at every one of these cells.
- docs/excel-semantics/statistics.md "BETA.INV is #NUM! once the square of AS 109's first shape overflows (T-3862)".
- Code: functions/statistical/beta.rs `beta_quantile`, inverse_beta.rs `beta_quantile_crossing`.

## Visible-parity stage
Owner of the visibly-off cells of probes/t3862-beta-inv-huge-alpha D1:D9, probes/t3862-beta-inv-sqrt-dbl-max G5:G25 and
H4:H25, and Gate A BETAINV/s1 A179: Excel `#NUM!` where ours is a number (an error against a number, `semantics`).

## Acceptance criteria
- [x] The boundary in alpha is bisected at two or more betas above 1 and p values, and the rule (a quantity of
  AS 109 or Excel's cdf that overflows, underflows or cannot move) is derived and recorded in statistics.md; or the
  negative result is recorded.
- [x] The probe cells above and BETAINV/s1 A179 pass where the rule covers them.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The sqrt(DBL_MAX) cut (T-3862) and the values below it at a tiny beta (T-4700).

## Probe 1 (predictions, committed before the fill)
probes/t4713-beta-inv-beta-above-1-cut: alpha bisected at 40 points a decade in columns A to D (four (p, beta) pairs,
each spanning a quantile 1 - x of about 4E-15 to 4E-16), E the flipped p = 0.9, F beta = 1.
- Hypothesis Q (the quantile): A to D are `#NUM!` exactly where the true 1 - x falls below one threshold T, the same
  T in every column, somewhere in (3.9E-16, 4.5E-15). AS 109's start is within 15% of the quantile in 1 - x here, so a
  start rule would put each column's cut at nearly the same 1 - x; telling them apart needs the cut row to differ.
- E: flipped (pp = beta = 2, qq = alpha): numbers in every row if the cut belongs to the unflipped first shape.
- F: beta = 1 is a closed form (x = p^(1/alpha)), predicted a number in every row.

## Probe 1 result, probe 2 (predictions, committed before the fill)
Probe 1 refutes hypothesis Q: A38:A40 (alpha 8.4E+15 to 9.4E+15) are 1 - 2^-51, the same double as AS 109's start at
A41 (alpha 1E+16), which is `#NUM!`; B (beta 10) is a number up to 1E+17 with 1 - x = 4.4E-16; C (beta 3) a number to
1E+16 with 1 - x = 2.2E-16. Excel's value is AS 109's unmoved start, bit for bit, on every row of A and C and most of D;
in B it is often the moved root. E (flipped) and F (beta 1) are numbers throughout (F is 0.999999 from 5.6E+16).
Probe 2 (probes/t4713-beta-inv-cut-pieces): extends A to D above their last numbers, and at alpha 7.9E+15 to 2E+16
reads BETA.DIST's cdf and density at the start 1 - 2^-51 and GAMMALN(alpha), GAMMALN(alpha + 2).
- Hypothesis K (the kernel): BETA.INV is `#NUM!` exactly where BETA.DIST's cdf or density at the iterate is `#NUM!`,
  so K is `#NUM!` exactly on the rows where G or H is.
- Hypothesis L (ln B): Excel's ln B = GAMMALN(a) + GAMMALN(b) - GAMMALN(a + b) is a multiple of the ulp of
  GAMMALN(a) (64 at 1E+16); `#NUM!` follows a value of J - I (0 or very large), so K's `#NUM!` rows match a J - I pattern.

## Probe 2 result
Hypothesis K holds row for row: K1:K41 (`BETA.INV(0.1,a,2)`) is `#NUM!` on exactly the 30 rows where H
(`BETA.DIST(1-2^-51,a,2,FALSE)`) is, and a number on the other 11. Hypothesis L (ln B from GAMMALN) does not separate
them (J - I is 64 on both kinds of row). The density's `#NUM!` rows are exactly where Loader's count
nk = fl(fl(a + b) - 2) - fl(a - 1) is 0: our density already gave Excel's own odd values on the other rows (H9
1.236E+15, nk = 2) and 0 on these.

## Outcome
- Rule (statistics.md "BETA.INV and BETA.DIST's density are #NUM! where Loader's n - k is not positive (T-4713)"):
  with both shapes above 1, Excel's Loader density is `#NUM!` at every x where nk <= 0, and BETA.INV/BETAINV are
  `#NUM!` at every p on those shapes after AS 109's flip. It predicts all 469 BETA.INV cells with both shapes above 1
  (below the sqrt(DBL_MAX) cut) of the four probes and all 41 densities; it is not a fit (no constant, no cut chosen).
- Code: loader.rs `beta_density_counts` (k, n, nk, now shared with `excel_beta_density`) and
  `beta_density_counts_vanish`; beta.rs's density returns `#NUM!` there, and `beta_quantile` returns NaN
  (`newton_density_vanishes`, flipped shapes for p > 1/2). Unit tests in both modules.
- Cells: probes/t3862-beta-inv-huge-alpha D1:D9, t3862-beta-inv-sqrt-dbl-max G5:G25 and H4:H25 now pass; Gate A
  accuracy/BETAINV/s1 A179 passes and its known-failures.tsv line is removed. New probes: t4713-beta-inv-beta-above-1-cut
  (222 cells) and t4713-beta-inv-cut-pieces (369 cells, 313 pass, the rest 15-digit-close cdf cells and ulp-level
  quantiles). Their visibly-off cells E41 and F12:F17 (beta 1's 0.999999 cap, the flipped quantile at 1) go to T-4958.
- In passing (recorded, not implemented): Excel's value here is often AS 109's unmoved start, bit for bit.
