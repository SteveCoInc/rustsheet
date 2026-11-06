---
id: T-4483
title: "rustsheet-accuracy math --function: regenerate only the sidecars of cases that call the named functions"
milestone: M0
lane: harness
status: in-progress
priority: P1
size: S
tier: opus
depends_on: []
---

## Goal
`rustsheet-accuracy math --suite S --function NAME` (repeatable) recomputes and rewrites only the math sidecars of
the cases whose formulas call one of the named functions, leaving every other sidecar of the suite untouched.

## Context
- Every Gate A table task that adds a definition must regenerate the probes and probes-numeric math sidecars so
  `make eval-math-check` passes (decision 0243). `write_suite` recomputes every case: about an hour of four CPUs for
  probes-numeric, and the cells that hit the time limit ("not converged or too slow") flip between a value and a skip
  from one machine to the next, so a full rewrite drags unrelated diffs into the PR (T-4083, #3157: 11 such files).
- About 25 open Gate A table tasks (T-4090 to T-4116) each need this step.

## Acceptance criteria
- [ ] `--function NAME` (repeatable, a registered definition's name) selects the cases whose case file calls
      `NAME(`; only those sidecars are written. An unknown name is an error. Without it, behaviour is unchanged.
- [ ] Tests in tools/tests; `make check-fast` green; reviewers approve.

## Out of scope
- Caching unchanged cells across runs.
