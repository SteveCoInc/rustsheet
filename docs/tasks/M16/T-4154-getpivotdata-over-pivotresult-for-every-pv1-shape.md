---
id: T-4154
title: "GETPIVOTDATA over PivotResult for every PV1 shape"
milestone: M16
lane: functions-lookup
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-4142, T-2085]
---

## Goal
GETPIVOTDATA reads the `PivotResult`: several data fields, page fields, hidden items, two or more fields per axis and their subtotals, number/date/boolean items, as T-2085's probes evidence.

## Context
SPEC-0015's unknowns answered `#REF!` until evidenced; T-2085 probes them. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

Owner of these visibly-off cells (decision 0245; T-4130, GETPIVOTDATA against a pivot collapsed to #SPILL!: Excel gives #REF! except for a field and item pair that names an item, which gives #SPILL!. The engine gives #SPILL! for the grand total by either name (B1, B2) and for an unknown item (B4), and 27 for a range holding the anchor (B6); B3 and B5 already agree; pivots.md "GETPIVOTDATA against a pivot collapsed to #SPILL!"):
- probes/t4130-pivot-spill-getpivotdata Q!B1:B2, Q!B4, Q!B6

Owner of these visibly-off cells (decision 0245; T-4170, pivots.md "Manual groups and date levels as Excel for the web
renders them"). Cause: the engine answers `#REF!` for an item of a field on an axis that holds two or more fields
(SPEC-0015 unknown 10), here DC's rows Years, Quarters and Date, where Excel answers the total (the grand total B17 and
DX's pair B20, one field per axis, already agree):
- probes/t4170-pivot-date-levels Calc!B1:B3, Calc!B5:B6, Calc!B8:B9, Calc!B11:B14

## Acceptance criteria
- [ ] probes/getpivotdata-* pass; the SPEC-0015 unknowns T-2085 resolved are implemented.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Show-as values in GETPIVOTDATA (PV2).
