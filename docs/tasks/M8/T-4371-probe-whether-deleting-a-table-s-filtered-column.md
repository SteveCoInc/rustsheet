---
id: T-4371
title: "Probe whether deleting a table's filtered column in Excel unfilters its hidden rows for SUBTOTAL"
milestone: M8
lane: engine
status: in-progress
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
- [ ] A probes case (excel-graph) covers the three edits with SUBTOTAL 3, 9, 103 and 109; the engine passes it.
- [ ] numbers.md records the result.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Re-applying filters.
