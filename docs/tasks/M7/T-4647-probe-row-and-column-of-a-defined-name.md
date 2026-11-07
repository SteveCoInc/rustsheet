---
id: T-4647
title: "Probe ROW and COLUMN of a defined name bound to an array constant, and of an IF or CHOOSE array pick"
milestone: M7
lane: research
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
Oracle evidence says what ROW and COLUMN return for an array that reaches them through a defined name or an IF or
CHOOSE pick, and arrays.md states it.

## Context
- T-4610 made ROW and COLUMN lift over any array value a name, IF, CHOOSE or LET gives (each element `#VALUE!`, an
  error element its own error; arrays.md "ROW, COLUMN and RANDARRAY over an array value"). Only the LET-bound and
  LAMBDA-parameter forms are probed (probes/t4519-row-column-randarray-over-arrays, t4610-row-column-array-error-element);
  the defined-name and IF/CHOOSE forms follow by inference (T-4610 excel-fidelity review).
- Excel for the web refuses a workbook holding ROW of an array constant or a function call (FileCorruptTryRepair,
  T-4519), so `ROW(CHOOSE(1,{1;2}))` may be refused too; a defined name bound to `{1;#N/A}` (`ROW(arr)`) should load.
  Wrap each in ARRAYTOTEXT and ROWS/COLUMNS, as the t4610 probe does.
- Code: `Model::row_or_column` (functions/lookup_and_reference/mod.rs), `reference_ranges_or_array`.

## Acceptance criteria
- [ ] A probes case covers ROW and COLUMN of a defined name bound to `{1;#N/A}` and `{1,2;3,4}`, and the IF and
      CHOOSE array picks (or records that Excel for the web refuses them), oracle-filled.
- [ ] arrays.md states the result; a functions/engine task is filed for any cell the engine gets wrong.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Engine changes (a follow-up if the probe needs one).
