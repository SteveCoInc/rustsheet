---
id: T-4171
title: "Pivot grouping in refresh and PivotResult, reusing the M6 group rules"
milestone: M16
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: [T-4170, T-4141]
---

## Goal
A refreshed pivot builds its groups from the source as Excel does, and PivotResult lays out grouped fields in every form, reusing the M6 rules (`pivot/number_groups.rs`, `pivot/date_groups.rs`, the importer's `import/pivots/groups.rs`) rather than a second implementation; manual groups and nested date levels follow T-4170's evidence.

## Context
Rescoped 2026-11-06. M6 implemented grouping at import (group labels become cache items) and a compact one-field layout for grouped spec pivots (`pivot/layout.rs`, T-4235/T-4246). What is missing is grouping during a refresh from the source, grouped fields in multi-field and tabular/outline layouts, and manual and multi-level groups. SPEC-0032 decides where the shared group code lives (directive 3). Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

The owner lines this task held (decision 0245: probes/t4128-pivot-spike G!A3:B7, every pivot of
probes/t4170-pivot-manual-groups and probes/t4170-pivot-date-levels, and t4170-pivot-manual-groups Calc!B16, Calc!B18)
now pass; the manual-group GETPIVOTDATA cells still off (Calc!B1:B5, B8:B10, B12, B14) moved to T-4154, their cause now
SPEC-0015 unknown 10 (see the Outcome).

## Acceptance criteria
- [x] The importer accepts a `refreshOnLoad` cache's pivotField `<item x>` entries past its rebuilt groups (the refresh lays the field out again), so probes/t4128-pivot-spike G renders Excel's 6 items.
- [ ] T-4170 probes pass; the M6 grouping probes keep passing; corpus group pivots re-render.
- [x] No duplicated grouping logic (`scripts/check_duplication.py`).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Auto date grouping on field drop (a UI action).

## Outcome
Done in one PR (no split). The three owner causes are fixed by extending the M6 code, not a second implementation:
- (a) `read_item` (crates/rustsheet-xlsx/src/import/pivots/table.rs) drops a numeric range group's `<item x>` past
  its rebuilt groups when the cache is `refreshOnLoad` (the refresh lays the field out again), the case t4128 G
  evidences; otherwise, date and manual groups included, it still refuses the pivot (T-4802). probes/t4128-pivot-spike
  G renders Excel's 6 items.
- (b) `apply_groups` (crates/rustsheet-xlsx/src/import/pivots/groups.rs) reads a source field's bare `<fieldGroup
  par>` as naming its group field, not as a group, when `par` names an added field whose `base` is that field (any
  other bare group still skips the cache), and keeps each added field's base in the new
  `CacheField::added_group` (`AddedGroup`). The discrete rule moved into the engine as `group_by_discrete`, which the
  importer and the refresh share.
- (c) `refresh` derives every added field (Quarters, Years, a manual group field) again from its base's new values,
  before the base is grouped in place (crates/rustsheet-engine/src/pivot/added_groups.rs `derive_added`, through
  `regroup` and `group_by_discrete`), so `remap` finds the fields by name and the existing layout stages render the
  grouped pivots in compact, outline and tabular form with default and custom subtotals; manual groups follow the
  pivotField item order (MP). `plan_pivot_refresh` charges the derivation to the shared `PivotBudget` (records x added
  fields), and the record-value and item limits are checked before anything is derived. The export
  (`cache::check`) refuses a cache with an added group field as it refuses other grouped caches (T-4734).

Eval: `make eval-baseline` fixed 877 cells (probes known failures 8083 -> 7638, text 1756 -> 1324, on origin/main 973eb152b), no
regressions in any suite; corpus-bench 2 -> 2 known failures, from a local run after fetching the suite. Every pivot cell of probes/t4170-pivot-
manual-groups and probes/t4170-pivot-date-levels passes (205 of 215 and 420 of 431 checked cells), as do the M6
grouping probes. The cells still off are GETPIVOTDATA on pivots whose rows hold two fields (SPEC-0015 unknown 10):
the date-level ones were already T-4154's, and the manual-group ones (Calc!B1:B5, B8:B10, B12, B14) moved there.
Criterion 2 is left open on its corpus part only, with no follow-up task: the corpus-bench grouped caches (48357,
52379, 59932) are not `refreshOnLoad`, and nothing refreshes a cache without it except an explicit
`Txn::refresh_pivot_table`, which the eval never calls (pivots.md "Pivots without refreshOnLoad, and source edits"). So
no load re-renders them: they keep their saved cells and still pass (corpus-bench 2 -> 2 known failures, local fetch run). Checking a
re-render would need Excel's cells after refreshing those caches, which no sidecar holds; T-4802 owns the open
grouping evidence. Criterion 3 holds after review: the range-grouping mapping is one `RangeGrouping::of`, shared by
the refresh's in-place regroup and the added levels, and the added-field count is one `PivotCache::added_group_count`.

Tests: crates/rustsheet-engine/src/pivot/added_groups/tests.rs (refresh against the t4170 sidecars: MG's group totals,
DY's year totals, the saved group items; the limits and refusals), and two importer tests at the end of
crates/rustsheet-xlsx/src/import/pivots/tests.rs (the `par`-only group and the stale item, with and without
`refreshOnLoad`, on date and manual groups, and the refused `par` shapes).

UNVERIFIED, recorded in pivots.md and handed to T-4802: a base item new to the source becomes a group item of its own;
the refresh is refused when that label equals a group's, when a date level's base gains a blank, or when the source
gains a column named as an added field; an added field whose base left the source goes with it; the stale-item
tolerance without `refreshOnLoad` or on date and manual groups; and other bare `<fieldGroup par>` shapes.
