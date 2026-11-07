---
id: T-4879
title: "Probe the record order and columns of Excel's Show Details sheet for a pivot value cell"
milestone: M16
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Evidence for what Excel's Show Details (double-click on a pivot value cell) puts on the sheet it creates: the
row order of the records and the columns (which source fields, in which order, with which headers and number
formats), so the drill-down API can document or match them.

## Context
T-4184 built `Workbook::pivot_cell_records` (crates/rustsheet/src/pivot_view.rs) over the engine's
`pivot::records_behind` (crates/rustsheet-engine/src/pivot/layout/records.rs): it returns the record set, checked
against Excel's cell values, in cache record order. docs/excel-semantics/pivots.md "Show Details" marks the sheet's
order and columns UNVERIFIED. The oracle reads cell values and pivot parts; whether it can trigger Show Details
(an excel-graph call, or a case whose workbook already holds a Show Details sheet saved by desktop Excel) is part
of the task. SPEC-0032 "API"; decision 0274.

## Acceptance criteria
- [ ] A probe case with an oracle sidecar records Excel's Show Details sheet for an item cell, a subtotal and a
      grand total of a pivot whose source rows are out of item order, with a hidden item, and a source with
      a number-formatted column; or, if the oracle cannot produce one, the task records why.
- [ ] docs/excel-semantics/pivots.md "Show Details" states the row order and columns with that evidence.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the API (a follow-up task if the evidence asks for it).
