---
id: T-5019
title: "PV1 layout catalog rows pass: the gen-pivot cells left on axes, fields, items, location and form"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Each PV1 layout row below has no failing oracle cell: the gen-pivot pivots that evidence the axes, row and column fields, pivot field items, location and compact/outline form render as Excel does.

## Context
Filed by T-4992 as the `owner` (eval/catalog/pivot-features.tsv) of these rows, which the M16 exit row (report.json `milestone_exit`, `pivot.stages[].unowned`) listed as remaining with no open M16 owner at the 2026-11-07 run; the owner rule is docs/architecture/eval-harness.md "Milestone exit" (T-4952). A row is remaining while it has fewer than 30 oracle cells or a failing cell. Rows (34): PV-0001 `ST_Axis=axisCol` (3,125 of 25,977 cells off); PV-0002 `ST_Axis=axisPage` (3,125 of 25,752 cells off); PV-0003 `ST_Axis=axisRow` (3,125 of 25,977 cells off); PV-0004 `ST_Axis=axisValues` (3,125 of 26,217 cells off); PV-0782 `pivotTableDefinition` (3,125 of 25,828 cells off); PV-0826 `colFields` (3,125 of 25,977 cells off); PV-0827 `colFields/field` (3,125 of 25,752 cells off); PV-0828 `colFields/field@x` (3,125 of 25,752 cells off); PV-0829 `colFields@count` (3,125 of 25,752 cells off); PV-0834 `colItems` (3,125 of 25,977 cells off); PV-0835 `colItems/i` (3,125 of 25,752 cells off); PV-1016 `location` (3,125 of 25,772 cells off); PV-1019 `location@firstDataRow` (3,125 of 25,752 cells off); PV-1021 `location@ref` (3,125 of 26,049 cells off); PV-1032 `pivotFields` (3,125 of 25,752 cells off); PV-1033 `pivotFields/pivotField` (3,125 of 25,752 cells off); PV-1073 `pivotFields/pivotField/items` (3,125 of 25,752 cells off); PV-1074 `pivotFields/pivotField/items/item` (3,125 of 25,752 cells off); PV-1085 `pivotFields/pivotField/items/item@x` (3,125 of 26,006 cells off); PV-1086 `pivotFields/pivotField/items@count` (3,125 of 25,752 cells off); PV-1090 `pivotFields/pivotField@axis` (3,125 of 25,752 cells off); PV-1135 `pivotFields@count` (3,125 of 25,752 cells off); PV-1175 `rowFields` (3,125 of 25,977 cells off); PV-1176 `rowFields/field` (3,125 of 25,752 cells off); PV-1177 `rowFields/field@x` (3,125 of 25,752 cells off); PV-1178 `rowFields@count` (3,125 of 25,752 cells off); PV-1183 `rowItems` (3,125 of 25,997 cells off); PV-1184 `rowItems/i` (3,125 of 25,752 cells off); PV-1196 `pivotTableDefinition@compact` (3,125 of 25,977 cells off); PV-1223 `pivotTableDefinition@outline` (3,125 of 25,977 cells off); PV-1281 `x14:pivotField` (3,125 of 25,992 cells off); PV-0028 `ST_ItemType=blank`, PV-1083 `pivotField/items/item@sd`, PV-1106 `pivotField@insertBlankRow` (from T-4699, done on main: confirm they pass and clear or keep).

Every failing cell of these rows is a gen-pivot cell (the M16 row's `gen_visibly_off`, 3,125 cells at that run; no other case tagged with these rows fails). A row's cells are the pivot cells of every case whose `pivot_features` lists it, so a row passes only when all its cases' pivot cells do. The causes found so far are owned cell by cell by the T-4132 owner tasks (T-4807, T-4914 to T-4922 and T-4924 to T-4927; decision 0245); this task owns the rows. Start after those land: re-run `make eval`, then for each cell these rows still fail, fix it where the cause is this family's feature (with its Excel evidence in docs/excel-semantics/pivots.md), or file an owner task for the cause and name it here. The engine pivot code is another stream's: this task is that stream's to implement.

When this task closes, re-point or clear its rows' `owner` cells in the same PR (`scripts/tasks.py validate` fails on a done owner, scripts/pivot_owners.py).

## Acceptance criteria
- [ ] Each row listed has no failing cell in `make eval`'s Pivot features table, and no other case regresses (`make eval-baseline`).
- [ ] Each further cause found has an owner task or is fixed, with its pivots.md entry.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the ownership rule (T-4952).
