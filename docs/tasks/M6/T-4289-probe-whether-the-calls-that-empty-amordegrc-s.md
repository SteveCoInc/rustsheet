---
id: T-4289
title: "Probe whether the calls that empty AMORDEGRC's left-over value (SEQUENCE, SORT, XLOOKUP, LAMBDA, ...) are what empties WEEKNUM's stale state"
milestone: M6
lane: functions-date
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel evidence says whether WEEKNUM's stale `return_type` state is emptied by the same calls that empty AMORDEGRC's
left-over value, or by a dynamic-array formula as such. If the evidence calls for it, the engine matches it.

## Context
- T-3393 found that AMORDEGRC's left-over value is emptied by particular function calls (SEQUENCE, SORT, UNIQUE,
  XLOOKUP, TEXTJOIN, MMULT, a called LAMBDA, ...), at the point each one is evaluated. A dynamic-array formula is no
  exception, and TRANSPOSE, `={1,2}`, MDETERM, LET and INDEX keep the value. See financial.md "AMORDEGRC below
  period 1" and probes/amordegrc-fraction-state-ag to -da.
- dates.md "WEEKNUM's stale `return_type` state" (T-2068) says a dynamic-array formula empties WEEKNUM's state.
  In that evidence each such formula was `#SPILL!`, an error or a 1x1 result. The engine empties the state at every
  dynamic anchor's place (`run_excel_order` in excel_order.rs). The same function-call rule may be what Excel
  actually does there.
- T-2070 asks whether a formula that spills empties it.

## Acceptance criteria
- [ ] Probe cases, one workbook each, put a WEEKNUM reader above a Monday setter with one call between them. Probe
  at least SEQUENCE, SORT, a called LAMBDA, TRANSPOSE, `={1,2}` and LET, in a dynamic-array formula and inside SUM.
  They are filled on excel-graph.
- [ ] dates.md records the rule. The engine matches the probes, or a follow-up task owns every visibly-off cell.
- [ ] `make check` green; reviewers approve.

## Out of scope
- AMORDEGRC itself (done in T-3393).
