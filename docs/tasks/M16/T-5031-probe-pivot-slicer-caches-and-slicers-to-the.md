---
id: T-5031
title: "Probe pivot slicer caches and slicers to the evidence floor (slicerCacheDefinition, x14:slicer)"
milestone: M16
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Each PV5 slicer row below has at least 30 oracle cells: a slicer cache's tabular data, item count, sort and missing-item switches, its pivot table entries, names and source, and the slicer part's cache, caption and name are evidenced, and the engine follows the evidence.

## Context
Filed by T-4992 as the `owner` (eval/catalog/pivot-features.tsv) of these rows, which the M16 exit row (report.json `milestone_exit`, `pivot.stages[].unowned`) listed as remaining with no open M16 owner at the 2026-11-07 run; the owner rule is docs/architecture/eval-harness.md "Milestone exit" (T-4952). A row is remaining while it has fewer than 30 oracle cells or a failing cell. Rows (16): PV-1400 `x14:slicerCacheDefinition/data`; PV-1427 `x14:slicerCacheDefinition/data/tabular`; PV-1434 `x14:slicerCacheDefinition/data/tabular/items@count`; PV-1436 `x14:slicerCacheDefinition/data/tabular@customListSort`; PV-1438 `x14:slicerCacheDefinition/data/tabular@showMissing`; PV-1439 `x14:slicerCacheDefinition/data/tabular@sortOrder`; PV-1443 `x14:slicerCacheDefinition/pivotTables/pivotTable@name`; PV-1444 `x14:slicerCacheDefinition/pivotTables/pivotTable@tabId`; PV-1446 `x14:slicerCacheDefinition@name`; PV-1447 `x14:slicerCacheDefinition@sourceName`; PV-1458 `x14:slicers/slicer`; PV-1460 `x14:slicers/slicer@cache`; PV-1461 `x14:slicers/slicer@caption`; PV-1465 `x14:slicers/slicer@name`; PV-1431 `x14:slicerCacheDefinition/data/tabular/items/i@nd`; PV-1514 `x15:slicerCacheHideItemsWithNoData@count` (moved from T-4956 in review: T-4956 only records them).

Each row has fewer than 30 oracle cells and none failing: no committed case lists it in `pivot_features`, even where a probe may already exercise it. The work is evidence first: tag the committed probes that exercise each row (their `pivot_features`) where the probe really varies or depends on it, add probe cells (case builder v2, `rustsheet-oracle fill` on excel-graph) until each row has at least 30, record what Excel shows in docs/excel-semantics/pivots.md, and then make the engine follow it, filing an engine task for any behaviour the probe shows the engine does not match. A row Excel for the web refuses gets `web` = `no` (capability matrix, pivots.md) with its evidence, and then is M14's. probes/t4767-pivot-slicers evidences one tabular slicer (pivots.md "Slicers and timelines filter the pivot they drive", T-4188, done) but lists none of these rows. Related: T-4962 (the engine's unprobed slicer readings), T-4956 (sidecar records of slicer caches), T-4963 (keeping the parts on save).

When this task closes, re-point or clear its rows' `owner` cells in the same PR (`scripts/tasks.py validate` fails on a done owner, scripts/pivot_owners.py).

## Acceptance criteria
- [ ] Each row listed has at least 30 oracle cells in `make eval`'s Pivot features table, or `web` = `no` with evidence.
- [ ] pivots.md records what Excel shows; the engine matches it, or an engine task owns the gap.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the ownership rule (T-4952).
