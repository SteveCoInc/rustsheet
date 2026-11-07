---
id: T-4989
title: "Leading-factor cdf residue above shape 20 at x/scale up to 1: two T-3412 sweep-2 cells 9 to 10 ulp off"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3412]
---

## Goal
GAMMA.DIST and CHISQ.DIST's cdf above shape 20 at x/scale up to 1 within half a unit of Excel's 15th digit at the
cells below, or a probe in statistics.md showing why not.

## Context
T-3412 takes Excel's leading-factor form there (`incomplete_gamma::lower_form::excel_lower_above_shape_one`). On
its second fresh sweep it is within 4 ulp at 82 of 90 cells and within 16 at all; two cells are 9 to 10 ulp off,
which is past the 15th digit:
- probes-numeric/t3412-fresh-sweep-2 Calc!A64 `CHISQ.DIST(0.113937,76,TRUE)`: Excel 9.364725748514775E-93, ours
  9.364725748514765E-93;
- probes-numeric/t3412-fresh-sweep-2 Calc!A36 `GAMMA.DIST(0.0176469,30.96,1,TRUE)`: Excel 7.150405746213197E-89, ours
  7.150405746213188E-89.
|a·LN(y)| is 109 and 125 there (an exponent ulp of 1.4E-14), and the residue may be Excel's
rounding of a·LN(y) or of GAMMALN(a + 1) at a large shape. Lead: probe adjacent doubles of y at those shapes and
compare Excel's staircase steps with the form's.

## Acceptance criteria
- [ ] The two cells within 15 digits of Excel, or the Outcome lists the variants tried with their counts.
- [ ] No gated cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Shapes below 20 (T-3412) and x/scale above 1 (T-4959).
