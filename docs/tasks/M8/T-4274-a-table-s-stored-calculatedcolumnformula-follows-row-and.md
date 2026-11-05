---
id: T-4274
title: "A table's stored calculatedColumnFormula follows row and column inserts and deletes as Excel saves it (today the text keeps its old references)"
milestone: M8
lane: engine
status: todo
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
- [ ] The `t4274-calc-column-*` cases are filled on excel-graph, their scored cells pass in `make eval` with no new F
      line in eval/baseline/probes.txt, and the saved `calculatedColumnFormula` text of each is quoted in xlsx-storage.md,
      replacing "unprobed here" (VERIFIED).
- [ ] An xlsx export test (crates/rustsheet-xlsx/src/export/test/test_tables.rs) applies each probed edit to a filled
      table and asserts the exported `calculatedColumnFormula` equals Excel's saved text.
- [ ] Undo of each edit restores the previous stored text (a user-model test).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Which row's text a fill typed below the first data row saves (T-3626).
- Whether Excel opens our exported column formula without repair, the empty `<calculatedColumnFormula/>` and the dropped
  `array="1"` attribute (the same xlsx-storage.md entry; separate questions).
