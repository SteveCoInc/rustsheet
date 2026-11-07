---
id: T-4182
title: "PivotTable Auto Refresh as evidenced"
milestone: M16
lane: engine
status: dropped
priority: P3
size: S
tier: opus
depends_on: [T-4130, T-4151]
---

## Goal
If the capability matrix shows Auto Refresh is stored in the file and honoured by Excel for the web, pivots marked so refresh after source edits as Excel does; otherwise this task records why not and is dropped.

## Context
excel-docs.md AUTOREFRESH (2025 Insider). Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.
T-4130's answer (pivots.md "PivotTable Auto Refresh is not stored or honoured by Excel for the web";
probes/t4130-pivot-auto-refresh): Excel for the web saves no Auto Refresh setting and does not refresh a pivot after a
source edit, with or without `refreshOnLoad`. Only a desktop-saved file could show a flag; without one this task is
dropped with that reason.

## Acceptance criteria
- [x] Probe-backed behaviour or a dropped task with the reason.
- [x] `make check` green; reviewers approve.

## Out of scope
-

## Outcome (dropped, 2026-11-07, research)
There is nothing to implement. Excel for the web neither stores nor honours Auto Refresh.
- **Evidence:** pivots.md "PivotTable Auto Refresh is not stored or honoured by Excel for the web"
  (probes/t4130-pivot-auto-refresh, excel-graph, 2026-11-07).
  - After a Graph write to the source and a full-rebuild calculate, a pivot with `refreshOnLoad` and one without it
    both keep their old values (East 5, West 20, Grand Total 25), while a SUM over the source shows 125.
  - The saved pivotCacheDefinition and pivotTableDefinition carry no Auto Refresh attribute or extension.
- **The engine already matches this:**
  - It refreshes a pivot only after a loaded workbook's first recalculation (for `refreshOnLoad` caches) and on
    `Txn::refresh_pivot_table`. It never refreshes on a source edit or a recalculation
    (crates/rustsheet-engine/src/pivot/grid.rs, module doc; pivots.md "Pivots without refreshOnLoad keep their saved
    cells; a source edit does not re-render", probes/t4586-pivot-refresh-on-load).
  - The probe's AR0 and AR1 areas match the engine's.
- **Out of reach:** whether desktop Excel, where Auto Refresh is an Insider feature, writes a flag into the file
  stays UNVERIFIED. The oracle cannot reach desktop Excel (decision 0266).
- **When to reopen:** if a desktop-saved workbook ever shows such a flag (for example in T-4131's corpus), file a
  new task that cites that file.
