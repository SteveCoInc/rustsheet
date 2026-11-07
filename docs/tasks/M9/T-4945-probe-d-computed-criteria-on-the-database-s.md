---
id: T-4945
title: "Probe D* computed criteria on the database's own sheet: ROW() and a #This Row reference"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence for a D* computed criterion on the database's own sheet: what `ROW()` with no argument and a `#This Row` structured reference (`T[@Col]`) read for each record.

## Context
- T-4858 matched the cross-sheet layout (probes/t3002-db-computed-criteria-absolute-mixed Calc B6:E14): `ROW()` is the criteria cell's row and `DbT[@Col]` reads the criteria cell's table row for every record. `db_computed_condition` (functions/database.rs) applies the same pin (`Node::pin_cell_row`) when the criteria range is on the database's sheet. No probe covers that layout (criteria.md "D* computed criteria", UNVERIFIED there).
- Design: copy the bitmask layout of probes/t3002 (Same!A5:C12, records rows 6-12, criteria in row 1:2) with `=ISODD(ROW())`, `=ISODD(INT(ROW()/2))`, `=ROW()>0`, and a table on the same sheet with a criteria cell inside its data rows (`=T[@Col]<0`, `=ISODD(ROW(T[@Col]))`). Run it through the oracle (excel-probe skill).

## Acceptance criteria
- [ ] A new probes case filled by excel-graph; criteria.md records the result (VERIFIED, or a new rule plus a follow-up task if ours differs).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Implementing a different rule: file it as its own task if the probe disagrees.
