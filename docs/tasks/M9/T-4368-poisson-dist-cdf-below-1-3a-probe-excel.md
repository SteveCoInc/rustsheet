---
id: T-4368
title: "POISSON.DIST cdf below 1.3a: probe Excel's P and Q at non-integer shapes and the cut's lower edge (T-4357 follow-up)"
milestone: M9
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
Excel's POISSON.DIST cdf at k 30 to 300 with the mean from k + 1 to 1.3 (k + 1) is reproduced, or the step that
carries Excel's error is located by probes that separate it.

## Context
- T-4357 (probes-numeric/t4357-poisson-companions, docs/research/t4357/score.py; statistics.md after the T-4345
  paragraph) ruled out the cdf as 1 - GAMMA.DIST's P, as Excel's density or pmf times the exact bracket, and as
  Temme's leading term on Excel's ERFC.PRECISE. Excel stays within 4 ulp of the correctly rounded Q at 29 of 42 points,
  1 to 17 ulp off elsewhere.
- Next probes: `GAMMA.DIST(m,a,1,TRUE)` at non-integer a near the same points (whether POISSON's cdf goes through the
  gamma code with an integer shape special case), the same k with m stepped by one ulp-scale decimal across 1.0a
  (where the series and the fraction may switch), and `CHISQ.DIST.RT(2m, 2a)` (T-4346 owns its cells; only as
  evidence here).
- Use the `excel-probe` skill; score with a hold-out fixed before scoring (decision 0188 R4).

The visibly-off cells this task owns (taken over from T-4357):
- probes-numeric/t4322-poisson-window-r4-fresh G3, B8, B9, A10 and C11.
- probes/t4291-poisson-far-above-mean D2 and H2; probes/t4302-poisson-cdf-beside-pmf B21 and E19.
- Any visibly-off cell of probes-numeric/t4357-poisson-companions column A.

## Acceptance criteria
- [ ] The probes are committed with oracle values, and the scored models (hold-out fixed before scoring) are recorded
      in statistics.md; a model Excel's on the hold-out and a fresh sweep is implemented.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off, so this task is in the campaign.

## Out of scope
- CHISQ.DIST.RT's own cells (T-4346, T-4320).
