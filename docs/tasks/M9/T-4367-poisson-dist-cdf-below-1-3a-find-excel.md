---
id: T-4367
title: "POISSON.DIST cdf below 1.3a: find Excel's operation order (backward error under 0.36 ulp of x; T-4357 follow-up)"
milestone: M9
lane: research
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-4357]
campaign: visible
---

## Goal
Excel's POISSON.DIST cdf at k 30 to 300 with the mean from k + 1 to 1.3 (k + 1) is reproduced bit for bit, or the
operation orders ruled out are recorded.

## Context
- T-4357 (statistics.md, the T-4357 paragraph; probes-numeric/t4357-poisson-cdf-companions) found that the cdf is
  backward-accurate: Excel's value is Q(a, x') for an x' within 0.36 ulp of x (median 0.1) at all 18 points, where
  one ulp of x moves Q by 9 to 119 ulp. It does not go through Excel's pmf or density, and it is not 1 - P. Excel's
  P (GAMMA.DIST TRUE) is within 1 ulp of the truth there.
- So the bits come from one or two double roundings of x-dependent intermediates (x - a, ln(x/a), x/a, ...). Next:
  score accurate algorithms (a double-double or extended-precision prefactor times a continued fraction or series)
  under candidate roundings of those intermediates. Fix the hold-out before scoring (decision 0188 R4): T-4345's
  train and hold-out split (t4357 rows 1:9 and 10:18). Draw a fresh sweep after the freeze.
- T-4345's models and scores: statistics.md, the T-4345 paragraph; docs/research/t4345/score.py.

The visibly-off cells this task owns (taken over from T-4357):
- probes-numeric/t4322-poisson-window-r4-fresh G3, B8, B9, A10 and C11.
- probes/t4291-poisson-far-above-mean D2 and H2; probes/t4302-poisson-cdf-beside-pmf B21 and E19.
- probes-numeric/t4357-poisson-cdf-companions, every visibly-off cell (the cdf, CHISQ.DIST.RT, GAMMA.DIST,
  the pmf and GAMMALN.PRECISE columns).

## Acceptance criteria
- [ ] A candidate operation order is scored on the fixed hold-out and taken if it is Excel's there and on a
      fresh sweep. Otherwise the orders ruled out go into statistics.md with their scores.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off, so this task is in the campaign.

## Out of scope
- CHISQ.DIST.RT's cells above 1.3a (T-4346, T-4320).
