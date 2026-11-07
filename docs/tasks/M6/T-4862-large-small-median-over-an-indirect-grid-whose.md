---
id: T-4862
title: "LARGE/SMALL/MEDIAN over an INDIRECT grid whose first element is a #REF! (bad sheet name) is #N/A beside an off-sheet reference, where a cell error value propagates"
milestone: M6
lane: functions-lookup
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
`LARGE(INDIRECT({"nosuch!A1";"A1";"Data!G1"}),1)` (and SMALL, MEDIAN) is `#N/A` as in Excel, while an error value that a cell holds (t4704) keeps propagating.

## Context
Found by T-4791 (probes/t4791-large-small-median-mixed-first-element H4:H6, excel-graph): expected `#N/A`, ours `#REF!`.
`RefGrid::order_statistic_error` (crates/rustsheet-engine/src/reference_operators/areas.rs) returns the first error element
when the first element is not a reference. Dropping that rule regresses probes/t4704-mixed-xlookup-grid-readers (a `#DIV/0!`
hit propagates), so the `#REF!` of a failed INDIRECT must be told apart from a value error (or the rule is about the
element's origin). Notes: lookup.md "A grid with references on both sheets".

## Acceptance criteria
- [ ] t4791 H4:H6 pass and t4704 does not regress; lookup.md records the rule.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Value-first grids (t4791 H7:H12 already pass).
