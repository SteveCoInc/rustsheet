---
id: T-4014
title: "Probe the 65-call cap for sheet-scoped names added through the API, and keep undo of a deleted over-cap name working"
milestone: M8
lane: research
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Oracle evidence for the 65-call cap on a sheet-scoped name added through the API, and an undo of a
deleted name that never fails on the cap.

## Context
- T-3264 probed workbook-scoped Graph `workbook/names/add` (probes/t3264-api-name-65 accepted,
  -66 refused) and made `Model::define_name` refuse a body over 65 calls on every non-loading path,
  so `Txn::define_sheet_name` too, by extension (comment in `insert_defined_name`,
  crates/rustsheet-engine/src/defined_names.rs; syntax.md "Formula nesting depth cap").
- Graph has `worksheets('S')/names/add`; the harness's `api_names` (tools/src/rustsheet_tools/api_names.py)
  defines workbook-scoped names only.
- `update_defined_name` is unchecked (Graph has no endpoint that sets a body), so a body edited to 66
  calls can exist; deleting it and undoing the delete goes through `new_defined_name`
  (user_model/undo_redo.rs, `Diff::DeleteDefinedName`), which now refuses it (T-3264 fidelity review).

## Acceptance criteria
- [x] A sheet-scoped `api_names` entry (spec key and both backends) and excel-graph probes at 65 and 66
  calls; syntax.md records the result, and the engine matches it.
- [x] Undoing the delete of a name whose body nests 66 calls restores it (a unit test), or the
  edit that made it is refused, whichever the evidence supports.
- [x] `make check` green; reviewers approve.

## Out of scope
- Desktop Excel's Name Manager (M14).

## Outcome
- Harness: an `api_names` entry takes an optional `"sheet"` (tools/src/rustsheet_tools/api_names.py `ApiName`, recorded
  as `[name, formula, sheet]`); excel-graph replays it with `worksheets('S')/names/add`, the eval adapter with
  `Txn::define_sheet_name` (`ApiName::sheet` in crates/rustsheet-eval/src/case.rs). Tests in
  tools/tests/test_spec2xlsx_api_names.py and crates/rustsheet-eval/src/engine/tests.rs.
- Evidence (excel-graph 2026-11-04): probes/t4014-sheet-api-name-65 is ADDED (`T!A1 =S!N` is 1) and -66 REFUSED with
  400 `InvalidArgument` (`S!N` is `#NAME?`): the cap holds for sheet-scoped names, and the engine already matched it
  (comment in `insert_defined_name` now cites the probe; unit test
  `a_sheet_scoped_name_takes_65_nested_calls_and_not_66`). Recorded in syntax.md "Formula nesting depth cap".
- Quirk found on the way: after the 65-call add, `S!A1 =N` on the name's own sheet stays `#NAME?` (the workbook-scoped
  twin gives 1). The engine gives 1: the case's one known failure, owned by the new T-4049 (rule unknown; probes listed
  there).
- Undo: an undo of a delete goes through the new `Model::restore_defined_name`, which skips the API cap and the
  reference-or-LAMBDA body rule (the name existed); unit test `undoing_the_delete_of_an_over_cap_name_restores_it`.
- Eval delta: 2 new cases (6 cells, 5 bit-exact, 1 new-coverage failure owned by T-4049); no regressions.
