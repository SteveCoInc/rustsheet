---
id: T-4940
title: "UserModel undo of a row or column edit restores the data table inputs and tables it deleted"
milestone: M11
lane: engine
status: todo
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

## Acceptance criteria
- [ ] Undo of a row delete and of a column delete that made an input `#REF!` restores it, and undo
      of one that deleted a table restores it with its results; redo deletes again. Unit tests.
- [ ] A diff whose data tables break `DataTable::check` is refused.
- [ ] `make check` green; reviewers approve.

- A range clear (or a paste) over every result cell deletes the table too (T-0452); its undo restores the
  cells but not the table. Restore it the same way.

## Out of scope
- The facade (its row and column edits are harness-only and have no undo).
