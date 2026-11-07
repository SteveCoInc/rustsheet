---
id: T-4506
title: "Pivot sources: shared caches, named and dynamic ranges"
milestone: M16
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4141]
---

## Goal
Several pivots share a cache; sources given as defined names (including dynamic OFFSET/INDEX ranges) and Tables resolve at refresh.

## Context
SPEC-0032. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

Owner of these check (c) cells (decision 0245; T-4972, decision 0349). Each `refreshOnLoad` pivot reads a Table or a defined name, so the engine refuses its refresh (`a pivot cache's source is not a worksheet range of this workbook`). Every cell of Excel's on-open area is then scored against nothing, and each non-empty one fails: 200 cells of 7 pivots.
- pivot-on-open/PivotMixedDataTypesInTableColumn.xlsx pvt1!A1:G4, pvt2!A1:C8, pvt3!A1:B12, pvt4!A1:G4, pvt5!A1:G19
- pivot-on-open/PivotTableWithNoneTheme.xlsx pvt!A1:C8
- pivot-on-open/TemplateWithTableSourcePivotTables.xlsx StressedAssetsPivot!A4:D8

## Acceptance criteria
- [ ] Probes pass.
- [ ] `make check` green; reviewers approve.

## Out of scope
- External sources.
