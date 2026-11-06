---
id: T-4475
title: "SCAN and REDUCE initial_value through IF, CHOOSE, INDEX, XLOOKUP, OFFSET, INDIRECT, a union and a 3-D span: probe whether the LAMBDA gets a reference (ISREF, PRODUCT, COUNT) and match"
milestone: M7
lane: functions-logic
status: in-progress
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
SCAN and REDUCE hand the LAMBDA Excel's kind of initial_value (a reference or a value) for every form that can
produce a cell, and keep (or drop) the reference-ness of an accumulator a step returns as Excel does, so ISREF,
PRODUCT and COUNT over the accumulator give Excel's results. The rules are probed first, then implemented, and the
"Unprobed initial values" list in docs/excel-semantics/arrays.md is replaced by evidence.

## Context
- docs/excel-semantics/arrays.md "SCAN and REDUCE keep errors in the LAMBDA" (gen-m7/2c5ea078f25443a2 Calc!A1767,
  18b80d3fd292d63a A467 and A285, excel-graph, T-2225/T-2307): a plain cell initial_value is a reference
  (`REDUCE(Data!A12,Data!C11,PRODUCT)` with both cells TRUE is 0; `REDUCE(Data!C7,Data!E12,COUNT)` with C7 the text
  "1" is 1). It lists as unprobed: a function result such as `IF(TRUE,A1)`; an INDEX or XLOOKUP that picks a cell (ours:
  a reference); a union or a 3-D span (ours: read as values); ISREF of the accumulator (ours: TRUE for a cell).
- arrays.md "BYROW, BYCOL, MAP, SCAN and REDUCE hand a range's slices to the LAMBDA as references" (T-1395, T-1442): "a REDUCE or SCAN accumulator stays as returned (what
  Excel does with `ISREF(a)` on an accumulator is unprobed)".
- Ours: `Model::lambda_initial_value` (crates/rustsheet-engine/src/functions/logical/map_reduce.rs) takes
  `eval_ref_or_index` (reference_operators/mod.rs: a reference or a cell-picking INDEX stays a reference, anything else
  is evaluated with `evaluate_node_with_reference`), and reads a union or a 3-D span as values; its doc says the
  OFFSET, INDIRECT, TRIMRANGE and grid-variable forms of `Model::bind_argument` (functions/logical/let.rs), which a
  LAMBDA call's argument does take, "are not taken here". `fn_scan` (scan.rs) and `fn_reduce` (map_reduce.rs) pass
  the step's result on unchanged.
