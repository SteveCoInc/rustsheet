---
id: T-4959
title: "GAMMA.DIST and CHISQ.DIST cdf at 1 < x/scale < a + 1: neither Excel's leading-factor form nor the accurate pair (T-3412 sweep edge cells)"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-3412]
---

## Goal
GAMMA.DIST, GAMMADIST and CHISQ.DIST's cdf at a shape above 1 and 1 < x/scale < a + 1 reach Excel's value within
half a unit of the 15th digit; or the Outcome lists the forms tried with their counts (decision 0056).

## Context
T-3412 made the cdf Excel's leading-factor form up to x/scale = 1 (`incomplete_gamma::excel_lower_above_shape_one`).
Just past 1 neither that form nor the accurate pair is Excel's:
- probes-numeric/t3412-fresh-sweep A251:A260 (shapes 1.9 to 18.8, x 1.02 to 1.49): 8 of 10 over 4 ulp off either way
  (`GAMMA.DIST(1.07551,13.41,1,TRUE)` 30 ulp, `(1.33726,18.84,1,TRUE)` -28);
- t3412-fresh-sweep-2 A91:A110 (shapes 20 to 170): 5 of 20 over 16 ulp;
- the literal cells with Excel's arguments: the form is bit-exact at 6 of 51, the pair at 10 (T-3412 Outcome).
Forms already tried there (docs/research/t3412, T-3412 Outcome): the leading-factor form in eight operation orders,
`EXP(a*LN(x)-x-GAMMALN(a+1))` times the series (the series in x87 registers was tried only up to x = 1). Leads: the cut may not be at x = 1
(probe x from 0.9 to 2 by adjacent steps at shapes 3 and 10), or Excel may switch to the continued fraction below
a + 1.

## Acceptance criteria
- [ ] The sweep edge cells above are within half a unit of Excel's 15th digit, or a probe in statistics.md shows why not.
- [ ] Any fit passes decision 0188 R4; no gated cell regresses.
- [ ] `make check` green; reviewers approve.

## Out of scope
- x/scale up to 1 (T-3412) and x >= a + 1 (the continued fraction, T-3285, T-3269).
