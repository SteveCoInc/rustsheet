---
id: T-4654
title: "Probe pivot shared-item flags and saved item order when stale cache items remain after a refresh"
milestone: M16
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
pivots.md "Shared-item flags Excel accepts for mixed fields" states how Excel counts the stale items a refresh leaves in the cache (old items no record holds any more) in a field's `<sharedItems>` flags and in its saved item order, and `SharedItemsFlags` (crates/rustsheet-engine/src/pivot/refresh.rs) follows it.

## Context
T-4612 (probes/t4612-pivot-date-items-and-casing; the sidecar's `cache_fields` carry each saved shared item's type and the `<sharedItems>` attributes) left these open:
- Stale items and the flags. T1 (dates beside stale `<n>` items) saved `containsNonDate="0"`, so the stale numbers did not count there, yet `containsMixedTypes="1"`. C1 (texts beside a stale `<m/>`) saved `containsBlank="1"`, so the stale blank did count. Probe a field of dates only with no stale item (a cache the builder lists empty or as `<d>`), dates with a stale `<s>`, and numbers with a stale `<s>`, to see which flags stale items set.
- `minDate` and `maxDate`. Every date field saved a right `minDate` but a `maxDate` below it (T1 `1900-01-10T12:50:04` for 45000 and 45001, T2 `1899-12-31T03:45:03`, T3 `1899-12-31T00:03:04`). Is this stable across runs, and does desktop Excel write it (M14 Phase 4 if only desktop can answer)?
- Saved item order. Excel saved the shared items in source order first, the stale ones after them (T1: `<d>` 45000, `<d>` 45001, then the old `<n>` 45000 and 45001; C1: `EAST`, then `<m/>`), while the pivot field's items keep the old order with new ones appended (rowItems x = 2, 3). `FieldReader` keeps one list, old items first. The export (T-4176) needs the shared order and the pivot-field order separately; record which one, and whether the records index the new shared order.

## Acceptance criteria
- [ ] A filled probe answers each point above; pivots.md states the rules with cells.
- [ ] `SharedItemsFlags` follows the evidence, with unit tests citing the probe.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Writing the cache (T-4176).
