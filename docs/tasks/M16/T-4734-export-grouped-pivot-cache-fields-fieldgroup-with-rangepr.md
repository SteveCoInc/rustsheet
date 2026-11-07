---
id: T-4734
title: "Export grouped pivot cache fields (fieldGroup with rangePr, discretePr and groupItems)"
milestone: M16
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
A pivot over a grouped cache field (numeric or date ranges, manual groups) survives an xlsx save: the export writes the field's `<fieldGroup>` (`rangePr` or `discretePr`, `groupItems`) and the grouping fields Excel adds (`databaseField="0"`), so Excel opens it and shows the same groups.

## Context
T-4176 exports pivots, but drops a cache with a grouped field, and every pivot on it, with a log line (`cache::check` in crates/rustsheet-xlsx/src/export/pivots/cache.rs). The importer turns a group into a field of group items and keeps no base values (crates/rustsheet-xlsx/src/import/pivots/groups.rs `apply_groups`): a field grouped in place loses its own values, so nothing is left to write the base field's shared items and records from. The model needs the base field's items and records kept beside the group (or the import must keep the base field and derive the group view), then the export writes `fieldGroup@base`/`par`, `rangePr` (`NumberGroups`, `DateGroups` bounds and `groupBy`) or `discretePr`, and `groupItems`. Evidence for what Excel writes: probes/t4128-pivot-spike G, t4249-pivot-range-group-*, t4236 date groups; docs/excel-semantics/pivots.md "Numeric and date group labels are rewritten by Excel".

## Acceptance criteria
- [x] Export then import of a grouped cache (numeric range, date range, discrete) gives an equal model; GETPIVOTDATA on group items answers the same after the round trip.
- [x] An engine-exported grouped pivot opens in Excel for the web without repair and renders the same cells (oracle recalc, as T-4176 did).
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome
Grouped pivot caches now export with their `<fieldGroup>`s instead of being dropped
(crates/rustsheet-xlsx/src/export/pivots/groups.rs; `cache::check` calls `groups::check` in place of the drop).
- Model: a field grouped in place by a range keeps its own values, `CacheField::ungrouped` (`UngroupedValues`:
  items and records). `group_by_numbers` and `group_by_dates` fill it, so the importer, the refresh (`regroup`) and
  `Txn::add_pivot_table` all keep it; an added field (Quarters, Years, Product2) keeps none, because its values come
  from its base. Each field holds at most one copy, and the export's item limit counts it.
- Shapes written, as the case builder's goldens (t4170-*) and t4128 G have them:
  - a field grouped in place: its own values as shared items and `<x>` records, then `<fieldGroup [par] base>`
    with `rangePr` (`autoStart="0" autoEnd="0"`; numbers or `groupBy` dates) and `groupItems`;
  - date levels: `databaseField="0"` fields with `<fieldGroup base>`, `rangePr` and `groupItems`, and `par` on the
    base naming the last level;
  - a manual group: a `databaseField="0"` field with `<fieldGroup base>`, `discretePr` and `groupItems`, and a
    `par`-only `<fieldGroup>` on the base.

  Records hold only the database fields. All text goes through `escape_xml`. `DateGroupBy::name` is the inverse
  of `parse`.
- Faithfulness: a grouping is written only when the import's own rule (`group_by_numbers`, `group_by_dates`,
  `group_by_discrete`), applied to the written base values, gives back the field the model holds. Every other shape
  is still dropped with a log line: kept values without a grouping, a range grouping without kept values, a numeric
  level, a manual group beside a range group, two manual groups of one field, an added field before a database
  field, or a date group in a 1904 workbook (the import refuses that last one).
- Oracle (excel-graph, scratch, pivots.md "Excel for the web opens engine-exported grouped pivot caches"): Excel for
  the web opened engine exports of t4170 manual groups and date levels, t4236 no-blank, t4228 and t4128 without
  repair, and of an API-built cache, and rendered the same cells and GETPIVOTDATA as the sidecars. The first run
  showed that a date field grouped in place also needs `containsNonDate="0"` and `minDate`/`maxDate`; without
  them Excel dropped the months. The export now writes those flags (`write_date_flags`).
- Tests: group_tests.rs round-trips a numeric range group, a date group of one level, nested date levels and a
  manual group, checking GETPIVOTDATA on group items before and after, plus the shapes that are dropped.
  rustsheet-eval's `every_probe_pivot_round_trips_through_the_export` now requires the 15 grouped probe pivots to
  come back equal; before this task they were dropped.
- Review fixes:
  - The check is linear in the cache. The added fields are indexed by base in one pass (`Added`, skipped when
    the cache has none), and each grouped field's own values are copied once, to group them again.
  - The XML is written from borrowed slices (`SharedItemsFlags::of_items` and the free fn
    `has_case_variant_texts`).
  - `group_by_numbers` and `group_by_dates` no longer copy their base. A caller grouping in place moves the base
    into `ungrouped` with `keep_ungrouped`: the importer, `refresh`, and `Txn::add_pivot_table` (edit/pivot.rs).
    Added levels get no copy.
  - The kept records are left out of the record-value budget. The factor of two is documented at
    `MAX_RECORD_VALUES` as an accepted bound. Follow-up T-4907 stores an item-to-group map instead.
  - The re-render cases moved to T-4177 and the flag questions to T-4654.
- UNVERIFIED (T-4177 commits re-render cases; T-4654 covers the flags): Excel's own `maxDate` is one day past the
  last date; groups renamed through `n`; groups over blanks; and whether `autoStart`/`autoEnd` on would matter. The
  model does not keep those two flags.
