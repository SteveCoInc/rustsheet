---
id: T-5088
title: "Restore Gate A point LINEST/s1 A184 with a double-double Householder reflector dot"
milestone: M9
lane: functions-stat
status: todo
priority: P2
size: M
tier: opus
depends_on: []
---

## Goal
accuracy/LINEST/s1!Calc!A184 is back in the batch's `check` list and inside the envelope (Excel 5206016).

## Context
Decision 0351 removed it. In simulation, summing the Householder reflector's dot product v·t in double-double and rounding once gives exactly 5206016; it was never run against Gate A.

Filed by T-4975's review (PR #4073).

## Acceptance criteria
- [ ] The goal holds, shown by tests or eval cells.
- [ ] `make check` green; reviewers approve.

## Out of scope
-
