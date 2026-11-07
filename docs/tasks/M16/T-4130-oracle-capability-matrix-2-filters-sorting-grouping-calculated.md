---
id: T-4130
title: "Oracle capability matrix 2: filters, sorting, grouping, calculated fields/items, slicers, timelines"
milestone: M16
lane: research
status: done
priority: P1
size: M
tier: opus
depends_on: [T-4122, T-4125]
---

## Goal
The same matrix for the remaining families, plus three questions: whether the Auto Refresh setting is stored in the file and honoured by Excel for the web, whether the web rewrites PivotChart caches (c:numCache/strCache) when it re-renders a pivot, and what GETPIVOTDATA returns against a pivot collapsed to `#SPILL!`.

## Context
excel-docs.md (AUTOREFRESH, PSPILL); plan §7 and §10. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [x] `web` filled for every row of the filtering, sorting and calculated families, each citing a probe; the grouping
  and slicers-timelines rows are split off to T-4767 (Outcome).
- [x] The three questions answered in docs/excel-semantics/pivots.md.
- [x] `make check` green; reviewers approve.

## Out of scope
- Implementing any of it.

## Outcome
Split to stay size M: the filtering, sorting and calculated families (245 rows) are probed, and the grouping (125)
and slicers-timelines (114) rows are left `unknown` for the new T-4767. Each filled row cites a probe and sheet in
`evidence`:
- **filtering**: 146 `yes`, 11 `no`, and 4 `unknown`: the record-tuple rows PV-0699, PV-0720, PV-0740 and PV-0761
  opened only in uncommitted scratch files, so they are UNVERIFIED, and T-4767 teaches the builder to write tuples
  and commits their probe;
- **sorting**: 42 `yes`;
- **calculated**: 2 `yes` (the calculated field), 40 `no` (calculated items).

Refused (`no`, `FileCorruptTryRepair`; pivots.md "Pivot flags Excel for the web refuses"): a filter of type
`unknown`, a `colorFilter` criterion, `iMeasureHier`, an item `s="1"`, `hideNewItems="1"`, `hiddenLevel="1"`, tuples
under a cache shared item, and **every calculated item** (its item `f="1"`; with `f="0"` the file opens and the item
is not computed). No M14 task is filed (decision 0266). Whether an Excel-saved calculated item opens is T-4505's.

Rendered, among the findings (pivots.md "Pivot sorts and filters as Excel for the web renders them"):
- caption filters ignore case, and captionBetween is inclusive;
- for a captionEqual the autoFilter decides, not `stringValue1`;
- a stored Top-N `filterVal` and a relative filter's `val`/`maxVal` are recomputed;
- relative date filters are evaluated against the day of opening;
- a dateEqual filter on a field the file caches as numbers is dropped;
- a value sort can sort by a second data field (SV2, by Max of Sales);
- `multipleFieldFilters="0"` does not stop two filters on a field;
- customListSort orders day names.

The three questions:
1. **Auto Refresh**: not stored in the file Excel for the web saves (seen in the oracle's cached download), and not
   honoured after a Graph range write to the source: it re-renders neither a `refreshOnLoad` pivot nor one without
   it. A UI edit is not reachable. Desktop is UNVERIFIED, recorded in T-4182.
2. **PivotChart caches**: rewritten. The web rewrites the series references to the rendered ranges, and the
   `c:strCache`/`c:numCache` to the plotted points, replacing stale ones. For a `#SPILL!` pivot all references become
   the anchor.
3. **GETPIVOTDATA against a `#SPILL!` pivot**: `#REF!` for the grand total, for an unknown item and for a reference
   that is not the anchor. A field/item pair naming an item gives `#SPILL!`.

New probes, all excel-graph, 2026-11-07:
- probes/t4130-pivot-label-filters, probes/t4130-pivot-value-filters, probes/t4130-pivot-date-filters,
  probes/t4130-pivot-relative-date-filters, probes/t4130-pivot-filter-attrs and probes/t4130-pivot-field-filters;
- probes/t4130-pivot-sort and probes/t4130-pivot-calc-items;
- probes/t4130-pivot-spill-getpivotdata, probes/t4130-pivot-chart-cache and probes/t4130-pivot-auto-refresh.

Harness changes:
- the case builder (Python and the Rust mirror, with three new golden parts) writes a field's raw `autoSortScope`,
  `pageField` attributes, and calculated items (`calculated_items`, appended to the field's shared items, with Excel's
  pivotArea or a raw one, and `f` from `item_attrs` when given);
- the Rust mirror repeats the builder's refusals for these slots (unknown calculated-item keys, managed attributes, a
  raw pivotArea that is not one element: `pivot_definition/checks.rs`);
- spec2xlsx writes a sheet's `pivot_charts` (tools/src/rustsheet_tools/pivot_chart.py).

Owners of the new off cells (decision 0245):
- T-4165: filters, Top-N, AutoShow and value sorts;
- the new T-4768: label sorts and customListSort;
- T-4174: the calculated field;
- T-4154: GETPIVOTDATA against `#SPILL!`.
