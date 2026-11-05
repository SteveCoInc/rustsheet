---
id: T-4022
title: "xlsx import: a defined name stored as a padded bare column T[ Col ] loads as #REF!"
milestone: M8
lane: engine
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
A workbook read by the xlsx importer whose `<definedName>` body is a padded bare column (`T[ Col ]`, `T[ Col]`,
`T[Col ]`) gives `#REF!` for the name, as Excel loads it, not the column.

## Context
Follow-up of T-3258. Excel loads such a name as `#REF!` (`=SUM(N)`, `=ROWS(N)` `#REF!`, `=ISREF(N)` FALSE;
probes/t3258-name-padded-column-0..2, excel-graph 2026-11-04; syntax.md "A defined name whose stored body is a padded
bare column loads as `#REF!`"). T-3258 made `Model::define_loaded_name` (the eval harness's path) read the body as a
file's text, but `rustsheet-xlsx/src/import/mod.rs` `reparse_formula_hack` re-parses name bodies with no tables and
stores the text, which the engine later re-parses as its own text (the column). First check with a round-trip test
(spec → xlsx → `load_from_xlsx`) whether the importer indeed reads the column, then make it read the body as loaded
text once the tables exist (reuse the parser's `set_loaded`; do not copy the lexer rule).

## Acceptance criteria
- [ ] An xlsx with `<definedName name="N">T[ Col ]</definedName>` over a table `T` loads `N` as `#REF!` (test for each
      of the three spellings); `T[ [Col] ]` still loads as the column.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Names defined through the API (Name Manager typing is unprobed).
