---
id: T-4260
title: "Date-grouped pivot over a blank: the adapter drops the grouping as Excel's refresh does (after T-4246)"
milestone: M6
lane: engine
status: in-progress
priority: P3
size: M
tier: opus
depends_on: [T-4246]
---

## Goal
A date-grouped spec pivot whose row field holds a blank source cell is laid out and answered as Excel does
after its refresh: ungrouped, each second-rounded date its own item, the blank as `(blank)`. Every cell of
probes/t4236-pivot-date-groups that T-4246 left visibly off passes.

## Context
- Split from T-4246, which registers date-grouped spec pivots whose row field holds no blank and leaves the
  rest unregistered: `Txn::add_pivot_table` refuses them with `PivotError::DateGroupOverBlank` and the eval
  adapter (crates/rustsheet-eval/src/spec_pivots.rs) skips them.
- Excel's rule (docs/excel-semantics/web.md "Grouped pivot fields import as their group items", the date-group
  "Quirk: on refresh" bullet): the grouping is dropped. The items are the dates rounded to the second, listed in
  record (first-appearance) order, not sorted, with `(blank)` at its record's place (pivot M: `... 43876, (blank),
  43631`). GETPIVOTDATA takes `"(blank)"` or an omitted item for the blank, and a date serial (`DATE(2019,1,1)`)
  for a date item, while every group label is `#REF!`.
- What is needed: the engine lays out an ungrouped date row field (row labels are dates as the source cells
  format them, so check the probe's rendered reads for their text or number type). GETPIVOTDATA needs a number
  item on an ungrouped date field (today SPEC-0015 unknown 3, `None`), and the cache must round dates to the
  second. Share `row_layout` (crates/rustsheet-engine/src/pivot/layout.rs) and do not copy it.
- The blank case's rendered reads stop at row 48 (Calc rows 41 to 58), which cuts off the ungrouped Q and Y
  layouts' last rows. Extend the reads if it refills.

## Acceptance criteria
- [ ] The adapter registers every pivot of probes/t4236-pivot-date-groups, and the probe's visibly-off cells
      pass; baseline updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The importer: an Excel-saved file with a grouping over a blank keeps it (59932), and the importer follows the
  file.
