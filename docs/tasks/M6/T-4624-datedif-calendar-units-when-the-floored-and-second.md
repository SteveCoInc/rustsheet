---
id: T-4624
title: "DATEDIF calendar units when the floored and second-rounded date orders differ: probe and match"
milestone: M6
lane: functions-date
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
DATEDIF matches Excel when its two dates fall on the same floored day but in a different order once rounded to the
second (e.g. start N-0.4 s, end N-0.6 s), and near the top of the calendar where rounding carries past 9999-12-31.

## Context
- T-4574 found that DATEDIF "D" floors its serials while "M", "YM", "MD" and "YD" read them rounded to the nearest
  second (`days360_serial`; dates.md "Other date functions floor a serial just below midnight").
- `datedif_value` (crates/rustsheet-engine/src/functions/date_and_time.rs) checks the order and the calendar range on
  the floored serials and computes the calendar units from the rounded ones, so `DATEDIF(N-0.4/86400,N-0.6/86400,"m")`
  computes from a reversed pair (-1 months in the engine today), and `DATEDIF(1,2958465.99999999,"m")` is #NUM!
  because the rounded end is off the calendar. Neither is evidenced.

## Acceptance criteria
- [ ] An excel-graph probe covers every unit with start and end on one floored day in reversed rounded order, and
  with an end just below 2958466, against integer controls.
- [ ] dates.md records the result with cells; the engine matches every probe cell.
- [ ] `make check` green; reviewers approve.

## Out of scope
- The ordinary near-midnight reads (T-4574).
