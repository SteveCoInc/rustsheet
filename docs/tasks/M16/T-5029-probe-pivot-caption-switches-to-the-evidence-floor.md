---
id: T-5029
title: "Probe pivot caption switches to the evidence floor (cacheField@caption, showHeaders, showMultipleLabel)"
milestone: M16
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Each PV1 captions row below has at least 30 oracle cells: a cache field's `caption`, `showHeaders` and `showMultipleLabel` are evidenced, and the engine follows the evidence.

## Context
Filed by T-4992 as the `owner` (eval/catalog/pivot-features.tsv) of these rows, which the M16 exit row (report.json `milestone_exit`, `pivot.stages[].unowned`) listed as remaining with no open M16 owner at the 2026-11-07 run; the owner rule is docs/architecture/eval-harness.md "Milestone exit" (T-4952). A row is remaining while it has fewer than 30 oracle cells or a failing cell. Rows (3): PV-0354 `cacheFields/cacheField@caption`; PV-1242 `pivotTableDefinition@showHeaders`; PV-1246 `pivotTableDefinition@showMultipleLabel`.

Each row has fewer than 30 oracle cells and none failing: no committed case lists it in `pivot_features`, even where a probe may already exercise it. The work is evidence first: tag the committed probes that exercise each row (their `pivot_features`) where the probe really varies or depends on it, add probe cells (case builder v2, `rustsheet-oracle fill` on excel-graph) until each row has at least 30, record what Excel shows in docs/excel-semantics/pivots.md, and then make the engine follow it, filing an engine task for any behaviour the probe shows the engine does not match. A row Excel for the web refuses gets `web` = `no` (capability matrix, pivots.md) with its evidence, and then is M14's. `showMultipleLabel` decides the `(Multiple Items)` page caption that T-4921 and T-4969 touch; `cacheField@caption` may be OLAP only (if the probe shows so, propose it to the scope task).

When this task closes, re-point or clear its rows' `owner` cells in the same PR (`scripts/tasks.py validate` fails on a done owner, scripts/pivot_owners.py).

## Acceptance criteria
- [ ] Each row listed has at least 30 oracle cells in `make eval`'s Pivot features table, or `web` = `no` with evidence.
- [ ] pivots.md records what Excel shows; the engine matches it, or an engine task owns the gap.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the ownership rule (T-4952).
