---
id: T-4806
title: "GETPIVOTDATA answers what the pivot shows: page fields, hidden items, nested row fields, a saved render (T-2085 evidence)"
milestone: M6
lane: functions-lookup
status: todo
priority: P3
size: M
tier: opus
spec: docs/specs/SPEC-0015-pivot-tables-and-getpivotdata.md
depends_on: []
---

## Goal
GETPIVOTDATA answers the value the pivot shows on pivots with page fields, hidden items or two row fields, and on a pivot whose saved render disagrees with its cache.

## Context
T-2085 probed SPEC-0015 unknowns 4, 9, 10 and 11 (docs/excel-semantics/web.md "GETPIVOTDATA on other pivot shapes (T-2085)", the "Page fields and hidden items", "Two row fields", "Summation order" and "A saved render" paragraphs). Excel answers what the pivot shows:
a total or item without a shown cell is `#REF!`, a saved render wins over the cache, and totals follow pivots.md
"Summation order of Sum, Average and Product totals". Reading the engine's rendered pivot layout may serve all of
these at once, in place of aggregating the cache.

Owner of these visibly-off cells (decision 0245):
- probes/t2085-getpivotdata-layouts Q!A1:A3, Q!A5:A7, Q!A9, Q!A11, Q!A12, Q!B2, Q!B3, Q!B5, Q!C2, Q!C3, Q!C5, Q!D2,
  Q!D3, Q!D5, Q!E3, Q!E5, Q!F1:F3, Q!H1:H3

## Acceptance criteria
- [ ] The cells above pass, and no other case regresses (`make eval-baseline`).
- [ ] SPEC-0015's resolution rules and web.md's "Ours" line say what the engine now does.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Calculated fields (T-4174 owns the CF cells of this case).
