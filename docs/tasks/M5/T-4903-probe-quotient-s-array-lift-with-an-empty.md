---
id: T-4903
title: "Probe QUOTIENT's array lift with an empty argument and SERIESSUM's empty coefficients against an x/n/m error"
milestone: M5
lane: functions-math
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Close the two open points T-4873 left: whether a written-empty QUOTIENT argument beside an array lifts to `#N/A`
in each element, and which error wins when SERIESSUM's coefficients are empty and x, n or m is an error.

## Context
- T-4873 review follow-up (PR #3930, excel-fidelity MINOR): probes/t4873-empty-arg-quotient-ceiling Calc!A7:A8
  (`QUOTIENT({7,8},)`, `QUOTIENT(,{2,4})`) spill into occupied cells and show only `#SPILL!`; numbers.md "A
  written-empty argument of a ToolPak function is #N/A" marks the coefficient order UNVERIFIED.
- Ours: `fn_quotient` (mathematical.rs) gives each element `#N/A` through `Model::atp_number`; `series_coefficients`
  (seriessum.rs) refuses empty coefficients, and that error is reported after x, n and m are checked.
- Design MINOR from the same review: `crates/rustsheet-engine/src/functions/logical/let.rs` near line 283 has
  `matches!(node, Node::EmptyArgKind) || self.is_omitted_variable(node)`, a copy of `Model::is_empty_argument`. Call
  the helper instead.

## Acceptance criteria
- [ ] A probes case (excel-graph) has `INDEX(QUOTIENT({7,8},),1,2)`, `ROWS(QUOTIENT(,{2;4}))`, and
      `SERIESSUM(1/0,1,1,)`, `SERIESSUM(2,"x",1,)` and `SERIESSUM(2,1,NA(),)`, each in a cell with room to spill.
      Their predictions are committed before the fill, and each cell passes.
- [ ] numbers.md drops the UNVERIFIED note or states the order the probe shows.
- [ ] let.rs uses `is_empty_argument` at the duplicated test.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The 2010+ roundings (T-4873 settled them).
