---
id: T-1521
title: "Column index upkeep: drop it after more updates than cells, skip it in undo snapshots"
milestone: M1
lane: perf
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Once a sheet's column index (`SheetData::column_rows`, T-1502) is built, heavy rewrites of the sheet (structural edits moving every cell, or a bulk re-import) cost no more than rebuilding the index on the next walk, and undo snapshots of a sheet do not copy the index.

## Context
T-1502 builds the index lazily on the first union walk of an area over more than 1,024 rows, then keeps it in step on every add/remove (`index_cell`/`unindex_cell` in sheet_data.rs). The T-1502 performance review noted: (1) insert/delete rows or columns move cells one by one (`move_cell`), each paying an O(log n) B-tree update once the index exists; dropping the index (`None`) once updates since the build exceed the cell count bounds that at the cost of a lazy rebuild (20-32 ms per 1M cells, measured in T-1502); (2) `SheetData` derives `Clone`, so `delete_sheet`'s undo snapshot (`user_model/common.rs`, `worksheet.clone()`) copies a built index; a manual `Clone` that leaves `column_rows: None` would avoid that, as it rebuilds lazily.

## Acceptance criteria
- [x] A unit test: after more index updates than cells the index is dropped and the next long union walk rebuilds it with the same results.
- [x] A cloned `SheetData` has no index; walks on it still give the same results.
- [x] `make check` green; reviewers approve.

## Out of scope
- When the index is built and which walks use it (T-1502).

## Outcome
`SheetData` now counts the writes that keep a built column index in step (`index_updates`) and drops the
index once they exceed the cells it holds, so a structural edit that moves every cell costs no more than
the lazy rebuild. `SheetData` has a manual `Clone` that leaves the index unbuilt, so `delete_sheet`'s undo
snapshot no longer copies it. Two tests in test_multi_area_arguments.rs cover both. Eval delta: none.
Review fix: `for_each_indexed_position` rebuilds a dropped index before each re-read, since a write by
`f` (a spill replacing its cells) can now drop it mid-walk and a missing index reads as no rows.
