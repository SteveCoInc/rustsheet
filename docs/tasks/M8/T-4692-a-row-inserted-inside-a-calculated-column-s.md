---
id: T-4692
title: "A row inserted inside a calculated column's data rows gets the column formula, as Excel fills it (T-4274 review follow-up)"
milestone: M8
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A whole-row insert inside a calculated column's data rows gives the new row's cell the column's formula, as Excel
does, and a probe cell scores it.

## Context
- Found by T-4274 (PR #3738, excel-fidelity review): in probes/table-edits/t4274-calc-column-insert-row-3 (T on
  S!A1:C4, C2 typed `=A2*2`, then insert_row 3) Excel's cached workbook has `<c r="C3" s="1"><f>A3*2</f>` (value 0):
  the inserted data row gets the column formula. Ours leaves S!C3 empty (`get_cell_formula(0,3,3)` is `None` on a
  user-model copy of the layout). The probe does not score it: F's references to S are displaced too, so F!A2 reads
  S!C4. xlsx-storage.md's calculatedColumnFormula entry records Excel's side (T-4274).
- Engine: the row insert in `Model::displace_formulas` / `resize_tables` (table_edits.rs) and the fill in
  calculated_columns.rs (`fill_calculated_column` writes a formula index into data cells).

## Acceptance criteria
- [x] A probe case scores the inserted row's cell (a reference written after the op, or `INDEX(S!C:C,3)` and
      FORMULATEXT), filled on excel-graph; it passes. Also the twin with a structured `=[@Col]*2` column, and an
      insert at the first data row and just above the totals row if Graph allows.
- [x] Undo of the insert removes the filled cell. Unit test.
- [x] xlsx-storage.md (and references.md "Calculated columns") state ours, citing the case.
- [x] `make check` green; reviewers approve.

## Out of scope
- The stored column-formula text (T-4274).

## Outcome
- A whole-row insert that adds rows to a table's data rows (inside them, at the first data row, or at the totals
  row) fills each calculated column's stored formula into the new rows, as Excel does
  (`Model::fill_inserted_data_rows`, called from `Model::displace_formulas` after the tables follow the insert).
  Undo deletes the inserted rows, so the filled cells go with them; redo fills again (unit test
  `an_inserted_data_row_gets_the_column_formula`; a plain column fills nothing, `an_inserted_row_fills_no_plain_column`).
- The stored column formula after an insert at the first data row is now `A2*2`, as Excel saves it (the cached
  `table1.xml` of t4692-insert-row-first-data); before, it was the moved cell's `A3*2`
  (`set_displaced_column_formulas` reads the displaced text where the first data cell moved).
- Probes (excel-graph 2026-11-06): probes/table-edits/t4692-insert-row-inside, -insert-rows-inside (two single
  inserts; the oracle's sheet op has no `count`), -insert-row-first-data, -insert-row-totals,
  -structured-insert-row-inside: 60 cells, all pass (all five fail without the fill). The cells read through
  `INDEX(S!C:C,n)`, which the insert does not displace.
- Docs: xlsx-storage.md ("A calculated column is saved as `calculatedColumnFormula`...") and references.md
  ("Calculated columns") state Excel's rule and ours. Excel saves each filled cell with `s="1"`; ours keeps the cell's
  own style (unscored).
- Eval delta: +5 probe cases, +60 passing cells; known failures 6392 -> 6392, regressions 0.
