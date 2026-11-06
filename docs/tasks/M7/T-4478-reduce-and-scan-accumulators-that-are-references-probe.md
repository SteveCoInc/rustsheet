---
id: T-4478
title: "REDUCE and SCAN accumulators that are references: probe ISREF, ROWS and a:v ranges built across steps, and the spill of a returned reference, and match"
milestone: M7
lane: engine
status: done
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
When a REDUCE or SCAN LAMBDA returns a reference, the next step's accumulator, the final REDUCE result and each SCAN
output cell are a reference or a value exactly as in Excel, evidenced on excel-graph, so `ISREF(a)`, `ROWS(a:v)`,
`SUM(REDUCE(..))` and a cell holding a REDUCE that ends on a multi-cell reference (spill extent included) match.

## Context
- docs/excel-semantics/arrays.md, "BYROW and BYCOL pass references" ("Ours" paragraph): "MAP, SCAN and REDUCE pass
  each cell of a one-sheet range as a one-cell reference too (`lambda_cells`, T-1442); a reference the LAMBDA returns
  is read when it becomes a MAP or SCAN output cell, while a REDUCE or SCAN accumulator stays as returned (what Excel
  does with `ISREF(a)` on an accumulator is unprobed)". No case in eval/suites reads ISREF, ROWS or AREAS of an
  accumulator: probes/lambda-helper-range-slices only reads the element (`SCAN(0,A1:A3,LAMBDA(a,v,a+ISREF(v)))`).
- Follow-up named in T-1465's Outcome and never filed: "Not done: carrying a reference in a REDUCE accumulator, which
  needs an `ISREF(a)` probe first (no Excel evidence); file it if wanted."
- Only the initial value is evidenced: `lambda_initial_value` (crates/rustsheet-engine/src/functions/logical/
  map_reduce.rs) keeps a cell or range reference (gen-m7/2c5ea078f25443a2 A1767, 18b80d3fd292d63a A467; arrays.md
  "SCAN and REDUCE keep errors in the LAMBDA", T-2307). The comment says a union and a 3-D span initial value "are
  read as before (unprobed)".
- Ours: `fn_reduce` (map_reduce.rs) and `fn_scan` (functions/logical/scan.rs) pass the `CalcResult` the LAMBDA
  returned (`call_lambda_caller`) straight on as the next accumulator, so a returned `Range` stays a reference to the
  end; `lambda_result_value` (bycol_byrow.rs) reads SCAN's output cells. Whether Excel keeps it, reads it to a value
  (so `a:v` on the next step is `#VALUE!`), or keeps it only for some steps is unknown.
- Probe (excel-probe skill, excel-graph, tag M7), A1:A4 = 1, 2, 3, "x", B1:B2 = 10, 20, each result read through
  `ISREF`, `ROWS(..)&"x"&COLUMNS(..)`, `ERROR.TYPE` and `ARRAYTOTEXT` where it is not a reference test:
  - accumulator after a step: `REDUCE(A1,A2:A3,LAMBDA(a,v,IF(ISREF(a),1,0)+IF(ISREF(v),10,0)))` with a step that
    returns `v`, so the second step's `a` was returned by the LAMBDA: `REDUCE(A1,A2:A3,LAMBDA(a,v,IF(ISREF(a),v,"no")))`;
  - ranges built across steps: `ROWS(REDUCE(A1,A2:A3,LAMBDA(a,v,a:v)))`, `SUM(REDUCE(A1,A2:A3,LAMBDA(a,v,a:v)))`,
    `AREAS(REDUCE(A1,A2:A3,LAMBDA(a,v,(a,v))))`, and OFFSET on the accumulator, `REDUCE(A1,{1,2},LAMBDA(a,v,OFFSET(a,1,0)))`;
  - the omitted initial value (the first element is the seed): `REDUCE(,A1:A3,LAMBDA(a,v,IF(ISREF(a),1,0)))`;
  - SCAN: `SCAN(A1,A2:A3,LAMBDA(a,v,v))` then ISREF of the next `a` (`SCAN(A1,A2:A4,LAMBDA(a,v,IF(ISREF(a),v,0)))`),
    and `SCAN(A1,A2:A3,LAMBDA(a,v,ROWS(a:v)))`;
  - a union and a 3-D span as initial value: `AREAS(REDUCE((A1,A3),{1},LAMBDA(a,v,a)))`,
    `REDUCE(Sheet2:Sheet3!A1,{1},LAMBDA(a,v,ISREF(a)))` (if the spec format can hold a second and third sheet);
  - a cell holding `=REDUCE(A1,A2:A3,LAMBDA(a,v,a:v))` bare, with the cells below it empty, so its spill extent is
    scored (a multi-cell reference result spills, or is one value, or `#CALC!`).
