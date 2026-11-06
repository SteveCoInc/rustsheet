---
id: T-4057
title: "Freeze probes-numeric once decision 0267's Tier A and B tasks close: a check that refuses new cases"
milestone: M0
lane: harness
status: blocked
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
probes-numeric is frozen. A check on every PR refuses a branch that adds a probes-numeric case or adds cells to an
existing one, unless a decision record names an exception. Oracle refills of existing cells are still allowed. The
freeze starts once decision 0267's Tier A and B work closes, as the owner asked ("after we get through a-d then I
want to freeze the probes and move on").

## Context
- Decision 0267. Until the freeze, new probes-numeric cases are allowed only for a task with a "Decision 0267" or
  "Decision 0267 cells" section, and only for the cells it owns.
- The check belongs with the governance gates (scripts/check_governance.py, or a new script that `make check` runs,
  against the origin/main merge base). scripts/ is a protected path, so the PR cites decision 0267.

## Blocked because
Decision 0267's Tier A and B tasks are still open. The task is unblocked when this command prints nothing, that is,
when every task with a "Decision 0267" or "Decision 0267 cells" section is done, blocked or dropped:
`grep -l '^## Decision 0267' -r docs/tasks | xargs grep -l '^status: \(todo\|in-progress\)'`

## Acceptance criteria
- [ ] A check, run by `make check` and CI, fails on a branch that adds a file under eval/suites/probes-numeric/cases/
      or adds cells to an existing case, unless a decision record names the exception. It passes on oracle sidecar
      refills of existing cells.
- [ ] Tests cover an added case, an added cell, a refill and an exception.
- [ ] `docs/process/quality-gates.md` lists the gate. `make check` is green; reviewers approve.

## Out of scope
- Freezing the other probe suites (`probes` and the generated suites).
