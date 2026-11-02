---
id: T-3860
title: "M2 exit cells probes/date-text-mixed-separator-typed B10 and C10 pass"
milestone: M2
lane: functions-date
status: todo
priority: P1
size: S
tier: sonnet
depends_on: [T-3796]
---

## Goal
The two M2 exit cells probes/date-text-mixed-separator-typed `S!B10` and `S!C10` pass, so
`make exit-owners MILESTONE=M2` no longer lists them.

## Context
Steward 2026-11-02: `make exit-owners MILESTONE=M2` listed these two cells as having no open
M2 owner. The case is tagged M2 (T-3751), so its cells count toward M2's exit (decision 0051),
but the fix is T-3796 (M6, functions-date; typed clock-first date-time text such as
`10:00 7/1/2025` is a number in Excel, ours keeps it as text), which a worker has in draft PR
#2855. This task is the M2 owner of the cells (decision 0096) and does not duplicate T-3796's
work: once T-3796 merges, check the two cells with
`make eval-show CASE=probes/date-text-mixed-separator-typed` and close this task. If they still
fail, fix the remainder here (S) or file a follow-up naming them.

## Acceptance criteria
- [ ] probes/date-text-mixed-separator-typed B10 and C10 pass in `make eval`.
- [ ] `make exit-owners MILESTONE=M2` does not list them.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Anything T-3796 already covers beyond these two cells.
