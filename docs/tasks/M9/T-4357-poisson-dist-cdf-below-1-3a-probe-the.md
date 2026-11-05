---
id: T-4357
title: "POISSON.DIST cdf below 1.3a: probe the companion functions at the same points (T-4345 follow-up)"
milestone: M9
lane: research
status: done
priority: P3
size: M
tier: opus
depends_on: [T-4345]
campaign: visible
---

## Goal
Excel's POISSON.DIST cdf at k 30 to 300 with the mean from k + 1 to 1.3 (k + 1) is reproduced, using probes of
the functions Excel may share a kernel with at the same points, or the next models ruled out are recorded.

## Context
- T-4345 (docs/research/t4345/score.py; docs/excel-semantics/statistics.md, the T-4345 paragraph) scored 14
  models below 1.3a with a hold-out fixed before scoring, and none is Excel's: the best is bit-exact on 1 of 9
  hold-out cells. They cover Cephes' continued fraction with four prefactors, Temme's erfc leading term with
  four exponents, SciPy's `gammaincc`, and pmf sums in three orders. Excel stays 1 to 17 ulp from the correctly
  rounded Q, and every candidate is 13 to 18 ulp off at A10 too, so the error is probably in a step none of
  them shares.
- What is unprobed at these points: `GAMMA.DIST(m, k + 1, 1, TRUE)` and `1 - GAMMA.DIST(..., TRUE)` (whether
  the upper tail comes from P), `GAMMA.DIST(m, k + 1, 1, FALSE)` and `POISSON.DIST(k, m, FALSE)` (which prefactor
  goes with the cdf), and `ERFC.PRECISE` of the Temme argument sqrt(B) (whether Excel's erfc is the leading
  term). Write them with the `excel-probe` skill, beside the cdf, at the T-4345 train and hold-out points and at a
  fresh set drawn after the freeze.
- T-4345 also found that Cephes' continued fraction in doubles does not explain the residue of T-4322's window
  form above 1.3a (fit 13/15 and fresh 47/57 within 4 ulp, against 14/15 and 51/57 with the exact fraction).

The visibly-off cells this task owns (taken over from T-4345):
- probes-numeric/t4322-poisson-window-r4-fresh G3, B8, B9, A10 and C11.
- probes/t4291-poisson-far-above-mean D2 and H2; probes/t4302-poisson-cdf-beside-pmf B21 and E19.

## Acceptance criteria
- [x] The companion probes are committed with oracle values.
- [x] A candidate is scored with a hold-out fixed before scoring (decision 0188 R4) and taken if it is Excel's
      on the hold-out and a fresh sweep. Otherwise the rejected models go into statistics.md with their scores.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the cells listed under Context are visibly off, so this task is in the campaign.

## Out of scope
- CHISQ.DIST.RT's cells (T-4346, T-4320).

## Outcome
- Probe probes-numeric/t4357-poisson-companions (docs/research/t4357/gen_probe.py; excel-graph, 2026-11-05): at
  T-4345's 9 train and 9 hold-out points and 24 fresh points (seed 4357, drawn before scoring), the cdf beside
  `GAMMA.DIST(m,a,1,TRUE)`, `1-GAMMA.DIST(...)`, `GAMMA.DIST(m,a,1,FALSE)`, `POISSON.DIST(k,m,FALSE)` and
  `ERFC.PRECISE` of Temme's argument, a = k + 1.
- Scored (docs/research/t4357/score.py, split fixed before scoring): five models that take one Excel companion value
  and the rest exactly. None is Excel's on the hold-out (best 0/3 of 9 bit-exact/within 4 ulp, against 1/4 for the
  correctly rounded Q); Excel's `1-GAMMA.DIST` is up to 389215 ulp off (P cancels), and the density, pmf and erfc
  models are 22 to 86 ulp off at their worst. The rejected models and their scores are in statistics.md after the
  T-4345 paragraph. No candidate was taken, so no engine change and no post-freeze sweep.
- Eval delta: the new case's cells (see the eval-check summary in the PR); the visibly-off cells the task owned stay
  open and move to T-4368 (`campaign: visible`), with any visibly-off cell of the new case's column A.
