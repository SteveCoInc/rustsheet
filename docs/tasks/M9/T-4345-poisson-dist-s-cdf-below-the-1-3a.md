---
id: T-4345
title: "POISSON.DIST's cdf below the 1.3a window cut (x/a 1 to 1.3), after T-4322"
milestone: M9
lane: research
status: todo
priority: P3
size: M
tier: opus
depends_on: [T-4322]
campaign: visible
---

## Goal
Excel's POISSON.DIST cdf at k 30 to 300 with the mean from k + 1 to 1.3 (k + 1) is reproduced, or the models
ruled out are recorded; the few-ulp residue of T-4322's window form is explained or left on record.

## Context
T-4322 (docs/excel-semantics/statistics.md, the T-4322 paragraph; docs/research/t4322/score.py) found that from
1.3a to 1.5a (a = k + 1) Excel's cdf is sqrt(a/2 pi) exp(-B - c) times the fraction with T-4308's window
exponent B, and that the cut is in (1.298a, 1.3a] at k = 100 and 300. Below the cut Excel is none of the
tried prefactors: it is 1 to 17 ulp from the correctly rounded Q (probes-numeric/t4322-poisson-window-r4-fresh
A8:C10, probes/t4291-poisson-far-above-mean D1:H1), and the window form is up to 62 ulp off. 1.3 is Cephes'
`SMALLRATIO`, the cut of its asymptotic (Temme) igamc series: score that series computed in doubles first. The
cut is bracketed at k = 100 and 300 only: at k = 200 the window form is as close as the correctly rounded Q just
below it (t4322 fresh B8:B10, -3, 4 and -2 ulp against -9, 10 and -1), so check whether the cut depends on k.
- probes-numeric/t4322-poisson-window-r4-fresh G3 `POISSON.DIST(80,118.464,TRUE)` (x/a 1.4625) is 64 ulp from
  the window form and 27 to 92 from every other tried form; its neighbours fit.
- Split (T-3229): T-4337 owns the t4291 cells that are off by a few ulp but not visibly (row 1 among them), cited
  above as evidence; this task owns only the visibly-off cells listed below.
- The window form's own residue (2 to 6 ulp, the accurate fraction against Excel's) leaves a few cells
  visibly off.

The visibly-off cells this task owns are these:
- probes-numeric/t4322-poisson-window-r4-fresh G3, B8, B9, A10 and C11.
- probes/t4291-poisson-far-above-mean D2 and H2; probes/t4302-poisson-cdf-beside-pmf B21 and E19.

## Acceptance criteria
- [ ] A candidate for x/a 1 to 1.3 is scored with a hold-out fixed before scoring (decision 0188 R4) and
      taken if it is Excel's on the hold-out. Otherwise the rejected models go into statistics.md with their
      scores.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off, so this task is in the campaign.

## Out of scope
- CHISQ.DIST.RT's cells (T-4346, T-4320).
