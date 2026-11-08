---
id: T-4912
title: "Read a written-empty branch of IF, CHOOSE and IFERROR as an omission in ToolPak functions and the newer roundings (T-4901 follow-up)"
milestone: M5
lane: functions-math
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
MROUND, QUOTIENT, SERIESSUM and the newer roundings (CEILING.MATH, FLOOR.PRECISE) read an IF, CHOOSE or IFERROR whose
chosen branch is written empty as an omission, as Excel does, while a formula that merely returns a blank stays 0.

## Context
- Decision 0355: this task owns foundational-tier cells, which the tier gate needs 100% right on type, error and text, so it is P1.
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
- [x] A3, A10, A12:A16 of the probe and the t5067 cells above pass; the cells owned by this task leave
      eval/baseline/probes.txt.
- [x] Other IF/CHOOSE/IFERROR/IFNA/IFS-style wrappers probed (excel-probe) and the rule recorded in numbers.md
      (T-5067).
- [x] `make check` green; reviewers approve.

## Out of scope
- LET/LAMBDA omissions (already evidenced).

## Outcome
- **Design:** no new `CalcResult` variant. `Model::is_empty_argument` (functions/util/empty_args.rs), the one test
  every reader of a written-empty argument and LET's and LAMBDA's binding use, now looks through an IF, CHOOSE,
  IFERROR or IFNA (in parentheses or under a unary plus) one of whose branches is written empty, as found from the
  node alone (`may_hand_omission`). Only then it reads the wrapper's condition, index or value, through the wrapper's
  own pick step (`Model::pick_step`; IFERROR's and IFNA's error test is now one function, `replaces_error`), and
  tests the branch picked. Every other argument costs a node match, as before.
- That read comes before the call's own, so the condition is evaluated twice for these forms. A LAMBDA parameter
  read once is copied first (`copy_read_once`, as IF over an array condition does; unit test
  `a_parameter_read_once_survives_the_probe` fails without it). Probes inside probes nest at most 2 deep
  (`MAX_OMISSION_PROBES`) so nested readers cannot double the work per level: a DEVIATION entry in numbers.md.
- IMSUB and IMDIV read their empty flags before the call lifts, not per element in the kernel.
- **Evidence added:** probes/t4912-written-empty-branch-readers (28 cells, excel-graph, 2026-11-08). The other
  readers (DEC2BIN, IMSUB, COMPLEX, ISEVEN, WEEKNUM, WORKDAY, NETWORKDAYS, YEARFRAC) take the omission too, a LET
  name or LAMBDA parameter bound to one is ISOMITTED TRUE, and a unary plus keeps it. 28/28 pass.
- **Cells fixed:** probes/t4901-mround-blank-let-lambda-roundings Calc!A3, A10, A12:A16 and
  probes/t5067-written-empty-branch-wrappers Calc!A1:A3, A11, A12, A14:A16, A20:A26, A28, A29 (24 cells), which
  were held by T-5084 for M7. Unit tests: test/test_written_empty_branch.rs.
- **Docs:** numbers.md "Which wrappers hand a written-empty branch on as an omission (T-5067)" gains the t4912 rows
  and the new "Ours"; new DEVIATION entry "A written-empty branch probed inside another one's condition".
