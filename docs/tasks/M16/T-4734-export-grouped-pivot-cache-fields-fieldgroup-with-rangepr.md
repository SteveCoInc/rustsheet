---
id: T-4734
title: "Export grouped pivot cache fields (fieldGroup with rangePr, discretePr and groupItems)"
milestone: M16
lane: engine
status: todo
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
- [ ] Export then import of a grouped cache (numeric range, date range, discrete) gives an equal model; GETPIVOTDATA on group items answers the same after the round trip.
- [ ] An engine-exported grouped pivot opens in Excel for the web without repair and renders the same cells (oracle recalc, as T-4176 did).
- [ ] `make check` green; reviewers approve.

## Out of scope
-
