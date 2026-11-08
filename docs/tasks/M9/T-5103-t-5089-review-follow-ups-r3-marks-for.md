---
id: T-5103
title: "T-5089 review follow-ups: R3 marks for the BETA.INV/F.INV p < 1E-15 region, an F.INV point in it, the identity-fit probe and small tidy-ups"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: S
tier: opus
usage: expert
depends_on: []
---

## Goal
The R3 region the owner ruled in T-5089 (BETA.INV/F.INV at p < 1E-15 in either tail, decision 0351 addendum) is
held to the math by Gate A at committed points, and T-5089's round-2 review MINOR findings are closed.

## Context
From PR #4100's round-2 review (all APPROVE, MINOR only):
- **Excel fidelity:**
  - accuracy/BETA.INV/s1 Calc!A169 and probes/t4974-beta-inv-dbl-min-residual P!H7 and P!C7 have no line in
    eval/accuracy/math-over-excel.tsv. They pass only through the envelope's math side, so Gate A's R3 check
    (T-5097) never holds them to the math alone.
  - The region has no F.INV evidence point between 1E-150 and 1E-15, or in the right tail.
  - The MATH-OVER-EXCEL entry's `Rule:` line in docs/excel-semantics/statistics.md still says "p < 1e-150 … left
    tail", while its new T-5089 Region paragraph widens it.
  - LINEST's identity-fit exception (linest.rs `householder_qr`) rests on linest-modes K85:N85 alone.
- **Design:** fisher.rs `f_quantile_as109` returns an anonymous `(f64, bool, bool)`; return a named struct or the
  `As109Outcome`. `householder_qr`'s doc comment does not mention the identity-fit exception, and its wrapping is
  uneven.
- **Security:** in linest.rs's standard-error step, `unit[i] = 1.0` indexes `vec![0.0; rank]` by the position in
  `kept`. Prove `kept.len() == rank` or use `.get_mut(i)`.

## Acceptance criteria
- [ ] R3 marks for BETA.INV/s1 A169 and t4974-beta-inv-dbl-min-residual H7 and C7, with math sidecar values;
      Gate A holds them to the math.
- [ ] One F.INV point in (1E-150, 1E-15) or in the right tail, probed with the excel-probe skill and marked or
      recorded.
- [ ] The MATH-OVER-EXCEL `Rule:` line matches the region.
- [ ] A probe of LINEST with y a scaled copy of an x column, and with y as a later pivot, shows whether Excel's
      identity-fit exactness is general; record the result in statistics.md.
- [ ] `f_quantile_as109` returns a named type; `householder_qr`'s doc names the exception; the `unit[i]` index is
      shown safe.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
