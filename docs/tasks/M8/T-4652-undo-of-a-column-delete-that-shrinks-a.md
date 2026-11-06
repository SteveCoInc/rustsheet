---
id: T-4652
title: "Undo of a column delete that shrinks a table at its right edge restores the columns but not the table's extent"
milestone: M8
lane: engine
status: in-progress
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Undoing a column delete that took a table's rightmost columns restores the table's extent and its column list along
with the columns, so undo leaves the workbook exactly as it was.

## Context
- Found by T-3634 (2026-11-06), the row twin. A table `Sales` over S!B3:D7 (columns Region, Qty, Extra):
  `UserModel::delete_columns(0, 4, 1)` shrinks it to B3:C7, and `undo()` re-inserts column D and its cells but the
  table stays B3:C7. The undo of `Diff::DeleteColumns` (`user_model/diff_undo.rs` `undo_delete_columns`) inserts at
  the first deleted column, just right of the table, where an insert does not grow it.
- Unlike a row delete, a column delete also drops `TableColumn` entries (names, calculated-column formulas, dxf ids)
  and filter columns, so the diff must carry those too, not only the reference: T-3634's `TableExtent` (reference and
  totals row count, restored by `Model::restore_table_extents` with checks against a peer's diff, directive 5) is the
  model to extend; a whole-`Table` snapshot needs its own checks (column count matches the reference width, unique
  names).
- Whether a delete of a middle column has the same gap for the column list (the extent is restored by the insert
  inside the table, but the new column's name may differ from the deleted one) is part of this task to check.

## Acceptance criteria
- [ ] Undo of a delete of a table's rightmost column(s) restores its reference and its column names; redo shrinks it
      again. Unit tests in test_structured_references.rs.
- [ ] A peer's undo diff with an extent or column list that does not fit the table is refused and changes nothing.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Row deletes (T-3634).
