---
id: T-4673
title: "Salvage duplicate PR #3637's T-4574 near-midnight probes and its T-4624 follow-up onto main"
milestone: M6
lane: functions-date
status: in-progress
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The probe evidence of PR #3637 (T-4574, branch `agent/functions-date/T-4574-near-midnight-serial-60`) is on `main`:
its two excel-graph probes `probes/t4574-near-midnight-serial-60` (234 cells) and
`probes/t4574-near-midnight-calendar-units` (112 cells, at N = 45017) with their oracle and math sidecars, and its
follow-up task T-4624 (DATEDIF calendar units when the floored and rounded date orders differ, "Y"'s reading, a
rounded end past the calendar).

## Context
Duplicate-PR check, steward 2026-11-06 (evening run). T-4574 was implemented twice: #3647 merged it (b85d6584,
2026-11-06 15:10 UTC), so #3637, idle since 15:18 UTC, is closed as the duplicate and its branch is kept. #3637's
probes and T-4624 are not on `main` (`main` has only `probes/t4574-feb29-near-midnight-date-args`). #3637 says
DATEDIF's calendar units read each date rounded to the nearest second; #3647 says they carry half a second. These
may be the same rule: the probes decide.

Copy the probe files and T-4624's task file from the branch with `git show` (do not run its scripts; check that
T-4624's id is still free on `main`, else allocate a fresh one with `make new-task`). Run `make eval`: if every new
cell passes with `main`'s engine, record them with `make eval-baseline`; for any that fail, file or extend a task
that owns them (the branch's engine change in `date_and_time.rs` and `yearfrac.rs` is a reference, not to be copied
blindly). Reconcile the dates.md entry with what landed in #3647.

## Acceptance criteria
- [ ] Both probes and their sidecars are on `main`, and every failing cell in them has an owning task.
- [ ] T-4624 (or its renumbered copy) is on `main`; dates.md agrees with the probes.
- [ ] Eval: 0 regressions.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing #3647's rule beyond what the probes require.
