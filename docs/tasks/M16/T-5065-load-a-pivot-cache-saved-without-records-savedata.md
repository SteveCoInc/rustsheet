---
id: T-5065
title: "Load a pivot cache saved without records (saveData=0) and build its records from the source on refresh"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-4506]
---

## Goal
The importer loads a pivot whose cache was saved without a records part (`saveData="0"`): the cache definition and the pivot load, and a refresh builds the records from the cache's source. A `refreshOnLoad` pivot over such a cache then renders as Excel does on open.

## Context
The importer skips such a cache ("no records part (saveData=\"0\")") and every pivot over it ("cacheId N has no loaded cache"). Check (c) (T-4972, decision 0349) scores Excel's on-open render of each such `refreshOnLoad` pivot against nothing. The caches of these pivots read one-row Tables, so the refresh also needs T-4506's Table sources. Which items Excel's rebuilt cache keeps is T-5057; what Excel does with `saveData` in general is T-5028.

Owner of these check (c) cells (decision 0245; T-4972). Each pivot is over a `saveData="0"` cache the importer skips, so every cell of Excel's on-open area is scored against nothing, and each non-empty one fails. That is 113 cells over 12 pivots, all in `pivot-on-open/TemplateWithTableSourcePivotTables.xlsx`:
- Final liability!A1:C6, Liability Detail!A1:B6, LiabilityChanges!A1:F6
- StressedLiabilityPivot!A1:C7, HedgeSummary!A1:E4
- Hedge detail - Nominal!A5:E8, Hedge detail - Nominal!A50:C54
- Hedge detail - ILB!A5:E8, Hedge detail - ILB!A34:C38
- ShockedCurvePivot!A1:B7, StressedYieldCurvePivot!A1:C7, StressedYieldCurvesFull!C1:D2

## Acceptance criteria
- [ ] A `saveData="0"` cache and its pivots load; a refresh reads the source records, and with none it fails, as now.
- [ ] The cells above pass check (c) or are owned by a narrower task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Caches over external or OLAP sources.
