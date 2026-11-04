---
id: T-4023
title: "External untyped entry paths beyond set_user_input (redo of an external edit, set_user_array_formula, legacy formulas) read a padded bare column as Excel does"
milestone: M2
lane: engine
status: todo
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
- [ ] Undo then redo of an external `T[ Col ]` entry gives the same cell as the entry; redo of a paste of the engine's
      printed `T[ Col ]` still reads the column (tests).
- [ ] External `set_user_array_formula` and `set_cell_with_legacy_formula` read `T[ Col ]` as no column; their engine
      re-entries keep the column (tests).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Defined names (T-4022 for the xlsx importer).
