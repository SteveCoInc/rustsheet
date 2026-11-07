---
id: T-4869
title: "Probe the item order of a pivot Excel creates, and order the API's new pivots the same"
milestone: M16
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-4178]
---

## Goal
Evidence of the order in which Excel lists the items of a pivot it creates from a range, and a pivot
created through `Txn::add_pivot_table` that lists them the same way.

## Context
- T-4178 made `Txn::add_pivot_table` public. Its cache is the source read as a refresh reads it, and a
  pivot field with no `<items>` shows its items in cache order, which is source order (first
  appearance; docs/excel-semantics/pivots.md "Items show in the order the pivot field lists them";
  "A pivot created through the API").
- Excel's UI may sort a newly created pivot's items ascending (by type), unprobed; pivots.md calls it
  a UI action no file reaches. The oracle can still create a pivot (Graph or a workbook the probe saves after an
  Excel-side "Insert PivotTable"), and the `<items>` order it saves is the answer: text against
  numbers, booleans, errors, blanks, case variants and dates.
- Code paths: crates/rustsheet/src/edit/pivot.rs (`build_pivot`, `pivot_fields`): writing each
  axis field's `<items>` in Excel's order there would change only API-created pivots.
- Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [ ] A probe case whose sidecar records the items order of an Excel-created pivot over mixed item
  types; pivots.md entry updated.
- [ ] If the probe shows a sort, `Txn::add_pivot_table` lists the items in that order, with a test.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The order of items a refresh appends (already VERIFIED, pivots.md).
