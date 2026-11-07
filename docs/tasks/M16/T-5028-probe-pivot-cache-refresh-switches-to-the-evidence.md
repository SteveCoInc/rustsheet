---
id: T-5028
title: "Probe pivot cache refresh switches to the evidence floor (enableRefresh, missingItemsLimit, saveData, x15)"
milestone: M16
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Each PV1 source-refresh row below has at least 30 oracle cells: `enableRefresh`, `missingItemsLimit`, `saveData`, `x15:pivotCacheIdVersion` and `x15:pivotTableReferences` are evidenced (what Excel for the web renders and refreshes with each), and the engine follows the evidence.

## Context
Filed by T-4992 as the `owner` (eval/catalog/pivot-features.tsv) of these rows, which the M16 exit row (report.json `milestone_exit`, `pivot.stages[].unowned`) listed as remaining with no open M16 owner at the 2026-11-07 run; the owner rule is docs/architecture/eval-harness.md "Milestone exit" (T-4952). A row is remaining while it has fewer than 30 oracle cells or a failing cell. Rows (6): PV-0657 `pivotCacheDefinition@enableRefresh`; PV-0661 `pivotCacheDefinition@missingItemsLimit`; PV-0670 `pivotCacheDefinition@saveData`; PV-1476 `x15:pivotCacheIdVersion`; PV-1500 `x15:pivotTableReferences`; PV-1501 `x15:pivotTableReferences/pivotTableReference`.

Each row has fewer than 30 oracle cells and none failing: no committed case lists it in `pivot_features`, even where a probe may already exercise it. The work is evidence first: tag the committed probes that exercise each row (their `pivot_features`) where the probe really varies or depends on it, add probe cells (case builder v2, `rustsheet-oracle fill` on excel-graph) until each row has at least 30, record what Excel shows in docs/excel-semantics/pivots.md, and then make the engine follow it, filing an engine task for any behaviour the probe shows the engine does not match. A row Excel for the web refuses gets `web` = `no` (capability matrix, pivots.md) with its evidence, and then is M14's. Related: T-4654 (stale items a refresh leaves, which `missingItemsLimit` bounds), T-4664 (refresh around other pivots and shared caches).

When this task closes, re-point or clear its rows' `owner` cells in the same PR (`scripts/tasks.py validate` fails on a done owner, scripts/pivot_owners.py).

## Acceptance criteria
- [ ] Each row listed has at least 30 oracle cells in `make eval`'s Pivot features table, or `web` = `no` with evidence.
- [ ] pivots.md records what Excel shows; the engine matches it, or an engine task owns the gap.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the ownership rule (T-4952).
