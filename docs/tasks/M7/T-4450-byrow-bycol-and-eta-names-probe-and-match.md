---
id: T-4450
title: "Eta names of special forms and reference functions: probe and match _xleta.LET, LAMBDA, SUBTOTAL, OFFSET, INDEX, CHOOSE, IFERROR, ROW and COLUMN in BYROW, MAP and GROUPBY, and an eta call with no arguments"
milestone: M7
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A function passed by name that is a special form (LET, LAMBDA, CHOOSE, IFERROR), takes or returns references (SUBTOTAL,
OFFSET, INDEX, ROW, COLUMN), or is called through a LET name with no arguments (`LET(f,_xleta.SUM,f())`) gives Excel's
result in BYROW, BYCOL, MAP, MAKEARRAY and GROUPBY. The rules are probed first, then implemented, and recorded in
docs/excel-semantics/arrays.md "Functions passed by name (`_xleta.`)".

## Context
- docs/excel-semantics/arrays.md "Functions passed by name (`_xleta.`)" (probes/eta-reduced-names, T-1466;
  probes/m7-argument-modes F28:H51) and "LAMBDA helpers refuse a function they cannot call" (T-3655, T-3753) evidence
  plain functions by name and every non-function argument (an error, a number, an array constant: whole-call `#VALUE!`).
  The first entry still says: "Unprobed: other errors (`#DIV/0!`, `#REF!`) and an array as BYROW's or BYCOL's function
  ..., a special form passed by name (`_xleta.LET`, `_xleta.LAMBDA`, `_xleta.SUBTOTAL`, `_xleta.OFFSET`) and a call with
  too few arguments (`LET(f,_xleta.SUM,f())`): the engine gives whatever the function gives for those arguments." The
  errors and arrays are now evidenced by T-3655's probes (`BYCOL({1,2},1/0)`, an array constant: `#VALUE!`), so that
  half of the sentence is stale; the rest has no Excel cell in any suite (no `_xleta.LET`, `LAMBDA`, `SUBTOTAL`, `OFFSET`,
  `INDEX`, `CHOOSE`, `IFERROR`, `ROW` or `COLUMN` appears in eval/suites).
- GROUPBY's zero-minimum functions (ROW, COLUMN, SHEET, SHEETS, RANDARRAY) are extrapolated from NOW (whole-call
  `#VALUE!`; arrays.md "GROUPBY and PIVOTBY", "Unprobed (T-1706)").
- Ours: `Model::unbound_name` and `eta_definition` (crates/rustsheet-engine/src/functions/logical/lambda.rs) build
  `FunctionKind { kind, args }` over parameter names for any function the stored-name table knows, so `_xleta.LET` and
  `_xleta.LAMBDA` become a call of the special form over bare parameter names, and `_xleta.OFFSET`/`_xleta.INDEX` return a
  reference whose read in a MAP or BYROW cell is unprobed; the arity check is `eta_takes_count` (`arg_counts::takes_count`).
- Write one probe case, probes/t4450-eta-special-forms (excel-probe skill, excel-graph, tag M7), A1:B3 = 1..6, C1:C3 =
  "x", "", 7, each result read through `ROWS(r)&"x"&COLUMNS(r)`, `ERROR.TYPE(r)` and `ARRAYTOTEXT(r)`:
  - `BYROW(A1:B3,_xleta.LET)`, `MAP(A1:A3,_xleta.LAMBDA)`, `MAKEARRAY(2,2,_xleta.LET)`, `LET(f,_xleta.LAMBDA,f(1))`;
  - `MAP(A1:A3,B1:B3,_xleta.CHOOSE)`, `MAP(A1:A3,B1:B3,_xleta.IFERROR)`, `MAP({1;0},{5;6},_xleta.IF)` as a control;
  - `BYROW(A1:B3,_xleta.ROW)`, `BYCOL(A1:B3,_xleta.COLUMN)`, `MAP(A1:A3,_xleta.ISREF)` as a control, `MAKEARRAY(2,2,_xleta.OFFSET)`,
    `MAP(A1:A3,{1;2;1},_xleta.INDEX)`, `MAKEARRAY(2,2,_xleta.SUBTOTAL)`, `BYROW(A1:B3,LAMBDA(r,SUBTOTAL(9,r)))` as a control;
  - `LET(f,_xleta.SUM,f())`, `LET(f,_xleta.NOW,f())`, `LET(f,_xleta.ROW,f())`, `LET(f,_xleta.PI,f())`;
  - `GROUPBY(A1:A3,B1:B3,_xleta.ROW)`, `GROUPBY(A1:A3,B1:B3,_xleta.COLUMN)`, `GROUPBY(A1:A3,B1:B3,_xleta.SHEET)`.

