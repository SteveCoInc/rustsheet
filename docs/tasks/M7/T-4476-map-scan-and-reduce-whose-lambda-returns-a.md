---
id: T-4476
title: "MAP, SCAN and REDUCE whose LAMBDA returns a multi-cell range for some elements only, an all-blank range, or as REDUCE's last accumulator: probe and match"
milestone: M7
lane: functions-logic
status: done
priority: P2
size: S
tier: opus
depends_on: [T-4475]
---

## Goal
MAP, SCAN and REDUCE give Excel's result when their LAMBDA returns a multi-cell range for only some elements, a
range of blank cells, or (REDUCE) a multi-cell range as the last or a passed-on accumulator. The rules are probed
first, then implemented, and the "unprobed" clauses of docs/excel-semantics/arrays.md "MAP and SCAN with a LAMBDA
that returns a multi-cell range" are replaced by evidence.

## Context
- arrays.md "MAP and SCAN with a LAMBDA that returns a multi-cell range" (probes/map-scan-range-result, excel-graph
  2026-10-18, T-1792/T-1769): a multi-cell reference result makes the whole call one `#CALC!` whatever the range
  holds; "an all-blank range and a LAMBDA that returns a range for only some elements are unprobed". Every probed
  formula returns the range for every element (`MAP(A1:A2,LAMBDA(r,B1:B3))`).
- arrays.md "BYROW and BYCOL: a LAMBDA result that is an array" (probes/byrow-bycol-array-results, T-1670) evidences
  the mixed case for BYROW (`BYROW(A1:B3,LAMBDA(r,IF(SUM(r)>5,r,0)))` is `#CALC!`), not for MAP or SCAN.
- The same entry, on array steps (T-2306, T-2309): "a multi-cell array step of MAP or SCAN is the one `#CALC!`
  of a multi-cell range (`lambda_result_value`); REDUCE does not go through it". No probe returns a range from
  REDUCE's LAMBDA (probes/lambda-helper-range-slices asks only `REDUCE(0,A1:A3,LAMBDA(a,v,a+ISREF(v)))` and a COUNTA
  form), so what `REDUCE(0,A1:A2,LAMBDA(a,v,B1:B3))` gives (a spill of B1:B3, `#CALC!` or `#VALUE!`) is unevidenced.
- Ours: `Model::lambda_result_value` (crates/rustsheet-engine/src/functions/logical/bycol_byrow.rs) turns a
  multi-cell range into `by_slices_calc_error` after reading its existing cells, so MAP (`fn_map`, map_reduce.rs) and
  SCAN (scan.rs) stop at the first element that returns one; `fn_reduce` (map_reduce.rs) returns the last
  accumulator as it is, a `CalcResult::Range` included.
- Search gen-m7 for a REDUCE whose LAMBDA can return a multi-cell range before probing (`rg 'REDUCE\(' eval/suites/
  gen-m7`), and cite any cell found.
- Write one probe case, probes/t4476-lambda-range-results (excel-probe skill, excel-graph, tag M7). Data A1:A3 = 1,2,3,
  B1:B3 = "x","y","z", Z1:Z3 blank, J1 = 4; each result read through `ROWS(..)`, `ARRAYTOTEXT(..)` and `INDEX(..,1)`:
  - some elements only: `MAP(A1:A3,LAMBDA(r,IF(r=2,B1:B3,r)))`, `MAP(A1:A3,LAMBDA(r,IF(r=3,B1:B3,r)))`,
    `SCAN(0,A1:A3,LAMBDA(a,v,IF(v=2,B1:B3,v)))`, `SCAN(0,A1:A3,LAMBDA(a,v,IF(v=3,B1:B3,v)))`;
  - an all-blank range: `MAP(A1:A2,LAMBDA(r,Z1:Z3))`, `SCAN(0,A1:A2,LAMBDA(a,v,Z1:Z3))`, and `Z1:Z1` as the control;
  - REDUCE: `REDUCE(0,A1:A2,LAMBDA(a,v,B1:B3))`, `REDUCE(0,A1:A2,LAMBDA(a,v,Z1:Z3))`,
    `REDUCE(0,A1:A2,LAMBDA(a,v,IF(v=1,B1:B3,COUNTA(a))))` (a range accumulator handed on), `ISREF(REDUCE(0,A1:A2,
    LAMBDA(a,v,B1:B3)))`, `REDUCE(0,A1:A2,LAMBDA(a,v,J1:J1))` (control) and `REDUCE(0,A1:A2,LAMBDA(a,v,{1;2}))` (an array
    control).

## Acceptance criteria
- [x] probes/t4476-lambda-range-results is committed with oracle values (excel-graph) for every formula above.
- [x] Every cell of it passes (`make eval-show CASE=probes/t4476-lambda-range-results`), MAP and SCAN through
      `lambda_result_value` (one rule for the LAMBDA helpers, no copy), REDUCE's last-accumulator rule in `fn_reduce`
      commented with its probe cells.
- [x] probes/map-scan-range-result, probes/byrow-bycol-array-results, probes/lambda-helper-range-slices and
      probes/t4475-scan-reduce-reference-initial still pass; the eval gate shows 0 regressions in every suite.
- [x] Directive 5 holds: a REDUCE result over a whole-column range (`REDUCE(0,A1:A2,LAMBDA(a,v,C:C))`) stays within
      the 2^20-cell array bound of arrays.md "Array results above 1,048,576 cells are #ERROR!", pinned by a unit test.
- [x] docs/excel-semantics/arrays.md: the "MAP and SCAN with a LAMBDA that returns a multi-cell range" entry covers the
      probed forms and REDUCE with their cells, and its "unprobed" clause is gone.
- [x] `make check` green; reviewers approve.

## Out of scope
- Whether initial_value or an accumulator is a reference (ISREF, PRODUCT, COUNT): T-4475, which this follows because
  both change map_reduce.rs and scan.rs.
- BYROW and BYCOL (already evidenced, T-1670) and MAKEARRAY steps (T-2463).
- PIVOTBY subtotal work (T-4388, T-4401, T-4414, T-4428), function arrays (T-4449) and eta names (T-4450) are unrelated.

## Outcome
- **Probe:** probes/t4476-lambda-range-results (excel-graph, 2026-11-07, 81 cells), every formula of the Context, each with
  ROWS, ARRAYTOTEXT, `INDEX(..,1)` and ERROR.TYPE beside it. No gen-m7 REDUCE returns a multi-cell range (searched).
- **Excel:** MAP and SCAN are one `#CALC!` when the LAMBDA returns a multi-cell range for any one element (first, middle
  or last) and for an all-blank range; the one-cell blank range is 0. REDUCE returns its last accumulator as it is: a
  multi-cell range spills ("x", "y", "z"; blank cells as 0), ISREF of it is TRUE, and a range accumulator reaches the
  next step as a reference (COUNTA 3).
- **Engine:** already matched on main (MAP and SCAN through `lambda_result_value`; REDUCE through T-4478/T-4494's
  `reduce_reference` and T-4475). No behavior change: `fn_reduce` now cites the probe cells, and a unit test
  (`reduce_ending_on_a_whole_column_stays_within_the_array_bound`) pins directive 5 for a whole-column REDUCE result
  (ROWS 1048576, SUM over it sparse, `+0` over two whole columns `#ERROR!`).
- **arrays.md:** "MAP and SCAN with a LAMBDA that returns a multi-cell range" covers the probed forms and REDUCE with
  their cells; the "unprobed" clause is gone.
- **Eval delta:** the new case passes 81/81; 0 regressions in every suite.