- Write the engine's values as predictions before the fill (docs/research/t4478/, as T-4447 did), so the Outcome can
  say how many cells the current code already gives. This is a structural rule read off the cells, not a numeric fit.

## Acceptance criteria
- [x] probes/t4478-reduce-scan-reference-accumulator (excel-graph, canary-verified) covers every form listed in
      Context; its predictions (the engine before the change) are committed before the fill.
- [x] Every cell of the new case passes, with no spill-extent failure, or each that does not is recorded in arrays.md
      and owned by a filed task named in the Outcome.
- [x] arrays.md's "a REDUCE or SCAN accumulator stays as returned ... unprobed" sentence is replaced by the evidenced
      rule, citing the case's cells, and `lambda_initial_value`'s "unprobed" comment is updated if the union or 3-D
      span forms were probed.
- [x] Unit tests in crates/rustsheet-engine for the rule (accumulator reference kept or read, and the spill of a
      returned multi-cell reference).
- [x] No regressions in any baseline (`make eval-baseline` refuses none); `make check` green; reviewers approve.

## Out of scope
- MAP, BYROW and BYCOL output cells (evidenced since T-1395 and T-1442).
- The cost of carrying a reference (T-1465's performance note): only if the rule makes it observable.

## Outcome
- New probe probes/t4478-reduce-scan-reference-accumulator (excel-graph, 2026-11-06): 27 expressions, each read as
  ISREF, shape, ERROR.TYPE and ARRAYTOTEXT, plus four bare formulas whose spill is scored (117 cells).
  - Every form in Context is covered.
  - Because `a:v` is ambiguous with the column range A:V, the range forms are repeated with parameter names
    `p_a`/`p_v`.
  - Generator and predictions are in docs/research/t4478. The predictions (the engine before the change) were
    committed in 49133d71a before the fill.
- **Excel's rule:** REDUCE keeps a reference its LAMBDA returns, both as the next accumulator and as the result
  (ISREF TRUE). SCAN reads it to a value before the next step. A union initial value stays a reference, and a 3-D
  span initial value is `#VALUE!`.
- **Implemented:**
  - `fn_reduce` calls with a reference-keeping caller.
  - ISREF evaluates REDUCE in reference context.
  - `lambda_initial_value` makes a 3-D span `#VALUE!`.
  - SCAN is unchanged: it already reads each step.
  - BYROW and BYCOL now read a one-cell reference that a function in their LAMBDA returns
    (`call_lambda_on_slice`). Without that, gen-m7/611ce3bbfab27cef A1421:B1421
    (`BYCOL(..,LAMBDA(y,REDUCE(y,y,LAMBDA(acc,v,Data!D1))))`) regressed to `#VALUE!` once REDUCE returned the
    reference.
- **Score:** the engine before the change predicted 81 of 117 cells; it now passes 88. A first try that also made
  SCAN keep references lost Calc!H13, which is how the SCAN half of the rule was found.
- **Left failing (29 cells), recorded in arrays.md "REDUCE keeps a returned reference; SCAN reads it" and owned by
  T-4494:**
  - `a:v` between two parameters parses as columns A:V;
  - AREAS of a REDUCE call is `#ERROR!`;
  - a union initial value is read, not kept.
- Unit tests are in test_reduce_reference_accumulator.rs.
- **Eval delta:** 1 new case (88 of 117 cells pass; the 29 others are new known failures owned by T-4494).
  0 regressions.
