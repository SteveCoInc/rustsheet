---
id: T-4518
title: "SHEET and SHEETS of a LAMBDA parameter: BYROW(r,_xleta.SHEET) is 1 per row, GROUPBY's SHEET and SHEETS #N/A per group (today #NAME? and #N/IMPL)"
milestone: M5
lane: functions-logic
status: in-progress
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
- [ ] Every cell of probes/t4450-eta-special-forms named above passes; `LET(x,A1,SHEET(x))` and
      `MAP(A1:A3,LAMBDA(r,SHEETS(r)))` are probed alongside (excel-probe skill) and pass.
- [ ] The eval gate shows 0 regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SHEET and SHEETS of 3-D references (references.md owns them).
