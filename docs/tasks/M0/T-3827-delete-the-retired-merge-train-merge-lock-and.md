---
id: T-3827
title: "Delete the retired merge train, merge lock and step-mode code"
milestone: M0
lane: harness
status: done
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
- [x] The scripts, Makefile targets, tests and CI triggers above are deleted, or kept with a
      stated reason; `scripts/check_duplication.py` and the other ratchets are updated with
      `--update` where they shrink.
- [x] No skill or `docs/process/` text names a deleted target or mode.
- [x] `scripts/review_panel.py` goes with `merge_ready.py`, and the review skill says precisely what
      a task-only PR is: every changed path a regular-file task file (matching `tasks.TASK_FILE`)
      under `docs/tasks/`, so not `docs/tasks/README.md` and not a symlink (T-3759, dropped).
- [x] SPEC-0027 is marked superseded by 0247 (kept as history) or removed with a pointer.
- [x] The verdict records (0178) are the only machine check of reviewer verdicts: delete them only
      with a replacement (e.g. a required status check posted by the review step), or have the
      decision record say outright that the panel's verdicts are an agent attestation, unchecked
      (security review of T-3826).
- [ ] The `merge-train-ejected` and `ready-to-merge` labels are deleted from the repository (none is
      in use after the 2026-11-02 cleanup), and nothing in the repo applies or reads them.
- [x] A decision record amends 0247 with what was removed.
- [ ] `make check` green; reviewers approve.

## Out of scope
- Changing the merge-queue flow itself (0053, 0247).
- Repository settings and routines (owner steps).

## Outcome
Decision 0252 (amends 0247, supersedes 0178). Deleted 11 train/lock scripts plus
`merge_train.toml`, `review_panel.py` and 11 test files (about 10.4k lines), 14 Makefile targets,
the `merge-train/*` push triggers and train case of `ci.yml` (the pin is now `["main"]`), the
lock-flip waiver of `check_governance.py`, the carry rule (`carry_problem`, `renumbered_merge`),
and `.claude/skills/review/SKILL.md` from `ci_changes.CODE_READ_PATHS` (its only reader was
`test_merge_ready.py`).

Kept with reasons (decision 0252): new `scripts/git_ops.py` holds the git helpers six scripts
imported from `merge_lock.py`; `make catch-up` and `make renumber` stay for queue removals and
id collisions (they no longer depend on `merge_ready.py` and say "enqueue again" instead of
`mark-ready CARRY=1`; `check_mapping` now has direct tests in place of the carry-rule tests);
`make merge-mode` stays and always prints `lock`, because the owner-managed routine prompts call
it; `gh_client.py` stays for catch-up, its dead requests are follow-up T-3866.

Verdict records: deleted without a machine replacement; 0252 says outright that reviewer
verdicts are an unchecked agent attestation, recorded in the PR body's review table and bound to
the report's `HEAD:` line. The review skill now defines a task-only PR precisely (regular files
matching `tasks.TASK_FILE` under `docs/tasks/`, not README.md, not a symlink). SPEC-0027 is
marked superseded, kept as history. Skills, reviewer agents and `docs/process/` name no deleted
target or mode.

Not done here (owner steps, listed in 0252): deleting the `ready-to-merge`,
`merge-train-ejected`, `merge-train-stop` and `merge-hold` labels and the `merge-lock` /
`merge-train/*` branches. Agents have no label-delete tool and the session git proxy refuses
branch deletion; nothing in the repository applies or reads them any more.

Eval delta: none (tooling only). Duplication 2917 -> 2880 (`--update`).
