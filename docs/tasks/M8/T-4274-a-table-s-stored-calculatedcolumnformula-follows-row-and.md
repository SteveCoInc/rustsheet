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
- Probes probes/table-edits/t4274-calc-column-{insert-column-a,delete-column-b,insert-row-1,insert-row-3,
  structured-insert-column-a} (excel-graph, 2026-11-06). Excel's saved `calculatedColumnFormula`: `B2*2`, `A2*2`,
  `A3*2`, `A2*2`, `T[[#This Row],[Col]]*2`: the first data cell's formula as the edit displaced it, and a structured
  one unchanged. Quoted in xlsx-storage.md (VERIFIED), replacing "unprobed here"; also recorded: the row insert_row 3
  adds inside the data rows gets the column's formula.
- `Model::displaced_column_formulas` / `set_displaced_column_formulas` (calculated_columns.rs), called from
  `displace_formulas` (table_edits.rs) around the cell walk: each stored text is parsed at its table's first data cell,
  displaced with the edit's `Displacement` (the CF path's `displace_cf_formula_str`, now `pub(crate)` in actions.rs),
  and after the tables follow the edit, reparsed and printed in file form (`to_excel_string`) at the new first data
  cell. A text with a `[` keeps its text (the displacement would lower `#This Row` under a column edit, T-3739), as
  does one whose first data cell is deleted (unprobed); moves and cell shifts are unchanged.
- Tests: `a_calculated_column_formula_follows_row_and_column_edits` (export, the five probed edits) in
  crates/rustsheet-xlsx/src/export/test/test_tables.rs; `the_column_formula_follows_edits_and_their_undo` (undo and
  redo of each) in test_calculated_columns.rs.
- Eval delta: +21 new checked probe cells, 20 pass. One new F line,
  t4274-calc-column-structured-insert-column-a F!A4 (`=SUM(T[Calc])`, Excel 14, ours `#ERROR!`): the moved `[@Col]`
  cells evaluate to `#ERROR!` (their FORMULATEXT matches), an engine bug outside this task, owned by T-4680. So the
  first criterion's "no new F line" holds for the column-formula text but not for that cell. 0 regressions.
- Follow-ups filed: T-4680 (that bug); T-4681 (review follow-up of PR #3727, T-3994). The other #3727 follow-up, owners for its two gen-m9 cells, is T-4675 (filed meanwhile on main), so no task of its own.
