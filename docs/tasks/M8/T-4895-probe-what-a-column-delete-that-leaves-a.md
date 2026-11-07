---
id: T-4895
title: "Probe what a column delete that leaves a filter criterion does to hand-hidden, visible, case-differing and formatted rows"
milestone: M8
lane: research
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Excel evidence for the parts of T-4803's "a column delete that takes one criterion re-applies the kept ones" rule
that are recorded as ours and unprobed (docs/excel-semantics/numbers.md, "SUBTOTAL and AGGREGATE over hidden and
filtered rows"), so the engine follows the evidence there too.

## Context
- T-4803 (PR #3920) shows each hidden data row that every kept criterion keeps, by the row's displayed text
  (`rows_kept_by_criteria`, `FilterCriterion::keeps_text` in crates/rustsheet-engine). Only
  probes/t4755-two-criteria-filtered-table-delete-one backs it: one kept values filter on plain lowercase text, with no
  hand-hidden row.
- Unprobed (review findings on #3920):
  - a row hidden in the file that the kept criteria keep, with two criteria (the engine shows it);
  - a visible row that a kept criterion rejects (the engine leaves it visible; does Excel re-apply the whole filter?);
  - kept values that differ in case from the cells (the engine ignores case);
  - number-formatted values and blank cells in the kept column;
  - a kept custom, date-group, top10 or color criterion (the engine keeps every hidden row hidden).
- Also from the review of #3831 (T-4749): the eval adapter's every-row-hidden rule (`filters_hidden_rows`, which
  counts a values filter with no visible data row as no criterion) reaches `"criteria"` tables too. For those tables
  spec2xlsx writes one `<filters>` per criterion, and that `<filters>` still lists values unless the criterion hides
  every row. No case reaches this today. Once evidence exists, make the adapter count such a table as filtered when
  some criterion keeps a row.
- spec2xlsx's `"criteria"` writes only values filters (tools/src/rustsheet_tools/spec_tables.py). Ad-hoc uploads may
  be needed for the other kinds; record their sha256 values as T-4749 did.

## Acceptance criteria
- [ ] A probes case (excel-graph) for each unprobed point above that spec2xlsx can write, with the engine's predictions
      committed before the fill. The other points are probed by upload, with their sha256 recorded.
- [ ] numbers.md states each result. Every visibly-off cell has an owner task, or is fixed here if it is size S.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Re-applying filters after a value edit; worksheet autoFilters.
