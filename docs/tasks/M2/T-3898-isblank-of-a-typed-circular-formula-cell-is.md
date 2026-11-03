---
id: T-3898
title: "ISBLANK of a typed circular formula cell is TRUE (probes/t3212 J39, J50)"
milestone: M2
lane: engine
status: done
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
- [x] The owned cells match Excel, and a probe of the wider rule matches too.
- [x] docs/excel-semantics/recalc.md records the rule.
- [x] `make check` green; reviewers approve.

## Visible-parity stage
Decision 0083: the owned cells are visibly off (0 where Excel shows TRUE), so this task is in the campaign. Its target
is that every cell it owns has Excel's type and value.

## Out of scope
- R1C1 entry (T-3212).

## Outcome
- Probe probes/t3898-typed-circular-readers (28 cells, excel-graph, 2026-11-03): Excel recalculates after the load and
  after each typed entry, and a reader of a cycle keeps its value from just before the entry that closed it. A stored
  `ISBLANK(H10)` of a typed `=H10+1` is TRUE because it was computed while H10 was blank; ISNUMBER, TYPE, `&`,
  COUNTBLANK and IFERROR keep their blank-cell values too, a reader typed after it is 0, and a cycle typed one cell at a
  time keeps its first cell's value (H17 1). The engine already keeps a circular cell's stored value. The gap was the
  eval harness, which set every typed entry in the same edit as the stored cells.
- `crates/rustsheet-eval/src/engine.rs`: entered cells are collected during the load edit and typed afterwards by
  `type_entries`, one edit each, in the order the oracle types them, with the refused-entry rule unchanged. Unit test
  `spec_entries_are_typed_one_at_a_time_after_the_load`. docs/excel-semantics/recalc.md has a new entry, "A circular
  formula typed after the load leaves its readers their values from before".
- Eval delta: probes known failures 5281 -> 5279 (t3212 S!J39, S!J50 fixed; 0 regressions over the 219 typed-entry
  cases); probes +1 case (t3898-typed-circular-readers, 28 cells, all pass).
