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
- [ ] One overlap definition used by `add_table_with_rows` and `restore_tables_of_sheet`.
- [ ] A crafted `Diff::DeleteColumns` whose `tables` entry is refused changes nothing (columns and cells included);
      unit test in test_external_diffs_untrusted.rs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Formulas rewritten to `#REF!` by the delete (T-4668).

## Outcome
`add_table_with_rows` now uses `Bounds::overlaps` (new `Bounds::range`), so the table code has one overlap test.
`restore_tables_of_sheet` is split into `check_tables_of_sheet` (changes nothing; its structural checks moved into the
named `restorable` helper) and the apply; the `Diff::DeleteColumns` undo arm runs the check first, with the overlap test
on the other tables where the column insert will have put them (`Bounds::after_column_insert`, via `column_span`), so a
refused table leaves the columns deleted. The `Diff::DeleteRows` arm does the same with the new
`check_table_extents` (no row twin needed). Tests: `the_restored_tables_are_checked_before_the_columns_come_back`
(model level, incl. the predicted shift) and `a_restored_table_naming_no_table_is_refused` (external diff). No eval change.
