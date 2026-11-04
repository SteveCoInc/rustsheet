---
id: T-4014
title: "Probe the 65-call cap for sheet-scoped names added through the API, and keep undo of a deleted over-cap name working"
milestone: M8
lane: research
status: todo
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
- [ ] A sheet-scoped `api_names` entry (spec key and both backends) and excel-graph probes at 65 and 66
  calls; syntax.md records the result, and the engine matches it.
- [ ] Undoing the delete of a name whose body nests 66 calls restores it (a unit test), or the
  edit that made it is refused, whichever the evidence supports.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop Excel's Name Manager (M14).
