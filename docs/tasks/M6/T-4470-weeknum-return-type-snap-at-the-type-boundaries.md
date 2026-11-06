---
id: T-4470
title: "WEEKNUM return_type snap at the type boundaries (1, 18, 21, 22), negative types and the start-weekday state; link the two WEEKNUM return_type entries"
milestone: M6
lane: functions-date
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
WEEKNUM's return_type snap (T-4454) is evidenced at the edges it now moves, and the two dates.md WEEKNUM entries
agree with each other.

## Context
- T-4454 (#3490) showed that a positive return_type within 2^-22 + 2^-33 below an integer counts as that integer
  (probes/t4454-near-midnight-date-args AA1:AB4). The engine snaps before choosing the type, so these readings
  changed without a probe:
  - `1-2^-23` (was type 0, always `#NUM!`; now type 1)
  - `18-2^-23`, `21-2^-23` and `22-2^-23` (22 and up read the stale state)
  - the start-weekday state that `2-2^-23` sets (now Monday), as read by a following `WEEKNUM(100,99)`
- A negative type still truncates. `-2^-30` reads 0 (`#NUM!`) by truncation but -1 (the stale state) by a floor or
  snap, so one cell separates the two models.
- dates.md "WEEKNUM return_type edge cases" still says "a fraction truncates: 2.9 is 2, 21.5 is 21". It should point
  to "Other date functions floor a serial just below midnight; ... but WEEKNUM's return_type".
- Excel-fidelity review MINORs on #3490.

## Acceptance criteria
- [ ] An excel-graph probe covers the cells above.
- [ ] dates.md records the result, and the old entry links to the new one.
- [ ] The engine matches every probe cell.
- [ ] `make check` green; reviewers approve.

## Out of scope
- ISOWEEKNUM and WEEKDAY's return_type (file separately if needed).
