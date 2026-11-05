---
id: T-4322
title: "POISSON.DIST's cdf in the gamma window (x/a 1.29 to 1.5) and its density, after T-4302"
milestone: M9
lane: research
status: done
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
- [x] A candidate is scored with a hold-out fixed before scoring (decision 0188 R4) and taken if it is Excel's on the
      hold-out. Otherwise the rejected models go into statistics.md with their scores.
- [x] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off, so this task is in the campaign.

## Out of scope
- Above the window (T-4302 took the cdf; T-4319 the density; T-4320 CHISQ.DIST.RT).

## Outcome
- From 1.3a to 1.5a (a = k + 1, integer shapes 31 to 301) POISSON's cdf now takes Excel's form there:
  sqrt(a / 2 pi) exp(-B - c) times the accurate fraction with T-4308's window exponent B (excel_tail.rs
  `excel_window_upper`; the far form's product shared as `loader_upper`). A structural model (the window
  exponent was T-4308's) plus one cut, 1.3, `FITTED` and frozen (commit ff265dc32) before the fresh sweep
  probes-numeric/t4322-poisson-window-r4-fresh (seed 43220, 48 points at 1.3a to 1.5a plus a ladder across the
  cut at k = 100, 200, 300) was written and filled (excel-graph 2026-11-05).
- Within 4 ulp of Excel (docs/research/t4322/score.py): fit (the 15 seen T-4302 points from 1.3a) the form 14/15
  against the correctly rounded Q 9/15; fresh 51/57 against 10/57, bit-exact 8 against 2. Gate A's envelope holds
  all 57. The ladder puts Excel's cut in (1.298a, 1.3a].
- Below 1.3a Excel is none of the tried prefactors (window, direct, correctly rounded B, pmf times Q/pmf) and 1 to
  17 ulp from the correctly rounded Q: T-4345. G3 of the fresh sweep is a lone 64-ulp miss (T-4345).
- The window densities this task listed (A8 ... H36) already pass since T-4308. CHISQ.DIST.RT's window cells,
  the same values, go to T-4346.
- Eval delta: probes known failures -3 (t4291 F2, t4302 F24, t3252-gamma-prefactor-window A116); probes-numeric
  gains the 66-cell fresh case. statistics.md has the T-4322 paragraph.
- Follow-ups: T-4345 (below the cut, the residue), T-4346 (CHISQ.DIST.RT in the window).
