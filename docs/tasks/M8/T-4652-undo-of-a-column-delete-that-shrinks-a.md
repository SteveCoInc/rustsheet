---
id: T-4652
title: "Undo of a column delete that shrinks a table at its right edge restores the columns but not the table's extent"
milestone: M8
lane: engine
status: done
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
- [x] Undo of a delete of a table's rightmost column(s) restores its reference and its column names; redo shrinks it
      again. Unit tests in test_structured_references.rs.
- [x] A peer's undo diff with an extent or column list that does not fit the table is refused and changes nothing.
- [x] `make check` green; reviewers approve.

## Out of scope
- Row deletes (T-3634).

## Outcome
- `Diff::DeleteColumns` carries `tables`: the tables the delete changed but kept, whole, as they were before it
  (`Model::delete_columns_keeping_tables`, tables.rs). Its undo re-inserts the columns and then gives those tables back
  with `Model::restore_tables_of_sheet`; redo's delete changes them again. A whole table, not only a `TableExtent`,
  because a column delete drops `TableColumn`s (names, calculated-column formulas, dxf ids) and filter columns.
- `restore_tables_of_sheet` checks a peer's tables before changing anything (directive 5): the table exists on the
  sheet under that key with the same name, display name, sheet and header row count, keeps its rows, has at most one
  totals row and room for its data rows, one column per column of its reference with distinct non-empty names
  (ignoring case), filter columns inside it, and overlaps no other table. `Bounds::overlaps` (table_edits.rs) is the
  shared range test.
- The middle-column question: a delete inside the table had the column-list gap. The undo's insert inside the table
  grows it but names the column `Column1` (and the header cell is put back as the deleted name), so the table's list
  said `Column1` while the cell said `Qty`; the restored table now has the deleted column back. A delete at the left
  edge moved the table on undo (the insert at its first column shifts it right); that is fixed too.
- Tests: `undo_of_a_column_delete_restores_the_table` (right edge, two right columns, left edge, middle; undo, redo,
  undo) and `a_restored_table_is_checked` (ten refused tables and one accepted) in test_structured_references.rs.
- Found in passing, filed as T-4668: undo of a column delete does not put back the formulas the delete rewrote to
  `#REF!`, plain (`=SUM(S!D4:D7)`) or structured (`=SUM(Sales[Qty])`); the test checks `Sales[Qty]` only where Qty
  is kept. Also filed T-4667, the review follow-ups of adopted PR #3657 (T-4606).
- Eval delta: none (an API undo behavior, no eval case covers it); 0 regressions.
