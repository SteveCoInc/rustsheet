---
id: T-4749
title: "Probe Excel on the API-filter export's empty filters (every data row hidden) and blank=1 forms (T-4363 review follow-up)"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence for the two API-filter export forms T-4363's upload did not cover: `<filters blank="1">` (a visible
blank data row) and an empty `<filters></filters>` (every data row hidden).

## Context
- Found by T-4363's excel-fidelity review (PR #3795, MINOR). T-4363 exports an API-filtered table with no kept
  criterion as a values filter on its first column listing the visible rows' text (`api_filter_column`,
  crates/rustsheet-xlsx/src/export/tables.rs); an excel-graph upload of our export with values "a" and "b" read as
  filtered (xlsx-storage.md "Table parts in the export"). The `blank="1"` form and the empty form were not uploaded;
  our import reads both as a criterion (`has_active_criterion`).
- Method: export a model with `Txn::add_table(.., filtered: true)` and hidden rows, then
  `cd tools && uv run rustsheet-oracle recalc -b excel-graph <file.xlsx>` (as T-4363 did); read SUBTOTAL 9/109 and
  3/103 over the table.

## Acceptance criteria
- [ ] Both forms uploaded; whether Excel opens them without repair and reads them as filtered is recorded in
      xlsx-storage.md with the input sha256.
- [ ] If Excel reads either as not filtered or refuses it, the export writes a form it reads as filtered (or a task
      is filed for it).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Re-applying filters.
