---
id: T-4574
title: "DATEDIF, YEARFRAC and DAYS360 just below midnight at serials 59, 60 and 61 (the fictitious 1900-02-29): probe and match"
milestone: M6
lane: functions-date
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The floor rule T-4454 found for date-serial arguments just below midnight is evidenced for DATEDIF, YEARFRAC and
DAYS360 at the bases around Excel's fictitious 1900-02-29 (serial 60), and the engine matches it there.

## Context
- T-4454 (#3490, probes/t4454-near-midnight-date-args) showed that DATEDIF, YEARFRAC and DAYS360 floor a serial just
  below midnight (N-0.4 s and N-(2^-22+2^-33) read day N-1; DAYS360 rounds to the second). Its bases avoid serials
  59/60/61, which T-4454's Context suggested. The only evidence near serial 60 is one gen-m6 DATEDIF cell
  (59.999988426).
- Serial 60 is the day that does not exist, so a floor from just below 61 lands on it and a floor from just below 60
  lands on 1900-02-28. Month and year arithmetic (DATEDIF "m"/"ym"/"md", DAYS360's end-of-February rules, YEARFRAC's
  basis 1) can differ there.
- docs/excel-semantics/dates.md: "Other date functions floor a serial just below midnight" (T-4454).
- Excel-fidelity review MINOR on #3490.

## Acceptance criteria
- [ ] An excel-graph probe covers DATEDIF (units "d", "m", "y", "md", "ym", "yd"), YEARFRAC (bases 0 to 4) and
  DAYS360 (both methods) with a start or end of 59, 60 or 61 minus 0.4 s and minus 2^-22+2^-33, against the integer
  controls.
- [ ] dates.md records the result with cells.
- [ ] The engine matches every probe cell (or each gap has a filed task).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Negative WEEKNUM return types (T-4470).
