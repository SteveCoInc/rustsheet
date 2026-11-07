---
id: T-4751
title: "arrays.md: a blank range step of MAP, SCAN and REDUCE keeps blank elements (ARRAYTOTEXT shows them empty), not 0"
milestone: M7
lane: functions-logic
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
docs/excel-semantics/arrays.md "MAP and SCAN with a LAMBDA that returns a multi-cell range" and T-4476's Outcome say
what Excel's oracle shows for a blank one-cell or blank multi-cell range step: the elements are blanks, which show as 0
only when spilled or read through INDEX, not the number 0.

## Context
- Found by the Excel-fidelity review of T-4476 (PR #3802, MINOR). probes/t4476-lambda-range-results (excel-graph,
  2026-11-07): `MAP(A1:A2,LAMBDA(r,Z1:Z1))` and its SCAN form have ARRAYTOTEXT ", " (F25, F29), and
  `REDUCE(0,A1:A2,LAMBDA(a,v,Z1:Z3))` has ", , " (F37). Only the spilled cells (D25:D30, D37:D39) and `INDEX(..,1)`
  (G25, G29, G37) show 0.
- arrays.md currently says the blank `Z1:Z1` "gives 0, 0" and REDUCE over `Z1:Z3` "spills 0, 0, 0", and T-4476's Outcome
  says "blank cells as 0". The engine already matches every cell. The risk is that a later change turns the blank into
  the number 0.
- Docs only: no engine change.

## Acceptance criteria
- [x] arrays.md's entry says the step keeps blank elements (ARRAYTOTEXT ", " and ", , ", F25, F29, F37), shown as 0 when
      spilled and through INDEX (D25:D30, D37:D39, G25, G29, G37).
- [x] T-4476's Outcome wording is corrected the same way.
- [x] `make check` green; reviewers approve.

## Out of scope
- Any engine change: probes/t4476-lambda-range-results passes 81/81.

## Outcome
Docs only: arrays.md and T-4476's Outcome now say the blank range step keeps blank elements (ARRAYTOTEXT ", " / ", , "), shown as 0 only when spilled or through INDEX. No engine or eval change.
