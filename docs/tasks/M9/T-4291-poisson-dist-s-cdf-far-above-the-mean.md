---
id: T-4291
title: "POISSON.DIST's cdf far above the mean at k 50 to 200: which prefactor form Excel takes (split from T-4255)"
milestone: M9
lane: research
status: done
priority: P3
size: S
tier: opus
depends_on: []
campaign: visible
---

## Goal
Find which large-shape prefactor Excel's POISSON.DIST cdf takes far above the mean (mean/k 1.5 to 3 at k 50 to
200), so that `gamma_ur` (incomplete_gamma/mod.rs) can take it there or record why it cannot.

## Context
T-3252 (docs/excel-semantics/statistics.md, "Incomplete gamma: accurate kernel") found the following in
probes/t3252-gamma-prefactor-window:
- Above the mean, POISSON.DIST's cdf is closer to the ratio form than to `rcomp` at k 100 and 200: A129, A139
  and A140 are -4, 3 and -1 ulp from the ratio form, against 115, -115 and -327 from `rcomp`.
- At k 50 it is not closer to the ratio form (A117, A118).
T-4255 split this question out and did not probe it. Q(k + 1, mean) at mean/k 1.5 to 3 puts x/a beyond the
window T-4290 owns, where the density is the ratio form (T-2638). So the k 50 cells may be on another path:
Excel's far upper tail (`excel_tail`, T-3269), or a series that does not use the prefactor.

Visibly-off cells this task owns: probes/t3252-gamma-prefactor-window A117, A118, A129, A139 and A140.

## Acceptance criteria
- [x] The form is scored on a hold-out fixed before scoring (decision 0188 R4): POISSON.DIST at k 30 to 300 for
      mean/k 1.3 to 3, as a new probe. It is adopted in `gamma_ur`'s POISSON path if it is Excel's there.
      Otherwise the rejected forms go into statistics.md with their scores.
- [x] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off, so this task is in the campaign.

## Out of scope
- The in-window gamma exponent (T-4290).

## Outcome
Neither prefactor form is Excel's far above the mean; the engine keeps the accurate kernel. The rejected forms and
their scores are in statistics.md (the POISSON sentences under "Incomplete gamma: accurate kernel").
- New hold-out probe probes/t4291-poisson-far-above-mean (48 cells, excel-graph): k 30 to 300 at mean/k 1.3 to 3.
  The generator and scorer are in docs/research/t4291/.
- Within 2 ulp of Excel on fit, then hold-out: the accurate value 4/20 and 9/48; the ratio form 0/20 and 1/48.
- Excel's implied exponent is off the double grid of E, so the chain itself differs, as in T-4290.
- The visibly-off cells pass to T-4302.
- Eval delta: 4 of 48 new probe cells pass. The 29 visibly off are owned by T-4302. Regressions 0.
