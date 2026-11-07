---
id: T-4940
title: "UserModel undo of a row or column edit restores the data table inputs and tables it deleted"
milestone: M11
lane: engine
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Undoing a row or column delete in the `UserModel` gives back the data tables the delete changed:
an input it made `#REF!` (`DataTableInput::Deleted`) is the cell it was, and a table it deleted
with its whole region is there again.

## Context
- T-0452 made `insert_rows`, `delete_rows`, `insert_columns` and `delete_columns` move a sheet's
  data tables (crates/rustsheet-engine/src/data_table_edits.rs, docs/excel-semantics/recalc.md,
  "Data tables follow row and column edits around them").
- The `UserModel` undoes a delete by inserting the lines again and restoring their cells
  (`Diff::DeleteRows`/`DeleteColumns` in crates/rustsheet-engine/src/user_model/undo_redo.rs,
  `undo_delete_rows` in diff_undo.rs). The insert moves a table that the delete moved back where it
  was, but cannot know a `Deleted` input's cell or a dropped table. The list tables have the same
  problem solved with the `tables` snapshot in the diff (`check_table_extents`,
  `restore_table_extents`); the data tables need the same, checked as untrusted input
  (`DataTable::check`), since external diffs are untrusted
  (test/user_model/test_external_diffs_untrusted.rs).
- A range clear (or a paste) over every result cell deletes the table too (T-0452); its undo
  restores the cells but not the table.

## Acceptance criteria
- [x] Undo of a row delete and of a column delete that made an input `#REF!` restores it, and undo
      of one that deleted a table restores it with its results; redo deletes again. Unit tests.
- [x] Undo of a range clear that deleted a table restores it; redo deletes it again.
- [x] A diff whose data tables break `DataTable::check` is refused.
- [x] `make check` green; reviewers approve.

## Out of scope
- The facade (its row and column edits are harness-only and have no undo).

## Outcome
A new snapshot diff `Diff::SetDataTables` (old == new, pushed before the edit like
`SetMergedCells`) records a sheet's data tables for `UserModel::delete_rows`/`delete_columns`,
`range_clear_all` and `range_clear_contents` when the edit changes them, so undo gives back a
`#REF!` input or a deleted table and redo deletes again. Applying it checks every table with
`DataTable::check`, so a crafted diff is refused unchanged. Tests: `test_data_tables_undo.rs`.
No eval delta (UserModel only); no new Excel quirk.
