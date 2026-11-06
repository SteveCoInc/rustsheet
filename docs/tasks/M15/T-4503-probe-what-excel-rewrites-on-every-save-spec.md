---
id: T-4503
title: "Probe what Excel rewrites on every save (SPEC-0033 normalization list N1-N12, RQ2)"
milestone: M15
lane: research
status: blocked
priority: P2
size: S
tier: opus
depends_on: [T-4167]
spec: docs/specs/SPEC-0033-xlsx-round-trip-contract.md
---

## Goal
For each candidate normalization N1–N12 of SPEC-0033 D4.4, we have evidence of whether Excel rewrites it on every
save. Evidenced items become normalizations in the round-trip differ; the others stay `changed` items.

## Context
SPEC-0033 X1 and RQ2: a normalization is allowed only with evidence (directive 1). Method:
1. Take a handful of Excel-saved workbooks that carry the candidate items. The Calipers goldens and our fixtures
   cover them all.
2. Have Excel re-save each one, unedited and after one cell edit, through excel-graph. Use desktop Excel where
   Calipers already shows it (`init.xlsx` against `golden.xlsx`).
3. Diff with the differ of T-4172, or with docs/research/xlsx-round-trip/rtdiff.py until it lands.

Record per item: always rewritten, rewritten only on edit, or kept. Note where Excel for the web differs from
desktop (RQ1, which T-4180 owns more widely).

## Acceptance criteria
- [ ] docs/excel-semantics/xlsx-storage.md "Round trips" has one evidence row per N item, with the probe workbook
      and backend.
- [ ] The differ's normalization data marks exactly the evidenced items, as a follow-up in T-4172's lane if T-4172
      has merged.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Normalizations not on the candidate list, which need a spec change first.

## Blocked because
Deferred to Phase 2 (decisions 0027, 0275, 0306): M15 waits until Phase 1 parity exits, except for the task the
owner-directed session claims. Unblock when docs/status.md has the line `**Phase 1:** exited <date>`, or by claiming
it in that session.
