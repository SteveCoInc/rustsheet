---
id: T-2612
title: "A plain reference to a deleted sheet is stored and exported as #REF!C4:C7, so a same-name sheet does not revive it (T-2340)"
milestone: M8
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
After a sheet is deleted, a plain reference to it (`S!C4:C7`) loses the sheet for good, as in Excel: it stays
`#REF!` when a sheet with the old name is added later, and the xlsx export writes `#REF!C4:C7`.

## Context
T-2340 (references.md "Structured references under row deletes, column edits, convert, table and sheet
delete") matches Excel's FORMULATEXT `=SUM(#REF!C4:C7)` (probes/table-edits/table-delete-sheet) only in the
display (`stringify/wrong_reference.rs`): the stored R1C1 text keeps `S!`, so
- `add_sheet("S")` revives the reference (test_structured_references.rs
  `a_sheet_delete_is_a_ref_error_and_restore_brings_the_tables_back` pins the plain twin reading 0), and
- `to_excel_string` writes `S!C4:C7`, where Excel's file holds `#REF!C4:C7`.
The stored text keeps the name because `UserModel` undo of a sheet delete re-inserts the sheet and relies on
the other sheets' formulas reviving; a fix rewrites them on delete (as T-2340 does for structured
references, `Model::rewrite_table_references`) and records them for the undo (`TakenSheet`,
`Diff::DeleteSheet`). Probe first: does Excel revive nothing when a same-name sheet is added after a delete
(excel-graph `worksheets/add` after `DELETE worksheets`)? Also unprobed: a reference to a sheet that never
existed (a loaded `Nope!A1`), which the engine now displays as `#REF!A1`; and whether Excel accepts
`#REF!C4:C7` typed in (the lexer reads a cell-shaped tail after `#REF!` as part of it, so `#REF!LOG10(1)` lexes
as `#REF!` then `(1)`).

## Acceptance criteria
- [x] A probe (excel-graph) of delete sheet S, then add sheet S: the plain `S!C4:C7` value and FORMULATEXT.
- [x] The engine matches it; the export writes Excel's text; undo of the delete still restores the reference.
- [x] `make check` green; reviewers approve.

## Out of scope
- Desktop-only undo (T-2325).

## Outcome
- Probe probes/table-edits/t2612-delete-sheet-then-add (excel-graph, 2026-11-06): delete S, add a new blank S.
  Excel revives nothing: `SUM(S!C4:C7)`, `S!C4`, `COUNTBLANK(S!C4:C7)` stay `#REF!`; FORMULATEXT `=SUM(#REF!C4:C7)`,
  `=#REF!C4`; `ISREF(#REF!A1)` FALSE; `INDIRECT("S!C4")` reads the new sheet. The engine passes all 10 cells (it
  had 2 before: A1, A3, A5 read the new sheet, the FORMULATEXT cells kept `S!`, ISREF was TRUE).
- `delete_sheet_in_node` (stringify/sheet_edits.rs) now also turns each reference and range on the deleted sheet
  into a wrong reference to `DELETED_SHEET` (`*#REF!`, wrong_reference.rs; `*` cannot be in a sheet name, and `:`
  would have read as `#NAME?` through `wrong_sheet_error`). `is_missing_sheet` prints it as `#REF!` in the display
  and the xlsx export; only the stored R1C1 text holds the sentinel. Undo (`restore_sheet`, `UserModel`
  `Diff::DeleteSheet`) restores the old text through the existing rewritten-formula list.
- Tests: `a_sheet_added_under_a_deleted_name_does_not_revive_its_references` (sheet_undo.rs: value, formula, undo);
  T-2340's `a_sheet_delete_is_a_ref_error_and_restore_brings_the_tables_back` now pins `#REF!` for the plain twin;
  rustsheet-xlsx `deleted_sheet_reference_export` checks the saved `<f>SUM(#REF!C4:C7)+#REF!$A$1</f>`.
- references.md "A deleted table or sheet" records the probe and replaces the "Known gaps (T-2612)" sentence.
- Eval delta: probes +1 case (10 cells), all pass.
- Follow-up: T-4628 (a loaded `#REF!C4:C7` loses its range in the lexer; probe first).
