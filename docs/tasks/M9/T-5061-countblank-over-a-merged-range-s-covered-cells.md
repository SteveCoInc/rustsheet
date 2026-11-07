---
id: T-5061
title: "COUNTBLANK over a merged range's covered cells (fixtures merged-covered-values I4): Excel 1, ours 3"
milestone: M9
lane: functions-math
status: in-progress
priority: P1
size: S
tier: sonnet
depends_on: [T-5044]
---

## Goal
COUNTBLANK over a range that holds a merged area counts its covered cells as Excel does.

## Context
- Found by the Phase 1 owners check (2026-11-07): an M9 exit cell (`rate:COUNTBLANK`) with no open M9 owner.
- fixtures/merged-covered-values.xlsx Sheet1!I4: `COUNTBLANK(A1:D1)`, Excel 1, ours 3. The covered cells of the merged
  range keep content in Excel's file that the import drops; T-5044 (M11, xlsx import keeps the content of covered cells)
  owns the import fix, which should fix this cell too. This task owns the COUNTBLANK cell so the M9 rate row has an
  owner, and checks it once T-5044 lands.

## Acceptance criteria
- [ ] merged-covered-values Sheet1!I4 matches Excel (expected after T-5044); if not, fix COUNTBLANK's reading of
      covered cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The import change itself (T-5044).
