---
id: T-4587
title: "A bare name in an API-added name body binds to its sheet-scoped definition on the alphabetically first sheet, else the workbook name (probes/t4554-api-name-*)"
milestone: M8
lane: api
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A name added through the API (`Txn::define_name`, `Txn::define_sheet_name`) whose body is or contains a bare defined
name reads the definition Excel binds it to: a sheet-scoped definition of that name wins over the workbook-scoped one,
and among sheet-scoped definitions the one on the alphabetically first sheet name wins, whichever sheet is active.

## Context
- Evidence (excel-graph, 2026-11-06, T-4554). A book of First, S2 and Last, with B1 = 1, 7, 9 and B2 = 10, 70, 90.
  Graph `workbook/names/add` of `W_x` = `=Dupx` gives the same value on every sheet, and the same with First, S2 or
  Last active (probes/t4554-api-name-scoped-name-choice, -first, -last, D1:D9 on each sheet):
  - workbook + S2: S2's (70)
  - workbook + Last: Last's (90)
  - workbook + First + S2: First's (1)
  - First + Last: First's (1)
  - Last only: Last's (90)
  - First only: First's (1)
  - workbook + S2 + Last: Last's (90)
  - S2 + Last: Last's (90)
  - workbook + First: First's (1)
  
  The order First < Last < S2 is alphabetical, not sheet order (First, S2, Last). probes/t4554-api-name-local-name-body
  agrees: `WL` = `=LocLoc` (S2's only) is 7, `WD` = `=Dup` (workbook + S2 + Last) is 90, and the Last-scoped `LD` =
  `=Dup` is 90.
- Ours: `insert_defined_name` (crates/rustsheet-engine/src/defined_names.rs) reads an API body on the active sheet
  (`formula_to_internal_between`). That resolves a bare name to the active sheet's definition, else the workbook one.
  `qualify_unqualified_references` then qualifies cell references only, so the name is stored bare and re-read from
  the first sheet: `#NAME?` for a name only another sheet defines, the workbook definition otherwise.
- The fix is probably to qualify each bare `DefinedNameKind` of an API body with the chosen sheet (`Sheet!Name`) before
  it is stored, so a later sheet rename rewrites it.
- docs/excel-semantics/syntax.md "An unqualified reference in an API-defined name binds to the active sheet" records
  the cells.

## Acceptance criteria
- [x] These cells pass (all but probes/t4554-api-name-local-name-body Last!D3, which is not a binding cell: see Outcome;
      T-4618 owns it):
  - probes/t4554-api-name-local-name-body: First!D1:D3, S2!D1:D3, Last!D1:D3.
  - probes/t4554-api-name-scoped-name-choice: First!D1, First!D2, First!D5, First!D7, First!D8, S2!D1, S2!D2, S2!D5, S2!D7, S2!D8, Last!D1, Last!D2, Last!D5, Last!D7, Last!D8.
  - probes/t4554-api-name-scoped-name-choice-first: First!D1, First!D2, First!D5, First!D7, First!D8, S2!D1, S2!D2, S2!D5, S2!D7, S2!D8, Last!D1, Last!D2, Last!D5, Last!D7, Last!D8.
  - probes/t4554-api-name-scoped-name-choice-last: First!D1, First!D2, First!D5, First!D7, First!D8, S2!D1, S2!D2, S2!D5, S2!D7, S2!D8, Last!D1, Last!D2, Last!D5, Last!D7, Last!D8.
- [x] The name order is case-insensitive, or a probe shows otherwise (sheets `b` and `A`), and the case is recorded in
      syntax.md.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Uses typed before the add (T-4391).
- Bodies that are a LAMBDA call or a name inside LET (unprobed).

## Outcome
- The parser reads a bare defined name of an API-added body (`Parser::set_api_name_body`, set by
  `Model::insert_defined_name` for `NameSource::Api`) as `Sheet!Name` on the first defining sheet by lower-cased
  name (`Parser::api_body_sheet_name`, crates/rustsheet-engine/src/expressions/parser/defined_names.rs). The body is
  stored qualified, so a sheet rename rewrites it; a name only the workbook defines stays bare. `bind_bare_name`
  (operands.rs) now checks the written identifier for a prefix, so a LET name or LAMBDA parameter of such a body
  still declares the variable. Unit tests: crates/rustsheet-engine/src/test/test_api_name_bare_name_binding.rs.
- Name order: probes/t4587-api-name-sheet-order-case (excel-graph 2026-11-06; tabs Z, B, a, _u) shows it ignores
  case (`a` before `B`, which an ordinal compare reverses; this covers the `b`/`A` question, which cannot tell the
  two apart) and puts `_` before the letters. Recorded in syntax.md.
- Last!D3 of t4554-api-name-local-name-body (`=LD` on Last, loaded before the Last-scoped add) is not about body
  binding: it is the T-4049 rule ("A sheet-scoped name added after load" leaves such a use `#NAME?`), which Excel
  breaks here. Four probes (t4587-sheet-api-name-inactive-sheet, -sheet-has-names, -name-body,
  -qualified-use-first) keep the rule and do not explain it; recorded in syntax.md, handed to T-4618.
- Eval delta: probes known failures 6282 -> 6229 (53 cells fixed, 0 regressions); 5 new probe cases (27 cells), all
  passing.
