---
id: T-3831
title: "Workers adopt every PR the merge queue dropped, without the 4 h idle wait"
milestone: M0
lane: harness
status: done
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
- [x] Work-loop step 0.3 and the steward skill's "Orphaned agent PRs": a PR whose last merge-queue
      timeline event is `removed_from_merge_queue` counts as an orphan at once, with no 4 h wait, as
      long as its branch has not moved since the drop. The opus worker adopts every such PR at the
      start of its run, oldest first, within its session budget (decision 0229). The other orphans
      keep the 4 h rule and one per run.
- [x] A cheap way to find drops: a `make` target or a `scripts/` helper that lists open PRs whose
      last queue event is a removal, from the issue timeline API. A GitHub Action that labels drops
      `queue-dropped` is an acceptable alternative.
- [x] `docs/process/routines.md` opus prompt step (2b) matches; a decision record amends 0248.
- [ ] `make check-fast` green; reviewers approve.

## Out of scope
- Changing the queue settings (decision 0053) or the never-fork rule (0248).

## Outcome
Decision 0253 (amends 0248). New `.github/workflows/queue-drops.yml` (on `pull_request`
`enqueued`, `dequeued`, `synchronize`, `closed`): labels an open same-repo PR `queue-dropped` when
the queue removes it and clears the label on a re-enqueue, a push or a close, so a labelled PR is
a drop nobody has touched since. The job checks out nothing and runs no PR code; write rights
(`issues`, `pull-requests`) sit on the job, the workflow level stays `contents: read`. This is the
acceptance criterion's Action alternative: the session proxy refuses GraphQL (where the
merge-queue timeline events live), so a `scripts/` helper reading the REST timeline could not be
relied on. Agents find drops with `list_pull_requests` (field `labels`).

Work-loop step 0.3, the steward skill's "Orphaned agent PRs" and routines.md's opus step (2b):
every labelled PR is an orphan at once and is adopted first, oldest first, within the session
budget; other orphans keep the 4 h rule, one per run. Work-loop step 6.3: commit a change before
disabling auto-merge and push at once, so an owner's own dequeue carries the label only briefly.

Owner step: update the opus worker routine's step (2b) to routines.md's text.
Follow-up filed in this PR: T-3868 (citation fix from #2860's fidelity review, T-3837).
Eval delta: none (process and CI only).
