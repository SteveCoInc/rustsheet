---
id: T-4164
title: "Probe pivot filters, Top-N, sort by value and their totals"
milestone: M16
lane: research
status: done
priority: P2
size: S
tier: sonnet
depends_on: [T-4130]
---

## Goal
Evidence for label, value and date filters, Top/Bottom N by items/percent/sum with ties, several filters on one field and their evaluation order, sort by a value field (ties, errors), manual item order, and whether subtotals and grand totals count filtered-out items.

## Context
Spike run 5 (Top-N changes subtotals); libreoffice.md (ties included). Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [x] Probe cases with sidecars; pivots.md entries.
- [x] `make check` green; reviewers approve.

## Out of scope
- Engine changes.

## Outcome
Six probes, all filled on excel-graph (2026-11-07), each recorded in docs/excel-semantics/pivots.md:
- **probes/t4164-pivot-top-n** ("Top-N filters at their edges"): a count Top-N keeps every item tied with the Nth (top
  2 shows three, bottom 1 two); percent and sum stop at the first item whose running sum reaches the threshold
  (exactly reaching stops) and keep no ties (the tied item first in the `<items>` order is taken, in either direction,
  not the cache's or the source's first: TP40i/TP40o, TSb10i/TSb10o); N past
  the item count keeps all; a zero is an ordinary item; sum compares signed (a bottom sum of 10 over a negative total
  keeps everything); percent over a negative total keeps all for top 50 and only the smallest for bottom 50.
- **probes/t4164-pivot-filter-order** ("Several filters on one field"): a hidden item or a label filter is applied
  before a Top-N, which ranks (and takes its percentage of) only what they kept, whatever the order in `<filters>`;
  a value filter and a count Top-N apply in the order `<filters>` lists them (TV1, TV0 keep Kiwi alone).
  `multipleFieldFilters` changes nothing.
- **probes/t4164-pivot-filter-totals** ("Filters on an inner field rank per parent"): Top-N, percent and value filters
  on an inner field are evaluated within each outer item; subtotals and grand totals never count removed items, for
  every filter kind, and `subtotalHiddenItems="1"` changes nothing (also the hidden-row-item control the "Record
  filters" entry was waiting for).
- **probes/t4164-pivot-value-sort** ("Value sorts: ties, blanks and errors"): ties keep the cache order descending and
  reverse it ascending, where the cache is the one the file wrote (STc, STca), not the source order, not alphabetical
  and not the `<items>` order (STo); a blank sorts as 0, an error last both ways; a
  column reference picks the column (Year item 1).
- **probes/t4164-pivot-item-order** ("A field shows its items in the `<items>` order"): with no sort or `manual` the
  field shows its `<item>` order, not the cache's; a label or value sort replaces it. For this the case builder gained
  `item_order` (Python and Rust, with tests and a golden, IOO), which orders the `<item>` elements without touching the
  cache; both builders refuse an `item_order` that is not a list, names an item not in the field or repeats one, and
  the Rust one also a list past `MAX_LIST`.
- **probes/t4164-pivot-date-times** ("useWholeDay"): without the x15 extension (WEqN, WNtN, WOeN, WBtN) or with
  `useWholeDay="0"` a date filter compares date-times; with `useWholeDay="1"` the item is cut to its day and the criterion is used as written (a
  dateBetween inside one day, or dateEqual at noon, keeps nothing).

A Top-N `sum` filter with a negative `val` makes the web refuse the file (bisected; "Pivot flags Excel for the web
refuses"; input sha256 7d676540… and 0dfd4906…, control 57396809…); NBS was rewritten with `val="10"`. Owners (decision 0245): T-4165 for the filter, Top-N, value-sort and
useWholeDay cells, T-4768 for IOA's label sort, T-4717 for the text of date item labels (WOe0, WOe1, WOeN); IO, IOM, IOO, SH0 and SH1H already pass. UNVERIFIED, filed as T-4799:
the mechanism of percent over a negative total (two readings fit), whether a blank ties with a real 0 in a value sort,
the order of several errors, a descending sort against an `<items>` order that differs from it (IOD coincides), and
the unprobed shapes each entry's Status names (first-item ties, several data fields, inner-field value-sort ties,
column fields, a hidden column item, percent or sum Top-N with a value filter, and `evalOrder`).
No M14 task (decision 0266).
