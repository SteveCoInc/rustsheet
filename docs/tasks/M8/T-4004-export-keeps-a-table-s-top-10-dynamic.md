---
id: T-4004
title: "Export keeps a table's top 10, dynamic, date-group, color and icon filter criteria so Excel still calls the rows filtered"
milestone: M8
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
An exported table keeps its top 10, dynamic, date-group, color and icon filter criteria, so Excel still calls its hidden
rows filtered after a round trip.

## Context
- Evidence: docs/excel-semantics/xlsx-storage.md "Table parts in the export" (T-3081; throwaway excel-graph uploads 2026-11-04, recipe there). A table with a top 10 filter (top 2 items of `Val`), imported and exported, is written as
  `<filterColumn colId="1"/>`. Excel for the web then counts its hidden rows as manually hidden:
  `SUBTOTAL(9,Sales[Val])` is 21 where the original filter made it 11 (SUBTOTAL 109 is 11 in both), and
  `SUBTOTAL(3,Sales[Cat])` is 6 where it was 2. So the round trip changes Excel's values.
- Code: `filter_column` in crates/rustsheet-xlsx/src/import/tables.rs and crates/rustsheet-xlsx/src/export/tables.rs
  (`FilterColumn`, crates/rustsheet-engine/src/table_types.rs). Keeping the criterion as validated, bounded data
  (directive 5) is enough; nothing needs to re-apply it.

## Acceptance criteria
- [ ] Top 10, dynamic, color, icon and date-group criteria survive import, export and import (unit tests on the
      written XML), and the exported workbook gives Excel's SUBTOTAL values (an excel-graph upload or a probes case).
- [ ] xlsx-storage.md "Table parts in the export" updated.
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Re-applying a filter. The criterion-less and out-of-range forms (T-4003).
