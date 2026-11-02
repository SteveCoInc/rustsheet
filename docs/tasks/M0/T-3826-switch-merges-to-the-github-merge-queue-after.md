---
id: T-3826
title: "Switch merges to the GitHub merge queue after the move to SteveCoInc"
milestone: M0
lane: harness
status: done
priority: P0
size: M
tier: opus
depends_on: []
---

## Goal
Merges to `main` go through GitHub's native merge queue (decision 0053) instead of the merge train
and the merge lock: workers and the steward enqueue a green, reviewed PR with
`enable_pr_auto_merge` (SQUASH), and the routines' prompts, skills and docs say so and name the
new repository, `SteveCoInc/rustsheet`.

## Context
Owner request (2026-11-02). The repository moved from `steveash/rustsheet` to the
`SteveCoInc` organization so the merge queue can be turned on (0075 could not, for a personal
account). Evidence from 2026-11-01/02: the train merged about 4.1-4.25 PRs/h while PRs arrived at
about 5.5/h; the ready queue grew from about 45 to 54, ejections from 6 to 13, and
ready-to-merged latency reached 16-19 h. Causes: the merger routine ran hourly (`35 * * * *`,
routines.md says `5,35`); `collect()` in `scripts/merge_train.py` checks each ready PR serially
under the lock (about 19 of the 40-min HOLD_BUDGET at 44 PRs); BATCH_MAX = 5; step mode (0169)
was fragile; long waits caused `conflict with main` and `eval red on main` ejections. Cutting CI
from 10 to 6 min (T-3699) and the 15-min CI timeout (T-3488, decision 0242) did not raise
throughput. Linear history and squash merges are not the bottleneck and stay.

This PR is the switch-over: it lands first, through the queue, before the routines resume. It is
not a one-file flip (decision 0157), so it takes the full panel and a decision record.

## Acceptance criteria
- [x] `scripts/merge_train.toml` says `mode = "lock"`.
- [x] A decision record reactivates 0053 and amends 0075 and 0157: in `lock` mode agents enqueue
      with `enable_pr_auto_merge` and take no merge lock for merges; 0053's rule (disable
      auto-merge before pushing to a queued PR, re-review, enqueue again) is kept.
- [x] The work-loop and steward skills make enqueueing the primary merge path and drop
      merge-lock acquisition for merges.
- [x] `docs/process/routines.md`: the worker, steward and issue-intake prompts enqueue instead of
      `make mark-ready` and the train fallback; the merger routine is marked retired.
- [x] `steveash/rustsheet` is replaced by `SteveCoInc/rustsheet` in skills, `docs/process/`,
      `scripts/tasks.py`, `Cargo.toml` and `bindings/python/pyproject.toml` (historical
      decisions and tasks are left alone).
- [ ] The PR merges through the merge queue (the live test that the queue is on).
- [x] `make check` green; reviewers approve.

## Out of scope
- Deleting the merge-train code (scripts/merge_*.py, the janitor, step mode, SPEC-0027): a
  follow-up task.
- Enqueueing the open `ready-to-merge` and `merge-train-ejected` PRs: done after this merges.
- Repository settings and routines (owner steps).

## Outcome
PR SteveCoInc/rustsheet#2835. Decision 0247 reactivates 0053 and amends 0075 and 0157, with
back-notes in all three. `scripts/merge_train.toml` is `lock`, which the skills now read as
"enqueue". The work-loop step 6, steward, review and issue-intake skills, agent-workflow.md and
routines.md enqueue with `enable_pr_auto_merge` (SQUASH): no merge lock, no `mark-ready`, no
up-to-date check of the agent's own, and the merger routine is retired. 0053's rule stays: turn
off auto-merge before pushing to a queued PR. A refused enqueue goes to the owner with no
fallback. `steveash/rustsheet` became `SteveCoInc/rustsheet` in the skills, `docs/process/`,
`scripts/tasks.py`, `Cargo.toml`, `pyproject.toml` and SPEC-0027.

No eval delta (docs, skills and one TOML value). CI was green on 07370e6. Design and security
approved on that head with MINOR findings only: the "P0 priority" wording and the 0157 wrap are
fixed, and the verdict-record caveat went into T-3827. The last criterion, merging through the
queue, is checked when this PR merges.

Follow-up: T-3827 deletes the retired train, lock and step-mode code.
