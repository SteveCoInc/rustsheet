---
id: T-4462
title: "Probe TIME's and DATE's near-integer windows to a relative 2^-14, as XMATCH's: 2^-22+2^-33 below a positive integer and the half window below a negative one"
milestone: M6
lane: functions-date
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
Excel cells pin both edges of the near-integer window of TIME's and DATE's arguments as tightly as lookup.md
already pins XMATCH's: to a relative 2^-14 (an absolute ~2^-36) below 1, 2 and 3 for a positive argument, and below
-1 and 0 for a negative one. The engine either matches every new cell or a follow-up task is filed.

## Context
- Gap triage 2026-11-06 (`make eval` on origin/main 16a00ebc7). dates.md "TIME: an argument past 32767 is `#NUM!`; a
  near-integer hour counts as that integer (T-2267)" says: "The exact constants inside those brackets are unprobed
  for TIME and DATE."
  - The measured brackets are wide: positive (2^-22+1E-12, 2.4E-7) and negative (2^-23+1E-12, 1.2E-7)
    (probes/time-argument-window, probes/date-argument-window).
  - The engine reads all of them with `near_integer_whole` (crates/rustsheet-engine/src/functions/util/near_integer.rs):
    `NEAR_INTEGER_TOLERANCE` = 2^-22 + 2^-33 below a positive value and half of it below a negative one.
- lookup.md "XMATCH and XLOOKUP floor a fractional mode, snapping near an integer (T-2656)" brackets that same
  constant for XMATCH's and XLOOKUP's modes: `1-2^-22*(1+2^-11)` reads 1 and `1-2^-22*(1+9*2^-14)` reads 0. That probe
  layout (snap-edge-fine, xlookup-mode-snap-edge column E) is the template here.
- Probe design: the same offsets for TIME's hour, minute and second and for DATE's year, month and day, below 1, 2
  and 3 and below -1 and 0 (the negative edge at 2^-23 times the same factors). Where possible use a form whose result
  shows the floor directly, such as `TIME(h,0,0)*24` and `DATE(2020,3,d)`. Mind Excel's precedence: write
  `-(2^-22)`, not `-2^-22`.

## Acceptance criteria
- [ ] A probe case under eval/suites/probes, oracle-filled from excel-graph, covers the offsets above for every
      TIME and DATE argument position.
- [ ] dates.md records the brackets in the TIME and DATE sections, citing the cells, and drops "The exact constants
      inside those brackets are unprobed for TIME and DATE".
- [ ] Every new cell passes. If any fails, the PR adds its known-failure lines and files a follow-up task with `make
      new-task` that names them (the constant or the negative half would then differ for TIME/DATE from XMATCH's).
- [ ] `make check` green; reviewers approve.

## Out of scope
- Other readers of `near_integer_whole` (LEFT, RIGHT, REPLACE, INDEX, XMATCH: text.md and lookup.md).
