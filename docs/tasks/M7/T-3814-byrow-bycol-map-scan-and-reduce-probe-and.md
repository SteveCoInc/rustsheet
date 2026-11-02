---
id: T-3814
title: "BYROW, BYCOL, MAP, SCAN and REDUCE: probe and match a LAMBDA of the wrong arity, optional parameters and eta-reduced functions"
milestone: M7
lane: functions-logic
status: done
priority: P2
size: S
tier: opus
depends_on: []
---

## Goal
BYROW, BYCOL, MAP, SCAN and REDUCE give Excel's result (one error, an error per element, or values) for a LAMBDA with
too few or too many parameters, a LAMBDA with optional parameters, and an eta-reduced function (`_xleta.NAME`) whose
argument count does not fit the helper, checked by one shared rule rather than per helper.

## Context
Lead: the 2026-11-01 holdout themes note (docs/research/holdout-themes-2026-11-01.md, theme 8, MAKEARRAY "errors for
arity and boundary sizes"), generalized to the sibling helpers as the note asks. It names a class of input, never a
case, and it is not Excel evidence: every behaviour change here needs fresh oracle cells from the excel-probe skill,
and the probe cases must not copy the note's wording. If the probes show the engine already matches Excel across the
class, record that in the Outcome and close the task.

Gap triage 2026-11-02 (report of `make eval` on 2026-11-02, excel-graph evidence): every visibly-off cell has an owner,
so this is an evidence gap, not a failing-cell group. The generated suites draw each helper's LAMBDA with the catalog's
arity (`arity:` in eval/catalog/functions.yaml), so no Excel cell holds a wrong-arity LAMBDA for these five helpers.
What is evidenced:
- BYROW and BYCOL refuse anything that is not a LAMBDA with `#VALUE!` (arrays.md, "BYROW and BYCOL: a LAMBDA result that
  is an array" and the paragraph after it);
- GROUPBY/PIVOTBY pass each eta-reduced function its catalog minimum argument count, and 0 or 3+ is `#VALUE!`
  (arrays.md, T-1706, probes/groupby-eta-arity G!A1-A141);
- `MAKEARRAY(2,2,_xleta.ABS)` and `BYROW(A1:B3,_xleta.NOW)` give an error excel-graph cannot decode (arrays.md, left
  pending): such cells cannot be committed as evidence; record them as refused or undecodable, as T-1387 did.
Unprobed: for each of BYROW, BYCOL (one parameter), MAP over 1, 2 and 3 arrays (one per array), SCAN and REDUCE (two):
a LAMBDA with one parameter fewer and one more than needed; zero parameters (`LAMBDA(1)`); a needed parameter made
optional (`LAMBDA(x,[y],..)` in BYROW, `LAMBDA(a,[b],..)` in REDUCE) and an extra optional one; a LET-bound and a
defined-name LAMBDA of the wrong arity; eta-reduced functions with a fixed count that does not fit (`_xleta.ABS` in
SCAN/REDUCE, `_xleta.MOD` in BYROW, `_xleta.PI`) and with a range of counts that does (`_xleta.SUM`, `_xleta.ROUND`); and
whether the error is one cell or one per element (over a 3x2 source, and over an empty-result source such as a 1x1).
Ours today: every call checks the count in `check_arg_count` (functions/logical/lambda.rs), so a wrong arity is a
`#VALUE!` per element, never one error before iterating; eta functions go through `function_value`
(`FunctionValue::Eta`). Helpers: functions/logical/bycol_byrow.rs, map_reduce.rs, scan.rs.
Coordinate with T-3655 (MAKEARRAY arity and sizes, functions-logic): whichever lands second reuses the first's arity
check; neither adds a second one (directive 3).

## Acceptance criteria
- [x] A probe case (excel-graph, canary-verified) covers the forms above for all five helpers; undecodable or refused
      forms are listed in the semantics entry, not committed as cells.
- [x] The engine matches every probed cell through one arity rule shared by the five helpers (and MAKEARRAY's, if
      T-3655 has landed).
- [x] docs/excel-semantics/arrays.md states the rule with the probe cells cited, in the same PR.
- [x] `make check` green; eval known failures only shrink (baseline updated, no new lines); reviewers approve.

## Out of scope
- MAKEARRAY (T-3655) and GROUPBY/PIVOTBY eta arity (T-1706, T-2644).
- LAMBDA results that are arrays or references (evidenced; arrays.md).

## Outcome
Probe `probes/t3814-lambda-helper-arity` (M7, excel-graph 2026-11-02, canary-verified, 248 cells over 124 forms):
BYROW and BYCOL (3x2 and 1x1 sources), MAP over 1, 2 and 3 arrays, SCAN and REDUCE (3x2 and 1x1) with LAMBDA(1),
one parameter fewer and one more, optional parameters (needed and extra), LET-bound and defined-name LAMBDAs of the
wrong arity, and `_xleta.ABS/MOD/PI/SUM/ROUND`. Column B gives the result's shape or the whole error's ERROR.TYPE,
column C the elements. Excel's rule: any arity mismatch is one `#VALUE!` for the whole result (never per element);
a function whose argument range covers the count runs. The engine already matched every cell (`check_arg_count`,
one rule shared by the five helpers), so no engine change. No form was undecodable: wrapping in ROWS/ERROR.TYPE/
TEXTJOIN avoids the error excel-graph cannot decode. arrays.md records the rule. Eval delta: +1 probe case, 248
cells, all bit-exact; no regressions, no new known failures.
