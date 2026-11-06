---
id: T-4519
title: "GROUPBY of ROW or COLUMN over a group's values: ARRAYTOTEXT of the result is #CALC!, and RANDARRAY's group value is a scalar"
milestone: M7
lane: engine
status: in-progress
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
GROUPBY's result for a function that returns an array of errors, or an array, per group reads in
ARRAYTOTEXT and INDEX as Excel's does.

## Context
- probes/t4450-eta-special-forms (T-4450, excel-graph): `GROUPBY(A1:A3,B1:B3,_xleta.ROW)` is 4x2
  with `#VALUE!` in each value cell (`ERROR.TYPE` of it is 3 in H109:H112; `INDEX(..,1,2)` is
  `#VALUE!`, J9), yet `ARRAYTOTEXT` of the whole result is `#CALC!` (F109; COLUMN F115;
  `LAMBDA(v,ROW(v))` J65), while `ARRAYTOTEXT(INDEX(..,0,1))` is "1, 3, 5, Total" (J15) and
  SHEET's `#N/A` cells print (F121). The engine prints "1, #VALUE!, ..." for F109, F115 and J65.
  A guess to test first: the group's value is ROW lifted over the values (an array of `#VALUE!`),
  which a cell shows as `#VALUE!` (an array result, as IF and ROUND, arrays.md "GROUPBY and
  PIVOTBY") but ARRAYTOTEXT refuses as a nested array.
- `ERROR.TYPE(INDEX(GROUPBY(A1:A3,B1:B3,_xleta.RANDARRAY),1,2))` is one `#N/A` cell in Excel (J63:J64): the
  engine's element is an array and the ERROR.TYPE spills (J64).

## Acceptance criteria
- [ ] F109, F115, J64 and J65 of probes/t4450-eta-special-forms pass, with a probe of the guess
      (excel-probe skill), recorded in docs/excel-semantics/arrays.md "GROUPBY and PIVOTBY".
- [ ] The eval gate shows 0 regressions (probes/groupby-*, probes/t4449-*).
- [ ] `make check` green; reviewers approve.

## Out of scope
- SHEET and SHEETS of a LAMBDA parameter (T-4518).
