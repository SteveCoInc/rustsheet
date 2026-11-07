---
id: T-4371
title: "Probe whether deleting a table's filtered column in Excel unfilters its hidden rows for SUBTOTAL"
milestone: M8
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Evidence for what Excel does to a filtered table's hidden rows (SUBTOTAL 9 vs 109) when the column that holds the
table's only filter criterion is deleted in the application, and the engine following it.

## Context
- Found by T-4003. The engine decided, without evidence, that such a delete unfilters the table
  (`kept_filter_inside` in crates/rustsheet-engine/src/table_edits.rs), so the engine agrees with what its export
  writes; docs/excel-semantics/numbers.md "SUBTOTAL and AGGREGATE over hidden and filtered rows" marks it unprobed.
- The spec's `sheet_ops` replay column deletes in Excel (excel-probe skill); a table with a values filter
  (spec2xlsx `"filter": true`) and SUBTOTAL formulas outside the table, then a delete of the filtered column, a delete
  of another column (the filter stays), and an insert.

## Acceptance criteria
- [x] A probes case (excel-graph) covers the three edits with SUBTOTAL 3, 9, 103 and 109; the engine passes it.
- [x] numbers.md records the result.
- [x] `make check` green; reviewers approve.

## Out of scope
- Re-applying filters.

## Outcome
- Probes (excel-graph, 2026-11-07): `probes/t4371-filtered-table-{delete-filtered-column,delete-other-column,insert-inside,
  insert-left}`, 42 cells. On a table A1:C7 whose values filter on `Cat` hides rows 3, 5, 7, **deleting the filtered
  column unfilters the table and shows the rows again** (SUBTOTAL 3 = 103 = 6, 9 = 109 = 21). Deleting a column with no
  criterion, inserting a column inside the table or left of it keeps the filter (3 = 103 = 3, 9 = 109 = 9).
- Engine: `resize_tables` (crates/rustsheet-engine/src/table_edits.rs) now unfilters an API-filtered table when a
  delete takes its first column, where its export writes the criterion (T-4363). Before, only a table whose criteria
  came from a file was unfiltered. When a column delete unfilters a table, its hidden data rows are unhidden in one
  pass over the stored rows. `UserModel::delete_columns` records the unhidden rows as `SetRowHidden` diffs after the
  delete's own diff, so undo hides them again before it restores the filter.
- Tests: `test_filtered_table_edits` covers the four edits, a hidden row outside the table, and undo/redo.
  `deleting_the_filtered_column_unfilters_the_table` (xlsx export tests) now checks that the hidden row comes back.
- numbers.md records the rule and what is still unprobed: a delete that leaves another criterion, and rows hidden by
  hand inside a filtered table (the engine shows those too).
- Eval delta: fixed 0 / new-coverage 42 cells passing (4 new probe cases, baseline +4 lines) / regressions 0. Before
  the engine change, the delete-filtered-column case failed 8 of 10 cells.
