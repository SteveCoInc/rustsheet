---
id: T-4330
title: "Probe the remaining WEEKNUM-state call shapes: HSTACK/DROP/TOROW/WRAPROWS/EXPAND beyond a range, CONCAT of INDEX/OFFSET/INDIRECT/IF, TEXTJOIN's delimiter (T-4315 review follow-ups)"
milestone: M6
lane: functions-date
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: [T-4315]
---

## Goal
The parts of WEEKNUM's call table that T-4315 implemented by extension are backed by oracle cells, and the engine
matches them (or owner tasks are filed for the off cells).

## Context
T-4315 (PR #3339) implemented dates.md "Which calls empty WEEKNUM's state" as a per-function table in
`Model::weeknum_effect_of_call` (crates/rustsheet-engine/src/functions/date_and_time/weeknum.rs). Its reviewers'
MINOR findings name the shapes it takes by extension, which dates.md marks unprobed:
- HSTACK, DROP, TOROW, WRAPROWS and EXPAND were probed only with the range `C1:D2`; the engine empties the state for
  any argument. Probe a constant array and one cell for each (as probes/weeknum-stale-call-shapes-2 S01:S11 did for
  CHOOSECOLS, TOCOL, VSTACK, TAKE).
- CONCAT/TEXTJOIN keep the state when a joined argument is a reference, read from the formula (`joins_a_reference`):
  a written reference, INDEX, OFFSET or INDIRECT. Only `CONCAT(INDEX(C1:D2,1,1))` is probed. Probe
  `CONCAT(INDEX({1,2},1))`, `CONCAT(OFFSET(C1,0,0))`, `CONCAT(INDIRECT("C1"))`, `CONCAT(IF(TRUE,C1))`,
  `CONCAT(CHOOSE(1,C1))`, `CONCAT(LET(x,C1,x))` and a defined name for a range (IF, CHOOSE, LET and names count as no
  reference today).
- TEXTJOIN's delimiter: `TEXTJOIN(C1,1,"a")` (a reference delimiter, no joined reference) separates "only joined
  arguments count" from "any reference counts".
- Also unprobed: a one-row range as ARRAYTOTEXT's argument (`ARRAYTOTEXT(C1:D1)`), and WRAPCOLS and TEXTSPLIT.
Layout: the T-4289 one (A1 `=WEEKNUM(100,99)`, A2 the call, A3 `=WEEKNUM(100,2)`; 16 keeps, `#NUM!` empties), C1:D2
holding 1 2 / 3 4, one sheet per call, as in probes/weeknum-stale-call-shapes.

## Acceptance criteria
- [ ] An excel-graph probe case of the shapes above; every cell passes, or an owner task is filed for each off cell.
- [ ] dates.md "Which calls empty WEEKNUM's state" states the probed results and drops them from its unprobed list;
      the engine's table changes where Excel differs.
- [ ] `make check` green; reviewers approve.

## Out of scope
- WEEKNUM calls inside a spilling formula (T-2070).
