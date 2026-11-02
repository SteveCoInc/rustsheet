---
id: T-3831
title: "Workers adopt every PR the merge queue dropped, without the 4 h idle wait"
milestone: M0
lane: harness
status: in-progress
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
A PR that the merge queue dropped is picked up within one opus worker run (2 h), even after its
own session has ended.

## Context
Decision 0248 (T-3828) makes an orphan of a PR whose branch has been idle 4 h. The opus worker
adopts the oldest orphan, one per run (work-loop step 0.3), and the steward sweeps orphans daily.
During the 2026-11-02 cleanup the queue dropped 18 of 49 enqueued PRs within about an hour, because
PRs ahead of them merged and conflicted with them. At one adoption per 2-hour run that is a day of
delay. The 4 h wait exists so that an adopter does not race a live session. A session that
enqueued a PR and moved on is not working on it, so a drop needs no such wait.

## Acceptance criteria
- [ ] Work-loop step 0.3 and the steward skill's "Orphaned agent PRs": a PR whose last merge-queue
      timeline event is `removed_from_merge_queue` counts as an orphan at once, with no 4 h wait, as
      long as its branch has not moved since the drop. The opus worker adopts every such PR at the
      start of its run, oldest first, within its session budget (decision 0229). The other orphans
      keep the 4 h rule and one per run.
- [ ] A cheap way to find drops: a `make` target or a `scripts/` helper that lists open PRs whose
      last queue event is a removal, from the issue timeline API. A GitHub Action that labels drops
      `queue-dropped` is an acceptable alternative.
- [ ] `docs/process/routines.md` opus prompt step (2b) matches; a decision record amends 0248.
- [ ] `make check-fast` green; reviewers approve.

## Out of scope
- Changing the queue settings (decision 0053) or the never-fork rule (0248).
