---
id: T-4221
title: "The facade's Input::LegacyFormula reads a padded bare column as loaded text (#REF!), not as an untyped parse failure"
milestone: M2
lane: api
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A legacy formula written through the facade (`Input::LegacyFormula`, a plain `<f>` as a pre-dynamic-array workbook
stores it) reads a padded bare column `T[ Col ]` as Excel loads stored text: `#REF!`, as `Input::StoredFormula` does.

## Context
- Follow-up of T-4023 (excel-fidelity review, MINOR). Since T-4023 `Model::set_cell_with_legacy_formula` reads its
  text as untyped external text, so `=SUM(T[ Col ])` is a parse failure (`#ERROR!`). The facade documents
  `Input::LegacyFormula` as stored text read "exactly as xlsx import reads a plain `<f>`", and Excel loads a stored
  padded bare column as `#REF!` (syntax.md padded bare column entry, probes/t2800-stored-ws-0..2, T-3258).
- Code: crates/rustsheet/src/edit.rs (`Input::LegacyFormula` calls `set_cell_with_legacy_formula`);
  `Model::set_loaded_user_input` (typed_input.rs) is the loaded reading `Input::StoredFormula` uses; the legacy rewrite
  is `legacy_formula_node` (legacy_formula.rs). Reuse the loaded reading (`Parser::set_loaded`), do not copy it.
- Same follow-up: syntax.md's padded bare column entry's "Implemented in" line still names only the lexer and parser;
  add `Model::as_external` (typed_input.rs), `reenter_array_formula` (model.rs), `reenter_legacy_formula`
  (legacy_formula.rs) and `Diff::SetCellValue`'s `external` (user_model/history.rs, undo_redo.rs).

## Acceptance criteria
- [ ] Through the facade, `Input::LegacyFormula("=SUM(T[ Col ])")` gives `#REF!` and `Input::LegacyFormula("=SUM(T[ [Col] ])")`
      the column's sum, with the legacy `@` rewrite kept (test in crates/rustsheet).
- [ ] syntax.md's "Implemented in" line lists the paths above.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The engine's own legacy re-entries (fill, move), which T-4023 keeps reading the column.
