---
id: T-4565
title: "Probe SHEET and SHEETS of a variable bound to a LAMBDA, an omitted parameter and a LET-bound array (T-4518 review follow-ups)"
milestone: M7
lane: functions-logic
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
SHEET and SHEETS of a LET variable or LAMBDA parameter bound to a LAMBDA, to an omitted parameter, or (SHEETS)
to a LET-bound array or a scalar value give Excel's result, probed.

## Context
- T-4518 (information.md "SHEET and SHEETS of a LET variable or LAMBDA parameter") probed the reference, text,
  number, boolean, error and array bindings of SHEET (probes/t4518-sheet-sheets-bound-variable). Its reviewers
  noted these as unprobed:
  - `LET(f,LAMBDA(x,x),SHEET(f))` and `LAMBDA(r,SHEET(r))()` now go through SHEET's sheet-name path
    (`fn_sheet`, crates/rustsheet-engine/src/functions/information/mod.rs);
  - SHEETS of a parameter bound to an array gives `#N/A` on GROUPBY-group evidence only (t4450-eta-special-forms
    J59), and SHEETS of a bound text, number or boolean stays `#N/IMPL!`.
- The oracle's canary guard (tools/src/rustsheet_tools/oracle/canary.py) refuses `SHEETS(<parameter>)` as
  written, because a count of sheets would include the canary. Probe the SHEETS cases through forms the guard
  accepts and whose answer cannot depend on the sheet count (`MAP({1;2},_xleta.SHEETS)`, `BYROW` of an array).
  If a case can only be written as `SHEETS(<parameter>)`, record it as unprobeable and leave it.

## Acceptance criteria
- [ ] A probe case answers each question above (excel-probe skill); ours matches every cell it scores.
- [ ] information.md records the results.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SHEET and SHEETS of 3-D references (references.md).
