---
id: T-90004
title: "A range whose right operand is a reference-returning call, in a cell, evaluates as Excel's"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A range whose right corner is a call returning a reference (`A$3:INDEX($A:$A,ROW()-1)`, `Data!$A$1:INDEX(Data!$B:$B,3)`) evaluates in a cell as Excel does.

## Context
Owns: probes/range-to-a-call-cells (every visibly-off cell).
- Calc!B5 `SUM(A$3:INDEX($A:$A,ROW()-1))` is 0; C1 and C2 `SUM(Data!$A$1:INDEX(Data!$B:$B,3))` 0; C3 `ROWS(Data!$A$1:INDEX(Data!$B:$B,COUNTA(Data!$A:$A)))` 5. Ours #ERROR! for all four, so the parser or evaluator rejects the call operand in a cell (probes/t4506-pivot-index-name-corner holds the same shape in a defined name).
- Source: (excel-graph fill, 2026-11-11, #4125 wave 2; Excel's value first, ours second).

## Acceptance criteria
- [ ] Every visibly-off cell of probes/range-to-a-call-cells matches Excel, or its verdict is recorded in docs/excel-semantics.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop-only behaviour (M14, decision 0039).
