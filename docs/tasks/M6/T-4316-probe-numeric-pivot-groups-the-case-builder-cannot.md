---
id: T-4316
title: "Probe numeric pivot groups the case builder cannot write yet: a label listed twice, a non-text group item, a numeric grouping over date cells alone"
milestone: M6
lane: engine
status: todo
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
- [ ] An excel-graph probe covers each point above, or the task names the ones excel-graph cannot show; web.md
      records the result.
- [ ] The engine matches every probe cell, or the cells are known failures owned by a task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Date groups (`groupBy`) and the points T-4287 settled.
