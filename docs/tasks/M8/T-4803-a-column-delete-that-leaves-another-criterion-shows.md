---
id: T-4803
title: "A column delete that leaves another criterion shows only the rows the deleted criterion hid"
milestone: M8
lane: engine
status: done
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
After a delete in the application takes one of a filtered table's criteria and another criterion stays, the
engine shows exactly the rows only the deleted criterion hid and keeps the rows a remaining criterion hides, as
Excel does.

## Context
- probes/t4755-two-criteria-filtered-table-delete-one (excel-graph 2026-11-07, T-4755): table Sales on S!A1:C7
  with a values filter on Cat (rows 3, 7 filtered) and one on Ext (row 5 filtered); delete_column at A takes Cat.
  Excel shows rows 3 and 7 and keeps row 5 hidden (SUBTOTAL 3 and 103 over Sales[Val]: 5; F!B5, F!C5: 0). The
  engine shows all six rows: 10 cells off (F!A1:A8, F!B5, F!C5). docs/excel-semantics/numbers.md, "SUBTOTAL and
  AGGREGATE over hidden and filtered rows", records it.
- Two engine paths miss it. The eval adapter adds a spec table through the facade (`add_table_with_rows`, a
  filtered flag and no criteria; `filters_hidden_rows` in crates/rustsheet-eval/src/engine.rs), so it cannot
  carry spec2xlsx's `"criteria"`, and `resize_tables` (crates/rustsheet-engine/src/table_edits.rs) unfilters an
  API-filtered table once its first column goes and unhides every data row. A table loaded from a file keeps its
  `filter_columns`; by the code, the delete of one criterion there keeps `has_filters` and unhides nothing, so rows 3 and 7
  stay hidden.
- Telling which rows the deleted criterion alone hid means evaluating the remaining criteria (`FilterCriterion`
  is kept as written and nothing re-applies it today), or recording per row which criteria reject it. The
  facade would need a way to give a table its criteria for the adapter to load the spec's.

## Acceptance criteria
- [x] probes/t4755-two-criteria-filtered-table-delete-one passes; the other t4755 and t4371 cases still pass.
- [x] A unit test covers the same edit on a table loaded from a file with two criteria.
- [x] `make check` green; reviewers approve.

## Out of scope
- Re-applying filters after a value edit, and worksheet-level autoFilters.

## Outcome
- A column delete that takes one of a filtered table's criteria and leaves another inside it now re-applies the kept
  criteria to the table's hidden data rows: a row every kept criterion keeps is shown, the others stay hidden
  (`Model::rows_kept_by_criteria` in `resize_tables`, crates/rustsheet-engine/src/table_edits.rs). Only a values filter
  without date groups is evaluated (`FilterCriterion::keeps_text`: blank rows by `blank`, others by displayed text among
  `values`, ignoring case); any other kept criterion keeps its rows hidden. The undo hides the shown rows again (the
  existing `row_shown` diffs).
- `Model::filter_table_column` / `Txn::filter_table_column` (crates/rustsheet/src/edit/table_filter.rs) give a filtered table a values filter that rejects exactly
  the given data rows (kept text of the other rows, as spec2xlsx writes `"criteria"`); the eval adapter uses it for spec
  `"criteria"`, so the criteria reach the engine and the export.
- Tests: engine `test_filtered_table_edits` (two criteria, undo, a hand-hidden row, an unevaluable criterion, the
  derived values filter and its refusals), xlsx `deleting_one_of_two_criteria_from_a_file_shows_the_rows_it_alone_hid`
  (saved and reloaded file), facade `filter_table_column_needs_a_filtered_table_and_its_data_rows`.
- Docs: numbers.md "SUBTOTAL and AGGREGATE over hidden and filtered rows" (the engine now matches; the rule's unprobed
  parts recorded as ours).
- Eval delta: probes 10 known failures fixed (probes/t4755-two-criteria-filtered-table-delete-one now 22/22); every other
  suite fixed 0 / regressions 0; the other t4755 and t4371 cases still pass.
