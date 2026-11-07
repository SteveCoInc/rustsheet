---
id: T-4997
title: "IRR probe misses from t4974: negative discount and caps"
milestone: M9
lane: functions-financial
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
IRR matches Excel on the visibly-off cells of the t4974 IRR probe (PR #4002 group G20).

## Context
Found by T-4974, which filled the probe suggestions of PR #4002 (docs/research/probe-suggestions-2026-11-07, merged to main as 8001d80b1)
with excel-graph. The predictions there came from a prototype of the 2026-11-07 Excel-parity fixes that is not on main;
compare against them before designing a fix.
- `IRR({0.3,-2.5,1.005})` (A1) Excel 6.909808543634389, ours 6.909808543634611; `IRR({-500,1,1,1,1,1})` (A18, B18) Excel `#NUM!`,
  ours -0.689...; 11 of these 14 cells match the prototype of PR #4002's fixes.

Owned cells (14, visibly off against Excel):
- probes/t4974-irr-negative-discount-and-caps.case.json (14): P!A1:A8, A18, B8, B13, B16:B18

## Acceptance criteria
- [ ] The owned cells are within half a unit of Excel's 15th significant digit (or the same error) and leave
      eval/baseline/probes.txt (`make eval-baseline`), with no new known-failure lines.
- [ ] docs/excel-semantics/statistics.md (financial.md for IRR) records the rule the probe settles.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The last-digit cells of the same cases.
