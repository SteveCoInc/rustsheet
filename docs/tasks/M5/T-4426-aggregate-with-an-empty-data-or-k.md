---
id: T-4426
title: "AGGREGATE with an empty data or k argument, or data from a LET or defined name: probe Excel (numbers.md 'Still unprobed')"
milestone: M5
lane: functions-math
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
AGGREGATE gives Excel's value or error when an argument is present but empty (`AGGREGATE(14,0,,1)`,
`AGGREGATE(9,0,)`, `AGGREGATE(14,0,E1:E3,)`), and when its data is a scalar or a range bound by LET or by a defined name.
numbers.md calls both cases unprobed today.

## Context
- Gap triage 2026-11-06 (`make eval` on origin/main 20b163db, excel-graph sidecars). `make unowned-off ALL=1` gives 0
  unowned visibly-off cells, so this triage read the rules the semantics docs mark unprobed. The functions-math lane had
  no ready task. (The INFO argument kinds this id was first made for are already T-2872's, so the task was rewritten.)
- numbers.md, the AGGREGATE entry, "Scalar data and error order" (probes/t4193-aggregate-scalar-data-error-order,
  excel-graph, 2026-11-04, T-4193):
  - in the reference form a literal number is `#VALUE!` (Calc!A1:A3);
  - in the array form it is a one-value array (Calc!A4:A7);
  - text and boolean scalars are skipped (Calc!A8:A10).
- It ends: "Still unprobed: an empty-argument data (`AGGREGATE(14,0,,1)`) and a scalar from a name or LET." AGGREGATE has
  catalog prior 0.5 (M5).
- Ours: `fn_aggregate` and `aggregate_scalar` in crates/rustsheet-engine/src/functions/aggregate.rs. A scratch case on a
  release build of 20b163db gives:
  - `#VALUE!` for `AGGREGATE(14,0,,1)`, `AGGREGATE(9,0,)`, `AGGREGATE(9,0,,E1:E3)`, `AGGREGATE(15,6,,2)`,
    `AGGREGATE(1,0,,)`, `AGGREGATE(4,0,E1:E3,)` and `LET(s,5,AGGREGATE(9,0,s))`;
  - `#NUM!` for `AGGREGATE(14,0,E1:E3,)`;
  - 5 for `LET(s,5,AGGREGATE(14,0,s,1))`.
- Whether Excel reads an empty argument as 0, as missing (as an omitted k) or as an error is the question. So is whether
  a LET- or name-bound scalar acts as a literal (as ours does) or as a value that the reference form refuses differently.
  The answers are error-or-number, so no fitting is involved.

## Probe
excel-probe skill, excel-graph, tag M5. Name the case probes/t4426-aggregate-empty-and-bound-data. Data Data!E1:E6 =
1, 2, 3, 4, 5, `#DIV/0!`.
- Empty data or k:
  - functions 1, 4, 9, 14, 15, 16 and 19 with options 0 and 6;
  - forms `(f,o,)`, `(f,o,,1)`, `(f,o,,E1:E6)`, `(f,o,E1:E6,)` and `(f,o,,)`.
- Bound data:
  - `LET(s,5,AGGREGATE(f,o,s[,1]))`, `LET(r,Data!E1:E6,AGGREGATE(f,o,r[,1]))`, and a LET-bound array `{1,2,3}`;
  - the same with a workbook name `n5` = `=5` and `nr` = `=Data!$E$1:$E$6`.
- `ERROR.TYPE` and `TYPE` companions on each.

## Acceptance criteria
- [ ] probes/t4426-aggregate-empty-and-bound-data is committed with excel-graph oracle values.
- [ ] Every cell passes. A cell ours gets wrong is fixed here, with a unit test in aggregate.rs, when the fix is in
      argument handling (`fn_aggregate`, `aggregate_scalar` or the shared empty-argument reader, reused not copied). Any
      other wrong cell is owned by a new functions-math task made with `make new-task` that names the case and cells
      (`campaign: semantics`, with a `## Visible-parity stage` section).
- [ ] numbers.md's "Still unprobed: an empty-argument data ... and a scalar from a name or LET" is replaced by the
      result, citing the cells.
- [ ] `make eval-baseline` records no regression; `make check` green; reviewers approve.

## Out of scope
- k rounding and a boolean k in LARGE/SMALL (T-1034).
- AGGREGATE over a LET-bound union's areas (T-1461, a performance task).
- SUBTOTAL and AGGREGATE inside a CSE array formula (T-4226).
