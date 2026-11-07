---
id: T-5025
title: "PV2 page-field catalog rows pass: the gen-pivot cells left on pageFields, pageField@fld and @hier"
milestone: M16
lane: engine
status: todo
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
Each PV2 filtering row below has no failing oracle cell: the gen-pivot pivots with page fields render as Excel does.

## Context
Filed by T-4992 as the `owner` (eval/catalog/pivot-features.tsv) of these rows, which the M16 exit row (report.json `milestone_exit`, `pivot.stages[].unowned`) listed as remaining with no open M16 owner at the 2026-11-07 run; the owner rule is docs/architecture/eval-harness.md "Milestone exit" (T-4952). A row is remaining while it has fewer than 30 oracle cells or a failing cell. Rows (5): PV-1023 `pageFields` (3,125 of 25,842 cells off); PV-1024 `pageFields/pageField` (3,125 of 25,842 cells off); PV-1027 `pageFields/pageField@fld` (3,125 of 25,842 cells off); PV-1028 `pageFields/pageField@hier` (3,125 of 25,842 cells off); PV-1031 `pageFields@count` (3,125 of 25,842 cells off).

Every failing cell of these rows is a gen-pivot cell (the M16 row's `gen_visibly_off`, 3,125 cells at that run; no other case tagged with these rows fails). A row's cells are the pivot cells of every case whose `pivot_features` lists it, so a row passes only when all its cases' pivot cells do. The causes found so far are owned cell by cell by the T-4132 owner tasks (T-4807, T-4914 to T-4922 and T-4924 to T-4927; decision 0245); this task owns the rows. Start after those land: re-run `make eval`, then for each cell these rows still fail, fix it where the cause is this family's feature (with its Excel evidence in docs/excel-semantics/pivots.md), or file an owner task for the cause and name it here. The engine pivot code is another stream's: this task is that stream's to implement. Related open tasks: T-4921 and T-4969 (page field captions), T-4633 (pageField@item and item@h), T-4650 (the page-field grid).

When this task closes, re-point or clear its rows' `owner` cells in the same PR (`scripts/tasks.py validate` fails on a done owner, scripts/pivot_owners.py).

## Acceptance criteria
- [ ] Each row listed has no failing cell in `make eval`'s Pivot features table, and no other case regresses (`make eval-baseline`).
- [ ] Each further cause found has an owner task or is fixed, with its pivots.md entry.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the ownership rule (T-4952).
