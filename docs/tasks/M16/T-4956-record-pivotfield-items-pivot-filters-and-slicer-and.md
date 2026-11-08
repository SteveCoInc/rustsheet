---
id: T-4956
title: "Record pivotField items, pivot filters and slicer and timeline caches in the oracle sidecar snapshot"
milestone: M16
lane: harness
status: done
priority: P3
size: S
tier: opus
depends_on: [T-4767]
---

## Goal
An Excel-saved file's oracle sidecar records what Excel saved for a pivot's fields and filters and for its slicers
and timelines, so claims about what Excel for the web saves can cite a committed sidecar, not the oracle's cached
downloads.

## Context
- T-4767 (pivots.md "Capability matrix 3") states several save claims as UNVERIFIED because the sidecar snapshot
  (crates/rustsheet-eval/src/xlsx_snapshot/pivots.rs) records no pivotField items, no pivot filters, and none of the
  slicer or timeline parts. Seen only in cached downloads: unselected slicer items saved as the pivotField's `h="1"`;
  a timeline selection saved as a `dateBetween` pivot filter with x15 `useWholeDay="1"`; dropped group-item
  attributes, slicer item `nd`, `slicerCacheHideItemsWithNoData@count`, `singleRangeFilterState="0"` and
  `x15:timelinePivotCacheDefinition`; sorted slicer items; added `xr10:uid` and `scrollPosition`.
- The snapshot's `group_items` maps `s` and `d` both to text (pivots.rs), so a saved date group item cannot be told
  from a string one; record the item's type.
- After the snapshot grows, refill probes/t4767-pivot-slicers, -timelines and -group-items (`rustsheet-oracle fill`
  re-reads cached downloads) and move the claims in pivots.md to VERIFIED or correct them.

## Acceptance criteria
- [x] The sidecar `pivots` section records each pivotField's items (with `h`, `m`, `s`, `x`), each pivot filter
  (type, field, values, extensions), each group item's type, and the slicer and timeline caches and parts.
- [x] The four t4767 probes are refilled and the pivots.md "Capability matrix 3" save claims cite them.
- [ ] `make check` green; reviewers approve. (`make check-fast` green locally.)

## Out of scope
- Writing slicers or timelines on export (T-4188).

## Outcome
The snapshot (crates/rustsheet-eval/src/xlsx_snapshot/pivots.rs, pivots/xml_tree.rs, pivots/slicers.rs) now keeps,
per pivot: `pivot_fields` and `filters` (each element as written, a bounded `XmlElement` tree), `cache_extensions`
(the cache definition's `<ext>`s), `slicers` and `timelines` (each cache that names the pivot, with the
`<slicer>`/`<timeline>` shapes naming it), and per cache field `group_items_written` (element, `v`, other
attributes); shared items keep their other attributes too. All additive: old sidecars still read. Bounds: the
workbook budget pays for every kept element and copy, trees stop at depth 32, cache and shape relationships at
`MAX_PIVOTS`. The four t4767 probes were refilled live (excel-graph, 2026-11-08: the cached downloads were not in
this checkout; the inputs' sha256 and every value are unchanged). Every save claim of pivots.md "Capability matrix 3"
is VERIFIED from the sidecars, with corrections: slicer items follow the `tabular` sort order (SM descending: West,
North, East), `xr10:uid` is on the shapes too, and the save also drops `slicerData="0"`, clears the pivot's own
`h="1"` on a selected item, and adds `<items>` on the timeline's date field and an `xlpdsc:dataSourceClean`
extension. The `tableSlicerCache` claim stays UNVERIFIED: T-5071.
