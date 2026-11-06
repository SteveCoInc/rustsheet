---
id: T-4671
title: "One table-overlap test and refuse a bad restored table before an undo re-inserts columns (T-4652 review follow-ups)"
milestone: M8
lane: engine
status: done
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
The table code has one rectangle-overlap test, and a column-delete undo refuses a bad restored table from a peer's
diff before it re-inserts any column.

## Context
- Review follow-ups of T-4652 (PR #3724; design and security, MINOR):
  - `Model::add_table_with_rows` (tables.rs) keeps its own inline overlap test
    (`c1 <= right && left <= c2 && r1 <= bottom && top <= r2`); T-4652 added `Bounds::overlaps` (table_edits.rs)
    with the same logic. Reuse it there (directive 3).
  - The `Diff::DeleteColumns` undo arm (user_model/undo_redo.rs) calls `restore_tables_of_sheet` after
    `undo_delete_columns`, so a refused table leaves the columns re-inserted. Split the checks out of
    `restore_tables_of_sheet` into a check that runs first (only the overlap test needs the post-insert state:
    run it on the predicted extents, or keep it after the insert and say why). The `Diff::DeleteRows` arm with
    `restore_table_extents` (T-3634) has the same order; fix both or file the row twin.
  - The `fits` closure of `restore_tables_of_sheet` is about 25 lines; a named helper reads better.

## Acceptance criteria
- [x] One overlap definition used by `add_table_with_rows` and `restore_tables_of_sheet`.
- [x] A crafted `Diff::DeleteColumns` whose `tables` entry is refused changes nothing (columns and cells included);
      unit test in test_external_diffs_untrusted.rs.
- [x] `make check` green; reviewers approve.

## Out of scope
- Formulas rewritten to `#REF!` by the delete (T-4668).

## Outcome
- `add_table_with_rows` now uses `Bounds::overlaps` (one overlap definition).
- Split `check_table_extents` / `check_tables_of_sheet` out of the two restore functions; the
  `DeleteRows` and `DeleteColumns` undo arms run them before moving any row or column. The overlap
  test of `restore_tables_of_sheet` stays after the column insert: the other tables' extents only
  settle once the columns are back. The 25-line `fits` closure is now `table_fits_restore`.
- New test `a_restored_table_naming_no_table_is_refused_before_columns_return`. No eval change.
