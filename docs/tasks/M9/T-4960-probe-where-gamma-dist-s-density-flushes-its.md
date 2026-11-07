---
id: T-4960
title: "Probe where GAMMA.DIST's density flushes its subnormal leading factor to 0 (one T-3412 sweep cell)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-3412]
---

## Goal
Evidence for where Excel's GAMMA.DIST and CHISQ.DIST density is 0 because its leading factor x^a / Gamma(a + 1) is
subnormal, so `incomplete_gamma::excel_lower_density`'s flush is no longer UNVERIFIED.

## Context
T-3412's density sweep found one cell: probes-numeric/t3412-density-sweep A69 `GAMMA.DIST(4.57398E-12,24.55,2,FALSE)`
is 0 in Excel, where the factor is a subnormal 3E-311 and a / x (5E12) would bring the density back to 2.5E-298. The
engine now returns 0 wherever the factor is below the smallest normal double; one cell is all that shows it. Probe
pairs of cells with the factor just above and just below 2^-1022 at shapes 2, 10 and 50, and at x where a / x is
large and where it is small.

## Acceptance criteria
- [ ] A filled probe case brackets the flush at three shapes; the doc comment and statistics.md cite it.
- [ ] If Excel's cut differs, the engine follows it; no gated cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The cdf (its subnormal results are flushed already, numbers.md).
