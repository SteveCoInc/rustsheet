---
id: T-3577
title: "BETA.DIST cdf is 0 at tiny shapes where Excel and the math are positive, and 0 where Excel is #NUM! at a huge beta"
milestone: M9
lane: functions-stat
status: done
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
BETA.DIST's cdf returns Excel's value (the math) at tiny shapes where the engine returns 0, and `#NUM!` where Excel
does at a huge beta, so the four Gate A known failures of accuracy/BETA.DIST/s1 pass.

## Context
Found by T-3529's Gate A table (excel-graph, 50-digit mpmath):
- A184 `BETA.DIST(2.65510449055274E-229,1.22900757873089E-163,2.29413050210964E-185,TRUE)`: Excel and the math
  1.8666528521155483E-22, engine 0.
- A187 `BETA.DIST(2.08276476656038E-168,6.75648251367539E-92,5.68983967596267E-231,TRUE)`: Excel and the math
  8.421304524131021E-140, engine 0.
- A193 `BETA.DIST(8.14458557003028E-43,1.60715427756387E-36,2.89371027189731E-284,TRUE)`: Excel and the math
  1.8005180412944583E-248, engine 0.
- A186 `BETA.DIST(1.16481965482714E-239,11670.8736712835,1.83368089638147E+214,TRUE)`: Excel `#NUM!`, the math and
  the engine 0 (statistics.md "Incomplete beta at a shape near the double range ... bpser is #NUM! once 1/B
  overflows (T-3007)" may be the rule).
At both shapes tiny, I_x(a, b) is about b/(a + b) · (x^a) for x far below 1, which is positive and not 0. The engine's
kernel is `functions/statistical/bratio` (TOMS 708).

## Acceptance criteria
- [x] The four points pass `rustsheet-eval accuracy`, and their lines leave `eval/accuracy/known-failures.tsv`.
- [ ] (moved to T-4733) The `#NUM!` rule is cited in docs/excel-semantics (a probe if A186 alone does not settle it).
- [ ] `make check` green; reviewers approve.

## Out of scope
- BETA.INV (T-3578).

## Outcome
- The eleven tiny-shape points are bratio's Tiny branch (both shapes below eps/1000), not fpser/apser. The engine had
  not ported that branch, so it fell to the Cephes kernel, which gave 0. `bratio::ported_tails_with` now returns TOMS
  708's `w = b/(a + b)`, `w1 = a/(a + b)`, which is Excel's value to the bit on all eleven Gate A points across three
  batches: BETA.DIST/s1 A184, A187, A193; BETADIST/s1 A183, A187, A193, A198; BETA.DIST/r4-T-4437 A185, A189, A192,
  A194. This is a verbatim branch, not a fit.
- A first attempt ported TOMS fpser/apser. It was dropped: statistics.md "Incomplete beta bpser" records that Excel has
  neither, and the eleven points never reach them.
- New statistics.md entry: "Incomplete beta at two vanishing shapes". New unit test:
  `bratio::tests::tiny_shapes_are_b_over_a_plus_b`.
- The two `#NUM!` points at a huge beta (BETA.DIST/s1 A186, BETADIST/s1 A184) are a different rule and move to the new
  T-4733, with their known-failures lines retargeted.
- Eval delta: Gate A loses 11 known failures (`eval-gates`: 0 failing). eval-check: 0 regressions, no other cell
  changed.
