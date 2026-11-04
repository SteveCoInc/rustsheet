---
id: T-4138
title: "Probe pivot layout geometry: location, header rows and page area"
milestone: M15
lane: research
status: todo
priority: P1
size: S
tier: sonnet
depends_on: [T-4122, T-4123]
---

## Goal
Evidence for where Excel puts a pivot's parts: location ref and firstHeaderRow/firstDataRow/firstDataCol for compact, outline and tabular with 0-3 row fields, 0-2 column fields and 1-3 data fields (Values on rows or columns), and the page-field area above the location (layout over/down, fields per column).

## Context
Calipers officejs/pivot_two_data shows no Values row with two data fields and no column field. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] Probe cases with sidecars (captured location and rowItems); pivots.md entries.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes.
