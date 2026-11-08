---
id: T-5079
title: "Deleting a sheet drops its sheet-scoped defined names, so the save no longer fails"
milestone: M15
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
A workbook whose deleted sheet had sheet-scoped defined names saves again. Today `Txn::delete_sheet` keeps a
deleted sheet's sheet-scoped names (`Print_Area`, `Print_Titles`, `_FilterDatabase`, user names), so the export
refuses the workbook: `defined name `Print_Area` is scoped to sheet id 33, which is not in the workbook`.

## Context
- T-5005's `R2/delete` scenario found it. It deletes the first sheet of every corpus case, and 78 cases fail to save
  (calipers 33, corpus-bench 36, poi 9, e.g. `roundtrip-calipers/pv_goldens/pv23_010/golden.xlsx`). They are
  recorded as `R2/delete scenario error` known failures in `eval/baseline/roundtrip-*.txt`, and this fix removes them by
  a re-record.
- The engine's `Model::take_sheet` (`crates/rustsheet-engine/src/sheet_undo.rs`) takes the sheet's tables and pivots
  and rewrites formulas to `#REF!`, but leaves `workbook.defined_names` entries whose `sheet_id` is the deleted sheet.
  `restore_sheet` (undo) must put them back.
- What Excel does with a deleted sheet's scoped names, and with a workbook-scoped name that pointed at it (`#REF!`),
  is to be probed (`excel-probe`) and recorded in docs/excel-semantics before the behaviour is fixed. Never write a
  name scoped to a missing sheet (directive 5: the save must not fail on an engine state an edit can reach).

## Acceptance criteria
- [ ] After `delete_sheet`, no defined name is scoped to the deleted sheet, as the probe shows Excel does; undo
      restores them (unit tests in the engine and the facade).
- [ ] The `R2/delete` scenario errors are gone from the round-trip baselines (re-record; entries only shrink).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The save report's `OwnerDeleted` entries for the deleted sheet's parts (T-5004).
