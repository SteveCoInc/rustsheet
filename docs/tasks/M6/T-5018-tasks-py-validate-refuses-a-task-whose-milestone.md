---
id: T-5018
title: "tasks.py validate refuses a task whose milestone has no docs/milestones.md heading, so a planning PR cannot break make progress-report (M17, #3878)"
milestone: M6
lane: harness
status: todo
priority: P2
size: S
tier: sonnet
depends_on: []
---

## Goal
A task file whose `milestone:` has no `## M<n>` heading in docs/milestones.md fails `tasks.py validate` (and so CI),
instead of first surfacing as a `make progress-report` refusal in the next steward run.

## Context
#3878 (tasks: plan M17) merged T-4808 to T-4822 under `milestone: M17` before decision T-4808 adds M17 to
docs/milestones.md. The progress-report generator refuses a milestone the task counts carry but the page has no
heading for (tools/src/rustsheet_tools/progress_report/goals.py, `_check_known`), so the 2026-11-07 evening steward
run could not regenerate docs/progress.md. Nothing in CI caught it: only the steward runs the generator. The check
belongs where task files are validated (scripts/tasks.py), reading the headings the way goals.py does (one definition:
reuse or share that parser, never copy it). Today's M17 tasks are left to T-4808, which adds the heading; this task
must not fail `main` meanwhile, so either land it after T-4808 or make the new check list the milestones it allows.

## Acceptance criteria
- [ ] `tasks.py validate` fails, naming the file, on a task whose milestone has no docs/milestones.md heading; a unit test shows it.
- [ ] `make check` passes on `main` when it lands.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Deciding M17's scope or phase (T-4808).
