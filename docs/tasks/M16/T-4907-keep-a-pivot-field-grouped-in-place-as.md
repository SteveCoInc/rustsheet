---
id: T-4907
title: "Keep a pivot field grouped in place as its own records plus an item-to-group map"
milestone: M16
lane: engine
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
A pivot cache field grouped in place by a range holds its records once. The model keeps the field's own records plus
a map from each own item to its group, instead of the grouped records and a full copy of the own records beside them.

## Context
T-4734 keeps a field's own values beside its range grouping (`CacheField::ungrouped`, `keep_ungrouped` in
crates/rustsheet-engine/src/pivot.rs), so that the xlsx export can write them back. A grouped field therefore holds
two record vectors, the group index per record and the own item per record, and a cache can hold up to twice the
record values `MAX_RECORD_VALUES` counts. This is an accepted factor, documented at the constant.

The group of a record is a function of its own item: `group_by_numbers` and `group_by_dates` build exactly such a
map (per base item, its group) before they map the records. Keeping that map and the own records would cost one
`u32` per own item rather than one per record. Every reader of `CacheField::records` on a grouped field would then
read through the map: the pivot result (`result.rs`), filters, GETPIVOTDATA (`pivot_index.rs`), `refresh` and the
export.

## Acceptance criteria
- [ ] A field grouped in place stores one record vector plus an item-to-group map. Pivots, GETPIVOTDATA, refresh
  and the xlsx export answer as before (the T-4734 round-trip tests and the pivot probe suites unchanged).
- [ ] `MAX_RECORD_VALUES`' factor-of-two note is removed, or states the new bound.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
