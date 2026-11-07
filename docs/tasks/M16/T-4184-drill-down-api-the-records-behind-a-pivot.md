---
id: T-4184
title: "Drill-down API: the records behind a pivot cell"
milestone: M16
lane: api
status: done
priority: P3
size: S
tier: opus
depends_on: [T-4142, T-4178]
---

## Goal
The API returns the source records behind any value cell (Excel's Show Details content).

## Context
SPEC-0032. Program: docs/research/pivot-tables-2026-11-04/plan.md; decision 0274.

## Acceptance criteria
- [x] Tests against probes.
- [x] `make check` green; reviewers approve.

## Out of scope
- Creating the new sheet as Excel's UI does.

## Outcome
`Workbook::pivot_cell_records(sheet, name, cell) -> Result<Vec<u32>, PivotError>` (crates/rustsheet/src/pivot_view.rs)
returns the cache record indices behind a value cell, in record order: the records the filters keep whose row and
column items are the cell's line tuples (all of them under a subtotal's or the grand total's prefix). A label,
header, outside or other-sheet cell, or any cell of a pivot with no data field or collapsed to `#SPILL!`, is the new
`PivotError::NotAValueCell`; a value cell no record reaches gives an empty list. It lays the pivot out as
`pivot_table_cells` does (`plan_pivot_render`, then `compute` and `lay_out`) under one default `PivotBudget`, and
returns at most `MAX_RECORDS` indices. The engine gained one additive function, `pivot::records_behind`
(crates/rustsheet-engine/src/pivot/layout/records.rs), mapping a laid-out cell to its row and column lines and their
nodes' records. Python: `Workbook.pivot_cell_records(sheet, name, a1) -> list[int]` (+ stub).
Tests: the engine checks every Excel number in the value areas of 23 probe pivots (486 cells: t4137 K2, K4, K6;
t4138 G1 to G9; t4164 SH0, SH1H with a hidden inner item; t4135 A_sum, A_count, A_average; t4634 T1 to T3, A1 to
A3), the grand total covering every kept record, hidden items dropping records, labels and outside cells, the work
budget and a pivot with no data field; the facade checks pivot_tables.xlsx (getpivotdata-real-pivot) against
Excel's GETPIVOTDATA values, an API-created t4128 R pivot against Excel's layout, Count and Average pivots against
t4135, non-value cells, an unknown pivot and a spilled pivot; Python checks the same contract. Excel's Show Details
sheet (row order, columns) is unprobed: pivots.md "Show Details" (UNVERIFIED), follow-up T-4879.
Top-N, value and label filters are not applied yet (T-4165), so the records of such a pivot include ones Excel's
totals leave out. Follow-up T-4896: expose the plan's result and layout so the call computes and lays out once.
