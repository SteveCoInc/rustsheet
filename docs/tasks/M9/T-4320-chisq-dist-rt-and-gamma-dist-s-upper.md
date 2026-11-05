---
id: T-4320
title: "CHISQ.DIST.RT and GAMMA.DIST's upper cdf far above the mean take POISSON's T-4302 prefactor"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4302]
campaign: visible
---

## Goal
CHISQ.DIST.RT (CHIDIST, CHISQ.TEST) and GAMMA.DIST's upper cdf far above the mean give POISSON.DIST's bits
there.

## Context
Excel's CHISQ.DIST.RT(2m, 2k + 2) is its POISSON.DIST(k, m, TRUE) bit for bit at all 96 probed points of
probes/t4302-poisson-cdf-beside-pmf (rows 13:18 and 37:42 against the cdfs). T-4302 gave POISSON's cdf the
prefactor sqrt(a/2 pi) exp(-B - c) with Loader's direct deviance B (excel_tail.rs `excel_far_above_mean`,
shapes 31 to 301 integer, 1.5a < x <= 3a). CHISQ.DIST.RT goes through `gamma_sf` with the ratio form, so it is
up to 247 ulp off at those cells.
- Route `gamma_sf` through `excel_far_above_mean` in its measured range, and check half-integer shapes (odd
  degrees of freedom) with a probe before widening to them.
- Check that the change does not move the T-3269 far tail (shapes up to 11).

The visibly-off cells this task owns are these:
- probes/t4302-poisson-cdf-beside-pmf A38, A39, B16, B17, B37, B38, B40, B42, C16, C18, C37, D15, D16, D39, E15,
  E37, E41, E42, F16, F18, F37, F39, G17, G38, G40, G42, H38 and H41.

## Acceptance criteria
- [ ] The CHISQ.DIST.RT cells listed below pass within 4 ulp, with no regression elsewhere.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off, so this task is in the campaign.

## Out of scope
- Inside the window (x/a <= 1.5; T-4308).
