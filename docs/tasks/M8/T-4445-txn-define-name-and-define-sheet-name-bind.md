---
id: T-4445
title: "Txn::define_name and define_sheet_name bind an unqualified reference to the active sheet, as Graph names/add does"
milestone: M8
lane: api
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A name defined through the API with an unqualified reference body binds to the workbook's active sheet, as Excel's
Graph `names/add` does, not always to the first sheet.

## Context
- T-4394 (docs/excel-semantics/syntax.md "An unqualified reference in an API-defined name binds to the active sheet"):
  in T-4353's book of First, S2 and Last, `=$B$1` and `=$B$1:$B$2` added with Graph `names/add` (workbook scope, or
  scoped to S2 or Last) read the active sheet's cells. probes/t4394-api-name-active-s2 (S2 active) has every use 7 or
  77, and probes/t4394-api-name-active-last (Last active) has every use 9 or 99.
- Ours: `Txn::define_name` and `Txn::define_sheet_name` (crates/rustsheet/src/edit.rs) parse the body against the
  first sheet (`an_unqualified_name_reference_binds_to_the_first_sheet`, crates/rustsheet/src/edit/tests.rs). The
  `rustsheet` API has no active sheet. The engine's user model has one (`get_selected_sheet`/`set_selected_sheet`,
  crates/rustsheet-engine/src/user_model/ui.rs), and xlsx export writes `<workbookView activeTab>` from the first view.
  Whether xlsx import reads `activeTab` is unchecked.
- The eval adapter (crates/rustsheet-eval/src/engine.rs `evaluate_spec`) ignores a case's `active_sheet` (spec2xlsx
  writes it, T-4394). It must set the active sheet before `define_api_names` for these cases to be scored on the rule.
- Visibly-off cells this task owns: probes/t4394-api-name-active-s2 First!A1, Last!A1, Last!A3, Last!A4, S2!A3, S2!A5
  and S2!A7; probes/t4394-api-name-active-last First!A1, Last!A1, Last!A3, Last!A4, S2!A3, S2!A5 and S2!A7. Ours reads
  First's 1 or 11 there.
- probes/t4394-api-name-active-last S2!A1 (`=Rate` typed on S2 before the add) is 9 in Excel, but `#NAME?` when First
  or S2 is active (T-4353, t4394-api-name-active-s2 S2!A1). That cell is T-4391's (uses typed before the add).

## Acceptance criteria
- [ ] The API has a way to set and read the active sheet (an xlsx import takes `activeTab`), and an API name binds
      its unqualified references to that sheet; unit tests for both scopes.
- [ ] The eval adapter honours `active_sheet`; the cells above match Excel.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Uses typed before the add (T-4391).
