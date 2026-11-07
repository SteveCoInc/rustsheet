---
id: T-4755
title: "Probe a row hidden by hand inside a filtered table, and a delete that leaves a criterion, through column edits"
milestone: M8
lane: engine
status: in-progress
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
- [ ] A probes case (excel-graph) for (a): a filtered table with a row hidden by hand that the filter keeps, then a
      delete of the filtered column; SUBTOTAL 3 and 103 tell whether that row stays hidden.
- [ ] A probes case for (b), or a recorded reason (and a follow-up task) if spec2xlsx cannot write it.
- [ ] numbers.md records both results; the engine passes the new cases, or an owner task is filed for each failing cell.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Re-applying filters, and worksheet-level autoFilters.
