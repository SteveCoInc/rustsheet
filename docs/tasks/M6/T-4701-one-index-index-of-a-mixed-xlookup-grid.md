---
id: T-4701
title: "One-index INDEX of a mixed XLOOKUP grid keeps the reference element (t4563 V25:V26)"
milestone: M6
lane: functions-lookup
status: in-progress
priority: P3
size: M
tier: opus
depends_on: [T-4570]
---

## Goal
`INDEX(X(r),2)` (one index into a column of elements of a mixed XLOOKUP grid) keeps the element it picks as a
one-element array of references, so `INDEX(INDEX(X(r),2),1,1)` hands the range back and spills it, as Excel does
(SPEC-0014 amendment rule 2).

## Context
- Evidence: probes/t4563-xlookup-not-found-ref-element V25:V26 (`INDEX(INDEX(XLOOKUP({2;9},..,Data!$D$1:$D$2),2),1,1)`
  spills x, y; excel-graph). T25:U25 (ROWS/COLUMNS 1) and W25:Z25 (`#REF!`) already pass.
- T-4570 makes `index_grid` (`functions/lookup_and_reference/index.rs`) give a one-element *value* array for one index
  into a column grid (`TODO(T-4701)` there): its value view is right, but the reference is lost. INDEX returns a
  `CalcResult`, so keeping it needs INDEX's reference form to return a `RefArg` (a 1x1 `RefGrid`) where a reference is
  taken (`index_reference`, `reference_arg_read` in `functions/util.rs`).
- Whether the same holds for OFFSET or INDIRECT grids (`INDEX(OFFSET(A1,{0;1},0),2)`, rule 3's `#VALUE!`) is
  unprobed: probe it first (excel-probe skill), since the change would reach them too.

## Acceptance criteria
- [ ] t4563 V25:V26 pass, with no regression (reference-lift-consumers-b1d E109 and t2577 A100 keep passing).
- [ ] SPEC-0014 rule 3 and the amendment's INDEX bullet say what is implemented.
- [ ] `make check` green; reviewers approve.

## Out of scope
- One index into a grid of several columns (still `#VALUE!`).
