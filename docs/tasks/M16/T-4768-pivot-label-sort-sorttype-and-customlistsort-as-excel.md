---
id: T-4768
title: "Pivot label sort: sortType and customListSort as Excel for the web renders them"
milestone: M16
lane: engine
status: done
priority: P2
size: S
tier: opus
depends_on: [T-4130]
---

## Goal
A pivot field's label sort renders in the order Excel for the web renders it: `sortType` ascending and descending
over the field's items, `manual` in the field's listed order, and with `customListSort` on (the default) the built-in
custom lists ahead of the alphabetical order (the day names are evidenced; the month lists are Excel's other
built-in lists, unprobed).

## Context
- T-4130 (probes/t4130-pivot-sort; pivots.md "Capability matrix 2" and "Pivot sorts and filters as Excel for the web
  renders them"): the engine renders every field in its cache order whatever `sortType` says. Excel sorts SA
  ascending (Ant, Bee, Cat), SD and NA descending (Cat, Bee, Ant; `nonAutoSortDefault` changes nothing visible), CL1
  Day ascending by the day-name custom list (Mon, Wed, Fri) and CL0, with `customListSort="0"`, alphabetically (Fri,
  Mon, Wed). SM (manual) already passes.
- Value sorts (`autoSortScope`) are T-4165's. Code: crates/rustsheet-engine/src/pivot/result.rs (`Order`).
- Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [x] The engine renders SA, SD, NA, CL1 and CL0 as the sidecar shows (unit tests against it, and the cells pass in the eval).
- [x] `make check` green; reviewers approve.

## Out of scope
- Sorting by value (T-4165) and user-defined custom lists (not in the file).

## Outcome
Done. `pivot::sort::labels::display_order` is a field's display order before any value sort: its `<items>` order
(`Order::of`), or under `sortType` ascending or descending with no `autoSortScope` a permutation of the cache items
(`Order::permuted`, now shared with T-4827's value sorts) by Excel's collation (`collation::cmp_text`, the caption
filters' collation). With `customListSort` on (the new `PivotOptions::custom_list_sort`, read by the importer from the
definition's kept attributes, so the export round trip stays verbatim) the names of Excel's built-in custom lists sort
by the list; the lists are `locale::excel_date_names::BUILT_IN_CUSTOM_LISTS`, built from the en-US date names the
formatter already holds. The axis (`AxisColumns::new`) and the value filters (Top-N ties) both take the order from
`display_order`. A value sort still wins: a field with a scope is never label-sorted. All the work is charged before
it is done: the items, the custom-list lookups, and a bound on the collation's ticks for the sort.

Tests: `pivot::sort::labels::tests` against the sidecars (t4130-pivot-sort SA, SD, NA, SM, CL1, CL0; t4164-pivot-item-order
IOA, IOD, IOM; t4128-pivot-spike K with its Top-N), the engine's own readings, and the budget; the importer's reading of
`customListSort` in table/tests.rs.

Eval: probes known failures 6833 -> 6793, text 142 -> 114 (fixed 68, REGRESSIONS 0); every owned cell passes, so
the owner lines are removed.

UNVERIFIED (T-4799, pivots.md "Label sorts: the engine's own readings"): how mixed item types order, the blank last,
collation ties, custom-list matching and precedence, the day, month and abbreviated month lists, range-group order,
a scope that sorts nothing, Top-N ties under a label sort, and a label sort on a column field.
