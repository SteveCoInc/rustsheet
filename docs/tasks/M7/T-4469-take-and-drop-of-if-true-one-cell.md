---
id: T-4469
title: "TAKE and DROP of IF(TRUE,<one-cell ref>) keep the reference: TYPE(TAKE(IF(TRUE,Z1),1)) is 1, ours 64"
milestone: M7
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
TAKE and DROP read `IF(TRUE,Z1)` (an IF that returns a one-cell reference) as a reference, the way Excel does. So
`TYPE(TAKE(IF(TRUE,Z1),1))` is 1, as `TYPE(TAKE(Z1,1))` already is.

## Context
- probes/t4448-lone-blank-reshapers Sheet1!B9 (excel-graph, 2026-11-06, T-4448): Excel gives 1 and the engine gives 64.
  Cells A9, C9 and D9 of the same row pass after T-4448: the blank stays blank.
- crates/rustsheet-engine/src/functions/lookup_and_reference/drop_take.rs: a source keeps its reference reading only
  when it is a reference, a defined name or INDEX (`evaluate_node_with_reference`). Every other source, IF included, is
  read through `value_or_grid` or `evaluate_picked_source`, so the result is a 1x1 array.
- Adding `Function::If` to that match did not change B9 (T-4448 tried it), so the value path of IF's picked branch is
  where to look.
- Review MINORs on #3492 (T-4448), to fix in this PR:
  - arrays.md's "TAKE, DROP and CHOOSEROWS of the reference itself already kept it (A6:D8, A10:D10)" names
    CHOOSEROWS, but those rows are TAKE and DROP only; CHOOSEROWS(Z1,1) (row 11) read 0 before T-4448. Drop
    CHOOSEROWS there, or cite the controls (rows 18 and 22).
  - arrays.md says TAKE/DROP of a value keep the blank, but no cell probes DROP of a value: add
    `DROP(IF(TRUE,Z1),0)` (and its TYPE) to this task's probe.
  - The SORTBY comment in spill_functions.rs cites SORT's cells (`SORT(Z1)&"x"`, A1:D5); cite rows 3 and 4
    (`SORTBY(Z1,Z1)`, `SORTBY(Z1,5)`), and narrow SORT's comment to A1:D2.
- Opus: this touches the reference-or-value reading shared with CHOOSE and the other picked sources (T-3072, T-3085).

## Acceptance criteria
- [x] probes/t4448-lone-blank-reshapers B9 passes, and its baseline line drops the cell.
- [x] arrays.md records the rule with the cell.
- [x] The #3492 review MINORs above are fixed, with the DROP cell in the probe.
- [x] 0 regressions; `make check` green; reviewers approve.

## Out of scope
- Other picked sources (CHOOSE, IFS, SWITCH) unless the probe shows they share the rule.

## Outcome
- New probe probes/t4469-take-drop-if-one-cell-reference (excel-graph, 2026-11-06), 52 cells: TAKE, DROP and TRIMRANGE
  over an IF, IFS or SWITCH that picks a one-cell reference (blank, number, text) or a two-cell one. In every row Excel
  keeps the reference: TYPE 1 (2 for text), ISREF TRUE, `&"x"` the cell's text, ISBLANK TRUE for the blank. IFS,
  SWITCH and TRIMRANGE share the rule, so the fix covers them too (the task's out-of-scope clause allows it).
- drop_take.rs: the value path reads TAKE's or DROP's array through `take_drop_value_source`. A picker whose pick is a
  reference form (`is_reference_form`: reference, defined name, INDEX) hands on that reference, read as CHOOSE reads its arm
  (`choose_arm_value`). Any other arm reads as before, through `value_or_grid` or the picker's own value. On the
  reference-argument path (ISREF, SUBTOTAL), `evaluate_picked_source` reads such an arm the same way, but only for array
  sources (`keep_cell_variables`: TAKE, DROP and TRIMRANGE, not OFFSET's counts). `value_or_grid` itself is
  unchanged, because SUMPRODUCT also reads through it.
- The #3492 MINORs: arrays.md no longer credits CHOOSEROWS of the reference itself (it cites the rows 18 and 22 controls); the
  "TAKE/DROP of a value" wording is now "of IF's pick", with the DROP cell (t4469 C1); the SORT and SORTBY comments in
  spill_functions.rs cite A1:D2 and A3:D4. arrays.md has the new entry "TAKE and DROP keep the one-cell reference a picker
  hands them".
- Unit test `take_drop_of_a_picker_of_one_cell_keep_the_reference`.
- Eval delta: probes/t4448-lone-blank-reshapers B9 now passes and its baseline line is gone, so that case is fully passing. The new
  t4469 case passes 52 of 52. 0 regressions.
