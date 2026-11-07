---
id: T-4988
title: "PivotChart export: a regenerated save keeps a pivot-bound drawing on a sheet whose row or column records changed without a structural edit"
milestone: M16
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: [T-4950]
---

## Goal
A save that regenerates the package keeps a sheet's PivotChart drawing when the sheet changed only in ways that do
not move cells (a hidden or resized row or column, a frozen pane, a view), and still drops it after a row or column
insert or delete (decision 0098's harness-only edits) until T-4567's spec says how a drawing moves.

## Context
T-4950 (crates/rustsheet-xlsx/src/export/pivot_drawings.rs) keeps a drawing only when the sheet's every record but
its cells equals the source's (`patch::sheet_structure_same`): the model holds no trace of a structural edit, so the
comparison stands in for one. It is safe but too strict: `Txn::set_row_hidden` on a pivot sheet drops its charts
(crates/rustsheet/tests/pivot_chart_save.rs, `a_drawing_on_a_renamed_restructured_or_moved_sheet_is_dropped`).
Telling a structural edit apart needs a signal the save can read, such as a per-sheet record the engine's
`insert_rows`/`delete_rows`/`insert_columns`/`delete_columns` leave, passed to `save_preserving`.

## Acceptance criteria
- [ ] A hidden row, a column width or a frozen pane on a pivot sheet keeps its drawing in a regenerated save; a row or
  column insert or delete still drops it with a log line; tests show both.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Moving a drawing's anchors with a structural edit (T-4567's spec, then M15's replay layer).
