---
id: T-4755
title: "Probe a row hidden by hand inside a filtered table, and a delete that leaves a criterion, through column edits"
milestone: M8
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Evidence for two column-edit corners of filtered tables that T-4371 left unprobed, and the engine following it:
(a) a data row hidden by hand inside a filtered table when the delete of the filtered column unfilters it, and
(b) a delete of one filtered column when another column still holds a criterion.

## Context
- T-4371 (probes/t4371-filtered-table-*, excel-graph 2026-11-07) found that deleting the column with a table's only
  criterion unfilters the table and shows the rows its filter hid; docs/excel-semantics/numbers.md "SUBTOTAL and
  AGGREGATE over hidden and filtered rows" records it and lists both corners as unprobed.
- The engine cannot tell a row hidden by hand from a filtered one (`row_hidden_status`,
  crates/rustsheet-engine/src/functions/subtotal.rs), so `resize_tables` (crates/rustsheet-engine/src/table_edits.rs)
  unhides every hidden data row of an unfiltered table. If Excel keeps a hand-hidden row hidden, the engine needs to
  know which rows the filter hid (for a table from a file: the rows its criteria reject).
- spec2xlsx writes one values filter per table (`"filter": true`, optionally `filter_column`); (b) needs a second
  criterion, which spec2xlsx may not support yet. If not, extend it in this task or split that part off.
- Found by the T-4371 Excel-fidelity review (PR #3807).

## Acceptance criteria
- [x] A probes case (excel-graph) for (a): a filtered table with a row hidden by hand that the filter keeps, then a
      delete of the filtered column; SUBTOTAL 3 and 103 tell whether that row stays hidden.
- [x] A probes case for (b), or a recorded reason (and a follow-up task) if spec2xlsx cannot write it.
- [x] numbers.md records both results; the engine passes the new cases, or an owner task is filed for each failing cell.
- [x] `make check` green; reviewers approve.

## Out of scope
- Re-applying filters, and worksheet-level autoFilters.

## Outcome
- spec2xlsx gains a table option `"criteria": [{"column": 0, "hides": [3, 7]}, ...]`: one values filter per listed
  column rejecting exactly its rows, so a hidden data row no criterion hides is one the filters keep, and a table can
  hold two criteria (`_criteria_columns`, tools/src/rustsheet_tools/spec_tables.py; `CriterionSpec` in the eval case
  schema). Unit tests in tools/tests/test_spec2xlsx_tables.py.
- Four probes (88 cells, excel-graph 2026-11-07), recorded in numbers.md "SUBTOTAL and AGGREGATE over hidden and
  filtered rows":
  - (a) `t4755-hand-hidden-filtered-table-control` / `-delete-filtered-column`: a row the filter keeps but the file
    writes hidden counts as filtered (SUBTOTAL 3 and 103 both skip it), and deleting the filtered column shows it with
    the filtered rows. The engine passes both.
  - (b) `t4755-two-criteria-filtered-table-control` / `-delete-one`: deleting one of two criteria shows only the rows
    it hid; the other criterion's row stays hidden and filtered. The engine passes the control and shows every row
    after the delete: 10 cells off, owned by T-4803 (new, engine, opus).
- Eval delta: probes +4 cases; 78 cells pass, 10 new known failures (all owned by T-4803), 0 regressions.
