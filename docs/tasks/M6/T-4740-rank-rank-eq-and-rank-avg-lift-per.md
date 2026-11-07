---
id: T-4740
title: "RANK, RANK.EQ and RANK.AVG lift per element over a mixed XLOOKUP grid (probes/t4704 A26, A29, A32, A35, A38)"
milestone: M6
lane: functions-lookup
status: in-progress
priority: P2
size: S
tier: opus
depends_on: [T-4704]
---

## Goal
`RANK(60,X)`, `RANK.EQ` and `RANK.AVG` over a mixed XLOOKUP grid `X = XLOOKUP({2;9},A1:A3,B1:B3,Data!G1:G2)` lift per element: the hit element (a number, not a reference) gives `#VALUE!`, the miss element ranks against its referenced range.

## Context
- Evidence: probes/t4704-mixed-xlookup-grid-readers (excel-graph, 2026-11-07, T-4704): probes 9-13 (Calc A25:C26 `RANK(60,..)` = `#VALUE!`, 1; A28:A29 RANK.EQ same; A31:A32 RANK.AVG same; A34:A35 `RANK(20,..)` = `#VALUE!`, `#N/A`; A37:A38 `RANK(50,..)` = `#VALUE!`, 2). The shape (2x1) already matches; ours gives `#VALUE!` for the second element (A26, A29, A32, A35, A38).
- T-4570 left RANK on the value view (`statistical/chisq.rs numbers_from_ranked_arg`); the evidence says the reference element is read as a range.

## Acceptance criteria
- [ ] Calc A26, A29, A32, A35 and A38 of probes/t4704 pass; SPEC-0014 "As implemented (T-4570)" drops its RANK fallback.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The other readers of a mixed XLOOKUP grid (T-4704).
