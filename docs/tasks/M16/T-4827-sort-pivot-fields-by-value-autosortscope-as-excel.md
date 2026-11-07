---
id: T-4827
title: "Sort pivot fields by value (autoSortScope) as Excel does"
milestone: M16
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: [T-4165]
---

## Goal
A pivot field sorted by a value (`sortType` ascending or descending with an `autoSortScope` on a data field) shows its
items in the order Excel for the web shows them, within each outer item, by the referenced data field and column item.

## Context
- Split from T-4165 (filters, Top-N, AutoShow and relative dates), which left value sorting for its own PR.
- Evidence: docs/excel-semantics/pivots.md "Pivot sorts and filters as Excel for the web renders them" (value sorts:
  SV, SVA, SVC, SVR, SXA, SXR, SV2 of probes/t4130-pivot-sort), "Value sorts: ties, blanks and errors"
  (probes/t4164-pivot-value-sort: ties follow the cache the file wrote when descending and its reverse when
  ascending, not the `<items>` order; a blank sorts as 0; an error sorts last both ways; the scope's column reference
  picks the column) and "A field shows its items in the `<items>` order, not the cache's, until a sort applies"
  (probes/t4164-pivot-item-order IOV).
- Model: type the field's `autoSortScope` (its `pivotArea` references: `field="4294967294"` is the data field, whose
  `x` names it; another `reference@field` with its `x` names a column item) from the `<pivotField>`'s kept
  `autoSortScope` element, as T-4165 types `<filters>` (crates/rustsheet-xlsx/src/import/pivots/filters.rs), keeping
  the verbatim round trip.
- Engine: the axis tree is built from display ranks (crates/rustsheet-engine/src/pivot/result.rs, `Order` and
  `AxisColumns::tree`); a value sort reorders each parent's children by the referenced value after the filters
  (`pivot::filter`), and charges the shared `PivotBudget`. Label sorts are T-4768's.
- Open points: T-4799 (a blank against a real 0, several errors, inner-field ties).

## Acceptance criteria
- [x] Unit tests against the probe sidecars, and the owned cells pass in the eval.
- [x] `make check` green; reviewers approve.

## Out of scope
- Label sorts and custom lists (T-4768).

## Outcome
Done. Every cell this task owned passes; its owner lines are removed.

- **Model.** `PivotField::auto_sort` (`pivot::AutoSortScope`): the data field (the first `x` of the reference whose
  `field` is 4294967294) and the other references with their `x`s. The importer reads it from the field's
  `autoSortScope`, which it keeps verbatim, so the export's round trip is unchanged
  (crates/rustsheet-xlsx/src/import/pivots/sort.rs). A refresh that renumbers the fields remaps the references of an
  API-made pivot and drops a scope whose field is gone.
- **Engine.** `pivot::sort::value_sorts` runs in `compute_on` after the filters, over the records they keep. It gives
  each sorted field's items a position within their parent. An outermost field's positions become its display
  `Order`, so its axis keeps the narrow u32 keys. Only an axis with a value-sorted inner field widens its keys to u64,
  with the position before the display rank (`result::axis::KeyPart`), so `AxisColumns::tree` orders each parent's
  children by value. An outermost sort orders only the items the kept records hold, plus a `showAll` axis's items
  as 0. The values come from
  T-4165's `ValueStep`, split into `ValueStep::cells` (the per-parent cell values, now also used by the value
  filters) and `run`. A `within` list restricts the values to the scope's column items. Descending ties keep the
  file's cache order, and ascending is its exact reverse. A cell with no value sorts as 0, and errors go last both
  ways. All the work is charged to the pivot budget first, and a unit test runs every budget below the cost and sees
  `Limit`.
- **Also found.** A Sum cell whose records are all blank renders blank, not 0 (SEd B7, SEa B4, owned here). This is
  added to pivots.md "Aggregates over text, blanks, booleans and errors" and implemented in
  `PivotResult::plain_as` for Sum only. The other aggregates are left to T-4633.
- **Evidence and readings.** pivots.md "Value sorts: ties, blanks and errors" lists the engine's own readings as
  UNVERIFIED (T-4799):
  - a blank against a real 0;
  - several errors;
  - inner-field ties;
  - `reference@x` read as an index into the field's `<items>`, and only its first `x` read;
  - references to a field off the other axis (on no axis, on the sorted field's own axis, or past the cache) being
    ignored;
  - scopes that name nothing, or that sit under `sortType="manual"`, which sort nothing;
  - `showAll` items, which sort as 0;
  - a sort on a column field;
  - a Count or Average item with no number in the scope, which sorts as its cell shows it.

  Also under T-4633: show-values-as modes read an all-blank Sum cell as empty, not 0.
- **Performance.** `cargo bench -p rustsheet --bench pivot -- "result tree"`, base dd67308d3 (this commit's parent)
  against HEAD. Each figure is the middle of three interleaved runs' medians, on a shared 4-core container:
  - over 100,000 records: 10.84 -> 11.09 ms;
  - with filters: 10.07 -> 9.88 ms;
  - with inner caption and Top-N filters: 4.20 -> 4.92 ms;
  - with outer and inner value sorts (new): 23.5 ms.

  The first review version, with u64 keys everywhere, measured 11.4, 10.3 and 4.4 ms. The narrow keys bring the first
  two cases back to base. The inner-filter case is about 0.7 ms slower in wall clock, but callgrind, cachegrind and
  branch simulation give it the same instruction, cache-miss and mispredict counts as the u64 version. The version
  differs from base by about 3.4M instructions per compute, from splitting `ValueStep::cells` out of
  `filter::apply`. Rebuilding with only filter.rs reverted moves the case by 0.3 ms, so the rest looks like
  code-layout sensitivity. Forcing alignment, pinning malloc thresholds and restoring the single row-and-column
  pass did not close it.
- **Eval.** probes known failures 6978 -> 6833 and text 280 -> 142 (fixed 283, REGRESSIONS 0), on b48104e92. The cells left in
  t4130-pivot-sort and t4164-pivot-item-order are the label sorts (SA, SD, NA, CL0, CL1, IOA), which are T-4768's.
