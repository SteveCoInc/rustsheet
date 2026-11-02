---
id: T-3827
title: "Delete the retired merge train, merge lock and step-mode code"
milestone: M0
lane: harness
status: todo
priority: P1
size: M
tier: opus
depends_on: [T-3826]
---

## Goal
The merge train, the merge lock and step mode, retired by decision 0247, are gone from the
repository: no dead scripts, Makefile targets, CI triggers or skill text are left to mislead an
agent or cost CI time.

## Context
Decision 0247 (T-3826) moved merges to GitHub's merge queue and retired the train (0157) and the
lock (0075) for merges, but left about 10.9k lines in place so that the switch-over PR stayed
small. What is left: `scripts/merge_*.py` and their tests, the janitor, step mode (0169,
`make merge-train-ack`, `make merge-train-next`), `make mark-ready` and `merge-ready-verify`,
`make catch-up` and `make renumber` (keep whatever is still useful outside the train, e.g.
renumbering on an id collision), the verdict records (0178) read only by `mark-ready`,
`scripts/merge_train.toml` and `make merge-mode`, the `merge-train/*` push trigger in `ci.yml`
(decision 0147) and its `check_ci_workflow.py` pins, the `merge-lock` branch, SPEC-0027, and the
train sections of the steward, work-loop and review skills.

## Acceptance criteria
- [ ] The scripts, Makefile targets, tests and CI triggers above are deleted, or kept with a
      stated reason; `scripts/check_duplication.py` and the other ratchets are updated with
      `--update` where they shrink.
- [ ] No skill or `docs/process/` text names a deleted target or mode.
- [ ] SPEC-0027 is marked superseded by 0247 (kept as history) or removed with a pointer.
- [ ] A decision record amends 0247 with what was removed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the merge-queue flow itself (0053, 0247).
- Repository settings and routines (owner steps).
