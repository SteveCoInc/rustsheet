---
id: T-4340
title: "GAMMA.DIST's density is #DIV/0! past the 2 pi a overflow (after T-4308)"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4308]
campaign: visible
---

## Goal
GAMMA.DIST's density returns Excel's #DIV/0! at shapes past the 2 pi a overflow, wherever Excel does.

## Context
T-4308's fresh sweep (probes-numeric/t4308-gamma-density-r4-fresh A151:A154, excel-graph 2026-11-05) put
`GAMMA.DIST(a,a,1,FALSE)` at a = 2E307, 5E307, 9E307 and 9.99999999999999E307. 2 pi a overflows above about
2.9E307. Excel gives 8.920620580763855E-155 at 2E307 and #DIV/0! at the other three. The engine returns about
5E-155 there (density.rs `loader_density` returns None, and the caller keeps the prefactor's form). The quirk is
in statistics.md (the T-4308 paragraph).
- Probe off the mode (x/a 0.7, 1.3, 2, and 0.1 with a scale) at the same shapes, CHISQ.DIST at 2 df past 5.8E307,
  and the cdf (TRUE), to bound where the #DIV/0! applies.
- Then return the error from the density path (it returns Option<f64> today, so the error needs plumbing).

Visibly-off cells this task owns: probes-numeric/t4308-gamma-density-r4-fresh A152, A153 and A154.

## Acceptance criteria
- [ ] The probed overflow cells match Excel, the error included.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off, so this task is in the campaign.

## Out of scope
- Shapes below the overflow.
