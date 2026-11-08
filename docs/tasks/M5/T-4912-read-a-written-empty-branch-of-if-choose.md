---
id: T-4912
title: "Read a written-empty branch of IF, CHOOSE and IFERROR as an omission in ToolPak functions and the newer roundings (T-4901 follow-up)"
milestone: M5
lane: functions-math
status: todo
priority: P3
size: M
tier: opus
depends_on: []
---

## Goal
MROUND, QUOTIENT, SERIESSUM and the newer roundings (CEILING.MATH, FLOOR.PRECISE) read an IF, CHOOSE or IFERROR whose
chosen branch is written empty as an omission, as Excel does, while a formula that merely returns a blank stays 0.

## Context
- probes/t4901-mround-blank-let-lambda-roundings (excel-graph, 2026-11-07, T-4901): `MROUND(IF(TRUE,),1)`,
  `MROUND(IF(FALSE,1,),1)`, `MROUND(CHOOSE(1,,2),1)`, `MROUND(IFERROR(,1),1)`, `MROUND(5,IF(TRUE,))` and
  `QUOTIENT(IF(TRUE,),1)` are `#N/A` (A3, A10, A12, A13, A14, A16), and `CEILING.MATH(2.675,IF(TRUE,))` is 3, the
  omitted significance (A15). Ours reads all of them as 0 (`Model::is_empty_argument` looks only at the argument node
  and a LET/LAMBDA variable). A blank from VLOOKUP/INDEX of an empty cell stays 0 (A1, A2, A8, A9), so the
  look-through must be on the written branch, not on `CalcResult::EmptyArg`, which IFERROR(<blank>,5) also gives.
- The condition (or index) of the wrapper is needed to pick the branch, so the predicate needs the cell and a
  `&mut self` evaluation; design that without double evaluation.
- probes/t5067-written-empty-branch-wrappers (excel-graph, 2026-11-08, T-5067) settles AC 2's wrappers. numbers.md
  "Which wrappers hand a written-empty branch on as an omission (T-5067)" has the rule:
  - IFNA hands the omission on too, and nesting, parentheses and a LET name keep it.
  - IFS, SWITCH and an operator read it as 0.
  - SQRTPI, FACTDOUBLE, EDATE, SERIESSUM and QUOTIENT's divisor read it as `#N/A`.
  - The chosen branch decides: `MROUND(IFERROR(Z1,),1)` and `MROUND(IF(TRUE,Z1,),1)` are 0.
  - An array IF reads it as 0 per element (A17:A19, already matched).
  Owned cells there: A1:A3, A11, A12, A14:A16, A20:A26, A28, A29.
- Design implication: a node-only look-through cannot tell `IFERROR(Z1,)` (0) from `IFERROR(1/0,)` (`#N/A`), and
  `CalcResult::EmptyArg` cannot either, because IFERROR and VLOOKUP return it for a blank. The omission needs a value
  of its own, for example a distinct "written-empty" result from evaluating `EmptyArgKind`:
  - IF, CHOOSE, IFERROR and IFNA pass it through;
  - IFS, SWITCH, operators and the lifts turn it into the empty value or 0;
  - `is_empty_argument` reads it.
  Size that change before starting (it touches every exhaustive match on `CalcResult`). Split it if it is past M.

## Acceptance criteria
- [ ] A3, A10, A12:A16 of the probe and the t5067 cells above pass; the cells owned by this task leave
      eval/baseline/probes.txt.
- [x] Other IF/CHOOSE/IFERROR/IFNA/IFS-style wrappers probed (excel-probe) and the rule recorded in numbers.md
      (T-5067).
- [ ] `make check` green; reviewers approve.

## Out of scope
- LET/LAMBDA omissions (already evidenced).
