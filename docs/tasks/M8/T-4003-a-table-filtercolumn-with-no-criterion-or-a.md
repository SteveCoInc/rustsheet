---
id: T-4003
title: "A table filterColumn with no criterion or a colId outside the table leaves its hidden rows unfiltered for SUBTOTAL and AGGREGATE"
milestone: M8
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
SUBTOTAL and AGGREGATE count a sheet's hidden rows as filtered only when one of its tables has a filterColumn with a
criterion inside the table, as Excel does.

## Context
- Evidence: docs/excel-semantics/xlsx-storage.md "Table parts in the export" (T-3081; throwaway excel-graph uploads 2026-11-04, recipe there). Excel for the web opens a table whose autoFilter holds only `<filterColumn colId="0"/>` (no
  criterion), or a values filter whose `colId` is past the table's width (5 on a 2-column table), but it does not call
  the hidden rows filtered: with rows 3 and 5 of `Sales` (A1:B7, values 1-6) hidden, `SUBTOTAL(9,Sales[Val])` is 21 and
  `SUBTOTAL(109,Sales[Val])` 15, `SUBTOTAL(3,Sales[Cat])` 6. Ours gives 15 for SUBTOTAL 9 and 4 for SUBTOTAL 3, since
  the import sets `has_filters` for any child of `<autoFilter>`. A values filter or custom filter inside the table, or
  a repeated colId, is filtered in both (SUBTOTAL 9 = SUBTOTAL 109).
- Code: `has_filters` in crates/rustsheet-xlsx/src/import/tables.rs (`filter.children().count() > 0`) and
  `sheet_has_filtered_table` in crates/rustsheet-engine/src/functions/subtotal.rs; `Txn::add_table(.., filtered)`
  (crates/rustsheet/src/edit.rs) and the column-delete fallback (table_edits.rs `edit_filter_columns`) also leave a
  criterion-less filter that we treat as filtered.
- Durable evidence first: the spec builder (tools/src/rustsheet_tools/spec2xlsx.py, `"filter": true`) writes only a
  values filter on the first column; add a spec option for a criterion-less filterColumn and an out-of-range colId,
  write the probes case and fill it.

## Acceptance criteria
- [ ] A probes case (excel-graph) covers a criterion-less filterColumn, a colId past the width, and a values filter,
      each with hidden rows and SUBTOTAL 3, 9, 103 and 109; it passes. (Split to T-4364: see the Outcome.)
- [x] What the API's filtered table with no criterion and the column-delete fallback should be is decided and written
      in docs/excel-semantics (numbers.md "SUBTOTAL and AGGREGATE over hidden and filtered rows").
- [x] `make check` green; reviewers approve.

## Out of scope
- Re-applying a filter. Top 10 and other criteria the export drops (T-4004).

## Outcome
- `Table::filters_hidden_rows` (crates/rustsheet-engine/src/table_types.rs) is the one rule SUBTOTAL and AGGREGATE
  read (`sheet_has_filtered_table`): a table's hidden rows are filtered when it has a kept filterColumn whose
  criterion is set and whose `colId` is inside the table, or when it is filtered with no kept column (the API's
  `filtered`, an autoFilter without filterColumns). The import reads a `<filterColumn>` with no criterion element as
  the new `FilterCriterion::Unset` (written back criterion-less, as before); a column delete that removes the last
  kept criterion leaves `Unset` on `colId` 0, the file the export writes. Evidence: the excel-graph uploads of
  docs/excel-semantics/xlsx-storage.md "Table parts in the export" (T-3081, 2026-11-04); the decision on the API and
  the column-delete fallback is in numbers.md "SUBTOTAL and AGGREGATE over hidden and filtered rows".
- Tests: engine `test_table_filter_criterion` (criterion-less, out-of-range, values, `Other`, API, column delete)
  and the import's `criterion_less_filter_columns_are_unset`.
- Split: the durable probe case needs new spec2xlsx filter modes, a public `Txn` filter-column API and the eval
  adapter, more than this size S task; T-4364 owns it (acceptance criterion 1).
- Eval delta: none (no suite case has a criterion-less or out-of-range filterColumn; verified with `make eval-check`).
