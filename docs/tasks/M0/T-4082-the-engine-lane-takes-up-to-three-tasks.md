---
id: T-4082
title: "The engine lane takes up to three tasks in flight at once"
milestone: M0
lane: harness
status: done
priority: P0
size: S
tier: opus
depends_on: []
---

## Goal
Up to three engine tasks can be in flight at once, so the P0 engine owners of the Phase 1 exit cells
are worked in parallel instead of one after another.

## Context
Owner request (2026-11-04). After #3126 made the exit-cell owners P0 and `engine` the only critical
lane, ten of the sixteen P0 owners sit in `engine`, and `make next-task` allowed one engine task in
flight across every worker (decision 0022), so they ran in series.

## Acceptance criteria
- [x] `scripts/tasks.py` lets `engine` hold three tasks in flight and every other lane one; a unit
      test covers both.
- [x] work-loop, agent-workflow.md and routines.md state the rule; decision 0269 amends 0022.
- [x] `make check-fast` green; reviewers approve.

## Out of scope
- Splitting `engine` into sub-lanes (revisit after Phase 1 exits).

## Outcome
Decision 0269. `LANE_SLOTS = {"engine": 3}` in `scripts/tasks.py`: `available()` counts in-flight
tasks per lane and treats a lane as busy only when its slots are full. New test
`test_engine_takes_up_to_three_tasks_in_flight_and_other_lanes_one`; the 66 `test_tasks.py` tests
pass. The work-loop skill, agent-workflow.md and the routines' worker prompts say "three in engine".
No eval delta.
