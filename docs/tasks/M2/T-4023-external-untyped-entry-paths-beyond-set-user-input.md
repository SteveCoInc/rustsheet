---
id: T-4023
title: "External untyped entry paths beyond set_user_input (redo of an external edit, set_user_array_formula, legacy formulas) read a padded bare column as Excel does"
milestone: M2
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Every path that takes formula text from outside the engine reads a padded bare column `T[ Col ]` as Excel reads typed
or loaded text (no column), and every path that re-enters the engine's own printed text reads it as the column.

## Context
Follow-up of T-3258, which split `Model::set_user_input` (external: `Parser::set_external`) from
`Model::reenter_printed_input` (the engine's own text: fill, paste, move, merge, `reenter_user_input`). Paths left as
they were, each reading engine text:
- redo of a `Diff::SetCellValue` (`user_model/undo_redo.rs`) re-enters `new_value` through `reenter_user_input`, so an
  external `UserModel::set_user_input("=SUM(T[ Col ])")` (`#ERROR!`) redoes as the column. The diff does not record
  whether its text came from outside (a paste's diff holds printed text);
- `Model::set_user_array_formula` (CSE entry) and `Model::set_cell_with_legacy_formula` parse with the default reading.
This is a design question (where to record the text's origin), hence opus. Evidence: syntax.md, probes/t2800-stored-ws-0..2.

## Acceptance criteria
- [x] Undo then redo of an external `T[ Col ]` entry gives the same cell as the entry; redo of a paste of the engine's
      printed `T[ Col ]` still reads the column (tests).
- [x] External `set_user_array_formula` and `set_cell_with_legacy_formula` read `T[ Col ]` as no column; their engine
      re-entries keep the column (tests).
- [x] `make check` green; reviewers approve.

## Out of scope
- Defined names (T-4022 for the xlsx importer).

## Outcome
- `Diff::SetCellValue` records `external`: set by `UserModel::set_user_input` (`Diff::set_cell_value`), a link label and
  pasted text (`paste_csv_string`); redo of such a diff goes through `Model::set_user_input`, every other one (fill,
  copy paste, merge) through the engine's re-entry as before.
- `Model::set_user_array_formula` and `Model::set_cell_with_legacy_formula` read text from outside (a new
  `Model::as_external`, shared with `set_user_input`); the engine's own re-entries use the new crate-internal
  `reenter_array_formula` (array formulas moved by row or column inserts, deletes and moves) and
  `reenter_legacy_formula` (`reenter_user_input`'s legacy branch). Redo of a `SetArrayValue` (only an external CSE entry
  records one) stays external.
- Tests: `external_array_and_legacy_entries_read_a_padded_bare_column_as_no_column`,
  `redo_keeps_where_a_padded_bare_column_came_from` (test_calculated_columns.rs). syntax.md's padded bare column entry
  lists the new paths.
- Eval delta: none (no eval case enters text through these paths with a padded bare column); 0 regressions.
