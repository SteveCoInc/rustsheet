---
id: T-4339
title: "GAMMA.DIST's large-shape density just above the window: why x/a = 1.5078125 takes the ratio prefactor (after T-4308)"
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
Excel's large-shape GAMMA.DIST density just above the window (x/a a little over 1.5) is the engine's, or the
probes that rule each model out are recorded.

## Context
T-4308 (docs/excel-semantics/statistics.md, the T-4308 paragraph; decision 0290; docs/research/t4308/score.py
part 4) found Excel's large-shape density to be Loader's chain exp(-c(a) - B) / sqrt(2 pi a) * (a / x) on bd0.
Outside the window B is the direct deviance a ln(a/y) + (y - a), which is Excel's at 156 of 160 probed cells above
the window. Row 223 of probes/t4290-gamma-window-large-pow2 (x/a = 1.5078125 exactly, shapes 4096, 1024 and 256)
is the ratio prefactor sqrt(a/2pi) exp(a ln(y/a) + a - y - c) / x to 0 to 2 ulp, where the direct deviance is 75 to
2,021 ulp off. Its neighbours x/a = 1.5039, 1.5098 and 1.5117 are the direct deviance bit for bit. Decision 0290
let B223 regress.
- Probe x/a on a fine grid in (1.5, 1.53) at shapes 256 to 4096 and at non-power-of-two shapes. Include the k/128
  points: 1.5078125 is 193/128, so a rounding of a/y or of ln(a/y) may decide it.
- Score each form against the exact B readout of score.py part 1.

Visibly-off cells this task owns: probes/t4290-gamma-window-large-pow2 A223, B223 and C223.

## Acceptance criteria
- [ ] Row 223 is explained and matched with no regression elsewhere, or the rejected forms are recorded with scores.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off, so this task is in the campaign.

## Out of scope
- Inside the window (T-4308, done).
