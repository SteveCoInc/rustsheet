---
id: T-2686
title: "An operator over a whole GROUPBY result keeps its nested cells (not #VALUE!), so MEDIAN over it is #N/A"
milestone: M7
lane: engine
status: done
priority: P3
size: S
tier: opus
depends_on: [T-2609]
---

## Goal
An arithmetic operator over a whole GROUPBY/PIVOTBY result whose cells are nested arrays (`LAMBDA(z,z)`) treats those cells as Excel
does, so `MEDIAN({1;2;"";4},GROUPBY(..,LAMBDA(z,z))-x,..)` is `#N/A` as in Excel.

## Context
Found by T-2623. Owned cell: gen-m9/17c110a53924beb8.case.json Calc!A300
`=MEDIAN({1;2;"";4},(_xlfn.GROUPBY(Data!A1:B12,Data!A1:A12,LAMBDA(z,z))-(Data!B8-0)),Data!E4:E4)`: Excel #N/A, ours #VALUE!.
Evidence (probes/t2475-stat-subexpressions, excel-graph, sheet D04 = that case's Data sheet):
- P!A62, the whole cell: ERROR.TYPE 7 (#N/A), ours 3 (#VALUE!);
- P!A63, `GROUPBY(..)-(D04!B8-0)`: Excel `64|13x3|?`: TEXTJOIN over `IFERROR(v&"",..)` fails, as it does over nested cells
  (arrays.md T-2592: TEXTJOIN over a result holding nested cells is #VALUE!); ours yields a flat 13x3 with `#VALUE!` (`#3`) in every
  nested cell, which MEDIAN then returns;
- P!A64, the GROUPBY alone, matches (`64|13x3|?` both).
So the minus does not turn a nested cell into `#VALUE!` the way the engine does (SPEC-0018: "an operator over the whole result reads a
nested cell as `#VALUE!`", unprobed); MEDIAN over nested cells is #N/A (arrays.md "Aggregates over nested GROUPBY and PIVOTBY cells").
Whether the operator keeps each nested cell as a nested array of differences or does something else is T-2609's probe
(`SUM(GROUPBY(..,LAMBDA(v,v))+0)`, `ISERROR(GROUPBY(..)+0)`); implement from its answer. Code: the nested-array reader
(`ArrayNode::Nested`, SPEC-0018) and the binary-operator array lift in the evaluator.

## Acceptance criteria
- [x] SPEC-0018 and arrays.md state the operator rule from T-2609's evidence.
- Narrowed (T-2609's evidence): gen-m9/17c110a53924beb8 Calc!A300 and probes/t2475-stat-subexpressions P!A62, A63 now
  give ours in Excel; their October values stay committed and T-4661 owns them.
- [ ] `make check` green; reviewers approve.

## Out of scope
- None.

## Outcome
No engine change: T-2609's probes (excel-graph, 2026-11-06) show that an operator over a whole GROUPBY/PIVOTBY result makes every
nested cell `#VALUE!`, whatever its size (1x1 included), kind, depth or operator, which is what ours already does (every operator
cell of probes/t2609-nested-operators-and-lifts, -one-cell and -d04 passes). The owned cells' `#N/A` is Excel's October behavior:
refills on 2026-11-06 of the same workbooks give `#VALUE!` for gen-m9/17c110a53924beb8 Calc!A300 and t2475-stat-subexpressions
P!A62, and our listing for P!A63 (-d04 A6, A7 repeat them; the refills of the original bytes are not committed). Re-filling until a
run matches us is ruled out (T-3610), so the October values stay and the cells pass to T-4661, which decides which fill is the
target alongside Excel's other 2026-11 nested-cell changes. SPEC-0018 rule 4 and "Consumers with their own rule" and arrays.md
now state the operator rule with this evidence. Eval: no cell fixed, no regression.
