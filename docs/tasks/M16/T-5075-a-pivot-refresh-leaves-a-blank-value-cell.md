---
id: T-5075
title: "A pivot refresh leaves a blank value cell empty, not an empty string, as Excel's on-open render does"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: [T-4506]
---

## Goal
A refreshed pivot leaves a value cell with no aggregate blank, as Excel's on-open render does, instead of writing an empty string.

## Context
Since T-4506, the engine refreshes pivots over a Table. Check (c) (T-4972, decision 0349) compares each refresh with Excel's on-open render, and these cells are empty in Excel's sidecar but `""` (an empty string) in the refresh. They look the same, but `ISBLANK` and `COUNTA` read them differently. The refresh writes the blank value of an empty aggregate (no record for that row and column item) as text. The excel-graph sidecars of these cases are the evidence.

Owner of these check (c) cells (decision 0245; T-4972): 52 cells.
- pivot-on-open/PivotMixedDataTypesInTableColumn.xlsx pvt1!G3:G4, pvt2!B8:C8, pvt3!B12, pvt4!G4
- pivot-on-open/PivotMixedDataTypesInTableColumn.xlsx pvt5!B3:G19 (44 cells: B3:B5, C6:C8, D3:D8, D12:D18, E12, E13, E17, F14:F18, G3:G19)
- pivot-on-open/PivotTableWithNoneTheme.xlsx pvt!B8:C8

## Acceptance criteria
- [ ] A value cell with no aggregate is left empty by a refresh; the cells above pass check (c).
- [ ] An engine unit test pins it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Number formats of the empty cells.
