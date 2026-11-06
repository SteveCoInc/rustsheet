---
id: T-4387
title: "Salvage draft PR #3177's BETA.DIST direct-bgrat findings into statistics.md"
milestone: M9
lane: functions-stat
status: todo
priority: P3
size: S
tier: sonnet
depends_on: []
---

## Goal
The findings of draft PR #3177 (T-3933, branch `agent/functions-stat/T-3933-bgrat-direct`) are recorded on `main`
in docs/excel-semantics/statistics.md and the task file, so they survive whatever happens to the draft.

## Context
Stale-draft triage, steward 2026-11-05 (second run), bucket 5 (decision 0264). #3177's head has not moved since
2026-11-04 14:26 UTC. T-3933 was parked when decision 0267 deferred it to Phase 1b (its cells are first wrong only
at the 14th or 15th digit). The branch adds a statistics.md entry, the research scripts under
`docs/research/t3933/` with their output, and Landing notes in the task file that say what reopening needs. `main` has
none of these.

Copy, from the branch, only docs: the statistics.md entry (marked as the branch's finding, its status as recorded
there), the `docs/research/t3933/` files, and the Landing notes into T-3933's task file on `main` (keep `defer:
phase-1b`). Read the branch with `git show`; do not run its scripts. The bratio code change stays on the branch.

## Acceptance criteria
- [ ] statistics.md, `docs/research/t3933/` and T-3933's task file on `main` hold the branch's findings and Landing
      notes; T-3933 stays deferred.
- [ ] A comment on #3177 links this task's PR.
- [ ] `make check` green; reviewers approve.

## Out of scope
- T-3933's bratio change.