- Write one probe case, probes/t4475-scan-reduce-reference-initial (excel-probe skill, excel-graph, tag M7). Data:
  A1 TRUE, A2 the text "1", A3 2.5, B1:B3 "x","y","z", and a second sheet with A2 = the text "1". Each initial form X
  of the cell A2 (or A1) is asked three ways that separate a reference from a value: `REDUCE(X,A3,COUNT)` (1 for a
  reference, 2 for the text value), `REDUCE(X1,A1,PRODUCT)` with X1 picking A1 (0 against 1) and
  `REDUCE(X,A3,LAMBDA(a,v,ISREF(a)))`, plus `SCAN(X,A3,LAMBDA(a,v,ISREF(a)))`. The forms X: `A2` (control),
  `IF(TRUE,A2)`, `CHOOSE(1,A2)`, `IFERROR(A2,0)`, `INDEX(A1:A3,2)`, `XLOOKUP("y",B1:B3,A1:A3)`, `OFFSET(A1,1,0)`,
  `INDIRECT("A2")`, `LET(c,A2,REDUCE(c,A3,COUNT))`, `(A2,A3)` (a union) and `Sheet1:Sheet2!A2` (a 3-D span; may be an
  error).
  The accumulator a step returns: `REDUCE(0,A1:A2,LAMBDA(a,v,IF(ISREF(a),"r",v)))` ("r" when the first step's `v`
  stays a reference in the next step's `a`, else "1") and `SCAN(0,A1:A3,LAMBDA(a,v,IF(ISREF(a),"r",v)))` read through
  `ARRAYTOTEXT`.

## Acceptance criteria
- [ ] probes/t4475-scan-reduce-reference-initial is committed with oracle values (excel-graph) for every formula above;
      a formula excel-graph refuses or cannot decode is left pending and named in arrays.md.
- [ ] Every filled cell passes (`make eval-show CASE=probes/t4475-scan-reduce-reference-initial`). Forms that Excel
      passes as references reuse the reference producers `Model::bind_argument` already uses (no second copy of its
      OFFSET/INDIRECT/grid-variable match): either `lambda_initial_value` calls a shared helper factored out of
      `bind_argument`, or it documents with probe cells why a form differs.
- [ ] gen-m7 (the T-2307 cells A1767, A467, A285 above in particular), probes/lambda-helper-range-slices and
      probes/scan-over-groupby still pass; the eval gate shows 0 regressions in every suite.
- [ ] docs/excel-semantics/arrays.md: the "Unprobed initial values" list and the "what Excel does with `ISREF(a)` on an
      accumulator is unprobed" clause are replaced by the probed rules with their cells.
- [ ] Unit tests pin `REDUCE(IF(TRUE,A2),A3,COUNT)` at its probed value (the accumulator formula belongs to T-4478).
- [ ] `make check` green; reviewers approve.

## Out of scope
- A LAMBDA result that is a multi-cell range (MAP/SCAN `#CALC!`, REDUCE's last accumulator): T-4476.
- Implementing the accumulator a step returns (REDUCE and SCAN keeping or dropping a returned reference, `a:v` across
  steps): T-4478, which shares this task's context cell gen-m7/2c5ea078f25443a2 A1767. This task only probes the
  accumulator (the row-11 cells) and hands what it finds to T-4478.
- ISOMITTED of an empty initial value (T-2670) and error or multi-cell-range initial values as argument kinds (T-2878).
- BYROW/BYCOL/MAP slices (already references, T-1395, T-1442). PIVOTBY subtotal work (T-4388, T-4401, T-4414,
  T-4428), function arrays (T-4449) and eta names (T-4450) are unrelated.

## Landing notes
State at the session's landing window (2026-11-06 07:10 UTC): branch `agent/functions-logic/T-4475-scan-reduce-reference-initial`,
no PR yet, no engine change yet. The commit after the claim adds probes/t4475-scan-reduce-reference-initial with oracle values
(excel-graph, 2026-11-06, 64 cells); 33 pass on main, 31 do not. The task's first acceptance criterion is met. What Excel showed
(data: Sheet1 A1 TRUE, A2 text "1", A3 2.5, B1:B3 x,y,z; Sheet2 A2 text "1"):
- **Eta COUNT/PRODUCT are useless here:** `REDUCE(A2,A3,COUNT)` and `REDUCE(A1,A1,PRODUCT)` are `#VALUE!` in Excel for every
  form (the first fill; the task's suggested `REDUCE(X,A3,COUNT)` design is wrong). The committed case uses
  `LAMBDA(a,v,COUNT(a))` and `LAMBDA(a,v,PRODUCT(a))` in columns D, E, H and I instead: a reference gives 0 and a value gives 1.
- **References (Excel):** `IF(TRUE,A2)` (D2, E2, F2, G2), `XLOOKUP(...)` (D6, E6, F6, G6), CHOOSE, INDEX, OFFSET and INDIRECT all
  pass a reference. Ours evaluates IF and XLOOKUP to values. `IFERROR(A2,0)` passes a value (D4, F4), and ours matches.
- **Union:** `(A2,A3)` is a reference (ISREF TRUE, F9 and G9). COUNT(a) of it is 1 (D9) and PRODUCT of `(A1,A3)` is 2.5 (E9), so
  the LAMBDA gets the whole multi-area reference. Ours reads it as values: 0 and `#VALUE!`.
- **3-D span:** `Sheet1:Sheet2!A1` gives `#VALUE!` (E10) where ours gives `#REF!`. ISREF of it is FALSE (F10) and COUNT is 0 (D10).
- **LET-bound initial value (H, I columns):** a LET variable bound to any reference form (A2, IF, CHOOSE, INDEX, XLOOKUP, OFFSET,
  INDIRECT) stays a reference: COUNT 0 and PRODUCT 0. Ours passes the variable's value (1). This is the grid-variable/`bind_argument`
  form the task names. The union variable gives 1 and 2.5 (H9, I9).
- **Accumulator:** REDUCE keeps a step's returned reference in the next step's `a`: `REDUCE(0,A1:A2,LAMBDA(a,v,IF(ISREF(a),"r",v)))`
  is "r" (D11) and the COUNT form is 0 (F11). Ours returns "1" and 2.5. **SCAN does not keep it:** the same SCAN is {TRUE;"1";2.5}
  (E11) and ISREF(a) is FALSE at every step (G11). Ours matches SCAN.
Next step: make `lambda_initial_value` (map_reduce.rs) share `Model::bind_argument`'s reference producers (let.rs). Add IF/XLOOKUP
results that are references (check `eval_ref_or_index`'s callers before widening it), pass a union as a multi-area reference, and
make the 3-D span `#VALUE!`. Then make `fn_reduce` (not `fn_scan`) carry a reference result into the next step. After that,
docs/excel-semantics/arrays.md and the unit tests. Do not repeat: eta COUNT/PRODUCT probes, which are all `#VALUE!`.
Design note (07:05 UTC): `lambda_initial_value` returns a `CalcResult`, and `call_lambda_caller` takes `[CalcResult; 2]`,
so a union (`Binding::Areas`), a LET grid variable (`Binding::Grid`) or a 3-D span (`Binding::Span`) cannot reach the LAMBDA
as a reference. The H/I-column and D9/E9 cells need a `Binding` path into the call: `bind_argument` (let.rs) already
produces it. A REDUCE accumulator that is a reference (D11, F11) needs the same path for each step's result. Check how
`helper_caller`/`call_lambda_caller` bind parameters before choosing between widening the caller to take `Binding`s
and adding a second entry point. That choice may push the task to size L: split it if so (planner).
