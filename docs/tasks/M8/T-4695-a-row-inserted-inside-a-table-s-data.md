---
id: T-4695
title: "A row inserted inside a table's data rows gets the calculated column's formula, as Excel fills it"
milestone: M8
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
After a row insert inside a table's data rows, each calculated column's cell on the new row holds the column formula,
read at that row, as Excel fills it; today the cell is left empty.

## Context
- Evidence (excel-graph, T-4274): probes/table-edits/t4274-calc-column-insert-row-3 F!A7:A8. T on S!A1:C4, C2 typed
  `=A2*2` (Calc filled), insert_row 3: Excel's S!C3 holds `=A3*2` (FORMULATEXT via INDIRECT `=A3*2`, ISFORMULA TRUE,
  value 0), saved as its own `<f>A3*2</f>` outside the shared formula `ref="C2:C5"`. Ours: empty (`#N/A`, FALSE).
  docs/excel-semantics/xlsx-storage.md "A calculated column is saved as `calculatedColumnFormula` plus one formula
  per row".
- Code: `Model::insert_rows` (actions.rs), the table growth of `table_edits.rs` (a row insert inside the data rows grows
  the table, T-2321), and the column formula `TableColumn::calculated_column_formula` (file form; parse with
  `Model::parse_internal_formula` at the new row, or copy the neighbour row's formula as a fill would; reuse
  `calculated_columns.rs`'s fill path rather than a new writer).
- An insert at the first data row is probed too: probes/table-edits/t4274-calc-column-insert-row-2 F!A5:A6 (Excel's
  C2 holds `=A2*2`). Unprobed: an insert at the totals row, and a column whose cells hold exceptions; probe them in the
  same PR or record them as hand.

## Acceptance criteria
- [ ] t4274-calc-column-insert-row-3 F!A7:A8 and -insert-row-2 F!A5:A6 pass; their F lines leave
      eval/baseline/probes.txt.
- [ ] The inserted cell's formula survives export (an xlsx export test) and the insert's undo removes it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The saved `calculatedColumnFormula` text (T-4274) and deletes that remove a read cell (T-4679).
