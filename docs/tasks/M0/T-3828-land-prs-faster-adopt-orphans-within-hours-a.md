---
id: T-3828
title: "Land PRs faster: adopt orphans within hours, a fast local gate, one CI run per PR"
milestone: M0
lane: harness
status: done
priority: P1
size: M
tier: opus
depends_on: []
---

## Goal
A PR lands with less session time and fewer CI runs, and a PR whose session ended is picked up
within hours, not a day: workers run a fast, path-aware local gate and leave the full gates to CI;
a PR normally needs one CI run before it is enqueued; and an orphaned PR, including one the merge
queue removed, is adopted by the next opus worker run.

## Context
Owner request (2026-11-02), after the switch to the merge queue (T-3826, decision 0247). Each PR
runs the full gates at least three times: local `make check` (about 25 min in a session
container), PR CI on every pushed head (about 6 min), and the merge-group run. The work-loop
marks the task `done` only after review, so nearly every PR pushes a second head and runs CI
twice (T-3826's PR did: 07370e6, then 6c6e99b for the Outcome and MINOR wording). The long local
run eats the session budget (decision 0229), which is what leaves PRs orphaned. Orphans are
adopted only by the steward's daily run, and only after 24 h open (decision 0050), and since
0247 dropped the train-mode clause, `claude/*` branches are not covered at all. A PR the queue
removes after its session ended waits for that daily run too.

## Acceptance criteria
- [x] `make check-fast` runs `fmt-check`, `hygiene` and `typos` always, and `clippy` and
      `eval-gates` when the change is code by `scripts/ci_changes.py`'s rule (doubt runs them),
      plus `py-check` when `tools/` changed and `py-bindings` when `bindings/` changed; unit
      tests cover the target selection.
- [x] The work-loop runs `make check-fast` plus the tests of the code it changed before pushing,
      and leaves the full `make check` gates to CI; CLAUDE.md's definition of done says so. The
      required CI checks, the merge-group run and `make check` itself are unchanged.
- [x] The work-loop writes `status: done` and the Outcome in the commit that goes to review, and
      does not push for MINOR findings after every reviewer approved (follow-up or noted instead).
- [x] An orphaned PR (agent/* or claude/*, not draft, not queued, branch idle 4 h or more,
      including one the queue removed) is adopted by the next opus worker run (work-loop step 0.3)
      and by the steward, under the existing conditions of the steward skill's "Orphaned agent
      PRs" rule minus the 24 h minimum.
- [x] routines.md's prompts match; a decision record amends 0050, 0247 and the work-loop steps.
- [x] `make check` green; reviewers approve.

## Out of scope
- Changing CI jobs, required checks or the merge-queue settings.
- Deleting the retired train code (T-3827).

## Outcome
Decision 0248 (amends 0050, 0229 and 0247's work-loop steps).
- `make check-fast` (`scripts/fast_gates.py`, 11 unit tests) picks the gates of `make check` that
  the changed paths can break and never runs the full Rust suite. It reuses `ci_changes`
  (`is_code_read`; `_git` became the public `git_bytes`). Docs, skills and scripts take about 2
  min (fmt-check, hygiene, typos), and an unknown path runs every fast target. The work-loop,
  review skill, agent-workflow.md, quality-gates.md and CLAUDE.md (commands, flow, definition of
  done; the stale "squash-merge" became "enqueue") use it. `make check` and the CI jobs are
  unchanged.
- Work-loop step 5 writes `status: done` and the Outcome in the commit that goes to review.
  After every reviewer approves, MINOR findings start no push. This PR followed that.
- Orphans: an `agent/*` or `claude/*` PR, not draft, not queued, branch idle 4 h, including one
  the queue removed, is adopted by the opus worker once per run (work-loop step 0.3, routines.md
  (2b)) and by the steward daily. The 24 h minimum is gone.
- Timing on this PR: a full `make check-fast` (every fast target, since the diff touches the
  Makefile) took 7.5 min, against about 25 min for `make check` in the same container.
- Review: design and security approved bdf1207 with MINOR findings only. One push followed,
  breaking 0248's own MINOR rule on purpose. The security finding that an orphan must come from
  this repository, never a fork, is a real gap this PR widened: opus workers now adopt orphans
  every run. The step 0.3 naming and the `fast_gates._known` comment rode along.

No eval delta.
