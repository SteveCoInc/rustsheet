---
id: T-4506
title: "Pivot sources: shared caches, named and dynamic ranges"
milestone: M16
lane: engine
status: done
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
- [x] Probes pass.
- [x] `make check` green; reviewers approve.

## Out of scope
- External sources.

## Outcome
Probes (excel-graph, 2026-11-07): probes/t4506-pivot-named-sources (N a static name, O an OFFSET name, I an
INDEX:INDEX name, T a Table) and probes/t4506-pivot-shared-cache (three pivots, two sheets, one cache). Every
`refreshOnLoad` cache saved as `<worksheetSource name>` is refreshed on open from what the name or Table stands
for then, and every pivot over a shared cache is refreshed (pivots.md "A pivot over a defined name or a Table
reads it again at the refresh", "Every pivot over a refreshOnLoad cache is refreshed on open"). That answers the
load half of T-4664 (c); the refresh call stays T-4664's.
- Case builder (pivot_definition.py / pivot.py and the Rust mirror pivot_definition.rs / spec_pivots.rs):
  `definition.source_name` writes `worksheetSource name=` alone, and `definition.shared_cache` writes a table
  over an earlier pivot's cache, refused unless that cache is the same one. Golden parts
  t4506-pivot-named-sources__N.
- Engine: `Model::pivot_source_area` (pivot/refresh.rs) resolves a range, then a Table (any case, header and
  body, without totals), then a workbook-scope defined name, read in reference context outside a cell
  (`evaluate_reference_outside_cell`). The attribute is only ever looked up as a name, never parsed. The body
  runs under the defined-name depth and evaluation caps and its own area budget, and the record limits refuse a
  whole-sheet OFFSET before any cell is read (directive 5). `plan_pivot_refresh` now takes `&mut self` and reads
  the source through `ResolvedSource`. Tests in pivot/grid/source_tests.rs.
- Split: the common idiom `Data!$A$1:INDEX(...)` does not parse in the engine. Its probe,
  probes/t4506-pivot-index-name-corner, is an engine error owned by T-5035. The unprobed edges (a ref beside a
  name, Table totals and header-less Tables, sheet-scoped, relative and non-range names) are T-5034.
