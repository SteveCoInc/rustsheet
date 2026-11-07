---
id: T-4445
title: "Txn::define_name and define_sheet_name bind an unqualified reference to the active sheet, as Graph names/add does"
milestone: M8
lane: api
status: done
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
- Visibly-off cells this task owns, where ours reads First's 1:
  - probes/t4394-api-name-active-s2 (5): First!A1, S2!A5, S2!A7, Last!A3 and Last!A4.
  - probes/t4394-api-name-active-last (5): First!A1, S2!A5, S2!A7, Last!A3 and Last!A4.
  Their typed-before uses (S2!A1, S2!A3 and Last!A1) are `#NAME?` in ours and belong to T-4391.
- probes/t4394-api-name-active-last S2!A1 (`=Rate` typed on S2 before the add) is 9 in Excel, but `#NAME?` when First
  or S2 is active (T-4353, t4394-api-name-active-s2 S2!A1). That cell is T-4391's (uses typed before the add).

## Acceptance criteria
- [x] The API has a way to set and read the active sheet (an xlsx import takes `activeTab`), and an API name binds
      its unqualified references to that sheet; unit tests for both scopes.
- [x] The eval adapter honours `active_sheet`; the cells above match Excel.
- [x] `make check` green; reviewers approve.

## Out of scope
- Uses typed before the add (T-4391).

## Outcome
- API: `Workbook::active_sheet` and `Workbook::set_active_sheet` (crates/rustsheet/src/workbook.rs), over new engine
  `Model::active_sheet`/`set_active_sheet` (the current workbook view's sheet). An xlsx import already sets that view
  from the sheet marked `tabSelected` (import/worksheets.rs); `activeTab` itself is not read, and a file whose
  `activeTab` and `tabSelected` disagree is unprobed. Export writes the view's sheet as `activeTab`, as before.
- Engine: a name added through the API (`NameSource::Api`) reads its body on the active sheet and stores it qualified
  (`Model::formula_to_internal_between`, `qualify_unqualified_references` beside `retarget_unqualified_references`), so a
  later change of the active sheet does not move it. Loaded and restored names are read as before.
- Eval: `CaseSpec::active_sheet`; the engine adapter sets it before `api_names`.
- Tests: `an_unqualified_name_reference_binds_to_the_active_sheet` (workbook scope and a name scoped to another sheet,
  a second add after the active sheet changes, an unknown sheet refused); the first-sheet test still passes (the
  default active sheet is the first).
- Eval delta: probes -10 known failures (t4394-api-name-active-s2 and -last: First!A1, S2!A5, S2!A7, Last!A3, Last!A4
  each); 0 regressions. The typed-before cells (S2!A1, S2!A3, Last!A1) stay with T-4391.
- syntax.md's entry now describes ours.
