---
id: T-4138
title: "Probe pivot layout geometry: location, header rows and page area"
milestone: M16
lane: research
status: done
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
- [x] Probe cases with sidecars (captured location and rowItems); pivots.md entries.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes.

## Outcome
probes/t4138-pivot-geometry (excel-graph, 2026-11-06), nine builder-v2 pivots:
- **Geometry:** for compact, outline and tabular form with 0–3 row fields, 0–2 column fields and 1–3
  data fields, Values on rows or on columns, Excel's location and its firstHeaderRow, firstDataRow and
  firstDataCol are tabulated in pivots.md "Pivot geometry".
- **Page area:** G8 shows wrapped page fields (pageWrap 2). They are consistent with the sidecar's
  page-area capture window, including the `colPageCount > 1` width (pairs at B and E of a five-column
  window). A capture cannot see past its own window, so this is consistency, not an independent
  confirmation.
- **Builder fix:** a Values field on rows makes Excel refuse the file unless the definition has
  `dataOnRows="1"`. Builder v2 now writes it, with a unit test.
