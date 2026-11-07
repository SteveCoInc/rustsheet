---
id: T-4998
title: "T.INV.2T at df near 1.2E+8 misses from t4974"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
T.INV.2T at df near 1.2E+8 matches Excel (PR #4002 group G11).

## Context
Found by T-4974, which filled the probe suggestions of PR #4002 (docs/research/probe-suggestions-2026-11-07 on that branch)
with excel-graph. The predictions there came from a prototype of the 2026-11-07 Excel-parity fixes that is not on main;
compare against them before designing a fix.
- `T.INV.2T(0.5,123456789)` (A34) Excel 0.6744897431508498, ours 0.6744897634720731 (8th digit); the prototype of PR #4002's
  fixes matches Excel on all four cells, so this closes with its joint absolute-and-relative stop.

Owned cells (4, visibly off against Excel):
- probes/t4974-t-inv-joint-abs-rel-stop.case.json (4): P!A30, A34, A37:A38

## Acceptance criteria
- [ ] The owned cells are within half a unit of Excel's 15th significant digit (or the same error) and leave
      eval/baseline/probes.txt (`make eval-baseline`), with no new known-failure lines.
- [ ] docs/excel-semantics/statistics.md (financial.md for IRR) records the rule the probe settles.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The last-digit cells of the same cases.
