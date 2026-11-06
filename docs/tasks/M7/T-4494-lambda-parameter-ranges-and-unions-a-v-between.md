---
id: T-4494
title: "LAMBDA parameter ranges and unions: a:v between two parameters is their range (not columns A:V), (a,v) is a union, AREAS of a REDUCE result, and a union REDUCE initial value stays a reference"
milestone: M7
lane: engine
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
The cells of probes/t4478-reduce-scan-reference-accumulator that T-4478 left failing pass. They cover:
- a range written between two LAMBDA parameters;
- a union of parameters;
- AREAS over a REDUCE call;
- a union as REDUCE's initial value.

## Context
- From T-4478 (excel-graph, 2026-11-06; docs/excel-semantics/arrays.md "REDUCE keeps a returned reference").
- `a:v`, where `a` and `v` are LAMBDA parameters, is the range between the two references they hold in Excel.
  `REDUCE(A1,A2:A3,LAMBDA(a,v,a:v))` is A1:A3 (3x1). The engine parses it as the column range A:V
  (1048576x22). The same probe with names that cannot be column letters (`p_a:p_v`, Calc!D20:H27) already passes.
  Cells: Calc!F5, H5, F6, G6, H6, F9, G9, H9, H14, G15, H15, J1, J2, J3.
  - In SCAN, step 2's `a` is a value, so `ROWS(a:v)` there is `#VALUE!` (H14 `{2;#VALUE!}`) and SCAN's
    `a:v` output is `#CALC!` (G15, H15).
  - The parser has to tell a LAMBDA/LET name from a column label inside the LAMBDA's scope.
- `AREAS(REDUCE(..))` is `#ERROR!` in ours, so the formula does not parse or evaluate. In Excel it is 3 for
  `REDUCE(A1,A2:A3,LAMBDA(a,v,(a,v)))` and for the `p_a` form, and 2 for `REDUCE((A1,A3),{1},LAMBDA(a,v,a))`.
  Cells: Calc!E7:H7, E16:H16, E22:H22.
- A union initial value stays a reference: `REDUCE((A1,A3),{1},LAMBDA(a,v,ISREF(a)))` is TRUE (Calc!H17).
  Ours reads it (`lambda_initial_value`, crates/rustsheet-engine/src/functions/logical/map_reduce.rs).

## Acceptance criteria
- [ ] Every cell named above passes; no regressions.
- [ ] arrays.md "REDUCE keeps a returned reference" and the `lambda_initial_value` comment are updated.
- [ ] `make check` green; reviewers approve.

## Out of scope
- SCAN's and REDUCE's accumulator rule itself (T-4478).
