---
id: T-4636
title: "Probe how an error ends a pivot's countA and count custom subtotals beyond one inner row field"
milestone: M16
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: [T-4622]
---

## Goal
Evidence for the scope of the error rule probes/t4622-pivot-count-subtotals found: under a Sum data field, an error ends the `countA` and `count` custom subtotal's count within each inner-item cell, in record order (the error is counted by countA, and nothing after it is). Is the unit the inner-item cell, the deepest row-and-column intersection, or something else?

## Context
t4622 (docs/excel-semantics/pivots.md "Which function the countA and count custom subtotals compute") has one outer field with the subtotals and one inner row field, and three placements of one `#N/A` (B1, B4, B5). Unprobed: an outer field with no inner field (the subtotal over the outer item's own records), two inner fields, a column field beside the row fields, a cell holding two errors, other data aggregates than Sum and Count (Average, Max, CountNums), and the other custom functions (sum, avg, ...) over an error. T-4168 renders custom subtotals and needs the rule.

## Acceptance criteria
- [ ] A probe case with sidecar that varies the inner fields, a column field and the data aggregate over records holding an error, the subtotal values captured.
- [ ] The pivots.md entry's error rule rewritten from the evidence (VERIFIED or corrected).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Rendering custom subtotals (T-4168).
