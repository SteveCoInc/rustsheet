---
id: T-4547
title: "Probe the lifted AVERAGEIFS and MINIFS forms T-4396 left UNVERIFIED: a later criteria range lifted alone, and MINIFS with two lifted pairs"
milestone: M9
lane: research
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The two lifted forms that docs/excel-semantics/lookup.md marks UNVERIFIED under "Statistical aggregates over a lifted
OFFSET" have excel-graph values cited there. They are a later criteria range of AVERAGEIFS or MINIFS lifted while the
first range and the first criteria range stay one cell, and MINIFS with two lifted criteria pairs. The engine's
inferred positions are confirmed or handed to a fix task.

## Context
- Gap triage 2026-11-06 (`make eval` on main 25779368). lookup.md, the AVERAGEIFS/MINIFS bullet (T-4396), says:
  "UNVERIFIED: a later criteria range lifted alone, and MINIFS with two pairs; the engine lifts every criteria range,
  as for SUMIFS and MAXIFS."
- T-4396 probed the other forms, one formula per workbook (probes/t4396-lift-*: -averageifs-both, -criteria, -first,
  -height, -pair2 and the MINIFS forms but -pair2). Its data: A1:A3 = 1, 100, 10000; B1:B3 = 2, 200, 20000; A4 "5",
  A6 TRUE, A7 7; the formula is in E1.
- What the engine does today (a scratch run on main 25779368): `AVERAGEIFS(A1:A3,A1:A3,">0",OFFSET(B1,{0;1},0),">0")`
  and `MINIFS(A1:A3,OFFSET(A1,{0;1},0),">0",OFFSET(B1,{0;1},0),">0")` both spill 2x1 (every criteria range lifts).
- Probe (excel-probe skill, excel-graph, tag M9). Use one formula per workbook, as T-4396 did, because Excel for the
  web refused some combined workbooks (T-2065). Copy T-4396's data. The forms:
  - `AVERAGEIFS(A1,A1,">0",OFFSET(B1,{0;1},0),">0")` and the MINIFS form (only the second criteria range lifted);
  - `AVERAGEIFS(A1:A2,A1:A2,">0",OFFSET(B1,{0;1},0),">0")` and the MINIFS form (a lifted one-cell element against
    two-cell ranges: is it the per-element shape `#VALUE!` of -first?);
  - `MINIFS(OFFSET(A1,{0;1},0),OFFSET(A1,{0;1},0),">0",OFFSET(B1,{0;1},0),">0")` (the -pair2 form for MINIFS);
  - `MAXIFS(A1,A1,">0",OFFSET(B1,{0;1},0),">0")`, the same question for MAXIFS (T-2833 covers its other partial
    lifts).
  Name each case probes/t4547-lift-<function>-<form>.
- Run the cases. Any cell where ours differs from Excel goes to a functions-criteria task filed with `make new-task`
  (`campaign: semantics`, with a `## Visible-parity stage` section) that names the case and cells.

## Acceptance criteria
- [ ] One probes/t4547-lift-* case per form in Context is committed with excel-graph oracle values.
- [ ] lookup.md: the UNVERIFIED sentence quoted in Context is replaced by the evidenced values, with the cases cited.
- [ ] Each cell where ours differs is recorded with `make eval-baseline` and owned by the filed functions-criteria
      task, named in the Outcome; no other new known failures.
- [ ] `make check` green; reviewers approve.

## Out of scope
- COUNTIFS, AVERAGEIF and the other MAXIFS partial lifts (T-2833).
- Moving lookup.md's "Which arguments lift is inferred" sentence (T-4418).
- The engine fix itself.
