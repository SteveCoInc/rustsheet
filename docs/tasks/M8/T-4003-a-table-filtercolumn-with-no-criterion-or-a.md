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
- [x] A probes case (excel-graph) covers a criterion-less filterColumn, a colId past the width, and a values filter,
      each with hidden rows and SUBTOTAL 3, 9, 103 and 109; it passes.
- [x] What the API's filtered table with no criterion and the column-delete fallback should be is decided and written
      in docs/excel-semantics (numbers.md "SUBTOTAL and AGGREGATE over hidden and filtered rows").
- [ ] `make check` green; reviewers approve.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Re-applying a filter. Top 10 and other criteria the export drops (T-4004).

## Outcome
- Durable evidence: spec2xlsx gains `"filter_column": {"col_id", "criterion"}` (`FilterColumnSpec` in
  crates/rustsheet-eval/src/case.rs, docs/architecture/eval-harness.md), and probes/t4003-filter-column-criterion
  (5 sheets, 30 cells, excel-graph 2026-11-05) covers a values filter, a criterion-less filterColumn on columns 0 and
  1, a values filter under `colId` 5 of a 2-column table, and a table without a filter, each with SUBTOTAL 3, 103, 9,
  109 and AGGREGATE(9,1) and (9,4). Excel calls the hidden rows filtered only for the values filter inside the table.
- The xlsx import sets `has_filters` only for a filterColumn with a criterion element and a `colId` inside the table
  (`has_active_criterion`, crates/rustsheet-xlsx/src/import/tables.rs). A column delete that removes a file-read
  table's last kept criterion inside the table unfilters it (`kept_filter_inside`, table_edits.rs), so the engine
  agrees with its export. The eval adapter adds a criterion-less or out-of-range spec filter unfiltered.
- Decided (numbers.md "SUBTOTAL and AGGREGATE over hidden and filtered rows"; xlsx-storage.md "Table parts in the
  export" updated): the API's `filtered` flag means a filter with a criterion, so it stays filtered; its export still
  writes a criterion-less filterColumn that Excel and now our import read as unfiltered, pinned by
  `a_filtered_table_without_a_criterion_exports_unfiltered`; follow-up T-4363 writes a criterion instead.
- Eval delta: +1 probes case, 30/30 cells pass; no other case changed.
- Follow-up: T-4363.
