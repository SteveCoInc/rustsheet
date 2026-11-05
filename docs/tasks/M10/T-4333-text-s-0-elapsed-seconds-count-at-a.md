---
id: T-4333
title: "TEXT [s].0 elapsed seconds count at a tenths tie (86908.95 s is 86909.9, 3497.95 s is 3497.9)"
milestone: M10
lane: functions-text
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
The elapsed `[s]` count before a fraction matches Excel at tenths ties: `TEXT(3497.95/86400,"[s].0")` is "3497.9", `TEXT(3733.95/86400,"[s].0")` "3733.9", `TEXT(45000+2740.95/86400,"[s].0")` "3888002740.9" (ours one second more each).

## Context
Found by T-4323. The engine counts `[s]` from the value rounded to the shown digit (`FractionalSeconds::elapsed`); visibly-off cells: probes/t4323-fraction-sweep-2 A9, A19, A27. Neither that count nor `floor(y*86400)` of the half-step register fits those and probes/t4323-fraction-ties A80, A90 together. See docs/excel-semantics/number-formats.md "Fraction digits of a second at a tie".

## Acceptance criteria
- [ ] probes/t4323-fraction-sweep-2 A9, A19, A27 pass without moving another cell.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
