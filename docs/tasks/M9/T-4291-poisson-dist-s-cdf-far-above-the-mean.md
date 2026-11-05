---
id: T-4291
title: "POISSON.DIST's cdf far above the mean at k 50 to 200: which prefactor form Excel takes (split from T-4255)"
milestone: M9
lane: research
status: in-progress
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
- [ ] The form is scored on a hold-out fixed before scoring (decision 0188 R4): POISSON.DIST at k 30 to 300 for
      mean/k 1.3 to 3, as a new probe. It is adopted in `gamma_ur`'s POISSON path if it is Excel's there.
      Otherwise the rejected forms go into statistics.md with their scores.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off, so this task is in the campaign.

## Out of scope
- The in-window gamma exponent (T-4290).
