---
id: T-3862
title: "BETA.INV is #NUM! in Excel at a huge alpha and a tiny p where the engine returns 1-2^-53"
milestone: M9
lane: functions-stat
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
BETA.INV returns `#NUM!` where Excel does at a huge alpha (above about 1E+160) and a tiny p, instead of 0.9999999999999999.

## Context
- Gate A table accuracy/BETA.INV/s1 (T-3578, excel-graph 2026-11-02): 12 log-uniform points A180, A181, A187, A188,
  A190, A191, A192, A193, A194, A196, A197 and A198, for example `BETA.INV(1.67153110503488E-76,1.06481105633372E+183,
  1.82075780727763E-162)`. Excel is `#NUM!`; the math is exactly 1 (the true quantile rounds to 1) and the engine returns
  1 - 2^-53. They are known failures in eval/accuracy/known-failures.tsv owned by this task.
- statistics.md "BETA.INV at a huge shape is AS 109's Newton search (T-1538)" says the search converges near 1 at a
  huge first shape (shapes up to 5E9 and 1E10 there). These points have alpha from 5E+161 to 4E+296 and p from 1E-277
  to 8E-10: find where Excel's search fails (a density or ln B that overflows, a start that cannot move), probe the
  boundary in alpha and p with the excel-probe skill, and record the rule in statistics.md.
- Code: `fn_beta_inv` (crates/rustsheet-engine/src/functions/statistical/beta.rs, inverse_beta.rs).

## Acceptance criteria
- [x] The boundary of Excel's `#NUM!` is probed and recorded in statistics.md.
- [x] The 12 known-failure lines of accuracy/BETA.INV/s1 owned by T-3862 pass and are removed.
- [ ] No eval cell regresses; `make check` green; reviewers approve.

## Out of scope
- BETA.INV near 1 at a tiny second shape (T-3351) and the 0.5 at a DBL_MIN shape (T-2808).

## Probe predictions (committed before the fill)
Case probes/t3862-beta-inv-huge-alpha (row r: alpha = 1E+(109+r), r = 1..51, alpha 1E+110..1E+160). Hypotheses:
- P1 (a threshold in alpha alone): every BETA.INV column (A-F, J, K) turns from a number to `#NUM!` at the same row,
  with alpha between 1E+115 and 1E+145 (the Gate A cells bracket it at 5.18E+114 / 1.55E+145), whatever p and beta.
- P2 (the search's cdf fails): BETA.DIST's cdf at x = 0.5 (H, I) is `#NUM!` from the same row as BETA.INV.
- P3 (AS 109's flip): G, `BETA.INV(0.9,1E-100,alpha)`, is 1 minus column J's mirror and errors where the
  unflipped call does.
A refutation of P1 (a row that depends on p or beta) means the rule is not in alpha alone.

First fill of t3862-beta-inv-huge-alpha (rows 1, 6, ..., 51 of A-D; excel-graph, 2026-11-06): with beta 1E-100 or
0.5, Excel is a number up to alpha 1E+150 and `#NUM!` from 1E+155 (0.999999 at p = 1E-10, 1 - 1/e at p = 1E-100);
with beta 10 it is `#NUM!` from 1E+110 already. So P1 is refuted as stated (beta above 1 fails lower), and for
beta below 1 the boundary lies in (1E+150, 1E+155), which holds sqrt(DBL_MAX) = 1.3407807929942596E+154.

## Probe predictions, second case (committed before its fill)
Case probes/t3862-beta-inv-sqrt-dbl-max. Hypothesis Q: BETA.INV is `#NUM!` once fl(pp*pp) overflows, pp AS 109's
first shape after its flip (alpha for p <= 1/2, beta above).
- Q1: A, B, C (alpha in rows 1-6, beta 1E-100 or 0.5) and D (`BETA.INV(0.6,0.5,a)`, flipped so pp = a) are numbers in
  rows 1-3 and `#NUM!` in rows 4-6: the change is between the literals 1.34078079299425E+154 (a*a finite) and
  1.34078079299426E+154 (a*a = inf).
- Q2: E (`BETA.INV(0.1,0.5,a)`, a the second shape, not squared) is a number in all six rows (as accuracy/BETA.INV/s1
  A186 at beta 3.8E+157 is).
- Q3 (exploratory, beta above 1): G (beta 10, p 1E-10) and H (beta 2, p 0.1) turn to `#NUM!` from about alpha 1E+18,
  where AS 109's start pp/(pp + qq*exp(2w)) rounds to 1.

Second fill (excel-graph, 2026-11-06): Q1 and Q2 hold. A, B, C and the flipped D are numbers at
1.34078079299425E+154 and `#NUM!` at 1.34078079299426E+154; E is a number in all six rows. Q3 is refuted: H
(beta 2, p 0.1) is `#NUM!` from alpha 1E+16, where AS 109's start is still 0.9999999999999996 (G, beta 10, turns at
1E+18).

## Outcome
- Rule, derived and bisected to the double's own overflow: Excel's BETA.INV and BETAINV are `#NUM!` exactly where
  fl(pp*pp) overflows, pp AS 109's first shape after its flip (alpha for p <= 1/2, beta above), i.e.
  pp > sqrt(DBL_MAX) = 1.3407807929942596E+154. The 15-digit literals either side of sqrt(DBL_MAX), 7E-15 relative
  apart, bracket the change at three (p, beta) pairs and in the flipped orientation, and a huge second shape
  does not cut (E1:E6, Gate A BETA.INV/s1 A186), which rules out a cut in a·b, (a + b)^2 or the true quantile
  rounding to 1. AS 109 as revised (AS R19) squares pp in its accuracy exponent; which of Excel's expressions squares
  it is not established (recorded as UNVERIFIED). No constant was fitted: the cut is `(pp * pp).is_infinite()`.
- Code: functions/statistical/beta.rs `first_shape_square_overflows`, checked first in `beta_quantile` (NaN, which
  BETA.INV reports as `#NUM!`). Unit test `quantile_is_nan_once_the_square_of_as109_first_shape_overflows`.
- Gate A: all 12 BETA.INV/s1 lines and 12 of the 13 BETAINV/s1 lines owned here pass and are removed from
  eval/accuracy/known-failures.tsv (412 known failures, 0 failing). BETAINV/s1 A179 (alpha 1.55E+145, beta
  4.2E+128) is below the cut: re-pointed to T-4713.
- Probes: probes/t3862-beta-inv-sqrt-dbl-max (80 cells) and t3862-beta-inv-huge-alpha (44 cells), predictions
  committed before each fill. A fill with BETA.INV(0.1,alpha,1E+100) at alpha 1E+110 to 1E+160 never finished
  (createSession retries exhausted), so those columns were dropped.
- Follow-ups (open; T-4713 campaign: semantics, T-4700 campaign: visible): T-4713 (`#NUM!` with beta above 1 from alpha about 1E+16; the start-rounds-to-1 and
  quantile-rounds-to-1 rules refuted), T-4700 (Excel's 0.999999 and 1 - 1/e below the cut at a tiny beta, an unmoved
  AS 109 start; and E1:E6, ours 0.5% off at a huge second shape). They own the probes' off cells.
- statistics.md: new entry "BETA.INV is #NUM! once the square of AS 109's first shape overflows (T-3862)"; the
  Gate A table entry's fourth class points to it.
