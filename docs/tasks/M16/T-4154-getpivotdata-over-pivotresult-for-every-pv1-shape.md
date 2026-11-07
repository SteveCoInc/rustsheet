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

## Acceptance criteria
- [ ] probes/getpivotdata-* pass; the SPEC-0015 unknowns T-2085 resolved are implemented.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Show-as values in GETPIVOTDATA (PV2).
