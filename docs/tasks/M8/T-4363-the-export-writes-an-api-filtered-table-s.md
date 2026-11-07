---
id: T-4363
title: "The export writes an API-filtered table's filter as a criterion Excel reads as filtered"
milestone: M8
lane: engine
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A table filtered through the API (`Txn::add_table(.., filtered: true)`, no kept criterion) keeps its hidden rows
filtered for SUBTOTAL and AGGREGATE after an xlsx export, in Excel and in our own re-import.

## Context
- Found by T-4003. Excel calls a table's hidden rows filtered only for a filterColumn with a criterion inside the
  table (probes/t4003-filter-column-criterion; docs/excel-semantics/numbers.md "SUBTOTAL and AGGREGATE over hidden
  and filtered rows"). The export (`auto_filter` in crates/rustsheet-xlsx/src/export/tables.rs) writes an API-filtered
  table with no kept criterion as `<filterColumn colId="0"/>`, which Excel and, since T-4003, our import read as not
  filtered, so a round trip changes `SUBTOTAL(9,..)` from the filtered sum to the full one
  (`a_filtered_table_without_a_criterion_exports_unfiltered` in export/test/test_tables.rs pins today's behavior).
- A criterion that Excel reads as filtered: spec2xlsx's values filter on the first column keeping exactly the visible
  data rows' text (tools/src/rustsheet_tools/spec2xlsx.py `_add_table`; Excel does not re-apply it on load). Which text a
  number or date cell lists in `<filter val>` (its formatted text?) and what to write when a value is both on a
  visible and a hidden row need evidence (`excel-probe`).

## Acceptance criteria
- [ ] An API-filtered table's export holds a criterion; Excel (excel-graph upload) and our re-import give the same
      SUBTOTAL 9 and 109 as the engine before the export.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Re-applying filters; top 10 and other kinds the export drops (T-4004).
