---
id: T-4680
title: "A #This Row formula inside a table that a column insert moves evaluates to #ERROR! (Excel keeps reading its row)"
milestone: M8
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A `#This Row` formula in a table's data cells reads its own row after a whole-column insert left of the table moves
the table, as in Excel, instead of `#ERROR!`.

## Context
- Found by T-4274 (2026-11-06): probes/table-edits/t4274-calc-column-structured-insert-column-a (T on S!A1:C4, C2
  typed `=[@Col]*2` so the Calc column fills, then insert_column A). Excel (excel-graph): FORMULATEXT of the moved
  cells is `=[@Col]*2` and their values 2, 4, 8, so F!A4 `=SUM(T[Calc])` is 14. Ours: FORMULATEXT matches
  (F!A1:A3 pass) but each cell is `#ERROR!`, so F!A4 is `#ERROR!` (the case's one failing cell).
- A unit check on `Model` (add_table, `=[@Col]*2` in C2, `insert_columns(0, 1, 1)`) gives the same `#ERROR!` with the
  table named `T` or `Tb`, so it is the edit, not the name.
- The column-edit path keeps the pre-T-3037 rule for `#This Row` (references.md "Structured references when the
  table changes"; `Rewrite::Displace` kept-table branch in stringify/structured.rs, row inserts only); T-3739 probes
  the rule for the other edits. The cell text is kept but its parsed tree apparently is not re-resolved against the
  moved table.

## Acceptance criteria
- [x] probes/table-edits/t4274-calc-column-structured-insert-column-a F!A4 passes; its baseline F line goes.
- [x] A unit test: a `[@Col]` cell in a table moved by a column insert, and by a column delete left of it, reads its
      row.
- [x] `make check` green; reviewers approve.

## Out of scope
- The stored `calculatedColumnFormula` text (T-4274).
- Excel's rule for `#This Row` through row deletes and columns edits inside the table (T-3739).

## Outcome
- **Cause:** `insert_columns` and `delete_columns` moved every cell with no kept tables. A `[@Col]` cell was then
  re-entered at its new column while the table was still at its old place, so it parsed as an error. The displace
  step also printed a `#This Row` reference to a followed table as its area under any edit but a row insert.
- **Fix:**
  - New `Model::column_edit_kept` (table_edits.rs) lists the tables a column edit keeps. Both column paths pass that
    list to `move_cell`, so a cell moved out of a table is parsed in that table (`kept_table_holding`), as row
    inserts already did (T-3037).
  - New `Displacement::moved_tables` lists the tables a column edit left of them moves whole, with no change to
    their column list. A `#This Row` reference to one keeps its text.
  - Column edits inside a table keep today's rule: Excel's is unprobed (T-3739).
- **Evidence:** probes/table-edits/t4274-calc-column-structured-insert-column-a (excel-graph): `=[@Col]*2` keeps its
  text and reads 2, 4, 8. The column delete left of the table follows by analogy and is unprobed (recorded in
  references.md).
- **Tests:** new `a_this_row_formula_in_a_table_a_column_edit_moves_reads_its_row` (test_this_row.rs) covers the
  insert and the delete.
- **Eval delta:** probes 1 fixed (F!A4, its baseline F line removed), 0 regressions in every suite.
- **Size gate:** `Displacement` and `DataBlock` moved, unchanged apart from the new field, from stringify.rs (1004 lines with
  it) to the new stringify/displacement.rs.
- **Docs:** references.md "Structured references when the table changes" (Implemented in) and xlsx-storage.md.
