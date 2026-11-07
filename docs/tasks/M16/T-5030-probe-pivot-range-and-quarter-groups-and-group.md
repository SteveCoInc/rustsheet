---
id: T-5030
title: "Probe pivot range and quarter groups and group-item values to the evidence floor (rangePr, groupItems)"
milestone: M16
lane: research
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Each PV3 grouping row below has at least 30 oracle cells: numeric range groups and their `rangePr` bounds and `groupBy`, quarter and range levels, `groupItems@count` and group item values (`v`, `u`) are evidenced, and the engine follows the evidence.

## Context
Filed by T-4992 as the `owner` (eval/catalog/pivot-features.tsv) of these rows, which the M16 exit row (report.json `milestone_exit`, `pivot.stages[].unowned`) listed as remaining with no open M16 owner at the 2026-11-07 run; the owner rule is docs/architecture/eval-harness.md "Milestone exit" (T-4952). A row is remaining while it has fewer than 30 oracle cells or a failing cell. Rows (20): PV-0023 `ST_GroupBy=quarters`; PV-0024 `ST_GroupBy=range`; PV-0133 `fieldGroup/groupItems/b@u`; PV-0134 `fieldGroup/groupItems/b@v`; PV-0141 `fieldGroup/groupItems/d@u`; PV-0142 `fieldGroup/groupItems/d@v`; PV-0161 `fieldGroup/groupItems/e@u`; PV-0163 `fieldGroup/groupItems/e@v`; PV-0182 `fieldGroup/groupItems/m@u`; PV-0202 `fieldGroup/groupItems/n@u`; PV-0204 `fieldGroup/groupItems/n@v`; PV-0223 `fieldGroup/groupItems/s@u`; PV-0225 `fieldGroup/groupItems/s@v`; PV-0226 `fieldGroup/groupItems@count`; PV-0227 `fieldGroup/rangePr`; PV-0230 `fieldGroup/rangePr@endDate`; PV-0231 `fieldGroup/rangePr@endNum`; PV-0232 `fieldGroup/rangePr@groupBy`; PV-0234 `fieldGroup/rangePr@startDate`; PV-0235 `fieldGroup/rangePr@startNum`.

Each row has fewer than 30 oracle cells and none failing: no committed case lists it in `pivot_features`, even where a probe may already exercise it. The work is evidence first: tag the committed probes that exercise each row (their `pivot_features`) where the probe really varies or depends on it, add probe cells (case builder v2, `rustsheet-oracle fill` on excel-graph) until each row has at least 30, record what Excel shows in docs/excel-semantics/pivots.md, and then make the engine follow it, filing an engine task for any behaviour the probe shows the engine does not match. A row Excel for the web refuses gets `web` = `no` (capability matrix, pivots.md) with its evidence, and then is M14's. Committed probes already render groups (probes/t4170-pivot-manual-groups, probes/t4170-pivot-date-levels and the T-4734 export cases; pivots.md "Manual groups and date levels") but list none of these rows. Related: T-4802 (shared groups and the shapes T-4170 left open, which owns the discretePr, par, base, months and years rows), T-4654 (autoStart, autoEnd), T-4941, T-4907.

When this task closes, re-point or clear its rows' `owner` cells in the same PR (`scripts/tasks.py validate` fails on a done owner, scripts/pivot_owners.py).

## Acceptance criteria
- [ ] Each row listed has at least 30 oracle cells in `make eval`'s Pivot features table, or `web` = `no` with evidence.
- [ ] pivots.md records what Excel shows; the engine matches it, or an engine task owns the gap.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the ownership rule (T-4952).
