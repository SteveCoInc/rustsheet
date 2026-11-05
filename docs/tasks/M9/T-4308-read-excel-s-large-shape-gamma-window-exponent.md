---
id: T-4308
title: "Read Excel's large-shape gamma window exponent on the mode chain (after T-4296)"
milestone: M9
lane: research
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-4296]
campaign: visible
---

## Goal
Excel's large-shape (a >= 20) gamma density inside the window (|x - a| <= 0.2 (x + a)) is reproduced away from the
mode. Otherwise the probes that rule out each exponent model are recorded.

## Context
T-4296 (docs/excel-semantics/statistics.md, the T-4296 paragraph under "Incomplete gamma: accurate kernel"; scorer
docs/research/t4296/score.py) found the rest of the window chain. The density is exp(-c(a) - B) / sqrt(2 pi a) times
(a / x): one exponential of the summed exponent, the exact Stirling error c, and the shape over the raw argument
last. It is Excel's bit for bit at x = a (B = 0) at 120 cells of probes/t4296-gamma-density-at-mode and at 40 of
the fresh sweep, and the engine now takes it (density.rs `mode_density`). What is still unknown is B, the exponent
a ln(a/y) + y - a:
- With the chain fixed, Excel's B minus the true B, per unit of shape, is within about +-0.5 ulp(ln u) of 0 at
  shapes 256 to 4096 (T-4290). It is not on the double grid of B/a: it is many ulps of B/a off near the mode. So
  the rounding enters at the size of a ln u.
- Rejected, with the T-4296 chain (rms of the per-unit difference in ulp(ln u), fit columns A:C of
  probes/t4290-gamma-window-large-pow2, then hold-out D; chance is 0.29):
  - Loader's bd0(a, y) series 0.31, 0.31;
  - -a ln(x/a) + (x - a) 0.39, 0.41;
  - -a log1p((x - a)/a) + (x - a) 0.41, 0.39;
  - a log1p((a - x)/x) + (x - a) 0.55, 0.54;
  - a ln(a/x) + (x - a), with a/x rounded, 3.4, 4.0;
  - a (ln a - ln x) + (x - a) 35.7, 19.6;
  - x phi(a/x) 0.57, 0.59.
  Folding 1/u into the exponent as (a - 1) ln u is 32 to 500 ulp off at shapes 256 to 4096.
- Seven cells that matched on the old chain now miss by 1 or 2 ulp (decision 0286): t4255 C29, C31 and t4290 C90,
  C123, D92, D93, D95. They come back with the exponent.

Visibly-off cells this task owns (taken over from T-4296, plus the fresh sweep):
- probes/t4290-gamma-window-large-pow2 A1:F226.
- probes/t4255-gamma-window-exact-ratio A5:D58 and G15.
- probes/t3252-gamma-prefactor-holdout-2 A5:A12, A17:A24 and A29:A36; probes/t3252-gamma-prefactor-holdout A5:A14,
  A19:A28 and A33:A42; probes/t3252-gamma-prefactor-window A4:A10, A11:A20, A24:A30 and A31:A40.
- probes/t3252-gamma-prefactor-sweep A9, A19, A20, A23, A53, A54, A55, A57, A58 and A63.
- probes-numeric/t4296-gamma-density-r4-fresh A41:A160.

## Acceptance criteria
- [ ] A candidate exponent on the T-4296 chain is scored with a hold-out fixed before scoring (decision 0188 R4),
      and adopted if it is Excel's on the hold-out. Otherwise each rejected family goes into statistics.md with its
      scores.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off, so this task is in the campaign.

## Out of scope
- Shapes below 20 (T-3219).
- The cdf inside the window (it decouples from the density from about 0.72a; T-4255).
