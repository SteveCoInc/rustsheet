---
id: T-4500
title: "Fuzz report job: keep going past a failed gh call, soften the log-bytes wording, and confirm the intake trigger fires"
milestone: M12
lane: harness
status: todo
priority: P2
size: S
tier: sonnet
depends_on: [T-4490]
---

## Goal
The nightly fuzz `report` job (decision 0302) still reports every failed target when one `gh`
call fails, its issue body makes no claim the job log may not meet, and the first real red run
shows whether the issue intake picked the issue up.

## Context
These are the MINOR findings from T-4490's review (#3509), on `.github/workflows/fuzz.yml`'s
`report` job and `scripts/fuzz_issue.py`. Both reviewers approved, so they wait for this task.
- **Design:** the step runs under `set -euo pipefail`, so one failed `gh issue create` or
  `gh issue comment` stops the loop, and later failed targets get no issue. The steward backstop
  covers it, but the job should go on past a failure and fail at the end.
- **Security:** the "Too large to embed" text says the job log prints each input's bytes.
  cargo-fuzz printed them for both 2026-11 timeouts, but that is not shown for every failure kind
  (for example a very large OOM input). Word it as "the job log usually prints", or point only to
  the run's artifact.
- **Design:** `scripts/fuzz_issue.py` has a shebang but is not executable. Drop the shebang, or
  set the mode to `100755`.
- **Design:** decision 0302 records as unverified whether the issue intake's GitHub trigger fires
  for an issue opened with the workflow token. On the first red scheduled run after T-4490,
  check whether the intake took the issue in on `issue.opened` or only through the steward's
  daily sweep, and record the answer in decision 0302.
- **Security (constraint, no change now):** the intake's "Fuzz finding" trust anchor is the author
  `github-actions[bot]`. It holds only while `report` is the only job in any workflow with
  `issues: write`. Note this in decision 0302 so a later workflow that wants `issues: write`
  revisits it.

The workflow, `scripts/` and decisions are protected paths, so this needs a decision-0302
revision line and the design and security reviewers' explicit approval.

## Acceptance criteria
- [ ] The `report` step records a failure per target and continues, then exits non-zero if any
      target failed. Shown by a stub-`gh` dry run, as T-4490's Outcome describes.
- [ ] The issue body's large-input text makes no unconditional claim about the job log;
      `scripts/test_fuzz_issue.py` covers the new wording.
- [ ] `scripts/fuzz_issue.py` has no shebang, or is executable.
- [ ] Decision 0302 gets a revision line: the `issues: write` constraint, and the trigger
      observation (or "not yet observed" while no red run has happened).
- [ ] `make check` green; reviewers approve.

## Out of scope
- The same blind spot for `.github/workflows/nightly.yml` (nightly eval).
