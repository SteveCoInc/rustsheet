---
id: T-4296
title: "Read Excel's in-window large-shape gamma path from its prefactor chain and the 2a/3 and 1.5a ties (after T-4290)"
milestone: M9
lane: research
status: in-progress
priority: P3
size: M
tier: opus
depends_on: [T-4290]
campaign: visible
---

## Goal
Excel's large-shape (a >= 20) gamma density inside the window (x/a from 2/3 to about 1.5) is reproduced, starting
from the prefactor chain, or the probes that rule out each model are recorded.

## Context
T-4290 (docs/excel-semantics/statistics.md, its paragraph under "Incomplete gamma: accurate kernel"; scorer
docs/research/t4290/score.py) found these facts:
- Outside the window Excel's density is the ratio form through a chain with a constant log offset per shape,
  against sqrt(a/2pi) exp(E - c(a))/x: -2.1E-14 at shape 4096, 4.3E-15 at 1024. Inside the window, next to the mode,
  that offset is absent. So the window path builds the prefactor differently. Find that chain first: probe Excel's
  density at x = a exactly (E = 0), at several power-of-two shapes, and compare with GAMMALN-based and Stirling-based
  prefactors.
- The per-unit exponent error is the same function of x/a at shapes 256 to 4096, within about +-0.6 ulp(ln u). No
  tested form beats chance.
- The lower end includes x = 2a/3. The upper end is past 1.5a(1 + 2^-20) at shape 2048 and is not one threshold.
- Ladder literals must have at most 15 significant digits (numbers.md).

The visibly-off cells this task owns are these:
- probes/t4290-gamma-window-large-pow2 A1:F226.
- probes/t4255-gamma-window-exact-ratio A5:D58 and G15.
- probes/t3252-gamma-prefactor-holdout-2 A5:A12, A17:A24 and A29:A36; probes/t3252-gamma-prefactor-holdout A5:A14,
  A19:A28 and A33:A42; probes/t3252-gamma-prefactor-window A4:A10, A11:A20, A24:A30 and A31:A40.
- probes/t3252-gamma-prefactor-sweep A9, A19, A20, A23, A53, A54, A55, A57, A58 and A63.

## Acceptance criteria
- [ ] The window path's prefactor chain is identified from probes at x = a, or the rejected chains are recorded
      with their scores.
- [ ] A candidate density is scored with a hold-out fixed before scoring (decision 0188 R4) and adopted if it is
      Excel's on the hold-out. Otherwise each rejected family goes into statistics.md with its scores.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off, so this task is in the campaign.

## Out of scope
- Shapes below 20 (T-3219).
- POISSON.DIST's far upper tail (T-4291).
