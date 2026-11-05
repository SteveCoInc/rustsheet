---
id: T-4316
title: "Probe numeric pivot groups the case builder cannot write yet: a label listed twice, a non-text group item, a numeric grouping over date cells alone"
milestone: M6
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The three numeric pivot range-group points T-4287 could not probe are settled by excel-graph probes, and the engine
matches them or the cells are owned.

## Context
- Follow-up of T-4287 (review MINOR). docs/excel-semantics/web.md "Grouped pivot fields import as their group items"
  lists them as UNVERIFIED; the case builder (tools/src/rustsheet_tools/pivot.py `_group`) refuses each today:
  - a `<groupItems>` label listed twice (the engine's `NumberGroups::new` keeps its first place);
  - a non-text group item, e.g. `<n v="3"/>` (the engine skips it);
  - a numeric grouping (`<rangePr>` without `groupBy`) over date-formatted cells only. T-4287 found that Excel's
    refresh drops a numeric grouping over dates mixed with numbers; over dates alone `Txn::add_pivot_table` keeps it,
    and `group_by_numbers` counts `CacheItem::Date` as a number in the whole-data test (number_groups.rs).
- Use the `excel-probe` skill; extend the builder (its own small change) where needed.

## Acceptance criteria
- [x] An excel-graph probe covers each point above, or the task names the ones excel-graph cannot show; web.md
      records the result.
- [x] The engine matches every probe cell, or the cells are known failures owned by a task.
- [x] `make check` green; reviewers approve.

## Out of scope
- Date groups (`groupBy`) and the points T-4287 settled.

## Outcome
- Builder (tools/ pivot.py `_group`): `<groupItems>` may now repeat a label and hold numbers (`<n v>`); the eval case
  schema (`PivotGroupItem`) reads both, and the adapter leaves a number item out (it matches no label).
- Probes (excel-graph, 2026-11-05), probes/t4316-pivot-group-label-twice, -number-item, -dates-alone,
  -dates-alone-frac:
  - a label listed twice takes its **last** place (`6-9`, `3-5`, `6-9`, ... shows `3-5`, `6-9`, ...); the engine took
    its first. `NumberGroups::new` now keeps the last.
  - a number item is skipped, as the engine already did.
  - a numeric grouping over date cells alone is **dropped** on refresh, like dates among numbers (T-4287);
    `Txn::add_pivot_table` kept it. `grouped` now drops it for any date cell (`holds_dates`).
- web.md records all three (VERIFIED); the UNVERIFIED notes are gone.
- Eval delta: probes gains 4 cases, 200 cells, all passing (eval/baseline/probes.txt records their coverage). No
  gated suite moves.
