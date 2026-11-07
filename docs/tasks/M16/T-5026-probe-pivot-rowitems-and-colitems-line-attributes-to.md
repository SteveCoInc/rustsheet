---
id: T-5026
title: "Probe pivot rowItems and colItems line attributes to the evidence floor (i@i, i@r, i@t, x, item@t)"
milestone: M16
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Each PV1 row below has at least 30 oracle cells: the `rowItems`/`colItems` line attributes (`i`, `r`, `t`, `count`, the `x` index), and a pivot field item's `t`, are evidenced, and the engine follows the evidence.

## Context
Filed by T-4992 as the `owner` (eval/catalog/pivot-features.tsv) of these rows, which the M16 exit row (report.json `milestone_exit`, `pivot.stages[].unowned`) listed as remaining with no open M16 owner at the 2026-11-07 run; the owner rule is docs/architecture/eval-harness.md "Milestone exit" (T-4952). A row is remaining while it has fewer than 30 oracle cells or a failing cell. Rows (13): PV-0836 `colItems/i/x`; PV-0837 `colItems/i/x@v`; PV-0838 `colItems/i@i`; PV-0839 `colItems/i@r`; PV-0840 `colItems/i@t`; PV-0841 `colItems@count`; PV-1084 `pivotFields/pivotField/items/item@t`; PV-1185 `rowItems/i/x`; PV-1186 `rowItems/i/x@v`; PV-1187 `rowItems/i@i`; PV-1188 `rowItems/i@r`; PV-1189 `rowItems/i@t`; PV-1190 `rowItems@count`.

Each row has fewer than 30 oracle cells and none failing: no committed case lists it in `pivot_features`, even where a probe may already exercise it. The work is evidence first: tag the committed probes that exercise each row (their `pivot_features`) where the probe really varies or depends on it, add probe cells (case builder v2, `rustsheet-oracle fill` on excel-graph) until each row has at least 30, record what Excel shows in docs/excel-semantics/pivots.md, and then make the engine follow it, filing an engine task for any behaviour the probe shows the engine does not match. A row Excel for the web refuses gets `web` = `no` (capability matrix, pivots.md) with its evidence, and then is M14's. The sidecar already records Excel's saved `rowItems`/`colItems` and the harness scores them (eval-harness.md, layout criterion), so these rows may need only tagging plus cells that vary each attribute (a data-field line `i`, a repeated-prefix `r`, each line type `t`). Related: T-4649, T-4650.

When this task closes, re-point or clear its rows' `owner` cells in the same PR (`scripts/tasks.py validate` fails on a done owner, scripts/pivot_owners.py).

## Acceptance criteria
- [ ] Each row listed has at least 30 oracle cells in `make eval`'s Pivot features table, or `web` = `no` with evidence.
- [ ] pivots.md records what Excel shows; the engine matches it, or an engine task owns the gap.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the ownership rule (T-4952).
