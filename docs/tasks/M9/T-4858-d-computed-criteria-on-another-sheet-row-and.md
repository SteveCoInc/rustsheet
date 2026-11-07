---
id: T-4858
title: "D* computed criteria on another sheet: ROW() and COLUMN() with no argument are the criteria cell's, a structured [@Col] reads its own table row (probes/t3002 B7:E9, B13:E13)"
milestone: M9
lane: functions-criteria
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A D* computed criterion evaluated for each record gives `ROW()` and `COLUMN()` with no argument the criteria cell's row and column, and a structured this-row reference (`DbT[@Col]`) the table row of the criteria cell, for every record, as Excel does.

## Context
- Evidence: probes/t3002-db-computed-criteria-absolute-mixed (excel-graph, 2026-11-07, T-3002), Calc B7:E9 (ROW() bits; ROW() is 2, the criteria row) and B13:E13 (`=DbT[@Col]<0`, every record matches); criteria.md "D* computed criteria and an unknown header in the gen-m9 database sweep (T-2258)". The cells that pass (ROW()>0, the higher ROW bits, COLUMN(), ROW(DbT[@Col])) must keep passing.
- Ours: `db_computed_condition` (functions/database.rs) evaluates the criterion as if it sat in each record's row, so ROW() is the record's row, and the structured reference does not resolve from there (no record matches).
- Only the cross-sheet layout is probed; the same rule on one sheet is likely (ours moves the evaluation cell the same way) but check it against probes/db-computed-criteria-cross-sheet and the T-2525 cases before changing it.

## Acceptance criteria
- [ ] probes/t3002-db-computed-criteria-absolute-mixed Calc B7:E9 and B13:E13 pass, the other 79 cells keep passing, and baselines only shrink.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The record-area rule for parsed references (T-2617, T-3002: already matched).
