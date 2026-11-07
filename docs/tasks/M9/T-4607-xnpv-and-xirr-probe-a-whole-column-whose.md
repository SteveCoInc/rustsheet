---
id: T-4607
title: "XNPV and XIRR: probe a whole column whose used area holds a cell error or a text date, and XIRR dates strictly between -1 and 0, and match"
milestone: M9
lane: functions-financial
status: done
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel cells decide the two unprobed XNPV/XIRR shapes in financial.md "XIRR reads empty cells as 0 and accepts earlier
dates; XNPV is #NUM! on empty cells". financial.md states both rules with the cells, and ours matches every cell.
- A whole-column `values` or `dates` whose used area holds a cell error or a text date.
- XIRR (and XNPV) dates strictly between -1 and 0.

## Context
- Gap triage 2026-11-06 (`make eval` on agent/tasks/refill-sonnet-1206, cut from origin/main). No unowned visibly-off
  cells were left, so this triage read the rules the semantics docs mark unprobed.
- The entry: "Unprobed: which error XNPV gives for a whole column whose used area holds a cell error or a text date
  (the engine returns that error, then #NUM! for the skipped cells); XIRR dates strictly between -1 and 0 (the engine
  truncates first, so -0.5 becomes -1 and is #NUM!)". Its evidence: probes/xirr-xnpv-whole-column-nogaps A1-A8 and
  probes/whole-column-empty-walkers (both pass).
- Ours (release build of this branch, scratch case; sheet D: A1:A3 = -100, 60, `=NA()`, B1:B3 = 39448, 39813, 40000;
  sheet E: the same with A3 = 70 and B3 the text "1/1/2010"):
  - `XNPV(0.1,D!A:A,D!B:B)` and `XIRR(D!A:A,D!B:B)` are `#N/A`;
  - `XNPV(0.1,E!A:A,E!B:B)` is `#NUM!` and `XIRR(E!A:A,E!B:B)` is 0.18863303065299983 (the text date is read and the
    empty cells count as 0);
  - `XIRR({-100,60,70},{-0.5,365,730})`, `XNPV(0.1,{-100,60,70},{-0.5,365,730})` and
    `XIRR({-100,60,70},{0,365,-0.5})` are `#NUM!`.
- Code: `fn_xnpv`, `fn_xirr` and `flow_count` in crates/rustsheet-engine/src/functions/financial/mod.rs.

## Probe
excel-probe skill, excel-graph, tag M9. Name the case probes/t4607-xnpv-xirr-column-errors-and-day-minus-one.
- Whole columns, the layout of probes/xirr-xnpv-whole-column-nogaps:
  - an error (`#N/A`, `#DIV/0!`) in the values column, then in the dates column;
  - a text date ("1/1/2010") and non-date text ("x") in the dates column;
  - a text number ("60") in the values column;
  - each once in the last used row and once in a middle row. Give the XNPV and the XIRR of each.
- The ranges form: the same cells over A1:A3/B1:B3, so the whole-column effect separates from the cell reading.
- Dates in (-1, 0): -0.5 and -1E-9 as the first date, as a later date, and with 0 as the first date, in arrays and in
  cells.

## Acceptance criteria
- [ ] probes/t4607-xnpv-xirr-column-errors-and-day-minus-one is committed with its excel-graph oracle sidecar.
- [ ] Every probe cell passes, or each visibly-off cell has an owner task.
- [ ] financial.md states both rules with the cells, and its "Unprobed" sentence is gone.
- [ ] If the engine changes, a unit test in crates/rustsheet-engine/src/test/test_fn_financial.rs pins one changed cell.
- [ ] No new known-failure line in eval/baseline/probes.txt.
- [ ] `make check` green; reviewers approve.

## Out of scope
- XIRR's solver (root, guess and stop rules: T-3287, T-4251) and boolean or text guesses (T-3749, T-4415).

## Outcome
Probe probes/t4607-xnpv-xirr-column-errors-and-day-minus-one (excel-graph, 88 formula cells, 96 checked): cell errors win over XNPV's empty-cell #NUM! in whole columns (XIRR has none; the range forms hold no empty cell and show only that the error passes through), text dates and numbers are read by XIRR but give XNPV #NUM!, and dates in (-1, 0) are #NUM!. The engine already matched every cell, so no code change. Rules recorded in financial.md; the "Unprobed" sentence is gone.
