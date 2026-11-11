---
id: T-90010
title: "FORMULATEXT prints a 3-D span over cell-like sheet names quoted whole"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: opus
usage: expert
depends_on: []
---

## Goal
FORMULATEXT prints a typed 3-D span over a sheet named like a cell with the whole span quoted.

## Context
Owns: probes/sheet-span-cell-like-names-typed (every visibly-off cell).
- Calc!B1: Excel prints `=SUM('Sales:T3'!C2)`; ours `=SUM(Sales:'T3'!C2)`. The span is quoted as one, not each sheet.
- Source: (excel-graph fill, 2026-11-11, #4125 wave 2; Excel's value first, ours second).

## Acceptance criteria
- [ ] Every visibly-off cell of probes/sheet-span-cell-like-names-typed matches Excel, or its verdict is recorded in docs/excel-semantics.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop-only behaviour (M14, decision 0039).
