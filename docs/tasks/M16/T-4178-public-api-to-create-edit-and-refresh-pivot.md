---
id: T-4178
title: "Public API to create, edit and refresh pivot tables, and the Python binding"
milestone: M16
lane: api
status: done
priority: P2
size: M
tier: opus
depends_on: [T-4151]
---

## Goal
The facade creates a pivot from a definition, edits its fields and options, refreshes it, and returns its result; the Python binding mirrors it.

## Context
SPEC-0032 API section; `Txn::add_pivot_table` is doc-hidden test support today. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [x] API tests and Python binding tests.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
The pivot API is public (SPEC-0032 "API"; the as-built note there lists the details):
- `Txn::add_pivot_table(&PivotTableInput)` lays out every new pivot from the top-left cell of its
  location and refuses a name the sheet already has (`PivotError::DuplicateName`).
- `Txn::set_pivot_table(sheet, name, &PivotTableInput)` replaces a pivot's definition and cache in
  place, keeping its anchor (`PivotError::Moved`). It lays the pivot out over its old area, so the
  cells it no longer writes are cleared, and rolls back with the edit (`PivotUndo::Replaced`).
- `Txn::refresh_pivot_table` is public.
- `Workbook::pivot_table(sheet, name) -> Option<PivotView>` returns the name, sheet, location, source
  range, and row, column and data field names, read from the definition only
  (crates/rustsheet/src/pivot_view.rs). `Workbook::pivot_table_cells(sheet, name)` computes the cells
  on request from the engine's read-only `plan_pivot_render`, the grid writer's own path, `#SPILL!`
  collapse included.
- `PivotTableInput::new` fills in the case builder's defaults.
- The cells each op writes and clears go through the edit's undo log, so the `ChangeSet` of
  `Workbook::edit` lists them.
- `PivotError::Limit` and `PivotError::Source` fold in the engine's limit and source errors. A pivot
  past a limit is not added, and nothing is written (tests/pivot_api.rs).
- The eval adapter's baked pivots go through the doc-hidden `Txn::register_pivot_table`, which keeps
  the old behavior: an ungrouped pivot is left unrendered, a grouped one is laid out.
- Python: `Transaction.add_pivot_table(name, **definition)`, `set_pivot_table(sheet, name,
  **definition)`, `refresh_pivot_table(sheet, name)`, `Workbook.pivot_table(sheet, name)` (a dict)
  and `Workbook.pivot_table_cells(sheet, name)` (bindings/python/src/pivots.rs,
  tests/test_pivots.py). It covers one row field, an optional column field and one data field. The definition is passed as keywords
  so that no binding exceeds clippy's argument limit.
- pivots.md has a new entry, "A pivot created through the API". New pivots list their items in source
  order; Excel's UI order is unprobed, and T-4869 is filed for it.
