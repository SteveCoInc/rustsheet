---
id: T-4554
title: "T-4445 review follow-ups: probe an API name body that uses a sheet-local name with a non-first sheet active; list the unprobed active-sheet choices in syntax.md; fix the stranded test comment and path"
milestone: M8
lane: api
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
An API-added name whose body uses a sheet-scoped name binds as Excel binds it when a non-first sheet is active, and
syntax.md lists which parts of our active-sheet handling are evidenced and which are our choice.

## Context
- PR #3557 (T-4445) review, MINOR findings (excel-fidelity, design, security):
  - Since T-4445, an API body is parsed on the active sheet but stored as written from the first sheet
    (`Model::formula_to_internal_between`, crates/rustsheet-engine/src/defined_names.rs). `qualify_unqualified_references`
    qualifies cell references only, so a body such as `=LocalOnS2` naming S2's sheet-scoped name, with S2 active, binds
    to S2's name at parse time but is stored bare and re-read from the first sheet (another binding or `#NAME?`).
    Excel's answer is unprobed.
  - docs/excel-semantics/syntax.md "An unqualified reference in an API-defined name binds to the active sheet" describes
    ours, but does not mark as unprobed: (a) the body is stored qualified, so a later change of the active sheet does not
    move an existing name; (b) renaming or deleting the bound sheet; (c) xlsx import takes the `tabSelected` sheet, not
    `activeTab`. It also cites the test in crates/rustsheet/src/edit/tests.rs; it is in edit/tests/name_binding.rs.
  - crates/rustsheet/src/edit/tests.rs: the `///` comment of the moved `an_unqualified_name_reference_binds_to_the_first_sheet`
    (T-4353 evidence) now sits on `sheet_names_are_local_and_roll_back`; move it to name_binding.rs.
  - `Model::set_active_sheet` returns Ok when the current view is missing; return an error or say why it cannot be.

## Acceptance criteria
- [ ] A probe case (excel-probe skill, excel-graph, `active_sheet` + `api_names`) adds a workbook name whose body is
      a sheet-scoped name of the active non-first sheet, and one whose body references a cell then has its sheet renamed;
      the engine matches every cell (owner tasks for any it cannot).
- [ ] syntax.md marks (a) to (c) as evidenced or UNVERIFIED and cites the right test path.
- [ ] The stranded comment is moved; `set_active_sheet`'s missing-view case is handled or explained.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Uses typed before the add (T-4391).
