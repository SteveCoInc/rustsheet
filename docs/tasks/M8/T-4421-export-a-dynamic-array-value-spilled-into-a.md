---
id: T-4421
title: "Export a dynamic-array value spilled into a table totals cell in a form Excel loads; reword xlsx-storage.md's constant-totals sentence"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4005]
---

## Goal
A table whose totals row holds a value spilled from a dynamic array exports in a form Excel loads, and xlsx-storage.md
describes the constant-totals export as it now is.

## Context
- From the Excel-fidelity review of T-4005 (PR #3449), MINOR.
- `totals_row_metadata` (crates/rustsheet-xlsx/src/export/tables.rs) writes bare `totalsRowFunction="custom"` for
  number, boolean and error constants since T-4005. A `Cell::SpillCell` (a value spilled into the totals row from
  a dynamic array outside it) still gets no metadata, and T-3081's evidence says Excel refuses a totals row whose
  cells lack it. Probe it with excel-graph uploads (recipe in xlsx-storage.md "Table parts in the export"): a
  spill from above the table, or from a cell beside the totals row, into a totals cell. Try bare `custom`, and
  check whether Excel allows a spill into a totals row at all (`#SPILL!`).
- xlsx-storage.md "Table parts in the export" still opens with "A totals-row cell holding a number, boolean or
  error constant gets no metadata ... and Excel refuses that file". Put it in the past tense ("got no metadata
  until T-4005").

## Acceptance criteria
- [ ] Evidence for a spilled value in a totals cell is recorded in xlsx-storage.md; the export writes the form Excel
      loads, and an excel-graph upload of the export loads.
- [ ] The constant-totals sentence describes the export as of T-4005.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
