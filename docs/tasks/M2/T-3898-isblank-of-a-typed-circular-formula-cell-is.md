---
id: T-3898
title: "ISBLANK of a typed circular formula cell is TRUE (probes/t3212 J39, J50)"
milestone: M2
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
campaign: semantics
---

## Goal
`ISBLANK` of a cell holding a typed circular formula is TRUE, as in Excel, where the engine gives 0.

## Context
- Found by T-3212: probes/t3212-r1c1-bare-offsets (excel-graph, 2026-11-03). S!H39 `=RC[0]*0+1` (A1 `=H39*0+1`) and
  S!H50 `=SUM(R:R[1])*0+ROWS(R[-50]:R[-49])` (reads its own row) are circular. Excel shows them as 0, and
  `ISBLANK(H39)` and `ISBLANK(H50)` (J39, J50) are TRUE. Ours returns 0 for both J cells, which is not even a boolean.
- Related: docs/excel-semantics/recalc.md has a circular spill whose cell is blank (`ISBLANK(B2)` TRUE). Check whether
  every circular formula cell reads as blank to ISBLANK, and what ISNUMBER, ISFORMULA and `=H39` give (probe).
- Owned cells: probes/t3212-r1c1-bare-offsets S!J39, S!J50.

## Acceptance criteria
- [ ] The owned cells match Excel, and a probe of the wider rule matches too.
- [ ] docs/excel-semantics/recalc.md records the rule.
- [ ] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the owned cells are visibly off (0 where Excel shows TRUE), so this task is in the campaign. Its target
is that every cell it owns has Excel's type and value.

## Out of scope
- R1C1 entry (T-3212).
