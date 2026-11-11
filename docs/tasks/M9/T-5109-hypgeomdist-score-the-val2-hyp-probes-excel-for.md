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
  "err" for another reason: Excel's pmf is 0 there (E37:F50), so the residual divides by 0. Ours gives normal numbers in E37:F50.
- Observed fit to score (statistics.md, "HYPGEOM.DIST of an empty population is 1, and a deep-tail pmf can be 0
  (T-4105)"): on every row where Excel's E is a number, E equals K = (H/J)*I (p = MAX(x, n-x)/N) where J is not 0, and
  Q = (N/P)*O (p = n/N) where J is 0 (hyp-02 rows 7 to 14 and 20 to 33), bit for bit; the E37:F50 zeros are K = 0
  because H underflows. Score this against HYP-01's and HYP-02's hypotheses, reading the raw cells (HYP-01's L column
  follows from it as 0 in every row). hyp-03 (the factor kernel at the target's p) has no off cell today but is scored here too.
- Also owns the HYPGEOMDIST and BINOM.DIST anchors of probes/val2-cal-01, P!H12:H14 and P!H16:H19 (H16
  `=HYPGEOMDIST(41966,46221,65797,74726)` is 0 in Excel, ours 8.103803254141709e-186; the rest differ in the low bits).
- Related: T-2481 (HYPGEOM.DIST's pmf formulation).

- val3 batch, docs/research/probe-handoff-2026-11-09-val3.md (excel-graph fill, 2026-11-09). Owns: probes/val3-hyp-01 (no visibly-off cell at the fill).
- val4 batch, docs/research/probe-request-val4-hyp.md (excel-graph fill, 2026-11-10; predictions frozen in
  docs/research/val4-hyp-predictions.json). Owns: probes/val4-hyp-01 and probes/val4-hyp-02 (no visibly-off cell at the
  fill).
- val4 HYP-03, docs/research/probe-request-val4-hyp.md and the case's own description (excel-graph fill, 2026-11-10;
  predictions frozen in docs/research/val4-hyp-03-predictions.json). Owns: probes/val4-hyp-03 (every visibly-off cell).
  HYPGEOMDIST and HYPGEOM.DIST(..,FALSE) are 0 at ordinary outcomes wherever H or I is flushed and J is not: P!E2:F4 and
  E8:F18, pmfs up to 1.4E-3, with T TRUE. On the same rows the cdf HYPGEOM.DIST(..,TRUE) is exactly 1 (U2:V2, U8:V18;
  math 0.9038 to 1) or exactly 0 (U3:V4 and V5; math about 0.096 to 0.098). Score and rule on the cdf with the pmf. Rows
  5 to 7, where J is flushed too, are the p = n/N product (statistics.md, "HYPGEOM.DIST of an empty population is 1, and
  a deep-tail pmf can be 0"). Following Excel's zero at a 2.8E-4 pmf needs the owner ruling of decision 0188 R3 / 0300.
  The J column's BINOM.DIST cells (P!J2:J4, J8:J10) sit just above DBL_MIN and are off in the low digits.
- probes-numeric/t5109-fresh was split into 18 parts (#4125). On 2026-11-11 excel-graph filled t5109-fresh-calc,
  -cdf, -closed and -poisson. Owns: probes-numeric/t5109-fresh-closed and probes-numeric/t5109-fresh-poisson (every
  visibly-off cell): Closed!G2, G3, G8 BINOM.DIST pmf is 0 in Excel where ours is subnormal (9.996E-311, 2.46E-318,
  6.26E-311). These are k = n cells, and the excel-graph oracle reads a subnormal as 0 (decision 0042,
  statistics.md "BINOM.DIST pmf: closed forms"), so they are an oracle artifact, not evidence against the rule. Poisson!C3, C7, C29, C34 differ from ours in the 8th to 9th digit.
- Refused (Graph's `calculate`, retries exhausted): t5109-fresh-cdftie-03 (twice), -cdftie-01, -02 and -cdfwide-01
  to -11 (once). They stay pending as an oracle limit, not as Excel values; score against the predictions only the
  parts that filled.

## Acceptance criteria
- [ ] Every hypothesis of the family in probe-requests-2026-11-08.md has a verdict (confirmed, refuted, undecided) citing
      the probe cells, written to the research note of the family.
- [ ] Each confirmed rule is a docs/excel-semantics entry citing its probe cells; each has an implementation task filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (the filed implementation tasks carry them).
- Desktop-only questions (M14, decision 0039).
