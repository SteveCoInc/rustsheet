---
id: T-4733
title: "BETA.DIST cdf is #NUM! where Excel's is at a huge beta beside a tiny x (A186, BETADIST A184)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
BETA.DIST's cdf is `#NUM!` where Excel's is, at a huge beta beside a tiny x, so the last two Gate A known failures
T-3577 left pass.

## Context
- From T-3577, which kept the tiny-shape points (bratio's Tiny branch) and handed these two on:
  - accuracy/BETA.DIST/s1 A186 `BETA.DIST(1.16481965482714E-239,11670.8736712835,1.83368089638147E+214,TRUE)` is
    `#NUM!` in Excel. The math and the engine give 0.
  - accuracy/BETADIST/s1 A184 `BETADIST(3.69979443365239E-134,2.15356248493825E+114,1.9782375667363E+66)` is `#NUM!` in
    Excel. The math and the engine give 0.
- Both have min(a, b) > 1 and a tiny x0, where bratio takes bpser (b0·x0 <= 0.7) or bfrac. statistics.md "Incomplete beta
  at a shape near the double range ... bpser is #NUM! once 1/B overflows (T-3007)" may be the rule: B(a, b) at these
  shapes is far below DBL_MIN, so 1/B overflows. T-3104 owns that entry's other off cells. Check whether this is the
  same rule and the same owner first.
- Use the excel-probe skill if the two points do not settle which form cuts (predictions committed before the fill).

## Acceptance criteria
- [ ] Both points pass Gate A, and their lines leave eval/accuracy/known-failures.tsv.
- [ ] The rule is cited in docs/excel-semantics/statistics.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Tiny shapes (T-3577, done).
