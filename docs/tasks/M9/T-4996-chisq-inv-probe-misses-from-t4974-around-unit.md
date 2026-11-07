---
id: T-4996
title: "CHISQ.INV probe misses from t4974 around unit x = .5"
milestone: M9
lane: functions-stat
status: todo
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
CHISQ.INV / CHISQ.INV.RT match Excel on the visibly-off round-trip cells of the t4974 unit-x .5 probe (PR #4002 group G02).

## Context
Found by T-4974, which filled the probe suggestions of PR #4002 (docs/research/probe-suggestions-2026-11-07 on that branch)
with excel-graph. The predictions there came from a prototype of the 2026-11-07 Excel-parity fixes that is not on main;
compare against them before designing a fix.
- G02 asks whether Excel keeps the leading-factor start and Halley steps past unit x = .5; 28 of these 32 cells match the
  prototype of PR #4002's fixes, so most close with it.

Owned cells (32, visibly off against Excel):
- probes/t4974-chisq-inv-unit-half-boundary.case.json (32): P!B6, B8, E7, E9, H1:H7, H9:H11, H14:H16, H22:H24, K9, K11, K15, K18, K25, N3:N7, N22, N24

## Acceptance criteria
- [ ] The owned cells are within half a unit of Excel's 15th significant digit (or the same error) and leave
      eval/baseline/probes.txt (`make eval-baseline`), with no new known-failure lines.
- [ ] docs/excel-semantics/statistics.md (financial.md for IRR) records the rule the probe settles.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The last-digit cells of the same cases.
