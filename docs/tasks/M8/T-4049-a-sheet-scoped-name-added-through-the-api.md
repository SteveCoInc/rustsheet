---
id: T-4049
title: "A sheet-scoped name added through the API after load is #NAME? to an unqualified reference on its own sheet"
milestone: M8
lane: engine
status: done
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
A sheet-scoped name defined through the API after the workbook loads resolves (or not) in every cell as Excel's does,
backed by probes that tell the possible rules apart.

## Context
- probes/t4014-sheet-api-name-65 (excel-graph 2026-11-04, T-4014): N scoped to sheet S is added by Graph
  `worksheets('S')/names/add` (65 nested ABS calls), then a FullRebuild. `T!A1 =S!N` is 1, but `S!A1 =N`, typed before
  the name existed, stays `#NAME?`. The workbook-scoped twin (probes/t3264-api-name-65) gives `=N` 1. The engine gives
  S!A1 1 (a known failure of the case).
- Unknown which rule it is: a cell's unqualified name is bound when it is entered and only a workbook-scoped add
  rebinds it; or a sheet-scoped add does not dirty its own sheet's dependants; or a Graph quirk. Probe: the same with
  `S!A1` typed after the add, with a workbook-scoped N present before the sheet-scoped one, and with another cell on S
  edited after the add.
- docs/excel-semantics/syntax.md "Formula nesting depth cap" (the T-4014 paragraph); engine: `insert_defined_name`
  (crates/rustsheet-engine/src/defined_names.rs) and name resolution.

## Acceptance criteria
- [x] The probes above are filled and their rule is in syntax.md.
- [x] probes/t4014-sheet-api-name-65 S!A1 passes, or the rule is shown to be a Graph artefact and recorded as such.
- [x] `make check` green; reviewers approve.

## Out of scope
- The 65-call cap itself (T-4014).

## Outcome
- Probes (excel-graph 2026-11-05): probes/t4049-sheet-api-name-plain (sheet-scoped N `=1` added through the API, no
  workbook N) and -over-book (a workbook N `=10` in the file, then the same add), with FORMULATEXT of the cells that
  move. Rule (syntax.md "A sheet-scoped name added after load"): Excel binds names at load/entry; a sheet-scoped add
  re-binds only uses of a defined workbook N on its own sheet (`=N` and `=S!N` there move; other sheets keep the
  workbook N), and an unresolved `=N` on the sheet stays `#NAME?` (shown `=N`). Not a Graph-only artefact as far as
  Graph can show; the 65-call body plays no part (the plain `=1` body behaves the same).
- Engine: `Model::bind_added_sheet_name` (defined_names.rs), run by an API sheet-scoped `define_name`, rewrites that
  sheet's stored formulas (`expressions/parser/stringify/name_binding.rs`): an unresolved use is pinned as `[0]!N`
  (parsed back as the undefined workbook name, shown `N`; `defined_name_text` stores it so), and a sheet-qualified read
  of the workbook name becomes `S!N`. `update_defined_name`'s rename loop now shares the new
  `Model::rewrite_shared_formulas` (formula_rewrite.rs).
- Eval delta: probes/t4014-sheet-api-name-65 S!A1 now passes (case passes); the two new cases pass 13/13 and 9/9.
  Unit tests: test/test_sheet_name_added_binding.rs.
