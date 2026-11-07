---
id: T-4771
title: "Probe the saved calculatedColumnFormula when a row or column delete removes a cell it reads"
milestone: M8
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for the `calculatedColumnFormula` text Excel saves when a row or column delete removes a cell the
column formula reads (`#REF!`?), so the engine's rewrite (T-4274, `Model::displaced_column_formulas`, calculated_columns.rs) is evidenced there too,
and an engine task is filed where it differs.

## Context
- T-4274 probed inserts and a delete of a column the formula does not read (and T-4769 carried the delete of the first data row, which saves `A2*2`) (probes/table-edits/t4274-calc-column-*;
  docs/excel-semantics/xlsx-storage.md "A calculated column is saved as `calculatedColumnFormula` plus one formula per
  row"). A delete of a column or row the formula reads is unprobed: ours prints the displaced text (`#REF!*2` for a
  deleted A), and the user model's undo re-inserts the column by an inverse edit, which cannot bring the old text back.
- Probe on the t4274 layout (T on S!A1:C4, C2 typed `=A2*2`): delete_column A; and a twin with `=B2*2` reading the Other
  column, delete_column B. Read the saved `xl/tables/table1.xml` from each sidecar's cached download, as T-4274 did.
- Undo is not reachable through Graph; if Excel's text is `#REF!`-bearing, file an engine task to store the old text in
  the delete's diff (`Diff::DeleteColumns`/`DeleteRows`, user_model/history.rs).

## Acceptance criteria
- [ ] Probe cases filled on excel-graph; the saved text of each quoted in xlsx-storage.md (VERIFIED).
- [ ] Where ours differs, an engine task is filed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Inserts and deletes that do not remove a read cell (T-4274).
