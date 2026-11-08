---
id: T-5022
title: "PV1 cache catalog rows pass: the gen-pivot cells left on cache fields, shared items and records"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Each PV1 cache row below has no failing oracle cell: the gen-pivot pivots refreshed from their source into cache fields, shared items and records render as Excel does.

## Context
Filed by T-4992 as the `owner` (eval/catalog/pivot-features.tsv) of these rows, which the M16 exit row (report.json `milestone_exit`, `pivot.stages[].unowned`) listed as remaining with no open M16 owner at the 2026-11-07 run; the owner rule is docs/architecture/eval-harness.md "Milestone exit" (T-4952). A row is remaining while it has fewer than 30 oracle cells or a failing cell. Rows (13): PV-0117 `pivotCacheDefinition` (3,125 of 25,752 cells off); PV-0118 `cacheFields` (3,125 of 25,752 cells off); PV-0119 `cacheFields/cacheField` (3,125 of 25,752 cells off); PV-0240 `sharedItems` (3,125 of 25,752 cells off); PV-0241 `sharedItems/b` (2,919 of 18,709 cells off); PV-0257 `sharedItems/e` (2,166 of 14,390 cells off); PV-0278 `sharedItems/m` (2,864 of 18,898 cells off); PV-0298 `sharedItems/n` (2,919 of 18,709 cells off); PV-0319 `sharedItems/s` (3,125 of 25,752 cells off); PV-0367 `cacheFields@count` (3,125 of 25,752 cells off); PV-0676 `pivotCacheRecords` (3,125 of 25,752 cells off); PV-0678 `pivotCacheRecords/r` (3,125 of 25,752 cells off); PV-0780 `pivotCacheRecords@count` (3,125 of 25,752 cells off).

Every failing cell of these rows is a gen-pivot cell (the M16 row's `gen_visibly_off`, 3,125 cells at that run; no other case tagged with these rows fails). A row's cells are the pivot cells of every case whose `pivot_features` lists it, so a row passes only when all its cases' pivot cells do. The causes found so far are owned cell by cell by the T-4132 owner tasks (T-4807, T-4914 to T-4922 and T-4924 to T-4927; decision 0245); this task owns the rows. Start after those land: re-run `make eval`, then for each cell these rows still fail, fix it where the cause is this family's feature (with its Excel evidence in docs/excel-semantics/pivots.md), or file an owner task for the cause and name it here. The engine pivot code is another stream's: this task is that stream's to implement. Related open tasks: T-4917 (date-time items), T-4654 (shared-item flags), T-4640 (field names).

When this task closes, re-point or clear its rows' `owner` cells in the same PR (`scripts/tasks.py validate` fails on a done owner, scripts/pivot_owners.py).

## Acceptance criteria
- [ ] Each row listed has no failing cell in `make eval`'s Pivot features table, and no other case regresses (`make eval-baseline`).
- [ ] Each further cause found has an owner task or is fixed, with its pivots.md entry.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the ownership rule (T-4952).
