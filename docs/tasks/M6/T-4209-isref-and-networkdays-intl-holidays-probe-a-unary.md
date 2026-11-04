---
id: T-4209
title: "ISREF and NETWORKDAYS.INTL holidays: probe a unary plus over a reference-returning function (+INDEX, +OFFSET, +XLOOKUP)"
milestone: M6
lane: functions-date
status: done
priority: P3
size: S
tier: sonnet
depends_on: [T-4190]
---

## Goal
ISREF and the NETWORKDAYS.INTL holidays split give Excel's answer for a unary plus over a reference-returning function
(`+INDEX(C1:C2,1)`, `+OFFSET(C1,0,0)`, `+XLOOKUP(1,{1},C1:C1)`, `+IF(TRUE,C1)`), which today is an inference.

## Context
- Follow-up from T-4190's Excel-fidelity review (MINOR). `written_reference` (crates/rustsheet-engine/src/functions/
  information/mod.rs) returns "no reference" for a unary plus over any operand. The evidence covers only a cell, plain,
  grouped or under `@`: probes/networkdays-intl-holiday-kinds G1:G8 and H1:H3 (`ISREF(+C1)` is FALSE, `ISREF(@C1)` is TRUE).
  The function-operand case is unprobed.
- Probe design (excel-probe skill, tag M6): use the -kinds layout, with C1 = 45003 (a Saturday) and C2 = 45006 (a Tuesday)
  under weekend 13. `NETWORKDAYS.INTL(45000,45100,13,h)` is 87 for a scalar Saturday holiday and 86 for a reference one,
  and the reverse for the Tuesday holiday. Add `ISREF(...)` of each form, and `T(+INDEX(D1:D2,1))` with D1 holding text.
- Also add T-4190 to the `Tasks:` line of dates.md "NETWORKDAYS.INTL: a scalar holiday ignores the weekend argument"
  (the same review's other MINOR).

## Acceptance criteria
- [x] A probe case holds the forms above, filled from excel-graph.
- [x] references.md "ISREF: a unary plus makes a value" states the rule for a function operand, and cites the cells.
- [x] The engine matches every new cell, through `written_reference` and nowhere else.
- [x] dates.md's `Tasks:` line names T-4190.
- [x] `make check` green; reviewers approve.

## Out of scope
- Unary minus (already a value) and the other holidays forms, which T-4190 evidenced.

## Outcome
- Probe `networkdays-intl-holiday-unary-plus-fn` (M6, excel-graph): a unary plus over `INDEX`, `OFFSET`, `XLOOKUP` or `IF` is a value for ISREF (FALSE) and a scalar NETWORKDAYS.INTL holiday; `T(+INDEX(D1:D2,1))` returns the text.
- Engine: `holiday_serials` read `+OFFSET(...)` and `+IF(TRUE,C1)` as references (86/87 swapped). It now takes the written plus from `written_reference` (read before IF's pick drops it). Probe 15/15 pass; `networkdays-intl-holiday-kinds` still 87/87.
- Docs: references.md "ISREF: a unary plus makes a value" extended; dates.md `Tasks:` names T-4190.
