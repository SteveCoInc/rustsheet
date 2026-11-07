---
id: T-5027
title: "Probe pivot cache item values in shared items and records to the evidence floor (v, u, x, count, longText)"
milestone: M16
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Each PV1 cache row below has at least 30 oracle cells: shared items and record values of each type (`b`, `d`, `e`, `m`, `n`, `s`, and a record's `x` index), their `v` and unused `u` flag, `sharedItems@count` and `@longText`, and the x14 cache extensions are evidenced, and the engine follows the evidence.

## Context
Filed by T-4992 as the `owner` (eval/catalog/pivot-features.tsv) of these rows, which the M16 exit row (report.json `milestone_exit`, `pivot.stages[].unowned`) listed as remaining with no open M16 owner at the 2026-11-07 run; the owner rule is docs/architecture/eval-harness.md "Milestone exit" (T-4952). A row is remaining while it has fewer than 30 oracle cells or a failing cell. Rows (52): PV-0247 `sharedItems/b@u`; PV-0248 `sharedItems/b@v`; PV-0255 `sharedItems/d@u`; PV-0256 `sharedItems/d@v`; PV-0275 `sharedItems/e@u`; PV-0277 `sharedItems/e@v`; PV-0296 `sharedItems/m@u`; PV-0316 `sharedItems/n@u`; PV-0318 `sharedItems/n@v`; PV-0337 `sharedItems/s@u`; PV-0339 `sharedItems/s@v`; PV-0348 `sharedItems@count`; PV-0349 `sharedItems@longText`; PV-1260 `x14:cacheField`; PV-1275 `x14:pivotCacheDefinition`; PV-0679 `pivotCacheRecords/r/b`; PV-0685 `pivotCacheRecords/r/b@u`; PV-0686 `pivotCacheRecords/r/b@v`; PV-0687 `pivotCacheRecords/r/d`; PV-0693 `pivotCacheRecords/r/d@u`; PV-0694 `pivotCacheRecords/r/d@v`; PV-0695 `pivotCacheRecords/r/e`; PV-0713 `pivotCacheRecords/r/e@u`; PV-0715 `pivotCacheRecords/r/e@v`; PV-0716 `pivotCacheRecords/r/m`; PV-0734 `pivotCacheRecords/r/m@u`; PV-0736 `pivotCacheRecords/r/n`; PV-0754 `pivotCacheRecords/r/n@u`; PV-0756 `pivotCacheRecords/r/n@v`; PV-0757 `pivotCacheRecords/r/s`; PV-0775 `pivotCacheRecords/r/s@u`; PV-0777 `pivotCacheRecords/r/s@v`; PV-0778 `pivotCacheRecords/r/x`; PV-0779 `pivotCacheRecords/r/x@v`; and the cache item flag `f` (moved from T-4175 in review: no evidence ties it to calculated items): PV-0132 `groupItems/b@f`; PV-0140 `groupItems/d@f`; PV-0156 `groupItems/e@f`; PV-0177 `groupItems/m@f`; PV-0197 `groupItems/n@f`; PV-0218 `groupItems/s@f`; PV-0246 `sharedItems/b@f`; PV-0254 `sharedItems/d@f`; PV-0270 `sharedItems/e@f`; PV-0291 `sharedItems/m@f`; PV-0311 `sharedItems/n@f`; PV-0332 `sharedItems/s@f`; PV-0684 `pivotCacheRecords/r/b@f`; PV-0692 `pivotCacheRecords/r/d@f`; PV-0708 `pivotCacheRecords/r/e@f`; PV-0729 `pivotCacheRecords/r/m@f`; PV-0749 `pivotCacheRecords/r/n@f`; PV-0770 `pivotCacheRecords/r/s@f`.

Each row has fewer than 30 oracle cells and none failing: no committed case lists it in `pivot_features`, even where a probe may already exercise it. The work is evidence first: tag the committed probes that exercise each row (their `pivot_features`) where the probe really varies or depends on it, add probe cells (case builder v2, `rustsheet-oracle fill` on excel-graph) until each row has at least 30, record what Excel shows in docs/excel-semantics/pivots.md, and then make the engine follow it, filing an engine task for any behaviour the probe shows the engine does not match. A row Excel for the web refuses gets `web` = `no` (capability matrix, pivots.md) with its evidence, and then is M14's. Stale and unused items (`u`): pivots.md "Shared-item flags" and T-4654. The OLAP-only item attributes (member properties, tuples, formatting) are the scope proposal's, not this task's.

When this task closes, re-point or clear its rows' `owner` cells in the same PR (`scripts/tasks.py validate` fails on a done owner, scripts/pivot_owners.py).

## Acceptance criteria
- [ ] Each row listed has at least 30 oracle cells in `make eval`'s Pivot features table, or `web` = `no` with evidence.
- [ ] pivots.md records what Excel shows; the engine matches it, or an engine task owns the gap.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the ownership rule (T-4952).
