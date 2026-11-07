---
id: T-4668
title: "Undo of a row or column delete puts back the formulas the delete rewrote to #REF!"
milestone: M8
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
Undo of a column (or row) delete puts back the text of every formula the delete rewrote to `#REF!`, so undo leaves
the workbook exactly as it was.

## Context
- Found by T-4652 (2026-11-06). On `S` with a table `Sales` over B3:D7 and `=SUM(S!D4:D7)` on `F`,
  `UserModel::delete_columns(0, 4, 1)` rewrites the formula to `=SUM(S!#REF!)` (as Excel does), and `undo()` re-inserts
  column D but the formula stays `=SUM(S!#REF!)`. A structured reference to a deleted table column
  (`=SUM(Sales[Qty])` after Qty's delete is `=SUM(#REF!)`) stays the same way, although T-4652 restores the table.
- The undo of `Diff::DeleteColumns` (`user_model/diff_undo.rs` `undo_delete_columns`) restores only the deleted
  columns' own cells; the formulas elsewhere that `Model::displace_formulas` rewrote are not in the diff. The sheet
  delete already solves this with `RewrittenFormula` (`formula_rewrite.rs`, `restore_rewritten_formulas`, carried in
  `Diff::DeleteSheet`): the column and row deletes need the same, collected by the displacement walk, with
  directive 5 checks on a peer's diff (each restored cell exists and held a formula).
- Whether a row delete has the same gap (`delete_rows`, `Diff::DeleteRows`) is part of this task to check; formulas
  in conditional formats and defined names too.

## Acceptance criteria
- [x] Undo of a column delete restores a plain reference and a structured reference that the delete made `#REF!`, on
      another sheet and on the edited sheet; redo makes them `#REF!` again. Unit tests.
- [x] The row delete checked and fixed the same way if it has the gap.
- [x] A peer's undo diff naming a cell that does not hold a formula is refused and changes nothing.
- [x] `make check` green; reviewers approve.

## Out of scope
- Table extents and column lists (T-3634, T-4652).

## Outcome
Done 2026-11-06. Undo of a row or column delete now puts back every cell formula the delete rewrote past what
re-inserting the lines undoes, so the workbook's formulas are as they were.
- `Model::shift_cell_formula` asks `Model::lost_on_reinsert` (`formula_rewrite.rs`) of each formula a row or column
  delete changes: it re-displaces the new text by the inverse insert and compares it with the old. A reference made
  `#REF!` (plain, or a structured reference to a deleted table column) and a range that lost an edge (`D1:E1` →
  `D1:D1`, which the re-insert makes `E1:E1`) are lost; a plain shift (`E1` → `D1`, `B1:F1` → `B1:E1`) is not and is
  not recorded, so the record stays small.
- The cell as it was is kept as a `RewrittenCell` at its place before the delete (a cell the delete moved is taken
  before the move, `MovedFormulas`/`before_moves`), returned by `Model::delete_columns_rewriting` /
  `delete_rows_rewriting` (the public `delete_columns` / `delete_rows` drop it) and carried in a new `rewritten`
  field of `Diff::DeleteColumns` and `Diff::DeleteRows`. The undo checks them first (`checked_rewritten_cells`: each
  passes `restored_cell`, holds a formula, and the place the delete moved it to holds a formula now; one inside the
  deleted lines or on no sheet is refused) and writes them back after the re-insert. Redo's delete rewrites them again.
- The row delete had the same gap (`=SUM(S!A4:D4)` stayed `#REF!`); fixed the same way.
- Tests: `test/user_model/test_delete_undo_formulas.rs` (column and row delete, other sheet and edited sheet, moved
  and unmoved cells, redo; a shifted formula is not recorded; six crafted peer diffs refused with the workbook
  unchanged). T-4652's `undo_of_a_column_delete_restores_the_table` now asserts `=SUM(Sales[Qty])` comes back for
  every delete, including the one that deleted Qty.
- No Excel quirk: Excel's undo restores the workbook as it was. Eval delta: none (the eval drives `Model`, not
  `UserModel` history); `make check-fast` green, full engine lib tests green.
- Follow-ups: T-4702 (conditional formats and defined names a delete rewrites; `displace_formulas` does not walk the
  defined names today). T-4703 is a probe task from the review of PR #3693 (T-4594), filed here.
