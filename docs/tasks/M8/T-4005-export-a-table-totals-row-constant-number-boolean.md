---
id: T-4005
title: "Export a table totals-row constant (number, boolean, error) in a form Excel loads; today Excel refuses the file"
milestone: M8
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A table whose totals row holds a constant (number, boolean or error) exports in a form Excel loads.

## Context
- Evidence: docs/excel-semantics/xlsx-storage.md "Table parts in the export" (T-3081; throwaway excel-graph uploads 2026-11-04, recipe there). Excel for the web refuses (`createSession` 501 `FileCorruptTryRepair` / `unsupportedWorkbook`) our
  export of a table whose totals row has a label in A8 and the number 42 in B8: B8's `<tableColumn>` has no totals
  metadata. Patching that column to `totalsRowLabel="42"`, or to `totalsRowFunction="custom"` with
  `<totalsRowFormula>42</totalsRowFormula>`, is refused too, while the same file with a SUBTOTAL formula in B8 (custom
  metadata) loads, re-zipped or not. So a constant needs some other form: probe what Excel writes for a typed number
  in a totals cell (a desktop-authored file, or another upload variant such as the cell written as a formula `=42`).
- Code: the totals-row metadata in crates/rustsheet-xlsx/src/export/tables.rs (T-1607, T-2196).

## Acceptance criteria
- [x] Evidence for the form Excel accepts for a number, a boolean and an error constant in a totals cell, recorded in
      xlsx-storage.md; the export writes it, and an excel-graph upload of the export loads.
- [x] `make check` green; reviewers approve.

## Out of scope
- Totals-row functions other than custom.

## Outcome
- Evidence (excel-graph uploads, 2026-11-06, recorded in xlsx-storage.md "Table parts in the export"): a number,
  boolean or error constant in a totals cell loads with `totalsRowFunction="custom"` and no `<totalsRowFormula>`,
  and `Sales[[#Totals],[Val]]` reads 42, TRUE and `#N/A` with the cell kept a constant. A label on a number, or a
  `<totalsRowFormula>` of the constant, is still refused. A `=42` formula cell with its `<totalsRowFormula>` loads,
  and so does the text "42" as a label, but both change the cell.
- `totals_row_metadata` (crates/rustsheet-xlsx/src/export/tables.rs) writes the bare `custom` for `NumberCell`,
  `BooleanCell` and `ErrorCell`. New test `totals_row_constants_get_custom_metadata` (export/test/test_tables.rs)
  checks the metadata and the round trip. An excel-graph upload of that test's export (with three
  `[#Totals]` reads added) loads and reads 42, TRUE, `#N/A`.
- Eval delta: none (no eval case covers table export); no regressions.
