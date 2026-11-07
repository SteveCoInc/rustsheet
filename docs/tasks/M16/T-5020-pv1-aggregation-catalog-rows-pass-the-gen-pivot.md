---
id: T-5020
title: "PV1 aggregation catalog rows pass: the gen-pivot cells left on the eleven functions and dataFields"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Each PV1 aggregation row below has no failing oracle cell: the gen-pivot pivots that evidence the eleven data-field functions and the dataFields part render as Excel does.

## Context
Filed by T-4992 as the `owner` (eval/catalog/pivot-features.tsv) of these rows, which the M16 exit row (report.json `milestone_exit`, `pivot.stages[].unowned`) listed as remaining with no open M16 owner at the 2026-11-07 run; the owner rule is docs/architecture/eval-harness.md "Milestone exit" (T-4952). A row is remaining while it has fewer than 30 oracle cells or a failing cell. Rows (17): PV-0005 `ST_DataConsolidateFunction=average` (3,125 of 25,752 cells off); PV-0006 `ST_DataConsolidateFunction=count` (2,539 of 20,675 cells off); PV-0007 `ST_DataConsolidateFunction=countNums` (3,125 of 25,752 cells off); PV-0008 `ST_DataConsolidateFunction=max` (2,715 of 21,559 cells off); PV-0009 `ST_DataConsolidateFunction=min` (2,998 of 23,843 cells off); PV-0010 `ST_DataConsolidateFunction=product` (2,588 of 19,650 cells off); PV-0011 `ST_DataConsolidateFunction=stdDev` (2,998 of 23,843 cells off); PV-0012 `ST_DataConsolidateFunction=stdDevp` (2,991 of 20,807 cells off); PV-0013 `ST_DataConsolidateFunction=sum` (3,125 of 26,049 cells off); PV-0014 `ST_DataConsolidateFunction=var` (1,886 of 20,050 cells off); PV-0015 `ST_DataConsolidateFunction=varp` (3,125 of 25,752 cells off); PV-0888 `dataFields` (3,125 of 25,752 cells off); PV-0889 `dataFields/dataField` (3,125 of 25,977 cells off); PV-0893 `dataFields/dataField@fld` (3,125 of 25,752 cells off); PV-0897 `dataFields/dataField@subtotal` (3,125 of 25,752 cells off); PV-0898 `dataFields@count` (3,125 of 25,752 cells off); PV-1094 `pivotFields/pivotField@dataField` (3,125 of 25,752 cells off).

Every failing cell of these rows is a gen-pivot cell (the M16 row's `gen_visibly_off`, 3,125 cells at that run; no other case tagged with these rows fails). A row's cells are the pivot cells of every case whose `pivot_features` lists it, so a row passes only when all its cases' pivot cells do. The causes found so far are owned cell by cell by the T-4132 owner tasks (T-4807, T-4914 to T-4922 and T-4924 to T-4927; decision 0245); this task owns the rows. Start after those land: re-run `make eval`, then for each cell these rows still fail, fix it where the cause is this family's feature (with its Excel evidence in docs/excel-semantics/pivots.md), or file an owner task for the cause and name it here. The engine pivot code is another stream's: this task is that stream's to implement.

When this task closes, re-point or clear its rows' `owner` cells in the same PR (`scripts/tasks.py validate` fails on a done owner, scripts/pivot_owners.py).

## Acceptance criteria
- [ ] Each row listed has no failing cell in `make eval`'s Pivot features table, and no other case regresses (`make eval-baseline`).
- [ ] Each further cause found has an owner task or is fixed, with its pivots.md entry.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the ownership rule (T-4952).
