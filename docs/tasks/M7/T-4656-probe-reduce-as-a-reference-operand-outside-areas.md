---
id: T-4656
title: "Probe REDUCE as a reference operand outside AREAS, a:v between LET names, and SCAN's union initial value"
milestone: M7
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
It is established, with excel-graph evidence recorded in docs/excel-semantics/, whether Excel takes a REDUCE call
as a reference operand outside AREAS, whether `a:v` between two LET names is their range, and whether SCAN keeps a
union initial value a reference.

## Context
- From T-4494 (review round 1, excel-fidelity). Only AREAS(REDUCE(..)), SUM(REDUCE(..)) and `a:v` between LAMBDA
  parameters are probed (probes/t4478-reduce-scan-reference-accumulator). Ours, after T-4494, with A1:A3 = 1, 2, 3:
  - `=REDUCE(A1,A2:A3,LAMBDA(a,v,v)) A3`, `=SUM(REDUCE(A1,A2:A3,LAMBDA(a,v,v)):A5)` and
    `=SUBTOTAL(9,REDUCE(A1,A2:A3,LAMBDA(a,v,v)))` are refused (parse error), since REDUCE is not in
    `Function::may_be_reference` (crates/rustsheet-engine/src/functions/util.rs).
  - `=REDUCE(0,A1:A3,LAMBDA(a,v,LAMBDA(x,x+v)))(2)` (calling a REDUCE result).
  - `=SUBTOTAL(9,TAKE(REDUCE(0,A1:A3,LAMBDA(a,v,VSTACK(a,v))),2))` spills `#VALUE!` per element (value accumulator),
    and `=SUBTOTAL(9,TAKE(REDUCE(A1:A3,{1},LAMBDA(a,v,a)),2))` (reference accumulator).
  - `=LET(a,A1,v,A3,SUM(a:v))` is 6 (`Parser::bound_names_range` binds LET names too; inferred) and
    `=LAMBDA(a,COLUMNS(a:C))(A1)` is 3 (column range A:C; inferred).
  - `=SCAN((A1,A3),{1},LAMBDA(a,v,ISREF(a)))` is FALSE (REDUCE's twin is TRUE, Calc!H17).
- docs/excel-semantics/arrays.md "REDUCE keeps a returned reference; SCAN reads it"; references.md "Intersection
  operands must be references", "Calling a call's result", "SUBTOTAL and AREAS take references only".
- Use the `excel-probe` skill; a refused workbook is recorded as a refusal, not committed.

## Acceptance criteria
- [ ] A probe case covers every form listed in Context, with excel-graph values or recorded refusals.
- [ ] arrays.md and the three references.md entries record the results; any mismatch with ours is a new engine task.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the engine (file a task for any mismatch).
