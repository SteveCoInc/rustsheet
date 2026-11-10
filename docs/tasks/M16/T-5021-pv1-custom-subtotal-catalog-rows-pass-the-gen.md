---
id: T-5021
title: "PV1 custom subtotal catalog rows pass: the gen-pivot cells left on avg to varP subtotals"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Each PV1 totals row below has no failing oracle cell: the gen-pivot pivots with custom subtotal functions (avg, max, min, product, stdDev, stdDevP, sum, var, varP) and data lines render as Excel does.

## Context
Filed by T-4992 as the `owner` (eval/catalog/pivot-features.tsv) of these rows, which the M16 exit row (report.json `milestone_exit`, `pivot.stages[].unowned`) listed as remaining with no open M16 owner at the 2026-11-07 run; the owner rule is docs/architecture/eval-harness.md "Milestone exit" (T-4952). A row is remaining while it has fewer than 30 oracle cells or a failing cell. Rows (19): PV-0027 `ST_ItemType=avg` (2,412 of 18,766 cells off); PV-0031 `ST_ItemType=data` (3,125 of 25,752 cells off); PV-0034 `ST_ItemType=max` (2,412 of 18,766 cells off); PV-0035 `ST_ItemType=min` (2,412 of 18,766 cells off); PV-0036 `ST_ItemType=product` (2,412 of 18,766 cells off); PV-0037 `ST_ItemType=stdDev` (2,576 of 18,583 cells off); PV-0038 `ST_ItemType=stdDevP` (2,576 of 18,583 cells off); PV-0039 `ST_ItemType=sum` (2,919 of 18,709 cells off); PV-0040 `ST_ItemType=var` (2,338 of 17,814 cells off); PV-0041 `ST_ItemType=varP` (2,338 of 17,814 cells off); PV-1089 `pivotFields/pivotField@avgSubtotal` (2,412 of 18,766 cells off); PV-1109 `pivotFields/pivotField@maxSubtotal` (2,412 of 18,766 cells off); PV-1111 `pivotFields/pivotField@minSubtotal` (2,412 of 18,766 cells off); PV-1117 `pivotFields/pivotField@productSubtotal` (2,412 of 18,766 cells off); PV-1126 `pivotFields/pivotField@stdDevPSubtotal` (2,576 of 18,583 cells off); PV-1127 `pivotFields/pivotField@stdDevSubtotal` (2,576 of 18,583 cells off); PV-1130 `pivotFields/pivotField@sumSubtotal` (2,919 of 18,709 cells off); PV-1133 `pivotFields/pivotField@varPSubtotal` (2,338 of 17,814 cells off); PV-1134 `pivotFields/pivotField@varSubtotal` (2,338 of 17,814 cells off).

Every failing cell of these rows is a gen-pivot cell (the M16 row's `gen_visibly_off`, 3,125 cells at that run; no other case tagged with these rows fails). A row's cells are the pivot cells of every case whose `pivot_features` lists it, so a row passes only when all its cases' pivot cells do. The causes found so far are owned cell by cell by the T-4132 owner tasks (T-4807, T-4914 to T-4922 and T-4924 to T-4927; decision 0245); this task owns the rows. Start after those land: re-run `make eval`, then for each cell these rows still fail, fix it where the cause is this family's feature (with its Excel evidence in docs/excel-semantics/pivots.md), or file an owner task for the cause and name it here. The engine pivot code is another stream's: this task is that stream's to implement. Related open tasks own parts of the same lines: T-4649 (placement probe), T-4925, T-4926 and T-4916 (custom subtotal lines), T-4636 (count and countA).

When this task closes, re-point or clear its rows' `owner` cells in the same PR (`scripts/tasks.py validate` fails on a done owner, scripts/pivot_owners.py).

## Acceptance criteria
- [ ] Each row listed has no failing cell in `make eval`'s Pivot features table, and no other case regresses (`make eval-baseline`).
- [ ] Each further cause found has an owner task or is fixed, with its pivots.md entry.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the ownership rule (T-4952).
