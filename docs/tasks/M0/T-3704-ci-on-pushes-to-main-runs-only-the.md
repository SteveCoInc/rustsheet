---
id: T-3704
title: "CI on pushes to main runs only the eval report"
milestone: M0
lane: harness
status: done
priority: P3
size: S
tier: opus
depends_on: []
---

## Goal
A push to main runs only what main needs (the informational full eval report and its artifact),
not a second copy of the gates the merge queue already ran on the exact merged sha.

## Context
Owner request (2026-11-01, CI-speed investigation). On 2026-11-01 (05:53-09:16Z) 12 of 16 CI runs
on main were cancelled by the next merge's run (concurrency group `ci-${{ github.ref }}`), after
0.2-5.6 min each, so the per-merge eval report of decision 0054 mostly does not exist. In `train`
mode (scripts/merge_train.toml) the train branch's CI tests the sha that lands. Since decision 0247
the merge queue does: the merge group's commit is the commit that lands on `main`.

## Acceptance criteria
- [x] A decision record states when main's push run may be trimmed (always, under the merge queue)
      and what keeps the nightly/full eval coverage; it amends 0054 and 0147.
- [x] CI on main runs the report job without being cancelled by the next merge, and skips the
      duplicated gates.
- [x] `scripts/check_ci_workflow.py` and its tests agree; `make check` green; reviewers approve.

## Out of scope
- PR, merge-group or merge-train CI.

## Outcome
Decision 0249 (amends 0054 and 0147). The owner ruled on 2026-11-02, under the merge queue, to run
only the report on `main`, which supersedes #2739 (written for the train and blocked on SPEC-0027
section 5).
- `ci.yml`: the `changes` job outputs `gates`, which is `'false'` on a push to `main`. Rust, python
  and hygiene are skipped there, the eval job skips `make eval-gates` and runs only `make eval` and
  its artifact, and `rust (fmt, clippy, test)` passes over the skipped shards.
- Each push to `main` has its own concurrency group with no cancelling. A `workflow_dispatch` on
  `main` still runs every gate.
- Checked with `actionlint` (clean before and after) and `check_ci_workflow.py` and its tests.

No eval delta. Bookkeeping in this PR: T-3759 is dropped (its scripts only fed the retired `make
mark-ready`), and its goal moved to T-3827.