## Acceptance criteria
- [x] probes/t4450-eta-special-forms is committed with oracle values (excel-graph) for every formula above; a formula
      excel-graph refuses or cannot decode is left pending and named in the arrays.md entry, as T-1466 did.
- [ ] Every filled cell passes (`make eval-show CASE=probes/t4450-eta-special-forms`): 124 of 135; the other 11 are owned by T-4518 and T-4519 (see Outcome).
- [x] The eval gate shows 0 regressions (probes/eta-reduced-names, probes/t3655-*, probes/t3814-lambda-helper-arity and
      gen-m7 in particular).
- [x] docs/excel-semantics/arrays.md "Functions passed by name (`_xleta.`)": the "Unprobed" sentence is replaced by the
      evidenced rules with their cells (the stale errors-and-arrays half pointing to "LAMBDA helpers refuse a function
      they cannot call"), and "GROUPBY and PIVOTBY"'s "Unprobed (T-1706)" sentence states the ROW, COLUMN and SHEET results.
- [x] Unit tests pin one special form by name and the no-argument `f()` result.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Typed entry of these formulas (`set_typed_user_input`'s argument-count refusals, T-2568).
- GROUPBY function arrays (T-4449) and LAMBDA arity in the helpers (T-3655, T-3814, done).
- ISOMITTED through LET, IF and CHOOSE (T-2670).

## Outcome
- New probe probes/t4450-eta-special-forms (excel-graph, 2026-11-06): every formula of the Context, plus follow-ups
  in column J (what the blank IF and CHOOSE return, OFFSET with three arguments, LET and LAMBDA given 2 or 3 values,
  `ERROR.TYPE(_xleta.LET)`, GROUPBY and PIVOTBY with SHEETS, RANDARRAY, NOW, PI and ROW). `ARRAYTOTEXT` of the
  volatile `LET(f,_xleta.NOW,f())` is read as ISNUMBER instead (F91). 135 cells, 124 pass.
- Engine (lambda.rs, variable_stack.rs, group_functions.rs):
  - `_xleta.LET` and `_xleta.LAMBDA` are `#NAME?` like an unknown name (`Model::unbound_name`).
  - A function by name called with a count it does not take is `#VALUE!` (`lambda_definition`): `LET(f,_xleta.SUM,f())`.
  - IF and CHOOSE by name read a reference value argument as a blank (`eta_blanks_references`, `Binding::is_reference`).
  - GROUPBY and PIVOTBY pass one argument to a minimum-0 function that takes one (ROW, COLUMN, SHEET, SHEETS,
    RANDARRAY), `eta_arity`; NOW and PI stay a whole-call `#VALUE!`.
- docs/excel-semantics/arrays.md: "Functions passed by name" replaces its "Unprobed" sentence with the rules and their
  cells; "GROUPBY and PIVOTBY" replaces "Unprobed (T-1706)". T-2644 (sonnet research) now names what is left (PIVOTBY
  with COLUMN, SHEET, SHEETS, RANDARRAY).
- Eval delta: probes +1 case, 124 of 135 cells pass; the 11 visibly-off cells are owned by T-4518 (SHEET and SHEETS of
  a LAMBDA parameter: F121, H121:H124, J53, J59) and T-4519 (GROUPBY's ROW and COLUMN results print `#CALC!` in
  ARRAYTOTEXT, and RANDARRAY's group value: F109, F115, J64, J65). 0 regressions in every suite.
- Unit tests: `special_forms_passed_by_name`, `function_by_name_called_without_arguments` (test_fn_lambda.rs),
  `groupby_passes_row_one_argument` (test_fn_groupby.rs).
