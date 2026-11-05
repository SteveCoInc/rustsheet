---
id: T-4364
title: "A durable probe of criterion-less and out-of-range table filterColumns: spec options, a Txn filter-column API and the adapter"
milestone: M8
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: [T-4003]
---

## Goal
A probes case (excel-graph) holds the evidence for T-4003's rule: SUBTOTAL 3, 9, 103 and 109 over a table whose
filter is a criterion-less filterColumn, a values filter past the table's width, and a values filter inside it,
each with hidden rows; the engine passes it.

## Context
- T-4003 implemented the rule (`Table::filters_hidden_rows`, crates/rustsheet-engine/src/table_types.rs) on the
  throwaway excel-graph uploads recorded in docs/excel-semantics/xlsx-storage.md, "Table parts in the export"
  (T-3081, 2026-11-04). It split the durable case off because a spec case needs three pieces that do not exist:
  - spec2xlsx (tools/src/rustsheet_tools/spec2xlsx.py, `_add_table`): a table `"filter"` mode besides `true`
    (the first-column values filter), e.g. `"unset"` writing `<filterColumn colId="0"/>` and `"outside"` writing the
    values filter on `colId` = width + 3; ruff and mypy strict, with tests.
  - `TableSpec::filter` (crates/rustsheet-eval/src/case.rs) reading those modes, and the engine adapter
    (crates/rustsheet-eval/src/engine.rs) giving the table those filter columns.
  - a public `Txn` way to set a table's kept filter columns (crates/rustsheet/src/edit.rs; `thiserror` errors, doc
    comments), which the adapter calls; today `Txn::add_table(.., filtered)` only sets `has_filters`.
- Also probe a repeated `colId` (Excel: as the values filter) and a sortState-only autoFilter (unprobed; ours
  counts an autoFilter without filterColumns as filtered).

## Acceptance criteria
- [ ] The probe case is filled (excel-graph) and passes.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The criteria the export drops (T-4004).
