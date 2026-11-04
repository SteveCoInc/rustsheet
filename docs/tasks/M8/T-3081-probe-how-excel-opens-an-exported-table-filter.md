---
id: T-3081
title: "Probe how Excel opens an exported table filter: kept criteria, a criterion-less filterColumn, SUBTOTAL over its hidden rows"
milestone: M8
lane: research
status: done
priority: P2
size: M
tier: opus
depends_on: [T-2838]
---

## Goal
Oracle evidence for how Excel reads a table filter our export writes, so xlsx-storage.md "Table parts in the
export" states it instead of calling it unprobed.

## Context
Split from T-2838, which keeps a table's `<filterColumn>` criteria through import and export
(`Table::filter_columns`, `filter_column` in crates/rustsheet-xlsx/src/export/tables.rs). A table that is filtered
with no kept criterion (`Txn::add_table` with `filtered`) is still written as `<filterColumn colId="0"/>`. The probe
cases are built by the spec→xlsx builder, not by our export, so this needs a way to put an exported workbook in
front of the oracle (or a builder path that writes the same table part); find or add one first.

## Acceptance criteria
- [x] For an exported workbook with (a) a table with a kept values filter and hidden rows, (b) one with a kept
      custom filter, (c) one with a criterion-less `<filterColumn colId="0"/>`, (d) a re-exported top 10 or date-grouped filter
      (written criterion-less): whether Excel opens it without
      repair, and what `SUBTOTAL(109, ...)` and `SUBTOTAL(9, ...)` over the hidden rows give.
- [x] Edge cases the export writes unchecked: a `colId` past the table's width or repeated, and a table whose
      only filtered column was deleted (it falls back to `<filterColumn colId="0"/>`); file a task for any repair.
- [x] Whether a totals row whose cells hold constants (no label or formula metadata) loads.
- [x] docs/excel-semantics/xlsx-storage.md "Table parts in the export" updated with the evidence; follow-up
      engine/xlsx tasks filed for any difference.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Re-applying a filter (computing which rows it hides). Sort state.

## Outcome
Findings, no code change. The excel-graph uploads (2026-11-04) were of our own export: openpyxl wrote the inputs, the
engine imported and exported them, and `rustsheet-oracle recalc -b excel-graph` uploaded the result. They were
throwaway, as T-1607's were; the recipe is in xlsx-storage.md "Table parts in the export".
- Every exported filter form loads without repair: a values filter, a custom filter, a criterion-less
  `<filterColumn colId="0"/>`, a top 10 filter written back criterion-less, a `colId` past the table's width, a
  repeated `colId`, and the `colId="0"` fallback after the filtered column is deleted.
- Excel calls the hidden rows filtered only for a criterion inside the table: a values filter, a custom filter or a
  repeated colId. For those, SUBTOTAL 9 equals SUBTOTAL 109 (9 and 9, 15 and 15), and ours matches. For the
  criterion-less, past-width and fallback forms, SUBTOTAL 9 and 3 count the hidden rows (21 against 109's 15, 11 or 9;
  6 against 103's 4, 2 or 3), where ours calls them filtered. That is T-4003.
- The round trip of a top 10 filter changes Excel's SUBTOTAL 9 from 11 to 21: T-4004.
- A totals row with a constant cell (a number, or a number and a boolean) is refused as corrupt, also with
  `totalsRowLabel` or custom `totalsRowFormula` metadata for it, while a formula cell loads: T-4005.
- No eval cases: the spec builder writes only a first-column values filter, so T-4003 adds the builder option and
  the durable probe. Eval delta: none (docs and tasks only).
