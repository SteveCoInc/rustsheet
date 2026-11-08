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
- probes/t2085-getpivotdata-layouts Q!H1:H3 (a saved render; the other cells were fixed by the M6 probe-cell PR, below)

## Acceptance criteria
- [ ] The cells above pass, and no other case regresses (`make eval-baseline`).
- [ ] SPEC-0015's resolution rules and web.md's "Ours" line say what the engine now does.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Calculated fields (T-4174 owns the CF cells of this case).

## Progress
The M6 probe-cell PR (with T-4154) moved GETPIVOTDATA onto the pivot's `PivotResult`: page fields, hidden items,
two row fields with their subtotals and summation order now answer what the pivot shows, and every cell above but
Q!H1:H3 passes. Left: a pivot whose cache does not refresh on load is still answered from its cache, where Excel reads
its saved cells (East 99, no West row). Reading the saved `rowItems`/`colItems` lines and the cells under them is the
remaining work; it must leave the calipers and corpus pivots whose saved render agrees with their cache unchanged.
