---
id: T-5109
title: "HYPGEOMDIST: score the val2 HYP probes (Excel for the web filled) against the conditional-binomial and factor-underflow hypotheses"
milestone: M9
lane: research
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Each HYPGEOMDIST hypothesis of the HYP probes (the conditional-binomial identity with Excel's own BINOM.DIST factors at
non-dyadic p and large N, the result when a binomial factor underflows) is scored against the filled Excel values, the
confirmed rules are recorded in docs/excel-semantics/statistics.md, and an implementation task is filed for each.

## Context
Owns: probes/val2-hyp-01, probes/val2-hyp-02 and probes/val2-hyp-03 (every visibly-off cell of these cases).
- Probes: the val2 batch of docs/research/probe-requests-2026-11-08.md (questions, hypotheses, model predictions) and
  docs/research/probe-handoff-2026-11-08.md (how it was filled, CAL-01 result). Filled on excel-graph 2026-11-09 by the
  PR that filed this task.
- CAL-01 passed on every anchor (the build reproduces the cached web corpus), but in-sheet `SUM(x,-y)*2^52` residuals
  snap to 0 (numbers.md, the SUM near-zero snap): read residuals from the raw value cells, never from a SUM column.
- No engine change lands until a probe confirms its rule; no fitted special cases (handoff, "What happens next").
- Off today: 41 cells in hyp-01 and 70 in hyp-02, almost all `IFERROR(SUM(x,-y)/x*2^52,"err")` residual columns. Those
  columns snap to 0 in Excel whenever x and y are close (CAL-01), so they cannot be read as ulp residuals: compute the
  residuals from the raw value cells, whose bits survive capture. 28 of the 70 hyp-02 cells (L37:L50, R37:R50) are
  "err" for another reason: Excel's pmf is 0 there (E37:F50), so the residual divides by 0. That is the factor-underflow
  result HYP-02 asks about: Excel's BINOM.DIST factor at p = MAX(x, n-x)/N is 0 in those rows (H37:H50, K37:K50) while
  the p = n/N product is not (Q37:Q50); statistics.md, "HYPGEOM.DIST of an empty population is 1, and a deep-tail pmf
  can be 0 (T-4105)". Ours gives normal numbers in E37:F50. hyp-03 (the factor kernel at the target's p) has no off cell today but is scored here too.
- Also owns the HYPGEOMDIST and BINOM.DIST anchors of probes/val2-cal-01, P!H12:H14 and P!H16:H19 (H16
  `=HYPGEOMDIST(41966,46221,65797,74726)` is 0 in Excel, ours 8.103803254141709e-186; the rest differ in the low bits).
- Related: T-2481 (HYPGEOM.DIST's pmf formulation).

## Acceptance criteria
- [ ] Every hypothesis of the family in probe-requests-2026-11-08.md has a verdict (confirmed, refuted, undecided) citing
      the probe cells, written to the research note of the family.
- [ ] Each confirmed rule is a docs/excel-semantics entry citing its probe cells; each has an implementation task filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (the filed implementation tasks carry them).
- Desktop-only questions (M14, decision 0039).
