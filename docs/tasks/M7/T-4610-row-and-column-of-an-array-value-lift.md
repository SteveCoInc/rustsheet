---
id: T-4610
title: "ROW and COLUMN of an array value lift: an array of #VALUE! in the value's shape, so GROUPBY(..,_xleta.ROW) holds nested #VALUE! arrays"
milestone: M7
lane: engine
status: todo
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
ROW and COLUMN of an array value (not a reference) give an array of `#VALUE!` of that value's shape, as Excel does. So
GROUPBY with `_xleta.ROW`, `_xleta.COLUMN` or `LAMBDA(v,ROW(v))` holds a nested `#VALUE!` array in each value cell, and
ARRAYTOTEXT of the result is `#CALC!`.

## Context
- probes/t4519-row-column-randarray-over-arrays (excel-graph, 2026-11-06, T-4519): `LET(x,{1;2},ROW(x))` is 2x1, TYPE 64,
  ARRAYTOTEXT "#VALUE!, #VALUE!", and each element's ERROR.TYPE is 3 (D1:H1). `ROW(LET(x,{7},x))` is a 1x1 array (D11:H11). Inside GROUPBY,
  `INDEX(GROUPBY(A1:A3,B1:B3,_xleta.ROW),4,2)` (the Total cell) is a 3x1 of `#VALUE!` and `..,1,2` a 1x1 (D5:H6); the same holds for
  `LAMBDA(v,ROW(v))` and `_xleta.COLUMN` (D9:H10).
- probes/t4450-eta-special-forms F109, F115 and J65: ARRAYTOTEXT of GROUPBY over ROW, COLUMN and `LAMBDA(v,ROW(v))` is `#CALC!`. ARRAYTOTEXT over a
  nested cell is `#CALC!` (arrays.md "GROUPBY and PIVOTBY", T-2594), so these pass once the group value is the array.
- Ours: `Model::row_or_column` (functions/lookup_and_reference/mod.rs) reads the argument through `pick_reference` and
  `reference_ranges`. A value that is not a reference becomes one scalar `#VALUE!` (`not_a_reference`). `evaluate_areas` drops the array, so its
  shape is lost. Re-evaluating the argument is wrong (RAND, cost): thread the value out of the one read instead.
- Excel for the web refuses a workbook holding ROW or COLUMN of an array constant, a function call or an operator (`ROW({1;2})`,
  `ROW(SEQUENCE(2,2))`, `ROW(A1:A3*1)`; FileCorruptTryRepair, T-4519), so only LET-bound, LAMBDA-parameter and picked values reach this path.
- Unprobed: an error element inside the array (`LET(x,{1;#N/A},ROW(x))`). Probe it with this task.
- Split from T-2644 (research: the GROUPBY/PIVOTBY eta probe, which lists the t4450 cells too) and T-4519: this task owns the engine fix for its cells.

## Acceptance criteria
- [ ] probes/t4519-row-column-randarray-over-arrays D1:F1, D5:F6, D9:F11 and probes/t4450-eta-special-forms F109, F115, J65 pass;
      the baseline lines drop them.
- [ ] The error-element form is probed and matched, and arrays.md records it.
- [ ] `make check` green; reviewers approve.

## Out of scope
- RANDARRAY (T-4611); SHEET and SHEETS (T-4518).
