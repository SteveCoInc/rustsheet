---
id: T-4692
title: "A row inserted inside a calculated column's data rows gets the column formula, as Excel fills it (T-4274 review follow-up)"
milestone: M8
lane: engine
status: todo
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
- [ ] A probe case scores the inserted row's cell (a reference written after the op, or `INDEX(S!C:C,3)` and
      FORMULATEXT), filled on excel-graph; it passes. Also the twin with a structured `=[@Col]*2` column, and an
      insert at the first data row and just above the totals row if Graph allows.
- [ ] Undo of the insert removes the filled cell. Unit test.
- [ ] xlsx-storage.md (and references.md "Calculated columns") state ours, citing the case.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The stored column-formula text (T-4274).
