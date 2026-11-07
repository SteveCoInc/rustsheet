---
id: T-4165
title: "Implement pivot label/value/date filters and Top-N"
milestone: M16
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: [T-4164, T-4149]
---

## Goal
PivotResult applies the filters and sorts of the catalog's filtering and sorting families as evidenced.

## Context
SPEC-0032; evidence from `p-filters`. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

Owner lines (decision 0245): every owned filter cell now passes in value, so T-4165 owns none. Moved:
- value sorts (probes/t4130-pivot-sort SV, SVA, SVC, SVR, SXA, SXR, SV2; probes/t4164-pivot-value-sort; t4164-pivot-item-order
  IOV) to T-4827, split from this task;
- the date item label cells of probes/t4130-pivot-date-filters and probes/t4164-pivot-date-times, now off in their
  text only (Excel shows the source's date format, the engine the serial), to T-4717, which owns that text.
The label sorts of probes/t4130-pivot-sort are T-4768's, as they were.

## Acceptance criteria
- [x] p-filters probes pass (filters, Top-N, AutoShow and relative dates; value sorts split to T-4827).
- [x] `make check` green; reviewers approve.

## Out of scope
- Slicers and timelines (PV5).

## Outcome
Partial split, done for its part: filters here, value sorting in T-4827 (size: one reviewable PR).

- **Model.** `PivotTable::filters` (`pivot::filter::PivotFilter`: `fld`, the `type` class, `iMeasureFld`,
  `evalOrder`, `id`, the autoFilter criterion typed as customFilters, top10, dynamicFilter, a values list with
  `dateGroupItem`s or an icon, and the x15 `useWholeDay`) and `PivotField::auto_show`. The importer reads them from
  the `<filters>` element and the `<pivotField>` attributes it keeps verbatim
  (crates/rustsheet-xlsx/src/import/pivots/filters.rs), so the export's round trip is unchanged. A date filter on a
  field the file caches as numbers is dropped on import, as Excel drops it (DLit).
- **Engine.** `pivot::filter::apply` runs after the page and hidden-item filter in `compute_on`: caption and date
  filters first, then each axis field's value filters and Top-N in `<filters>` order and its AutoShow, ranked within
  each tuple of the axis's outer fields, each dropping the records of what it removes, so every subtotal and grand
  total covers only what is shown. Count Top-N keeps ties; percent and sum stop at the threshold, take no tie and
  break ties by the `<items>` order; sum is signed and percent compares in absolute value. Caption matching uses the
  engine's wildcard pattern and Excel's text comparison, case-insensitive. Relative dates read today from the
  workbook clock as TODAY does (`Model::pivot_today`, which the grid writer's `place` and the export pass to
  `compute_on`), never the host clock directly. All work is charged to the pivot budget before it is done: each item
  test's text comparisons as their collation ticks (`area_budget::compared_units`), with each list value folded, each
  pattern compiled and each number parsed once per filter. The importer caps filters, conditions and values-list
  sizes (pivots.md "Pivot filters past the import limits", DEVIATION) and skips a pivot whose `<filters>` it cannot
  type (web.md). GETPIVOTDATA on a filtered pivot is `#REF!` (web.md). Code: `pivot/filter.rs` (apply and the item
  tests), `filter/model.rs` (the typed model), `filter/rank.rs` (value filters and Top-N); a bench case in
  crates/rustsheet/benches/pivot.rs times inner-field caption and Top-N filters over 100,000 records.
- **Tests.** crates/rustsheet-engine/src/pivot/filter/tests.rs compares every label, value, Top-N, filter-order,
  filter-totals, filter-attrs, AutoShow, date-times and fixed/month/quarter date sheet with its sidecar
  (`probe_cases::sidecar_differences`, shared with the render tests); the importer's reading is in
  crates/rustsheet-xlsx/src/import/pivots/table/tests.rs.
- **Eval.** `make eval-baseline` (on main after T-4530): probes known failures 8069 -> 7410, text 1756 -> 1175 (fixed
  1240, REGRESSIONS 0).
  Of the 659 owned filter cells all pass in value and 586 in their text too; the 73 date label texts left go to
  T-4717.
- The engine's own readings are UNVERIFIED in pivots.md "Pivot filters: the engine's own readings", listed in
  T-4799.

Bench baseline (`cargo bench -p rustsheet --bench pivot -- "result tree"`, medians, reviewer run 2026-11-07):
result tree over 100,000 records 10.2–11.1 ms (origin/main 10.5 ms); result tree with filters 9.1–9.9 ms
(origin/main 9.2 ms); result tree with inner caption and Top-N filters (new) 4.1–4.2 ms.
