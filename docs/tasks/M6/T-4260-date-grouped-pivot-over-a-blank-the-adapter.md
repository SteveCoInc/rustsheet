---
id: T-4260
title: "Date-grouped pivot over a blank: the adapter drops the grouping as Excel's refresh does (after T-4246)"
milestone: M6
lane: engine
status: done
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

## Outcome
Done, but 15 of the probe's cells are now T-4261's. `Txn::add_pivot_table` drops a date grouping over a blank, as
Excel's refresh does. The new `ungroup_dates` (pivot/date_groups.rs) uses the groups' own `split_serial` rule, so
each second-rounded date is its own `CacheItem::Date`, and the items keep record (first-appearance) order with the
blank at its record's place. `row_layout` writes a date label as its serial and the blank as `(blank)`. GETPIVOTDATA
picks an ungrouped date item by an equal serial (`pivot_number_item`) and the blank of an ungrouped field by
`"(blank)"` (`pivot_blank_text`). On grouped fields `"(blank)"` stays `#REF!`, as probed. The eval adapter now
registers every spec pivot, and `PivotError::DateGroupOverBlank` is gone. The unit test
`an_omitted_item_is_the_blank_item` had pinned the unprobed `"(blank)"` as `#REF!`; it now expects the blank item,
which this probe evidences.

Eval delta: probes known failures 5068 -> 4978 (90 cells of probes/t4236-pivot-date-groups fixed, 0 regressions; the
case goes from 225 to 320 of 335 cells). The 15 cells left (Calc E44:E50, F43:F50, pivot H) come from the half-second
point that T-4261 owns (Data!D4, 10:59:59.5 that is 2.2e-7 s below the half). The engine rounds it down and merges it
with 10:59:59, which shifts H's later rows. T-4261 now lists these cells. Calipers checked locally: 0 regressions.
