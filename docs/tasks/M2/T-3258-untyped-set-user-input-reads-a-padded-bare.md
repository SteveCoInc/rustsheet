---
id: T-3258
title: "Untyped set_user_input reads a padded bare column T[ Col ] as the engine's own text"
milestone: M2
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A padded bare column (`T[ Col ]`, `T[ Col]`, `T[Col ]`) entered from outside the engine through the untyped
`Model::set_user_input` (also the Python bindings' path) and in a defined-name body is read as Excel reads typed or
loaded text, while the engine's own re-parse of its printed `T[ Col ]` (after edits) keeps reading the column.

## Context
Follow-up from T-3069 (PR #2415 review). T-3069 added `Lexer::set_engine_text`, set in `Parser::parse` to
`!typed && !loaded`, so only typed and loaded text refuses a padded bare column. Untyped `set_user_input` is also the
path the engine's own print-then-parse callers use (`Model::extend_to`, `extend_copied_value`,
`move_cell_value_to_area`), so they need a separate marker before external untyped input can refuse it. Evidence:
probes/t2800-stored-ws-0..2 (stored: `#REF!`, FORMULATEXT `=SUM(#REF!)`), docs/excel-semantics/syntax.md
"Whitespace beside a missing argument". A defined name's body is unprobed: probe it first (excel-probe skill).

## Acceptance criteria
- [x] Untyped external `set_user_input("=SUM(T[ Col ])")` matches Excel's typed result; the engine's own re-parse
      paths still read `T[ Col ]` as the column (tests for each).
- [x] A defined-name body `T[ Col ]` is probed and matches Excel.
- [x] `make check` green; reviewers approve.

## Out of scope
- Printing changes for this-row references (T-3241).

## Outcome
- `Parser::set_external` marks untyped text from outside the engine; the lexer reads the engine's own text only when
  the formula is neither typed, loaded nor external. `Model::set_user_input` (the public, bindings' path) sets it and
  delegates to the new `Model::reenter_printed_input`, which the engine's own re-entries now call
  (`reenter_user_input`: fill, paste, move, insert/delete rewrites, redo; merged-cell re-entry). External
  `=SUM(T[ Col ])`, `T[ Col]`, `T[Col ]` are a formula that does not parse (`#ERROR!`), as an unknown column is
  untyped; a fill of the engine's printed `T[ Col ]` still reads the column (tests in test_calculated_columns.rs and
  parser test_tables.rs).
- Probed defined-name bodies: probes/t3258-name-padded-column-0..2 (excel-graph, 2026-11-04, M8). Excel loads a name
  stored as `T[ Col ]`, `T[ Col]` or `T[Col ]` as `#REF!` (`SUM(N)`, `ROWS(N)` `#REF!`, `ISREF(N)` FALSE).
  `Model::define_loaded_name` now parses its body as loaded text (unresolved structured reference → `#REF!`); before,
  `T[ Col ]` read the column and the one-sided forms made the whole case an engine error. Recorded in syntax.md.
- Eval delta: probes +3 cases, 9 new cells, all pass; no regressions, no other cell moved (`rustsheet-eval check`).
- Follow-ups: T-4022 (the xlsx importer's name-body path, `reparse_formula_hack`), T-4023 (redo of an external entry,
  `set_user_array_formula`, legacy formulas still read the engine's text).
