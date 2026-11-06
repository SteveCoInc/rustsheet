---
id: T-4517
title: "Probe FILTER's column filter by a one-column include as tall as the array is wide (include_axis), with and without an error element"
milestone: M7
lane: functions-lookup
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
FILTER's handling of an n x 1 include whose height equals the array's column count (not its row count) is decided by
oracle evidence: whether Excel filters the columns (as `include_axis` does today) or returns `#VALUE!`, with and without
an error element in that include.

## Context
- T-4477 review follow-up (PR #3528, excel-fidelity MINOR): `include_axis`
  (crates/rustsheet-engine/src/functions/lookup_and_reference/filter.rs) accepts a vertical include as tall as the array
  is wide as a column filter, e.g. `FILTER(A1:B3,{TRUE;FALSE})`. No probe covers it; T-4477 moved its unit test off this
  shape. docs/excel-semantics/arrays.md (T-3497 entry, "Ours") marks it "not probed by T-4477".
- Probe case probes/t4517-filter-transposed-include (excel-probe skill, excel-graph, tag M7), A1:B3 and A1:C2 numbers,
  C1 `#DIV/0!`, each result read through `ROWS(..)&"x"&COLUMNS(..)`, `ERROR.TYPE(..)` and `ARRAYTOTEXT(..)`:
  `FILTER(A1:B3,{TRUE;FALSE})`, `FILTER(A1:B3,{1;0})`, `FILTER(A1:C2,{1;0;1})`, `FILTER(A1:C2,{1,0,1})` (control),
  `FILTER(A1:B3,C1:C2)` with C2 TRUE, `FILTER(A1:B3,{1;#N/A})`, and the square control `FILTER(A1:B2,{1;0})`.

## Acceptance criteria
- [ ] The probe case is committed with oracle values for every formula above, and every cell passes.
- [ ] `include_axis` matches the evidence (changed only if Excel differs); arrays.md's "not probed by T-4477" note is
      replaced by the probed rule with its cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
- FILTER's one-cell error includes and indirect references (T-4477, done).
