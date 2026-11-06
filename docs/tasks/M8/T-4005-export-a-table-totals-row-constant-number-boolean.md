---
id: T-4005
title: "Export a table totals-row constant (number, boolean, error) in a form Excel loads; today Excel refuses the file"
milestone: M8
lane: engine
status: in-progress
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
- [ ] Evidence for the form Excel accepts for a number, a boolean and an error constant in a totals cell, recorded in
      xlsx-storage.md; the export writes it, and an excel-graph upload of the export loads.
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Totals-row functions other than custom.
