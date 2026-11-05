---
id: T-4319
title: "GAMMA.DIST's density above 1.5a at shapes 31 to 301 takes Loader's direct deviance in the T-4296 chain (after T-4302)"
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
Excel's GAMMA.DIST density above its window (x > 1.5a) at shapes 31 to 301 is the engine's, bit for bit.

## Context
T-4302 (docs/excel-semantics/statistics.md, the T-4302 paragraph; docs/research/t4302/score.py part 3) found
Excel's density there to be exp(-c(a) - B) / sqrt(2 pi a) * (a / x): the T-4296 chain with Loader's direct
deviance B = a ln(a/x) + (x - a), the difference x - a taken first, and c the exact Stirling error. It is
bit-exact at all 120 probed points above 1.5a: train 32/32, hold-out 40/40 and the fresh sweep 48/48 (probes/
t4302-poisson-cdf-beside-pmf rows 7:12 and 31:36, probes-numeric/t4302-poisson-cdf-r4-fresh rows 7:12). Ours
takes `gamma_prefactor`'s ratio form / x there (density.rs `density_with`), up to 481 ulp off.
- Take the chain in `density_with` at shapes 31 to 301 for x > 1.5a (x / scale), at integer shapes only
  unless a non-integer probe shows the same chain.
- T-2638 and T-4290 saw the ratio form, with a per-shape offset, outside the window at shapes 25, 50, 500 and
  256 to 4096. Score this chain on those cells too (it may explain the offset), and on below-the-window cells
  (x < 2a/3), before widening the range.
- Run a full eval A/B: every GAMMA.DIST, CHISQ.DIST and BETA path that calls `gamma_density` moves.

The visibly-off cells this task owns are these:
- probes/t4302-poisson-cdf-beside-pmf A10, A32, A33, B10, B11, B31, B32, B34, B36, C10, C12, C31, C33, D9, D10, D33,
  E9, E35, E36, F10, F12, F31, F33, G11, G32, G34, G36, H32 and H35.

## Acceptance criteria
- [ ] The visibly-off density cells listed below pass bit-exact, with no regression elsewhere.
- [ ] The R4 guards are recorded (the T-4302 splits can be reused: they were fixed before scoring).
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off, so this task is in the campaign.

## Out of scope
- Inside the window (T-4308).
