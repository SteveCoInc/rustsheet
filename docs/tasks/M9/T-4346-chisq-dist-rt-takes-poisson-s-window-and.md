---
id: T-4346
title: "CHISQ.DIST.RT takes POISSON's upper-window form of Q at even df (x/a 1.3 to 1.5)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4322]
campaign: visible
---

## Goal
CHISQ.DIST.RT (CHIDIST, CHISQ.TEST) in the upper gamma window gives POISSON.DIST's bits there.

## Context
Excel's CHISQ.DIST.RT(2m, 2k + 2) is its POISSON.DIST(k, m, TRUE) bit for bit at all 96 probed points of
probes/t4302-poisson-cdf-beside-pmf (T-4302). T-4322 gave POISSON's cdf the window form from 1.3a to 1.5a
(excel_tail.rs `excel_window_upper`, integer shapes 31 to 301); CHISQ.DIST.RT still goes through `gamma_sf`.
Route it through the same form in the same measured range (do it with or after T-4320, which does the same
for `excel_far_above_mean` above 1.5a), and probe odd df before widening to half-integer shapes.

The visibly-off cells this task owns are these:
- probes/t4302-poisson-cdf-beside-pmf G13, B14, C14, D14, F14, G14, A37, F38, B39, A40 and H42.

## Acceptance criteria
- [ ] The cells listed above whose cdf T-4322's form matches pass within 4 ulp, with no regression elsewhere.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off, so this task is in the campaign.

## Out of scope
- Below the 1.3a cut (T-4345).
