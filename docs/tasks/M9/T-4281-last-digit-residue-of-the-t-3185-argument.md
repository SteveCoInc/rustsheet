---
id: T-4281
title: "Last-digit residue of the T-3185 argument-kind cells: BETA.INV, F.DIST, F.INV, HYPGEOM.DIST, LINEST, LOGEST and NEGBINOMDIST base calls"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
The 16 last-digit residue cells of the T-3185 argument-kind probes each match Excel, or are owned by the task that
owns the same base call's residue.

## Context
- Found by T-3185 (excel-graph, 2026-11-04). In each cell the argument kind is coerced as Excel does (statistics.md
  "M9 statistical argument kinds of a scalar parameter (T-3185)"); the difference is the base call's own last digits.
- probes/t3185-m9-statistical-argument-kinds-a A14 (`BETA.INV(0.4,2,3,0,TRUE)`), A33 (`BETAINV(0.4,2,3,0,"1")`) and
  A114 (`F.DIST(1.5,4,TRUE,TRUE)`).
- probes/t3185-m9-statistical-argument-kinds-b A6 (`F.INV(0.3,TRUE,6)`, NumericClose, not ulp-level), A65, A68
  (`HYPGEOM.DIST(2,5,8,20,TRUE)` with text arguments), A76, A79, A82 (`HYPGEOMDIST(2,5,8,20)` with text
  arguments), A86, A87 (LINEST), A91, A93 (LOGEST) and A119 (`NEGBINOMDIST(3,"5",0.4)`).
- Gap triage 2026-11-05 (`make eval` on origin/main c3de9423b) found two more cells of the same probes that fail
  bit-level with no open task naming them: probes/t3185-m9-statistical-argument-kinds-b A62
  (`GROWTH(Data!A1:A5,Data!B1:B5,6,Data!Z5:Z6)`, Excel 6.597539553864471, ours ...475, 5 ulp) and
  probes/t3185-m9-statistical-argument-kinds-c A69 (`TREND(Data!A1:A5,Data!B1:B5,6,Data!Z5:Z6)`, Excel 5.8, ours
  5.800000000000001, 1 ulp). Their base calls' residue is the least-squares kernel's (T-2006, deferred).
- First step: compare each cell with the same call written with plain numbers. If an existing open task owns that
  base call's residue, move the cell's name into that task and close this one for those cells. Gate A context:
  T-4105 (discrete tables) and the F and beta inverse tasks.

## Acceptance criteria
- [ ] Each cell listed above matches Excel, or an open task names it beside its case.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The visibly-off cells of the same cases (T-4222).
