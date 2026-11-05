---
id: T-4302
title: "POISSON.DIST's cdf far above the mean: Excel's prefactor chain at k 30 to 300 (after T-4291)"
milestone: M9
lane: research
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-4291]
campaign: visible
---

## Goal
Excel's POISSON.DIST cdf far above the mean (mean/k 1.3 to 3, k 30 to 300) is reproduced by the engine, or the
models that are ruled out are recorded.

## Context
T-4291 (docs/excel-semantics/statistics.md, under "Incomplete gamma: accurate kernel", the POISSON sentences; scorer
docs/research/t4291/score.py) found that neither the accurate prefactor nor the ratio form is Excel's there:
- on the hold-out probes/t4291-poisson-far-above-mean, 9/48 and 1/48 cells are within 2 ulp;
- Excel is up to 265 ulp from the true Q, and its implied exponent is not on the double grid of E (G5: -0.65 ulp
  of E).
So the chain around the exponent differs, as T-4290 found inside the gamma window (T-4296). Start from whatever
T-4296 learns about that chain, then probe here with ulp runs in the mean at fixed k (literals of at most 15
significant digits), next to POISSON.DIST(k, mean, FALSE) for the density's own chain.

The visibly-off cells this task owns are these:
- probes/t4291-poisson-far-above-mean A1:H6.
- probes/t3252-gamma-prefactor-window A117, A118, A129, A139 and A140.

## Acceptance criteria
- [ ] A candidate is scored with a hold-out fixed before scoring (decision 0188 R4) and taken in `poisson_upper`
      (incomplete_gamma/mod.rs) if it is Excel's on the hold-out. Otherwise the rejected models go into
      statistics.md with their scores.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off, so this task is in the campaign.

## Out of scope
- The in-window gamma prefactor (T-4296).
- Shapes up to 11 past x = 30, which take Excel's far tail already (T-3269).
