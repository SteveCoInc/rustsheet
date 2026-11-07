---
id: T-4363
title: "The export writes an API-filtered table's filter as a criterion Excel reads as filtered"
milestone: M8
lane: engine
status: done
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
- [x] An API-filtered table's export holds a criterion; Excel (excel-graph upload) and our re-import give the same
      SUBTOTAL 9 and 109 as the engine before the export.
- [x] `make check` green; reviewers approve.

## Out of scope
- Re-applying filters; top 10 and other kinds the export drops (T-4004).

## Outcome
- The xlsx export writes an API-filtered table with no kept criterion as a values filter on its first column listing
  the formatted text of each visible data row (`blank="1"` for a visible blank; `export::tables::api_filter_column`),
  the form spec2xlsx writes, instead of the criterion-less `<filterColumn colId="0"/>`.
- Excel: an excel-graph upload of our export (2026-11-07; `Sales` A1:B7, `Cat` a,b,a,c,a,b, `Val` 1-6, rows 3 and 5
  hidden, "b" on both a visible and a hidden row) reads SUBTOTAL 9 and 109 both 15, 3 and 103 both 4, as the engine
  before the export. Our re-import keeps the table filtered with the same SUBTOTALs (unit test
  `a_filtered_table_without_a_criterion_exports_its_visible_values`, which replaces the test that pinned the old
  behavior). No probe case: a spec case cannot express an API filter (spec2xlsx writes its own criterion).
- Docs: xlsx-storage.md ("Table parts in the export") and numbers.md (SUBTOTAL over filtered rows).
- Follow-up filed: T-4744 (T-4692 review: style of filled calculated-column cells).
- Eval delta: none (export path; no eval case covers it).
