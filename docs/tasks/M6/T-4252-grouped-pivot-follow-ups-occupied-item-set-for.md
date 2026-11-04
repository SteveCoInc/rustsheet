---
id: T-4252
title: "Grouped pivot follow-ups: occupied-item set for GETPIVOTDATA and one layout footprint helper"
milestone: M6
lane: engine
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
GETPIVOTDATA on a range-grouped field checks an item's records in constant time, and a grouped pivot's layout footprint
has one definition.

## Context
- Review minors of T-4235 (PR #3238), deferred as follow-up per the work-loop:
  - `shown_group` (crates/rustsheet-engine/src/functions/lookup_and_reference/pivot_index.rs) scans every key of the
    index's `tuples` per call, before the memo: O(cells x tuples) on a hostile multi-axis grouped pivot. Precompute
    per axis position the set of occupied items when the index is built (`build`).
  - The eval harness derives a grouped pivot's footprint itself (crates/rustsheet-eval/src/spill_extent.rs: two columns,
    `max_layout_rows`). Expose one engine helper returning the area (crates/rustsheet-engine/src/pivot/layout.rs) and
    use it there and in `Txn::add_pivot_table`'s `lay_out_grouped` (crates/rustsheet/src/edit/pivot.rs).
  - Re-wrap the over-long doc comment on `aggregate` (pivot/layout.rs).
- Behaviour must not change: probes/t4228-pivot-range-groups and probes/t4235-pivot-range-rounding stay 168/168 and
  242/242.

## Acceptance criteria
- [ ] `shown_group` answers from a precomputed occupied-item set; a unit test covers an empty and a filled group.
- [ ] One engine helper gives a grouped pivot's footprint, used by the harness and the facade.
- [ ] `make check` green; reviewers approve.

## Out of scope
- New Excel behaviour (T-4249, T-4248).
