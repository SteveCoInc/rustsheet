---
id: T-4296
title: "Read Excel's in-window large-shape gamma path from its prefactor chain and the 2a/3 and 1.5a ties (after T-4290)"
milestone: M9
lane: research
status: done
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
- [x] The window path's prefactor chain is identified from probes at x = a, or the rejected chains are recorded
      with their scores.
- [x] A candidate density is scored with a hold-out fixed before scoring (decision 0188 R4) and adopted if it is
      Excel's on the hold-out. Otherwise each rejected family goes into statistics.md with its scores.
- [x] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off, so this task is in the campaign.

## Out of scope
- Shapes below 20 (T-3219).
- POISSON.DIST's far upper tail (T-4291).

## Outcome
- **Prefactor chain identified** (statistics.md, the T-4296 paragraph under "Incomplete gamma: accurate kernel";
  docs/research/t4296/score.py). The probe is probes/t4296-gamma-density-at-mode (120 cells, excel-graph
  2026-11-05) at x = a, where the exponent is 0. Excel's window density is exp(-c(a) - B) / sqrt(2 pi a) times
  (a / x): one exponential, the exact Stirling error, and the shape over the raw argument last. It is bit-exact on
  fit A 40/40, hold-out B 40/40 (seed 4296, fixed before scoring) and scale-3 C 40/40. The rejected chains and
  their scores are recorded there.
- **Engine**: density.rs `mode_density` takes the chain wherever the density keeps the accurate exponent at shape
  20 and up. P and Q are unchanged. This is a structural model (check 7 (c)), so it owes the R4 guards and no
  FITTED tag. Counts are bit-exact / within 4 ulp / within 15 digits.
  - Hold-out (B): new 40/40/40; main 22/40/40; correctly rounded 33/40/40.
  - Train (A): new 40/40/40; main 22/40/40; correctly rounded 21/40/40.
  - Fresh sweep probes-numeric/t4296-gamma-density-r4-fresh (160 log-uniform cells, generated and filled after the
    freeze, since GAMMA.DIST has no Gate A table yet, T-3925): at x = a new 40/40/40, main 20/40/40, correctly
    rounded 18/40/40. In the rest of the window new 38/51/64, main 38/51/63, correctly rounded 39/52/57.
  - Gate A envelope on the sweep, against 50-digit mpmath: 160/160 inside, none outside, for both main and new.
- **Eval delta**:
  - probes: 49 cells fixed and 7 regressions (1 to 2 ulp); known failures 5760 to 5718. The 7 are recorded by
    decision 0286 through `update-baseline --allow-regressions`.
  - Visible parity: 2530 cells visibly off, against an allowance of 3156.
  - The new case t4296-gamma-density-at-mode is 120/120 bit-exact.
- **Exponent still open**: it is moved to T-4308, with the rejected families and the visibly-off cells this task
  owned plus the fresh sweep's.
