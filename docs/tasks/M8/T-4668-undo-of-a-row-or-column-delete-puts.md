---
id: T-4668
title: "Undo of a row or column delete puts back the formulas the delete rewrote to #REF!"
milestone: M8
lane: engine
status: in-progress
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
- [ ] Undo of a column delete restores a plain reference and a structured reference that the delete made `#REF!`, on
      another sheet and on the edited sheet; redo makes them `#REF!` again. Unit tests.
- [ ] The row delete checked and fixed the same way if it has the gap.
- [ ] A peer's undo diff naming a cell that does not hold a formula is refused and changes nothing.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Table extents and column lists (T-3634, T-4652).
