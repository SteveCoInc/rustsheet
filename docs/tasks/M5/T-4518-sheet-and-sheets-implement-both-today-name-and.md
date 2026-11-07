---
id: T-4518
title: "SHEET and SHEETS of a LAMBDA parameter: BYROW(r,_xleta.SHEET) is 1 per row, GROUPBY's SHEET and SHEETS #N/A per group (today #NAME? and #N/IMPL)"
milestone: M5
lane: functions-logic
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
SHEET and SHEETS called with a LAMBDA parameter (a function passed by name, or a LAMBDA body
reading its parameter) give Excel's result: the sheet of a bound reference, `#N/A` for a bound
value array.

## Context
- probes/t4450-eta-special-forms (T-4450, excel-graph): `ARRAYTOTEXT(BYROW(A1:B3,_xleta.SHEET))` is
  "1, 1, 1" (J53); `GROUPBY(A1:A3,B1:B3,_xleta.SHEET)` and `..._xleta.SHEETS` give `#N/A` in each
  group's value cell (F121, H121:H124, J59). The engine gives `#NAME?` (SHEET) and `#N/IMPL`
  (SHEETS) for each: `fn_sheet` and `fn_sheets` (crates/rustsheet-engine/src/functions/information/mod.rs)
  match the argument node's kind (a reference, a defined name, text) and do not read a
  `Node::NamedVariableKind` bound by a LAMBDA call.
- docs/excel-semantics/arrays.md "Functions passed by name (`_xleta.`)" records the cells.

## Acceptance criteria
- [x] Every cell of probes/t4450-eta-special-forms named above passes; `LET(x,A1,SHEET(x))` and
      `MAP(A1:A3,LAMBDA(r,SHEETS(r)))` are probed alongside (excel-probe skill) and pass.
- [x] The eval gate shows 0 regressions.
- [x] `make check` green; reviewers approve.

## Out of scope
- SHEET and SHEETS of 3-D references (references.md owns them).

## Outcome
- `fn_sheet` reads a bound LET variable or LAMBDA parameter as what it is bound to. A reference
  gives its sheet, text a sheet name, an error that error, and an array one `#N/A`. Only an unbound
  name stays `#NAME?`. `fn_sheets` gives 1 for a parameter bound to a one-sheet reference and `#N/A`
  for one bound to an array. SHEETS of any other bound value stays `#N/IMPL!` (unprobed, below).
- New probe probes/t4518-sheet-sheets-bound-variable (26 cells, excel-graph, all pass):
  `LET(x,A1,SHEET(x))` and SHEET of a variable bound to a reference on each of three sheets, a
  union, text, a number, a boolean, an error or an array; MAP, BYROW, BYCOL, GROUPBY and a LAMBDA
  call. Recorded in information.md "SHEET and SHEETS of a LET variable or LAMBDA parameter".
  Quirk: `LET(x,1/0,SHEET(x))` is `#DIV/0!`, while a written error argument is `#N/A`.
- `MAP(A1:A3,LAMBDA(r,SHEETS(r)))` cannot be probed as written. The oracle's canary guard
  (tools/rustsheet_tools/oracle/canary.py) refuses `SHEETS(<parameter>)`, because a sheet count
  would include the canary sheet. Its equivalent `MAP(A1:A3,_xleta.SHEETS)` (D9) binds the same
  references and is probed instead, with BYROW and BYCOL forms (D11, D13, D23). Each of these is 1
  whatever the sheet count.
- t4450-eta-special-forms J53, F121, G121, H121:H124 and J59 pass.
- Eval delta: probes known failures 13446 -> 13439 (fixed 7), 0 regressions, 1 new case.
