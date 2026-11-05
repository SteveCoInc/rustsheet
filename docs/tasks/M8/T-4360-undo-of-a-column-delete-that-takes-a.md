---
id: T-4360
title: "Undo of a column delete that takes a table's right columns restores the cells but not the table's extent or columns"
milestone: M8
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Undoing a column delete that took a table's right columns gives the table back its extent and its columns (names,
ids, filter columns), so undo leaves the workbook exactly as it was.

## Context
- Found by T-3634 (2026-11-05), the row-delete counterpart. `UserModel::delete_columns(0, 3, 1)` on the
  `edge_model` table `Sales` (B3:C7, test_structured_references.rs) leaves B3:B7 with one column; `undo()` re-inserts
  column C and its cells, but the table stays B3:B7 with one column. The undo of `Diff::DeleteColumns`
  (`user_model/diff_undo.rs` `undo_delete_columns`) inserts at the first deleted column, just right of what is left
  of the table, and an insert there does not grow it.
- T-3634 added `Diff::SetTableExtents` (reference and totals row count, restored after the rows are back) for row
  deletes. A column delete also drops table columns (`table_edits.rs` `edit_columns`, `edit_filter_columns`), so the
  snapshot needs the column list too, with the same untrusted-peer checks as `undo_table_extents`.

## Acceptance criteria
- [ ] Undo of a delete of a table's right columns restores its reference, columns and filter columns; redo deletes
      them again. Unit tests next to T-3634's in test_structured_references.rs.
- [ ] A crafted extent from a peer cannot move, widen past its columns or overlap a table
      (test_external_diffs_untrusted.rs).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Row deletes (T-3634).
