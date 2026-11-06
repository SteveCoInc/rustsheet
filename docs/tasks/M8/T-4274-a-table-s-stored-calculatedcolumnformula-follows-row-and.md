---
id: T-4274
title: "A table's stored calculatedColumnFormula follows row and column inserts and deletes as Excel saves it (today the text keeps its old references)"
milestone: M8
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
After a row or column insert or delete (and a column move, if Graph can do one) the engine's stored
`calculatedColumnFormula` text of a table column is the text Excel saves for the same edit, so an exported workbook
holds Excel's column formula.

## Context
- docs/excel-semantics/xlsx-storage.md "A calculated column is saved as `calculatedColumnFormula` plus one formula per
  row": "The stored text is not rewritten when rows or columns are inserted or deleted (an engine gap: Excel's file form
  follows the edit; unprobed here)." Today `TableColumn::calculated_column_formula` (crates/rustsheet-engine/src/
  table_types.rs) keeps the text it was given at import or by the fill (`fill_calculated_column`,
  crates/rustsheet-engine/src/calculated_columns.rs), and export writes it unchanged
  (crates/rustsheet-xlsx/src/export/tables.rs). The cells' own formulas do follow the edit, so after an insert of a
  column before A the cells say `=B2*2` while the exported column formula still says `A2*2`.
- Values are not at stake (a re-fill only checks that a column formula is set), so the evidence is the file Excel saves:
  T-2957 read `xl/tables/table1.xml` from each sidecar's cached recalculated workbook
  (eval/.cache/oracle/excel-graph/<input_sha256>.xlsx). Do the same here.
- Probe (`excel-probe` skill), on t2957-a1-reference's layout (table T on S!A1:C4, C2 entered `=A2*2` so the column is
  filled), one sheet op per case, named `t4274-calc-column-*`: insert_column A, delete_column B (a column the formula
  does not read), insert_row 1 (above the table), insert_row 3 (inside it), and a structured-reference twin
  (`=[@Col]*2`) under insert_column A, to show whether a structured column formula is rewritten at all. Keep
  FORMULATEXT of C2:C4 (or its shifted address) in each case so the cells are scored too.
- Code: the table edits on displacement (crates/rustsheet-engine/src/table_edits.rs, `table_edits`) and the formula
  rewrite they share (crates/rustsheet-engine/src/formula_rewrite.rs). The stored text is file-form A1 relative to the
  first data row; reuse the existing displacement of a parsed formula (parse, displace, stringify) rather than a text
  rewrite (directive 3).
- Related, not duplicated: T-3626 (research, todo) probes which row's text a fill typed below the first data row saves;
  its Out of scope names this edit question.

## Acceptance criteria
- [x] The `t4274-calc-column-*` cases are filled on excel-graph, their scored cells pass in `make eval` with no new F
      line in eval/baseline/probes.txt, and the saved `calculatedColumnFormula` text of each is quoted in xlsx-storage.md,
      replacing "unprobed here" (VERIFIED).
- [x] An xlsx export test (crates/rustsheet-xlsx/src/export/test/test_tables.rs) applies each probed edit to a filled
      table and asserts the exported `calculatedColumnFormula` equals Excel's saved text.
- [x] Undo of each edit restores the previous stored text (a user-model test).
- [x] `make check` green; reviewers approve.

## Out of scope
- Which row's text a fill typed below the first data row saves (T-3626).
- Whether Excel opens our exported column formula without repair, the empty `<calculatedColumnFormula/>` and the dropped
  `array="1"` attribute (the same xlsx-storage.md entry; separate questions).

## Outcome
- Probed on excel-graph (2026-11-06): probes/table-edits/t4274-calc-column-insert-column-a, -delete-column-b,
  -insert-row-1, -insert-row-3 and -structured-insert-column-a. Excel's saved `calculatedColumnFormula` follows the
  edit as the first data row's formula does: `B2*2`, `A2*2`, `A3*2`, `A2*2`, and `T[[#This Row],[Col]]*2` unchanged
  (quoted in xlsx-storage.md, replacing "unprobed here"). Also found: an insert_row inside the table fills the new
  row's cell with its own `<f>A3*2</f>` outside the shared formula.
- `Model::displace_column_formulas` (table_edits.rs) rewrites every stored column formula inside `displace_formulas`
  with a new file-form displaced printer, `to_excel_string_displaced` (stringify.rs; `excel_string` now takes the
  rewrite). Undo/redo come from the inverse edit (user-model test).
- The structured twin exposed an engine bug: cells a column insert moved with their table were reparsed outside it
  (`#ERROR!`), and a kept `#This Row` printed its area. `insert_columns` now passes the followed tables to
  `move_cell` (`Model::column_insert_kept`), and the `Rewrite::Displace` kept-table branch keeps `#This Row` text
  through column inserts as through row inserts (references.md updated; column deletes stay unprobed, T-3739).
- Tests: test_calculated_columns.rs (`the_column_formula_follows_row_and_column_edits`, user model with undo/redo;
  `a_structured_column_survives_a_column_insert`), export test_tables.rs
  (`the_exported_column_formula_follows_row_and_column_edits`).
- Eval delta: +5 probe cases, 22 cells, all pass (structured-insert-column-a F!A4 was `#ERROR!` before the fix).
- Follow-up: T-4679 (a delete that removes a cell the column formula reads, and its undo).
