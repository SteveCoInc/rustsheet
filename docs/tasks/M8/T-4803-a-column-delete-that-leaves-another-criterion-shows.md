---
id: T-4803
title: "A column delete that leaves another criterion shows only the rows the deleted criterion hid"
milestone: M8
lane: engine
status: in-progress
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
- [ ] probes/t4755-two-criteria-filtered-table-delete-one passes; the other t4755 and t4371 cases still pass.
- [ ] A unit test covers the same edit on a table loaded from a file with two criteria.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Re-applying filters after a value edit, and worksheet-level autoFilters.
