---
id: T-4322
title: "POISSON.DIST's cdf in the gamma window (x/a 1.29 to 1.5) and its density, after T-4302"
milestone: M9
lane: research
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-4302, T-4308]
campaign: visible
---

## Goal
Excel's POISSON.DIST cdf at k 30 to 300 with the mean from 1.29 to 1.5 times k + 1 (inside the gamma window) is
reproduced, or the models ruled out are recorded.

## Context
T-4302 (docs/excel-semantics/statistics.md, the T-4302 paragraph; docs/research/t4302/score.py) found that above
1.5a the cdf is Excel's gamma density times x times the fraction F, and that this density is the T-4296 chain with
Loader's direct deviance B = a ln(a/x) + (x - a). Inside the window (x/a 1.29 to 1.5 in its probes) neither holds:
- the cdf is up to 63 ulp from Excel's density times x F (11 of 16 train cells within 4 ulp);
- that B is Excel's density at only 2 of 24 window densities. The window density's exponent is T-4308's question.
Start from T-4308's findings. CHISQ.DIST.RT(2m, 2k + 2) is the same value as the cdf (bit for bit at 96 points), so
its window cells go with the cdf.
- E19 of probes/t4302-poisson-cdf-beside-pmf (`POISSON.DIST(78,124.88,TRUE)`, x/a 1.58) is above the window but
  6 ulp from Excel's density times x F, so Excel's own fraction is a few ulp off there; it is listed here too.

The visibly-off cells this task owns are these:
- probes/t4291-poisson-far-above-mean B2, D2, F2, G2 and H2.
- probes/t4302-poisson-cdf-beside-pmf A19, B21, F20, H24 and E19 (cdf); A37, A40, B14, B39, C14, D14, E14, F14,
  F38, G13, G14 and H42 (CHISQ.DIST.RT); A8, A31, A34, B33, C8, D8, D31, E7, E8, F7, F32, G7, G8, H7 and H36 (the
  window density, which T-4308's exponent decides).

## Acceptance criteria
- [ ] A candidate is scored with a hold-out fixed before scoring (decision 0188 R4) and taken if it is Excel's on the
      hold-out. Otherwise the rejected models go into statistics.md with their scores.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off, so this task is in the campaign.

## Out of scope
- Above the window (T-4302 took the cdf; T-4319 the density; T-4320 CHISQ.DIST.RT).
