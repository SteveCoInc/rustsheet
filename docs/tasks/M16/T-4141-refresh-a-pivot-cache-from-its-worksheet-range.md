---
id: T-4141
title: "Refresh a pivot cache from its worksheet range or Table source"
milestone: M16
lane: engine
status: done
priority: P1
size: M
tier: opus
depends_on: [T-4139, T-4134, T-4585]
---

## Goal
Refreshing reads the source (range, Table, or defined name) and rebuilds the cache exactly as Excel does: field names (duplicates renamed), item typing, identity and order, sharedItems flags, record values.

## Context
SPEC-0032; evidence from task `p-items`. The CacheBuilder of SPEC-0015 is the one builder (directive 3). Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [x] Unit tests and the p-items probes' cache-level expectations pass.
- [x] Duplicate and blank source header names follow T-4585's probe: the verified cases, with the open points per T-4627.
- [x] `make check` green; reviewers approve.

## Out of scope
- Grouped and calculated cache fields (PV3).

## Outcome
`crates/rustsheet-engine/src/pivot/refresh.rs` adds `refresh(cache, source, workbook_values_left)`, its
first step `read_source` (a `CacheBuilder` for callers that add items before finishing), the `SourceCells`
trait (implemented by `Model` over stored, evaluated values: no formula evaluation, no I/O; a number in a
date format reads as a date), `regroup` (the range-grouping drop rules, moved from the facade) and
`SharedItemsFlags` (derived from the items, for the export, T-4176). The source's size is checked against
the limits before any cell is read. `Txn::add_pivot_table` now builds its cache through `read_source` and
`regroup`, so the eval adapter's pivots name, type and identify items as refresh does (eval unchanged:
REGRESSIONS 0, fixed 0). Rules, with unit tests built from the probes' data: field names per T-4585's probe
(repeats numbered per name from 2, blank headers `Field<n>` by position in the source), item identity per
t4134 I1/I4/I5, listed order kept with new items appended in source order (t4134 I2/I3, t4582 J2), a cached
text slot relabelled with the source's casing (t4582 J1), date values never matching cached numbers
(t4582 J3/J4). Left open: which casing relabels and whether appended dates are `<d>` (T-4612); `Field<n>`
by position or sheet column, an empty header cell, and name collisions (T-4627); defined-name and Table
sources, which refresh as `PivotCacheError::Source` (T-4506); the refresh-on-open trigger (T-4151); a number,
boolean or error header (refused as `PivotCacheError::Header`; a literal `#N/A` header was a field `#N/A`
in `Txn::add_pivot_table` before) and case-folded old-to-new field matching (T-4627); date flags in
`SharedItemsFlags` and time-only or elapsed formats read as dates (T-4612). A refresh returns the fields
in source order and does not remap pivot tables' field indices: T-4151's swap must.
