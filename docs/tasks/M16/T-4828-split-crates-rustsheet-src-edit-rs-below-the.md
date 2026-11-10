---
id: T-4828
title: "Split crates/rustsheet/src/edit.rs below the 1000-line limit (pivot and table-edit plumbing into edit/)"
milestone: M16
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
crates/rustsheet/src/edit.rs has room to grow: its pivot and table-edit plumbing moves into modules under crates/rustsheet/src/edit/, and edit.rs stays well below the 1000-line file-size limit, with no behaviour change.

## Context
edit.rs is 994 lines after T-4152, so the next API change (T-4178, T-4184) would fail scripts/check_file_size.py. T-4669 split workbook.rs and filed this as its sibling, as T-4669's context allowed. edit/ already holds pivot.rs; follow the workbook/ pattern of one `impl` block per module, moved verbatim.

## Acceptance criteria
- [x] edit.rs is under 700 lines; no public item changes path or signature; test bodies unchanged.
- [x] `make check` green
- [x] Reviewers approve.

## Out of scope
- Behaviour changes.

## Outcome
crates/rustsheet/src/edit.rs goes from 995 to 628 lines, as a verbatim move (only `use`, `mod`,
`impl` wrapper lines, one `pub(super)` and a module doc per new file differ): the sheet-list ops
(`add_sheet`, `rename_sheet`, `move_sheet`, `delete_sheet` and their `sheet_name` helper) into
edit/sheets.rs (121 lines), the table ops (`add_table`, `add_table_with_totals`,
`add_table_with_rows`, `add_data_table`) into edit/table.rs (154 lines), and the undo replay
`roll_back` into edit/rollback.rs (125 lines). No public item changed path or signature; test
counts are unchanged with and without `--all-features`, and `make check-fast` is green locally.
