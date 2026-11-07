---
id: T-5032
title: "Probe pivot timeline caches and timelines to the evidence floor (timelineCacheDefinition, x15:timeline)"
milestone: M16
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Each PV5 timeline row below has at least 30 oracle cells: a timeline cache's pivot table entries, bounds, name and source, `timelinePivotCacheDefinition@timelineData`, and the timeline part's cache, caption, name and selection level are evidenced, and the engine follows the evidence.

## Context
Filed by T-4992 as the `owner` (eval/catalog/pivot-features.tsv) of these rows, which the M16 exit row (report.json `milestone_exit`, `pivot.stages[].unowned`) listed as remaining with no open M16 owner at the 2026-11-07 run; the owner rule is docs/architecture/eval-harness.md "Milestone exit" (T-4952). A row is remaining while it has fewer than 30 oracle cells or a failing cell. Rows (15): PV-1524 `x15:timelineCacheDefinition/pivotTables`; PV-1525 `x15:timelineCacheDefinition/pivotTables/pivotTable`; PV-1526 `x15:timelineCacheDefinition/pivotTables/pivotTable@name`; PV-1527 `x15:timelineCacheDefinition/pivotTables/pivotTable@tabId`; PV-1529 `x15:timelineCacheDefinition/state/bounds`; PV-1530 `x15:timelineCacheDefinition/state/bounds@endDate`; PV-1531 `x15:timelineCacheDefinition/state/bounds@startDate`; PV-1606 `x15:timelineCacheDefinition@name`; PV-1607 `x15:timelineCacheDefinition@sourceName`; PV-1610 `x15:timelinePivotCacheDefinition@timelineData`; PV-1620 `x15:timelines/timeline`; PV-1622 `x15:timelines/timeline@cache`; PV-1623 `x15:timelines/timeline@caption`; PV-1625 `x15:timelines/timeline@name`; PV-1627 `x15:timelines/timeline@selectionLevel`.

Each row has fewer than 30 oracle cells and none failing: no committed case lists it in `pivot_features`, even where a probe may already exercise it. The work is evidence first: tag the committed probes that exercise each row (their `pivot_features`) where the probe really varies or depends on it, add probe cells (case builder v2, `rustsheet-oracle fill` on excel-graph) until each row has at least 30, record what Excel shows in docs/excel-semantics/pivots.md, and then make the engine follow it, filing an engine task for any behaviour the probe shows the engine does not match. A row Excel for the web refuses gets `web` = `no` (capability matrix, pivots.md) with its evidence, and then is M14's. probes/t4767-pivot-timelines evidences one timeline on a field on no axis (T-4188, done) but lists none of these rows. Related: T-4962 (the engine's unprobed timeline readings and the selection rows), T-4955 (Excel-saved timelinePivotFilter), T-4956, T-4963.

When this task closes, re-point or clear its rows' `owner` cells in the same PR (`scripts/tasks.py validate` fails on a done owner, scripts/pivot_owners.py).

## Acceptance criteria
- [ ] Each row listed has at least 30 oracle cells in `make eval`'s Pivot features table, or `web` = `no` with evidence.
- [ ] pivots.md records what Excel shows; the engine matches it, or an engine task owns the gap.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the ownership rule (T-4952).
