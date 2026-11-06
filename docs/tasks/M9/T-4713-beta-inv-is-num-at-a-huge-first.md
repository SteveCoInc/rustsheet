---
id: T-4713
title: "BETA.INV is #NUM! at a huge first shape with a second shape above 1 from about 1E+16 (T-3862 split)"
milestone: M9
lane: functions-stat
status: todo
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
- [ ] The boundary in alpha is bisected at two or more betas above 1 and p values, and the rule (a quantity of
  AS 109 or Excel's cdf that overflows, underflows or cannot move) is derived and recorded in statistics.md; or the
  negative result is recorded.
- [ ] The probe cells above and BETAINV/s1 A179 pass where the rule covers them.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The sqrt(DBL_MAX) cut (T-3862) and the values below it at a tiny beta (T-4700).
