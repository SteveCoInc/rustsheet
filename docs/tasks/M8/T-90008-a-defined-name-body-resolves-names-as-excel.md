---
id: T-90008
title: "A defined name body resolves names as Excel does when sheet-local names sit on the first sheet"
milestone: M8
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
Which names a defined name's body reads when the sheet holding sheet-local names is the first sheet matches Excel.

## Context
Owns: probes/defined-name-body-scope-first-sheet (every visibly-off cell).
- Rate is 1 in the workbook and Data!Rate 10; Data is the first sheet. Excel: `DoubleRate` (sheet-local Data!DoubleRate = Rate*2) is 2 from both sheets, so the body reads the workbook Rate, not Data's; ours 20. `TripleRate` 3 (ours 30). `DoubleOwn` (a body using a name only Data defines) is #NAME? and ISERROR TRUE; ours 20 / FALSE. `CallsDbl` and `LocalCallsDbl` 14; ours 21.
- Source: (excel-graph fill, 2026-11-11, #4125 wave 2; Excel's value first, ours second).

## Acceptance criteria
- [ ] Every visibly-off cell of probes/defined-name-body-scope-first-sheet matches Excel, or its verdict is recorded in docs/excel-semantics.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Desktop-only behaviour (M14, decision 0039).
