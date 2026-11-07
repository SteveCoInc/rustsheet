---
id: T-4827
title: "Sort pivot fields by value (autoSortScope) as Excel does"
milestone: M16
lane: engine
status: todo
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

Owner of these visibly-off pivot cells (decision 0245; moved from T-4165):
- probes/t4130-pivot-sort SV!A4:A6, SV!B4:B6, SV2!A5:A6, SV2!B5:B6, SV2!C5:C6, SVA!A5:A6, SVA!B5:B6, SVC!A6:A7, SVC!B6:B7, SVC!C6:C7, SVC!D6:D7, SVR!A9:A11, SVR!B9:B11, SXA!A4:A6, SXA!B4:B6, SXR!A4:A6, SXR!B4:B6
- probes/t4164-pivot-value-sort SC2!A5:A9, SC2!B5:B9, SC2!C5:C9, SC2!D7:D9, SCa!A5, SCa!A8:A9, SCa!B5, SCa!B8:B9, SCa!C5, SCa!C8:C9, SCa!D5, SCa!D8:D9, SCd!A5:A6, SCd!A8, SCd!B5:B6, SCd!B8, SCd!C5:C6, SCd!C8, SCd!D5, SCd!D8, SEa!A4:A8, SEa!B4:B8, SEd!A4:A6, SEd!A8, SEd!B4:B8, STa!A4:A8, STa!B4, STa!B7:B8, STc!A4:A7, STc!B4, STc!B7, STca!A4:A8, STca!B4, STca!B7:B8, STd!A4:A7, STd!B4, STd!B7, STo!A4:A8, STo!B4:B5, STo!B7:B8
- probes/t4164-pivot-item-order IOV!A4:A6, IOV!B4:B6

## Acceptance criteria
- [ ] Unit tests against the probe sidecars, and the owned cells pass in the eval.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Label sorts and custom lists (T-4768).
